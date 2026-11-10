use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use proptest::prelude::*;

use super::*;
use crate::collector::{Count, DocSetCollector};
use crate::directory::RamDirectory;
use crate::index::SegmentComponent;
use crate::query::{PhraseQuery, TermQuery};
use crate::schema::{IndexRecordOption, FAST, INDEXED, STORED, STRING, TEXT};
use crate::{Directory, DocAddress, Index, SingleSegmentIndexWriter, TantivyDocument, Term};

fn config(fields: &[&str]) -> ClusteringConfig {
    ClusteringConfig {
        fields: fields.iter().map(|name| (*name).to_owned()).collect(),
        compression: CompressionConfig::default(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn clustering_matches_reference_order(
        rows in prop::collection::vec((prop::option::of(0u8..3), 0u8..3, 0u8..11, any::<bool>()), 1..40),
    ) {
        let mut schema = Schema::builder();
        let id = schema.add_u64_field("id", INDEXED | FAST | STORED);
        let status = schema.add_text_field("status", STRING | STORED);
        let json = schema.add_json_field("json", TEXT | STORED);
        let body = schema.add_text_field("body", TEXT | STORED);
        let schema = schema.build();
        let mut clustering = config(&["status", r"json.labels.service\.name"]);
        clustering.compression.message_fields.push("body".into());
        let mut groups = DocIdGroups::new(&schema, &clustering.fields).unwrap();
        let mut writer = Index::builder().schema(schema.clone())
            .single_segment_index_writer(RamDirectory::default(), 15_000_000).unwrap()
            .with_clustering(clustering).unwrap();
        let mut remap = Index::builder().schema(schema)
            .settings(crate::IndexSettings { manual_doc_id_mapping: true, ..Default::default() })
            .single_segment_index_writer(RamDirectory::default(), 15_000_000).unwrap();
        let mut documents = Vec::new();
        let mut keys = Vec::new();
        for (ordinal, &(parent, child, kind, array)) in rows.iter().enumerate() {
            let text = ["", "api", "日本語"][child as usize];
            let value = match kind {
                0 => serde_json::json!(text),
                1 => serde_json::json!([text, "other"]),
                2 => serde_json::json!([]),
                3 => serde_json::Value::Null,
                4 => serde_json::json!(-1),
                5 => serde_json::json!(1),
                6 => serde_json::json!(1.5),
                7 => serde_json::json!(true),
                8 => serde_json::json!({"ignored": text}),
                9 => serde_json::json!([{"ignored": "other"}, text]),
                _ => serde_json::Value::Null,
            };
            let mut labels = if kind == 10 { serde_json::json!({}) }
                else { serde_json::json!({"service.name": value}) };
            if array {
                labels = serde_json::json!([labels, {"service.name": "other"}]);
            }
            let mut document = doc!(id => ordinal as u64,
                json => serde_json::json!({"labels": labels}), body => "hello grouped world");
            if let Some(parent) = parent {
                document.add_text(status, ["", "api", "日本語"][parent as usize]);
                document.add_text(status, "other"); // Only the first repeated value groups.
            }
            // Derive expected keys from the generated case, independently of path traversal.
            keys.push((parent, (kind < 2).then_some(child)));
            // Supplied IDs run backwards, so sorting by ID cannot mimic arrival order.
            groups.push(&document, (rows.len() - 1 - ordinal) as DocId).unwrap();
            writer.add_document(document.clone()).unwrap();
            remap.add_document(document.clone()).unwrap();
            documents.push(document);
        }
        // A flat reference: rank every prefix by population, then its first arrival.
        // No production key extraction, grouping tree, or traversal is reused.
        let mut order: Vec<_> = (0..rows.len()).collect();
        order.sort_by_key(|&i| {
            let parent: Vec<_> = (0..keys.len()).filter(|&j| keys[j].0 == keys[i].0).collect();
            let leaf: Vec<_> = (0..keys.len()).filter(|&j| keys[j] == keys[i]).collect();
            (Reverse(parent.len()), parent[0], Reverse(leaf.len()), leaf[0], i)
        });
        let sizes: Vec<_> = order.chunk_by(|&a, &b| keys[a] == keys[b]).map(|group| group.len()).collect();
        prop_assert_eq!(groups.group_sizes(), sizes.clone());
        prop_assert_eq!(writer.clustering_group_sizes(), Some(sizes));
        let expected: Vec<_> = order.iter().map(|&i| i as DocId).collect();
        let supplied_ids: Vec<_> = order.iter().map(|&i| (rows.len() - 1 - i) as DocId).collect();
        prop_assert_eq!(groups.into_order(), supplied_ids);
        let mapping = crate::indexer::DocIdMapping::new_permutation(expected).unwrap();
        let remapped = remap.finalize_with_doc_id_mapping(&mapping).unwrap();
        let remapped = remapped.reader().unwrap().searcher();
        let index = writer.finalize().unwrap();
        let searcher = index.reader().unwrap().searcher();
        prop_assert_eq!(searcher.num_docs(), rows.len() as u64);
        let reader = searcher.segment_reader(0);
        let ids = reader.fast_fields().u64("id").unwrap();
        let remapped_ids = remapped.segment_reader(0).fast_fields().u64("id").unwrap();
        for (new_id, original_id) in order.into_iter().enumerate() {
            let address = DocAddress::new(0, new_id as DocId);
            let stored: TantivyDocument = searcher.doc(address).unwrap();
            let manual: TantivyDocument = remapped.doc(address).unwrap();
            prop_assert_eq!(&stored, &documents[original_id]);
            prop_assert_eq!(stored, manual);
            prop_assert_eq!(ids.first(new_id as DocId), Some(original_id as u64));
            prop_assert_eq!(remapped_ids.first(new_id as DocId), Some(original_id as u64));
            let query = TermQuery::new(Term::from_field_u64(id, original_id as u64), IndexRecordOption::Basic);
            prop_assert_eq!(searcher.search(&query, &DocSetCollector).unwrap(), [address].into());
            prop_assert_eq!(reader.get_fieldnorms_reader(body).unwrap().fieldnorm(new_id as DocId), 3);
        }
        let phrase = PhraseQuery::new(vec![Term::from_field_text(body, "grouped"), Term::from_field_text(body, "world")]);
        prop_assert_eq!(searcher.search(&phrase, &Count).unwrap(), rows.len());
        let metas = index.searchable_segment_metas().unwrap();
        prop_assert!(!index.directory().exists(&metas[0].relative_path(SegmentComponent::TempStore)).unwrap());
    }
}

#[test]
fn clustering_indexes_keys_after_admission_stops() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let id = schema.add_u64_field("id", INDEXED | FAST | STORED);
    let json = schema.add_json_field("json", TEXT | STORED);
    let message = schema.add_text_field("message", TEXT | STORED);
    let mut options = config(&["json.service"]);
    options.compression = CompressionConfig {
        max_fields_per_path: 3,
        message_fields: vec!["message".into()],
    };
    let mut writer = Index::builder()
        .schema(schema.build())
        .single_segment_index_writer(RamDirectory::default(), 15_000_000)?
        .with_clustering(options)?;
    let document = |i: u64| {
        doc!(
            id => i,
            json => serde_json::json!({
                "service": if i.is_multiple_of(2) { "api" } else { "worker" },
                "customer": "acme corp",
                "dynamic": {"keep": "common", "other": "user value", format!("id_{i}"): "needle"}
            }),
            message => "request for customer acme corp succeeded"
        )
    };
    for i in 0..10_001 {
        writer.add_document(document(i))?;
    }
    let index = writer.finalize()?;
    let searcher = index.reader()?.searcher();
    assert_eq!(searcher.num_docs(), 10_001);
    let ids = searcher.segment_reader(0).fast_fields().u64("id")?;
    // The service key is stored inline and still determines cluster order.
    for doc_id in 0..5001 {
        assert_eq!(ids.first(doc_id).unwrap() % 2, 0);
    }
    let parser = crate::query::QueryParser::for_index(&index, vec![json]);
    for i in [0, 9999, 10_000] {
        let query = parser.parse_query(&format!("json.dynamic.id_{i}:needle"))?;
        let addresses = searcher.search(&query, &DocSetCollector)?;
        assert_eq!(addresses.len(), 1);
        let stored: TantivyDocument = searcher.doc(*addresses.iter().next().unwrap())?;
        assert_eq!(stored, document(i));
    }
    let phrase = PhraseQuery::new(vec![
        Term::from_field_text(message, "acme"),
        Term::from_field_text(message, "corp"),
    ]);
    assert_eq!(searcher.search(&phrase, &Count)?, 10_001);
    Ok(())
}

#[test]
fn clustering_keeps_document_addresses_unchanged() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let service = schema.add_text_field("service", STRING);
    let json = schema.add_json_field("json", STORED);
    let mut options = config(&["service"]);
    options.compression.max_fields_per_path = 2;
    let mut groups = DocGroups::new(options, &schema.build())?;
    groups.push(doc!(service => "api", json => serde_json::json!({"first": "original"})))?;
    let original = groups.root.children["api"].documents[0];
    for _ in 1..10_001 {
        groups.push(doc!(service => "api", json => serde_json::json!({"later": "frequent"})))?;
    }
    let documents = &groups.root.children["api"].documents;
    assert_eq!(documents.len(), 10_001);
    assert_eq!(documents[0], original);
    Ok(())
}

