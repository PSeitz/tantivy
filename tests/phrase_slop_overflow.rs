use tantivy::collector::TopDocs;
use tantivy::query::PhraseQuery;
use tantivy::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions};
use tantivy::{DocAddress, Index, Score, TantivyDocument, Term};

#[test]
fn phrase_query_slop_above_255_must_not_match_when_real_slop_exceeds_max()
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

    let mut text = String::from("a");
    for _ in 0..256 {
        text.push_str(" z");
    }
    text.push_str(" b z c");

    let mut doc = TantivyDocument::default();
    doc.add_text(body, &text);
    writer.add_document(doc)?;
    writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let mut phrase = PhraseQuery::new(vec![
        Term::from_field_text(body, "a"),
        Term::from_field_text(body, "b"),
        Term::from_field_text(body, "c"),
    ]);
    phrase.set_slop(256);

    let hits: Vec<(Score, DocAddress)> =
        searcher.search(&phrase, &TopDocs::with_limit(10).order_by_score())?;
    assert_eq!(
        hits.len(),
        0,
        "real slop is 256 + 1 = 257 which is greater than max_slop 256, \
         so the phrase must not match"
    );
    Ok(())
}
