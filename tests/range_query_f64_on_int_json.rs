// Regression test: a `RangeQuery` on a JSON numerical field where the user supplies an
// `f64` bound while the underlying column is integer (i64/u64) must not include integer
// values that fall on the wrong side of a non-integer bound.
//
// See `transform_from_f64_bounds` in `range_query_fastfield.rs`.

use std::collections::BTreeMap;
use std::ops::Bound;

use tantivy::collector::Count;
use tantivy::query::RangeQuery;
use tantivy::schema::{OwnedValue, Schema, FAST, STORED, TEXT};
use tantivy::{Index, IndexWriter, TantivyDocument, Term};

fn json_obj(entries: &[(&str, OwnedValue)]) -> BTreeMap<String, OwnedValue> {
    entries
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect()
}

fn build_index() -> tantivy::Result<(Index, tantivy::schema::Field)> {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("json", TEXT | STORED | FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;

    for v in [1_i64, 5, 10, -1, -5, -10] {
        let mut doc = TantivyDocument::default();
        doc.add_object(json_field, json_obj(&[("num", OwnedValue::I64(v))]));
        writer.add_document(doc)?;
    }
    writer.commit()?;
    Ok((index, json_field))
}

fn count_range(
    index: &Index,
    field: tantivy::schema::Field,
    lower: Bound<f64>,
    upper: Bound<f64>,
) -> tantivy::Result<usize> {
    fn bound_term(field: tantivy::schema::Field, b: Bound<f64>) -> Bound<Term> {
        match b {
            Bound::Included(v) => {
                let mut t = Term::from_field_json_path(field, "num", true);
                t.append_type_and_fast_value(v);
                Bound::Included(t)
            }
            Bound::Excluded(v) => {
                let mut t = Term::from_field_json_path(field, "num", true);
                t.append_type_and_fast_value(v);
                Bound::Excluded(t)
            }
            Bound::Unbounded => Bound::Unbounded,
        }
    }
    let q = RangeQuery::new(bound_term(field, lower), bound_term(field, upper));
    index.reader()?.searcher().search(&q, &Count)
}

#[test]
fn range_f64_lower_bound_with_fract_on_int_column() -> tantivy::Result<()> {
    let (index, field) = build_index()?;

    // `>= 5.5` over integer values [-10, -5, -1, 1, 5, 10] should match only 10.
    // The buggy path truncates 5.5 to 5 and turns the bound into `Included(5)`,
    // which incorrectly also matches the indexed value 5.
    let count = count_range(&index, field, Bound::Included(5.5), Bound::Unbounded)?;
    assert_eq!(
        count, 1,
        "Bound::Included(5.5) lower bound must not match the integer 5"
    );

    // `> 5.5` should likewise match only 10.
    let count = count_range(&index, field, Bound::Excluded(5.5), Bound::Unbounded)?;
    assert_eq!(
        count, 1,
        "Bound::Excluded(5.5) lower bound must not match the integer 5"
    );

    Ok(())
}

#[test]
fn range_f64_upper_bound_with_fract_on_negative_int_column() -> tantivy::Result<()> {
    let (index, field) = build_index()?;

    // `<= -5.5` over integer values [-10, -5, -1, 1, 5, 10] should match only -10.
    // The buggy path truncates -5.5 to -5 (round-toward-zero) and turns the bound into
    // `Included(-5)`, which incorrectly also matches the indexed value -5.
    let count = count_range(&index, field, Bound::Unbounded, Bound::Included(-5.5))?;
    assert_eq!(
        count, 1,
        "Bound::Included(-5.5) upper bound must not match the integer -5"
    );

    // `< -5.5` should likewise match only -10.
    let count = count_range(&index, field, Bound::Unbounded, Bound::Excluded(-5.5))?;
    assert_eq!(
        count, 1,
        "Bound::Excluded(-5.5) upper bound must not match the integer -5"
    );

    Ok(())
}

// Sanity check that the existing in-bounds cases still work the way users expect — these
// also pass on `main`, but they guard against an over-zealous fix.
#[test]
fn range_f64_lower_bound_with_fract_existing_behaviour() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    // `>= 4.5` should match 5 and 10.
    let count = count_range(&index, field, Bound::Included(4.5), Bound::Unbounded)?;
    assert_eq!(count, 2);
    // `>= -5.5` should match -5, -1, 1, 5, 10.
    let count = count_range(&index, field, Bound::Included(-5.5), Bound::Unbounded)?;
    assert_eq!(count, 5);
    Ok(())
}

#[test]
fn range_f64_upper_bound_with_fract_existing_behaviour() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    // `<= 4.5` should match -10, -5, -1, 1.
    let count = count_range(&index, field, Bound::Unbounded, Bound::Included(4.5))?;
    assert_eq!(count, 4);
    // `<= -4.5` should match -5, -10.
    let count = count_range(&index, field, Bound::Unbounded, Bound::Included(-4.5))?;
    assert_eq!(count, 2);
    Ok(())
}
