use std::collections::hash_map::Entry;
use std::marker::PhantomData;

use fnv::FnvHashMap;

use crate::index::list_segment_files;
use crate::indexer::operation::AddOperation;
use crate::indexer::segment_updater::save_metas;
use crate::indexer::{DocIdMapping, SegmentWriter};
use crate::schema::document::{BinaryValueSerializer, Document};
use crate::schema::{Field, Value};
use crate::{Directory, Index, IndexMeta, Opstamp, Segment, TantivyDocument, TantivyError};

struct ClusteredDocuments {
    fields: Vec<Field>,
    root: DocumentGroup,
    memory: usize,
}

impl ClusteredDocuments {
    fn mem_usage(&self) -> usize {
        let memory = self.memory + self.fields.capacity() * std::mem::size_of::<Field>();
        // Reserve 50% headroom for grouping and indexing. This is an estimate, not a limit.
        memory + memory / 2
    }
}

#[derive(Default)]
struct DocumentGroup {
    children: FnvHashMap<Vec<u8>, DocumentGroup>,
    documents: Vec<AddOperation<TantivyDocument>>,
}

impl DocumentGroup {
    fn push(
        &mut self,
        fields: &[Field],
        operation: AddOperation<TantivyDocument>,
        memory: &mut usize,
    ) -> crate::Result<()> {
        let Some((&field, remaining)) = fields.split_first() else {
            *memory += operation.document.mem_usage() - std::mem::size_of::<TantivyDocument>();
            let capacity = self.documents.capacity();
            self.documents.push(operation);
            *memory += (self.documents.capacity() - capacity)
                * std::mem::size_of::<AddOperation<TantivyDocument>>();
            return Ok(());
        };
        // Store full binary values, so hash collisions cannot combine distinct groups.
        let mut key = Vec::new();
        for value in operation.document.get_all(field) {
            BinaryValueSerializer::new(&mut key).serialize_value(value.as_value())?;
        }
        let capacity = self.children.capacity();
        let child = match self.children.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                *memory += entry.key().capacity();
                entry.insert(DocumentGroup::default())
            }
        };
        let result = child.push(remaining, operation, memory);
        // Account incrementally: mem_usage() must not walk the tree on every added document.
        // Estimate map storage as entries plus control bytes; 50% headroom covers table slack.
        *memory +=
            (self.children.capacity() - capacity) * (std::mem::size_of::<(Vec<u8>, Self)>() + 1);
        result
    }

    fn index(self, writer: &mut SegmentWriter) -> crate::Result<()> {
        for operation in self.documents {
            writer.add_document(operation)?;
        }
        for child in self.children.into_values() {
            child.index(writer)?;
        }
        Ok(())
    }
}

#[doc(hidden)]
pub struct SingleSegmentIndexWriter<D: Document = TantivyDocument> {
    segment_writer: SegmentWriter,
    segment: Segment,
    opstamp: Opstamp,
    clustering: Option<ClusteredDocuments>,
    _phantom: PhantomData<D>,
}

impl<D: Document> SingleSegmentIndexWriter<D> {
    pub(crate) fn new(index: Index, mem_budget: usize) -> crate::Result<Self> {
        let segment = index.new_segment();
        let segment_writer = SegmentWriter::for_segment(mem_budget, segment.clone())?;
        Ok(Self {
            segment_writer,
            segment,
            opstamp: 0,
            clustering: None,
            _phantom: PhantomData,
        })
    }

    /// Groups incoming documents by full field values, buffering them until indexing at
    /// finalization. Fields are hierarchical: `["status", "service"]` groups by status, then
    /// service. All values of each field participate, in document order; missing fields form
    /// their own group. Group order is unspecified, but arrival order within each group is
    /// preserved.
    ///
    /// Must be configured before adding documents. Documents are buffered as `TantivyDocument`s.
    /// `mem_usage()` includes buffer allocations plus 50% headroom; callers decide when to
    /// finalize. There is no automatic flush, and indexing errors may be deferred until
    /// finalization.
    pub fn with_clustering_fields(mut self, fields: &[&str]) -> crate::Result<Self> {
        let settings = self.segment.index().settings();
        if self.opstamp != 0 || fields.is_empty() {
            return Err(TantivyError::InvalidArgument(
                "clustering requires nonempty fields and must be configured before adding \
                 documents"
                    .to_string(),
            ));
        }
        if settings.sort_by_field.is_some() || settings.manual_doc_id_mapping {
            return Err(TantivyError::InvalidArgument(
                "clustering cannot be combined with sort_by_field or manual_doc_id_mapping"
                    .to_string(),
            ));
        }
        let schema = self.segment.index().schema();
        let fields = fields
            .iter()
            .map(|name| schema.get_field(name))
            .collect::<crate::Result<_>>()?;
        self.clustering = Some(ClusteredDocuments {
            fields,
            root: DocumentGroup::default(),
            memory: 0,
        });
        Ok(self)
    }

    pub fn mem_usage(&self) -> usize {
        self.segment_writer.mem_usage()
            + self
                .clustering
                .as_ref()
                .map_or(0, ClusteredDocuments::mem_usage)
    }

