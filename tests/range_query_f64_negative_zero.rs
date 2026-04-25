// Regression test: `f64_to_u64` (used by Tantivy's `MonotonicallyMappableToU64` for `f64`)
// maps `-0.0` and `0.0` to two *different* u64 values:
//   `f64_to_u64( 0.0) = 0x8000_0000_0000_0000`
//   `f64_to_u64(-0.0) = 0x7fff_ffff_ffff_ffff`
//
// Since IEEE-754 `0.0 == -0.0`, every well-behaved comparison in user code treats them as
// the same value. But the index does not, so a doc indexed with `-0.0`:
//   - never matches a `RangeQuery` whose `Bound::Included(0.0)`/`Bound::Included(0.0)`
//     range conceptually contains it.
//   - is sorted *before* docs indexed with `0.0` even though they should tie.
//
// This is observable through the public f64 fast-field and term APIs.

use std::ops::Bound;

use tantivy::collector::{Count, TopDocs};
use tantivy::query::{AllQuery, RangeQuery};
use tantivy::schema::{Schema, FAST};
use tantivy::{doc, Index, IndexWriter, Order, Term};

fn build_index() -> tantivy::Result<(Index, tantivy::schema::Field)> {
    let mut schema_builder = Schema::builder();
    let f64_field = schema_builder.add_f64_field("v", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
    writer.add_document(doc!(f64_field => -0.0_f64))?;
    writer.add_document(doc!(f64_field => 0.0_f64))?;
    writer.commit()?;
    Ok((index, f64_field))
}

#[test]
fn range_query_zero_inclusive_matches_negative_zero() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_f64(field, 0.0)),
        Bound::Included(Term::from_field_f64(field, 0.0)),
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(
        count, 2,
        "RangeQuery [0.0..=0.0] should match both 0.0 and -0.0 (they are equal)"
    );
    Ok(())
}

#[test]
fn range_query_geq_zero_matches_negative_zero() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_f64(field, 0.0)),
        Bound::Unbounded,
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(
        count, 2,
        "RangeQuery [>=0.0] should match -0.0 because -0.0 == 0.0"
    );
    Ok(())
}

#[test]
fn sort_by_f64_treats_zero_and_negative_zero_as_equal() -> tantivy::Result<()> {
    let (index, _field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let collector = TopDocs::with_limit(2).order_by_fast_field::<f64>("v", Order::Asc);
    let docs: Vec<(Option<f64>, _)> = searcher.search(&AllQuery, &collector)?;
    let values: Vec<f64> = docs.iter().filter_map(|(v, _)| *v).collect();
    // Both values are returned; they should sort as equals (both `0.0` to f64 PartialOrd).
    assert_eq!(values.len(), 2);
    for v in &values {
        assert_eq!(*v, 0.0, "expected 0.0 (and -0.0 == 0.0), got {v}");
    }
    Ok(())
}