#[test]
fn clustering_borrows_string_keys() {
    let mut groups = GroupNode::<()>::default();
    for text in ["", "api", "日本語"] {
        let value = crate::schema::OwnedValue::Str(text.into());
        for _ in 0..2 {
            let key = GroupKey::from_value(&value, &[]);
            assert!(matches!(key, GroupKey::Str(Cow::Borrowed(_))));
            groups.push(std::iter::once(key), ());
        }
        assert_eq!(groups.children[text].num_docs, 2);
    }
    groups.push(std::iter::once(GroupKey::Ignored), ());
    assert_eq!(groups.group_sizes(), vec![2, 2, 2, 1]);
}

#[test]
fn clustering_estimate_reserves_expanded_input() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let service = schema.add_text_field("service", STRING);
    let message = schema.add_text_field("message", TEXT);
    let mut groups = DocGroups::new(config(&["service"]), &schema.build())?;
    let empty_retained =
        groups.mem_usage + groups.compressor.mem_usage() - size_of::<DocumentCompressor>();
    assert_eq!(
        groups.mem_usage(),
        empty_retained * 2 + groups.compressor.indexing_headroom_bytes()
    );
    let text = "repeated text ".repeat(4_000);
    for _ in 0..100 {
        groups.push(doc!(service => "api", message => text.as_str()))?;
    }
    let retained =
        groups.mem_usage + groups.compressor.mem_usage() - size_of::<DocumentCompressor>();
    let expanded = groups.compressor.expanded_document_bytes();
    assert!(expanded > retained * 2);
    assert_eq!(
        groups.mem_usage(),
        retained + expanded + groups.compressor.indexing_headroom_bytes()
    );
    assert!(groups.mem_usage() >= 100 * text.len());
    Ok(())
}