    pub fn add_document(&mut self, document: D) -> crate::Result<()> {
        let opstamp = self.opstamp;
        self.opstamp += 1;
        if let Some(clustering) = &mut self.clustering {
            let document = document.into_tantivy_document();
            return clustering.root.push(
                &clustering.fields,
                AddOperation { opstamp, document },
                &mut clustering.memory,
            );
        }
        self.segment_writer
            .add_document(AddOperation { opstamp, document })
    }

    pub fn finalize(self) -> crate::Result<Index> {
        let Self {
            segment,
            mut segment_writer,
            clustering,
            ..
        } = self;
        if let Some(clustering) = clustering {
            clustering.root.index(&mut segment_writer)?;
        }
        let max_doc = segment_writer.max_doc();
        segment_writer.finalize()?;
        let did_remapping = segment.index().settings().sort_by_field.is_some();
        Self::finalize_inner(segment, max_doc, did_remapping, false)
    }

    pub fn finalize_with_doc_id_mapping(self, mapping: &DocIdMapping) -> crate::Result<Index> {
        if self.clustering.is_some() {
            return Err(TantivyError::InvalidArgument(
                "clustering cannot be combined with a manual doc id mapping".to_string(),
            ));
        }
        let Self {
            segment,
            segment_writer,
            ..
        } = self;
        let max_doc = segment_writer.max_doc();
        segment_writer.finalize_with_doc_id_mapping(mapping)?;
        Self::finalize_inner(segment, max_doc, true, true)
    }

