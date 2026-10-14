use std::sync::Arc;
use std::time::{Duration, Instant};

use tantivy::collector::Count;
use tantivy::directory::RamDirectory;
use tantivy::indexer::{IndexWriterOptions, NoMergePolicy, UserOperation};
use tantivy::query::TermQuery;
use tantivy::schema::{Field, IndexRecordOption, Schema, FAST, STORED, STRING};
use tantivy::{doc, ByteDeduplicator, Index, IndexWriter, SingleSegmentIndexWriter, Term};

/// Checks that postings and fast-field dictionaries keep each index's own document counts.
fn assert_index_values(index: &Index, text: Field, expected_count: usize) -> tantivy::Result<()> {
    let reader = index.reader()?;
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), expected_count as u64);
    let query = TermQuery::new(
        Term::from_field_text(text, "shared"),
        IndexRecordOption::Basic,
    );
    assert_eq!(searcher.search(&query, &Count)?, expected_count);
    for segment in searcher.segment_readers() {
        let column = segment.fast_fields().str("text")?.unwrap();
        let mut value = String::new();
        for doc_id in 0..segment.max_doc() {
            let ords: Vec<_> = column.term_ords(doc_id).collect();
            assert_eq!(ords.len(), 1);
            assert!(column.ord_to_str(ords[0], &mut value)?);
            assert_eq!(value, "shared");
        }
    }
    Ok(())
}

#[test]
fn single_segment_writers_share_bytes_and_release_them() -> tantivy::Result<()> {
    let byte_deduplicator = Arc::new(ByteDeduplicator::default());
    let mut schema = Schema::builder();
    let text = schema.add_text_field("text", STRING | FAST | STORED);
    let bytes = schema.add_bytes_field("bytes", FAST);
    let schema = schema.build();
    let text_bytes = byte_deduplicator.get_or_allocate(b"shared");
    let binary_bytes = byte_deduplicator.get_or_allocate(b"\xff\x00");
    // In-memory postings keys contain the field id followed by the term bytes.
    let term_bytes = byte_deduplicator
        .get_or_allocate(&[text.field_id().to_be_bytes().as_slice(), b"shared"].concat());
    let mut first: SingleSegmentIndexWriter = Index::builder()
        .schema(schema.clone())
        .single_segment_index_writer_with_byte_deduplicator(
            RamDirectory::create(),
            15_000_000,
            byte_deduplicator.clone(),
        )?;
    let mut second: SingleSegmentIndexWriter = Index::builder()
        .schema(schema)
        .single_segment_index_writer_with_byte_deduplicator(
            RamDirectory::create(),
            15_000_000,
            byte_deduplicator.clone(),
        )?;
    first.add_document(doc!(text => "shared", bytes => b"\xff\x00".as_slice()))?;
    second.add_document(doc!(text => "shared", bytes => b"\xff\x00".as_slice()))?;
    second.add_document(doc!(text => "shared"))?;
    for allocation in [&text_bytes, &binary_bytes, &term_bytes] {
        assert_eq!(Arc::strong_count(allocation), 3);
    }
    let first_index = first.finalize()?;
    for allocation in [&text_bytes, &binary_bytes, &term_bytes] {
        assert_eq!(Arc::strong_count(allocation), 2);
    }
    let second_index = second.finalize()?;
    for allocation in [&text_bytes, &binary_bytes, &term_bytes] {
        assert_eq!(Arc::strong_count(allocation), 1);
    }
    assert_index_values(&first_index, text, 1)?;
    assert_index_values(&second_index, text, 2)?;
    for index in [&first_index, &second_index] {
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let column = searcher
            .segment_reader(0)
            .fast_fields()
            .bytes("bytes")?
            .unwrap();
        assert_eq!(column.term_ords(0).collect::<Vec<_>>(), vec![0]);
        let mut value = Vec::new();
        assert!(column.ord_to_bytes(0, &mut value)?);
        assert_eq!(value, b"\xff\x00");
    }
    let released_text = Arc::downgrade(&text_bytes);
    drop(text_bytes);
    assert!(released_text.upgrade().is_none());
    Ok(())
}

