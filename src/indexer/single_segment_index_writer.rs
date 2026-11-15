use std::marker::PhantomData;

use crate::index::list_segment_files;
use crate::indexer::doc_clustering::DocGroups;
use crate::indexer::operation::AddOperation;
use crate::indexer::segment_updater::save_metas;
use crate::indexer::{ClusteringConfig, DocIdMapping, SegmentWriter};
use crate::schema::document::Document;
use crate::{Directory, Index, IndexMeta, Opstamp, Segment, TantivyDocument};

#[doc(hidden)]
pub struct SingleSegmentIndexWriter<D: Document = TantivyDocument> {
    segment_writer: SegmentWriter,
    segment: Segment,
    opstamp: Opstamp,
    doc_groups: Option<DocGroups>,
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
            doc_groups: None,
            _phantom: PhantomData,
        })
    }

    /// Retain documents in nested groups, then index them in group order at finalization.
    /// Documents are kept as [`TantivyDocument`]. Inputs of that type are moved without copying;
    /// other document types are converted once during ingestion.
    /// Must be configured before adding any documents. Indexing errors are deferred
    /// until finalization. Use [`Self::mem_usage`] to track retained memory and leave
    /// headroom for indexing. The caller decides when to finalize.
    /// An existing `sort_by_field` setting still applies after clustering.
    ///
    /// ```no_run
    /// # use tantivy::{ClusteringConfig, Index, TantivyDocument};
    /// # use tantivy::schema::{Schema, STRING, STORED};
    /// # use tantivy::directory::RamDirectory;
    /// # let mut schema = Schema::builder();
    /// # schema.add_text_field("status", STRING);
    /// # schema.add_json_field("json", STORED);
    /// # let writer = Index::builder().schema(schema.build())
    /// #     .single_segment_index_writer::<TantivyDocument>(RamDirectory::default(), 15_000_000)?;
    /// let writer = writer.with_clustering(ClusteringConfig {
    ///     fields: vec!["status".into(), "json.service".into()],
    /// })?;
    /// // Add documents and use mem_usage() to decide when to finalize.
    /// let index = writer.finalize()?;
    /// # Ok::<(), tantivy::TantivyError>(())
    /// ```
    pub fn with_clustering(mut self, config: ClusteringConfig) -> crate::Result<Self> {
        if self.opstamp != 0 {
            return Err(crate::TantivyError::InvalidArgument(
                "clustering must be configured before adding documents".to_owned(),
            ));
        }
        if self.segment.index().settings().manual_doc_id_mapping {
            return Err(crate::TantivyError::InvalidArgument(
                "clustering cannot be combined with manual doc ID mapping".to_owned(),
            ));
        }
        self.doc_groups = Some(DocGroups::new(config, &self.segment.schema())?);
        Ok(self)
    }

    /// Counts of leaf groups in traversal order, without exposing keys.
    /// Returns `None` when clustering is disabled. This allocates the result on demand.
    pub fn clustering_group_sizes(&self) -> Option<Vec<usize>> {
        self.doc_groups.as_ref().map(DocGroups::group_sizes)
    }

    /// Estimated retained memory, including buffered documents and grouping allocations.
    /// Leave headroom for indexing during finalization. This writer does not flush automatically.
    pub fn mem_usage(&self) -> usize {
        self.segment_writer.mem_usage() + self.doc_groups.as_ref().map_or(0, DocGroups::mem_usage)
    }

    pub fn add_document(&mut self, document: D) -> crate::Result<()> {
        let opstamp = self.opstamp;
        self.opstamp += 1;
        if let Some(groups) = &mut self.doc_groups {
            groups.push(document.into_tantivy_document());
            Ok(())
        } else {
            self.segment_writer
                .add_document(AddOperation { opstamp, document })
        }
    }

    pub fn finalize(self) -> crate::Result<Index> {
        let Self {
            segment,
            mut segment_writer,
            doc_groups,
            ..
        } = self;
        if let Some(groups) = doc_groups {
            groups.drain(&mut segment_writer)?;
        }
        let max_doc = segment_writer.max_doc();
        segment_writer.finalize()?;
        let did_remapping = segment.index().settings().sort_by_field.is_some();
        Self::finalize_inner(segment, max_doc, did_remapping, false)
    }

    pub fn finalize_with_doc_id_mapping(self, mapping: &DocIdMapping) -> crate::Result<Index> {
        if self.doc_groups.is_some() {
            return Err(crate::TantivyError::InvalidArgument(
                "a manual doc ID mapping cannot be applied to buffered clustering".to_owned(),
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