    fn finalize_inner(
        segment: Segment,
        max_doc: u32,
        did_remapping: bool,
        clear_manual_doc_id_mapping: bool,
    ) -> crate::Result<Index> {
        let segment: Segment = segment.with_max_doc(max_doc);
        let segment_meta = segment.meta();
        let mut index = segment.index().clone();
        if clear_manual_doc_id_mapping {
            index.settings_mut().manual_doc_id_mapping = false;
        }

        if did_remapping {
            // Untrack the temp docstore file from the segment metadata.
            segment_meta.untrack_temp_docstore();
        }

        let persisted_custom_extensions: Vec<String> = index
            .custom_plugins()
            .iter()
            .flat_map(|plugin| plugin.extensions().iter().copied())
            .map(str::to_string)
            .collect();
        let index_meta = IndexMeta {
            index_settings: index.settings().clone(),
            persisted_custom_extensions,
            segments: vec![segment_meta.clone()],
            schema: index.schema(),
            opstamp: 0,
            payload: None,
        };
        save_metas(&index_meta, index.directory())?;
        index.directory().sync_directory()?;

        if did_remapping {
            // Run the garbage collector to remove the temp docstore file from the directory.
            let mut living_files = list_segment_files(
                std::slice::from_ref(segment_meta),
                &index_meta.persisted_custom_extensions,
            );
            living_files.insert(crate::core::META_FILEPATH.to_path_buf());
            index.directory_mut().garbage_collect(|| living_files)?;
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Count;
    use crate::directory::RamDirectory;
    use crate::index::SegmentComponent;
    use crate::query::PhraseQuery;
    use crate::schema::{Schema, FAST, STORED, STRING, TEXT};
    use crate::{DocAddress, IndexSettings, IndexSortByField, Order, Term};

    fn writer(schema: Schema, settings: IndexSettings) -> crate::Result<SingleSegmentIndexWriter> {
        Index::builder()
            .schema(schema)
            .settings(settings)
            .single_segment_index_writer(RamDirectory::default(), 15_000_000)
    }

    #[test]
    fn clustering_indexes_in_final_order() -> crate::Result<()> {
        let mut schema = Schema::builder();
        let status = schema.add_text_field("status", STRING | STORED);
        let service = schema.add_text_field("service", STRING | STORED);
        let message = schema.add_text_field("message", TEXT | STORED);
        let id = schema.add_u64_field("id", FAST | STORED);
        let schema = schema.build();
        let values = [
            ("err", "api"),
            ("ok!", "web"),
            ("err", "web"),
            ("err", "api"),
            ("ok!", "api"),
        ];
        for fields in [["status", "service"], ["service", "status"]] {
            let mut writer = writer(schema.clone(), IndexSettings::default())?
                .with_clustering_fields(&fields)?;
            let initial_memory = writer.mem_usage();
            for (i, (status_value, service_value)) in values.into_iter().enumerate() {
                writer.add_document(doc!(
                    status => status_value, service => service_value,
                    message => if i % 2 == 0 { "alpha beta" } else { "beta alpha gamma" },
                    id => i as u64
                ))?;
            }
            assert_eq!(writer.segment_writer.max_doc(), 0);
            let buffer = writer.clustering.as_ref().unwrap();
            assert_eq!(buffer.root.children.len(), 2);
            let allocated = buffer.memory + buffer.fields.capacity() * std::mem::size_of::<Field>();
            assert_eq!(
                writer.mem_usage(),
                writer.segment_writer.mem_usage() + allocated + allocated / 2
            );
            assert!(writer.mem_usage() > initial_memory);

            let index = writer.finalize()?;
            let searcher = index.reader()?.searcher();
            let segment = searcher.segment_reader(0);
            assert_eq!(segment.num_docs(), 5);
            let ids = segment.fast_fields().u64("id")?.first_or_default_col(0);
            let fieldnorms = segment.get_fieldnorms_reader(message)?;
            let actual: Vec<_> = (0..5).map(|doc_id| ids.get_val(doc_id)).collect();
            let mut sorted_ids = actual.clone();
            sorted_ids.sort_unstable();
            assert_eq!(sorted_ids, [0, 1, 2, 3, 4]);
            // The repeated leaf stays contiguous and in arrival order.
            assert!(actual.windows(2).any(|pair| pair == [0, 3]));
            let outer: Vec<_> = actual
                .iter()
                .map(|&id| {
                    let (status, service) = values[id as usize];
                    if fields[0] == "status" {
                        status
                    } else {
                        service
                    }
                })
                .collect();
            // Each first-level group must be contiguous, regardless of hash-map iteration order.
            assert_eq!(
                outer.windows(2).filter(|pair| pair[0] != pair[1]).count(),
                1
            );
            for (doc_id, original_id) in actual.into_iter().enumerate() {
                let doc_id = doc_id as u32;
                assert_eq!(fieldnorms.fieldnorm(doc_id), 2 + original_id as u32 % 2);
                let doc: TantivyDocument = searcher.doc(DocAddress::new(0, doc_id))?;
                assert_eq!(doc.get_first(id).unwrap().as_u64(), Some(original_id));
            }
            let phrase = PhraseQuery::new(vec![
                Term::from_field_text(message, "alpha"),
                Term::from_field_text(message, "beta"),
            ]);
            assert_eq!(searcher.search(&phrase, &Count)?, 3);
            let meta = &index.searchable_segment_metas()?[0];
            assert!(!index
                .directory()
                .exists(&meta.relative_path(SegmentComponent::TempStore))?);
        }
        Ok(())
    }

    #[test]
    fn clustering_uses_full_value_sequences() -> crate::Result<()> {
        let mut schema = Schema::builder();
        let field = schema.add_text_field("key", STRING | STORED);
        let id = schema.add_u64_field("id", FAST);
        let mut writer =
            writer(schema.build(), IndexSettings::default())?.with_clustering_fields(&["key"])?;
        // Missing, empty and differently delimited multi-values must remain distinct.
        let values: &[&[&str]] = &[
            &["ab", "c"],
            &[],
            &["a", "bc"],
            &[""],
            &["ab", "c"],
            &[],
            &["a", "bc"],
            &[""],
        ];
        for (i, values) in values.iter().enumerate() {
            let mut doc = doc!(id => i as u64);
            for value in *values {
                doc.add_text(field, value);
            }
            writer.add_document(doc)?;
        }
        let index = writer.finalize()?;
        let searcher = index.reader()?.searcher();
        let ids = searcher
            .segment_reader(0)
            .fast_fields()
            .u64("id")?
            .first_or_default_col(0);
        let actual: Vec<_> = (0..8).map(|doc_id| ids.get_val(doc_id)).collect();
        let mut groups = Vec::new();
        for pair in actual.chunks_exact(2) {
            assert_eq!(pair[1], pair[0] + 4);
            groups.push(pair[0]);
        }
        groups.sort_unstable();
        assert_eq!(groups, [0, 1, 2, 3]);
        Ok(())
    }

    #[test]
    fn clustering_empty_segment() -> crate::Result<()> {
        let mut schema = Schema::builder();
        schema.add_text_field("key", STRING);
        let index = writer(schema.build(), IndexSettings::default())?
            .with_clustering_fields(&["key"])?
            .finalize()?;
        assert_eq!(index.reader()?.searcher().num_docs(), 0);
        Ok(())
    }

    #[test]
    fn clustering_rejects_invalid_configuration() -> crate::Result<()> {
        let mut schema = Schema::builder();
        let field = schema.add_u64_field("key", FAST);
        let schema = schema.build();
        for fields in [&[][..], &["missing"][..]] {
            assert!(writer(schema.clone(), IndexSettings::default())?
                .with_clustering_fields(fields)
                .is_err());
        }
        for settings in [
            IndexSettings {
                manual_doc_id_mapping: true,
                ..Default::default()
            },
            IndexSettings {
                sort_by_field: Some(IndexSortByField {
                    field: "key".to_string(),
                    order: Order::Asc,
                }),
                ..Default::default()
            },
        ] {
            assert!(writer(schema.clone(), settings)?
                .with_clustering_fields(&["key"])
                .is_err());
        }
        let mut late = writer(schema.clone(), IndexSettings::default())?;
        late.add_document(doc!(field => 1u64))?;
        assert!(late.with_clustering_fields(&["key"]).is_err());
        let clustered =
            writer(schema, IndexSettings::default())?.with_clustering_fields(&["key"])?;
        let mapping = DocIdMapping::new_permutation(vec![])?;
        assert!(clustered.finalize_with_doc_id_mapping(&mapping).is_err());
        Ok(())
    }
}
