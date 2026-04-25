// Regression test: a `RangeQuery` over a boolean fast field is in the public surface
// (booleans are accepted by `is_type_valid_for_fastfield_range_query`) but the
// `FastFieldRangeWeight` scorer does not actually handle the bool term encoding.
// `value.as_u64()` / `as_i64()` / `as_f64()` / `as_date()` all return `None` for a bool
// term, so the scorer returns `InvalidArgument: Expected term with u64, i64, f64 or date`.

use std::ops::Bound;

use tantivy::collector::Count;
use tantivy::query::RangeQuery;
use tantivy::schema::{Schema, FAST};
use tantivy::{doc, Index, IndexWriter, Term};

fn build_index() -> tantivy::Result<(Index, tantivy::schema::Field)> {
    let mut schema_builder = Schema::builder();
    let bool_field = schema_builder.add_bool_field("bool", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
    writer.add_document(doc!(bool_field => true))?;
    writer.add_document(doc!(bool_field => false))?;
    writer.add_document(doc!(bool_field => true))?;
    writer.commit()?;
    Ok((index, bool_field))
}

#[test]
fn range_query_bool_inclusive_true() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_bool(field, true)),
        Bound::Included(Term::from_field_bool(field, true)),
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(count, 2, "should match the two `true` documents");
    Ok(())
}

#[test]
fn range_query_bool_inclusive_full_range() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_bool(field, false)),
        Bound::Included(Term::from_field_bool(field, true)),
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(count, 3, "should match all three documents");
    Ok(())
}

#[test]
fn range_query_bool_inclusive_false() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_bool(field, false)),
        Bound::Included(Term::from_field_bool(field, false)),
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(count, 1, "should match the one `false` document");
    Ok(())
}
