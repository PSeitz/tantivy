// Regression test: a `RangeQuery` over a bool value inside a JSON fast field is on the
// public surface (you can build a Bool term inside a JSON path with `Term::from_field_json_path`
// + `append_type_and_fast_value(true)`) but `FastFieldRangeWeight::scorer` rejects it with
// `InvalidArgument: unsupported value bytes type in json term value_bytes Bool`.

use std::collections::BTreeMap;
use std::ops::Bound;

use tantivy::collector::Count;
use tantivy::query::RangeQuery;
use tantivy::schema::{OwnedValue, Schema, FAST, STORED, TEXT};
use tantivy::{Index, IndexWriter, TantivyDocument, Term};

fn build_index() -> tantivy::Result<(Index, tantivy::schema::Field)> {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("json", TEXT | STORED | FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
    for v in [true, false, true] {
        let mut doc = TantivyDocument::default();
        let mut obj: BTreeMap<String, OwnedValue> = BTreeMap::new();
        obj.insert("flag".to_string(), OwnedValue::Bool(v));
        doc.add_object(json_field, obj);
        writer.add_document(doc)?;
    }
    writer.commit()?;
    Ok((index, json_field))
}

#[test]
fn range_query_bool_in_json_full_range() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let lower = {
        let mut t = Term::from_field_json_path(field, "flag", true);
        t.append_type_and_fast_value(false);
        t
    };
    let upper = {
        let mut t = Term::from_field_json_path(field, "flag", true);
        t.append_type_and_fast_value(true);
        t
    };
    let q = RangeQuery::new(Bound::Included(lower), Bound::Included(upper));
    let count = searcher.search(&q, &Count)?;
    assert_eq!(count, 3, "should match all three documents");
    Ok(())
}

#[test]
fn range_query_bool_in_json_only_true() -> tantivy::Result<()> {
    let (index, field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let mut t = Term::from_field_json_path(field, "flag", true);
    t.append_type_and_fast_value(true);
    let q = RangeQuery::new(Bound::Included(t.clone()), Bound::Included(t));
    let count = searcher.search(&q, &Count)?;
    assert_eq!(count, 2, "two docs have flag = true");
    Ok(())
}