#[test]
fn clustering_compression_releases_unused_document_capacity() -> crate::Result<()> {
    let mut schema = Schema::builder();
    schema.add_text_field("service", STRING);
    let mut groups = DocGroups::new(config(&["service"]), &schema.build())?;
    let before = groups.mem_usage();
    let document = TantivyDocument::with_capacity(1_000_000);
    assert!(document.mem_usage() >= 1_000_000);
    groups.push(document)?;
    assert!(groups.mem_usage() > before);
    assert!(groups.mem_usage() - before < 1_000_000);
    Ok(())
}

#[test]
fn clustering_validates_configuration_and_handles_empty_segments() -> crate::Result<()> {
    let mut schema = Schema::builder();
    schema.add_text_field("status", STRING);
    let schema = schema.build();
    for fields in [&[][..], &["unknown"][..], &["status.child"][..]] {
        assert!(DocGroups::new(config(fields), &schema).is_err());
        assert!(DocIdGroups::new(&schema, &config(fields).fields).is_err());
    }
    let empty = DocIdGroups::new(&schema, &config(&["status"]).fields)?;
    assert!(empty.group_sizes().is_empty());
    assert!(empty.into_order().is_empty());
    let writer: SingleSegmentIndexWriter = Index::builder()
        .schema(schema.clone())
        .single_segment_index_writer(RamDirectory::default(), 15_000_000)?
        .with_clustering(config(&["status"]))?;
    assert_eq!(writer.finalize()?.reader()?.searcher().num_docs(), 0);
    let mut writer: SingleSegmentIndexWriter = Index::builder()
        .schema(schema)
        .single_segment_index_writer(RamDirectory::default(), 15_000_000)?;
    writer.add_document(doc!())?;
    assert!(writer.with_clustering(config(&["status"])).is_err());
    Ok(())
}