/// Waits for background indexing workers to hold the expected number of key references.
fn wait_for_references(bytes: &Arc<Vec<u8>>, expected_count: usize) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Arc::strong_count(bytes) != expected_count {
        assert!(
            Instant::now() < deadline,
            "indexing workers did not retain the shared bytes"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn index_writers_share_bytes_across_commits_and_rollback() -> tantivy::Result<()> {
    let byte_deduplicator = Arc::new(ByteDeduplicator::default());
    let mut schema = Schema::builder();
    let text = schema.add_text_field("text", STRING | FAST);
    let schema = schema.build();
    let first_index = Index::create_in_ram(schema.clone());
    let second_index = Index::create_in_ram(schema.clone());
    let mut single: SingleSegmentIndexWriter = Index::builder()
        .schema(schema)
        .single_segment_index_writer_with_byte_deduplicator(
            RamDirectory::create(),
            15_000_000,
            byte_deduplicator.clone(),
        )?;
    single.add_document(doc!(text => "shared"))?;
    let options = IndexWriterOptions::builder()
        .num_worker_threads(2)
        .byte_deduplicator(byte_deduplicator.clone())
        .build();
    let mut first: IndexWriter = first_index.writer_with_options(options.clone())?;
    let mut second: IndexWriter = second_index.writer_with_options(options)?;
    first.set_merge_policy(Box::new(NoMergePolicy));
    second.set_merge_policy(Box::new(NoMergePolicy));
    let text_bytes = byte_deduplicator.get_or_allocate(b"shared");
    let term_bytes = byte_deduplicator
        .get_or_allocate(&[text.field_id().to_be_bytes().as_slice(), b"shared"].concat());
    first.add_document(doc!(text => "shared"))?;
    second.add_document(doc!(text => "shared"))?;
    wait_for_references(&text_bytes, 4);
    wait_for_references(&term_bytes, 4);
    first.commit()?;
    assert_eq!(Arc::strong_count(&text_bytes), 3);
    assert_eq!(Arc::strong_count(&term_bytes), 3);
    second.rollback()?;
    assert_eq!(Arc::strong_count(&text_bytes), 2);
    assert_eq!(Arc::strong_count(&term_bytes), 2);
    assert_index_values(&first_index, text, 1)?;
    assert_index_values(&second_index, text, 0)?;

    // Commit and rollback both replace workers; the replacement workers must retain the setting.
    first.add_document(doc!(text => "shared"))?;
    second.add_document(doc!(text => "shared"))?;
    wait_for_references(&text_bytes, 4);
    wait_for_references(&term_bytes, 4);
    first.commit()?;
    second.commit()?;
    assert_eq!(Arc::strong_count(&text_bytes), 2);
    assert_eq!(Arc::strong_count(&term_bytes), 2);
    let single_index = single.finalize()?;
    assert_index_values(&single_index, text, 1)?;
    assert_eq!(Arc::strong_count(&text_bytes), 1);
    assert_eq!(Arc::strong_count(&term_bytes), 1);
    assert_index_values(&first_index, text, 2)?;
    assert_index_values(&second_index, text, 1)?;
    first.wait_merging_threads()?;
    second.wait_merging_threads()?;
    Ok(())
}

#[test]
fn index_writer_shares_bytes_after_memory_triggered_flush() -> tantivy::Result<()> {
    let byte_deduplicator = Arc::new(ByteDeduplicator::default());
    let mut schema = Schema::builder();
    let text = schema.add_text_field("text", STRING | FAST);
    let payload = schema.add_bytes_field("payload", FAST);
    let index = Index::create_in_ram(schema.build());
    let options = IndexWriterOptions::builder()
        .memory_budget_per_thread(15_000_000)
        .byte_deduplicator(byte_deduplicator.clone())
        .build();
    let mut writer: IndexWriter = index.writer_with_options(options)?;
    writer.set_merge_policy(Box::new(NoMergePolicy));
    let text_bytes = byte_deduplicator.get_or_allocate(b"shared");
    writer.add_document(doc!(text => "shared"))?;
    wait_for_references(&text_bytes, 2);

    // One batch exceeds the budget. The worker checks memory after the entire batch,
    // so these documents cause one flush rather than leaving another segment half full.
    writer.run((0u32..1000).map(|i| {
        let mut bytes = vec![0; 16_384];
        bytes[..4].copy_from_slice(&i.to_le_bytes());
        UserOperation::Add(doc!(payload => bytes))
    }))?;
    wait_for_references(&text_bytes, 1);

    writer.add_document(doc!(text => "shared"))?;
    wait_for_references(&text_bytes, 2);
    writer.commit()?;
    assert_eq!(Arc::strong_count(&text_bytes), 1);
    let reader = index.reader()?;
    let searcher = reader.searcher();
    assert_eq!(searcher.segment_readers().len(), 2);
    assert_eq!(searcher.num_docs(), 1002);
    let query = TermQuery::new(
        Term::from_field_text(text, "shared"),
        IndexRecordOption::Basic,
    );
    assert_eq!(searcher.search(&query, &Count)?, 2);
    writer.wait_merging_threads()?;
    Ok(())
}
