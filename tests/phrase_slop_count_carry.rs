use tantivy::collector::Count;
use tantivy::query::PhraseQuery;
use tantivy::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions};
use tantivy::{Index, TantivyDocument, Term};

/// 3-term phrase "a b c" against the document "a x b x c" with slop=1.
///
/// The total slop budget required to match is 2 (one slip between a and b,
/// one between b and c), so the phrase must NOT match. The same scenario
/// is already covered for the scoring path by `test_phrase_slop` in
/// src/query/phrase_query/mod.rs and correctly returns 0 hits there. The
/// non-scoring path (used by the Count collector, since it returns false
/// from `requires_scoring`) takes a different branch in PhraseScorer that
/// fails to carry the accumulated slop into the final intersection check
/// and incorrectly reports the document as a match.
#[test]
fn phrase_query_count_collector_three_terms_slop_must_carry()
-> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let text_indexing = TextFieldIndexing::default()
        .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        .set_tokenizer("default");
    let body = schema_builder
        .add_text_field("body", TextOptions::default().set_indexing_options(text_indexing));
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer_with_num_threads(1, 15_000_000)?;

    let mut doc = TantivyDocument::default();
    doc.add_text(body, "a x b x c");
    writer.add_document(doc)?;
    writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let mut phrase = PhraseQuery::new(vec![
        Term::from_field_text(body, "a"),
        Term::from_field_text(body, "b"),
        Term::from_field_text(body, "c"),
    ]);
    phrase.set_slop(1);

    let count = searcher.search(&phrase, &Count)?;
    assert_eq!(
        count, 0,
        "matching `a b c` over `a x b x c` requires total slop 2 > 1, \
         the Count collector must not return any hit"
    );
    Ok(())
}