#[test]
fn clustering_rejects_manual_mapping_before_buffering() -> crate::Result<()> {
    let mut schema = Schema::builder();
    schema.add_text_field("status", STRING);
    let writer: SingleSegmentIndexWriter = Index::builder()
        .schema(schema.build())
        .settings(crate::IndexSettings {
            manual_doc_id_mapping: true,
            ..Default::default()
        })
        .single_segment_index_writer(RamDirectory::default(), 15_000_000)?;
    assert!(matches!(
        writer.with_clustering(config(&["status"])),
        Err(TantivyError::InvalidArgument(_))
    ));
    Ok(())
}

#[test]
fn clustering_config_defaults_and_rejects_removed_options() -> crate::Result<()> {
    let options: ClusteringConfig = serde_json::from_str(r#"{"fields":["group"]}"#)?;
    assert_eq!(options, config(&["group"]));
    for json in [
        r#"{"fields":["group"],"sort_group_by":"timestamp"}"#,
        r#"{"fields":["group"],"min_group_size":6}"#,
    ] {
        assert!(serde_json::from_str::<ClusteringConfig>(json).is_err());
    }
    Ok(())
}

struct TrackedDocument {
    document: TantivyDocument,
    dropped: Arc<AtomicUsize>,
}

impl Document for TrackedDocument {
    type Value<'a> = <TantivyDocument as Document>::Value<'a>;
    type FieldsValuesIter<'a> = <TantivyDocument as Document>::FieldsValuesIter<'a>;

    fn iter_fields_and_values(&self) -> Self::FieldsValuesIter<'_> {
        self.document.iter_fields_and_values()
    }
}

impl Drop for TrackedDocument {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn clustering_compresses_and_releases_caller_documents_during_ingestion() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let service = schema.add_text_field("service", STRING | STORED);
    let mut writer = Index::builder()
        .schema(schema.build())
        .single_segment_index_writer(RamDirectory::default(), 15_000_000)?
        .with_clustering(config(&["service"]))?;
    let dropped = Arc::new(AtomicUsize::new(0));
    for ordinal in 0..10 {
        writer.add_document(TrackedDocument {
            document: doc!(service => "api"),
            dropped: dropped.clone(),
        })?;
        assert_eq!(dropped.load(Ordering::Relaxed), ordinal + 1);
    }
    let index = writer.finalize()?;
    assert_eq!(dropped.load(Ordering::Relaxed), 10);
    assert_eq!(index.reader()?.searcher().num_docs(), 10);
    Ok(())
}
