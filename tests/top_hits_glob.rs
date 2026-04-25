use serde_json::json;
use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::AggregationCollector;
use tantivy::query::AllQuery;
use tantivy::schema::{Schema, FAST};
use tantivy::{Index, TantivyDocument};

/// `top_hits` accepts shell-style globs in `docvalue_fields`. The glob
/// `name*` should match the field literally named `name` (zero
/// characters after the prefix), as well as `name.first`, `name1`,
/// etc.
///
/// On main `globbed_string_to_regex` calls `.replace('*', ".*")` a
/// second time *after* `regex::escape` and the first
/// `\*` → `.*` substitution, which mutates the just-inserted `.*`
/// into `..*`. The compiled regex therefore requires at least one
/// character after the literal prefix, so the field `name` is not
/// matched and `validate_and_resolve_field_names` panics on the
/// internal `assert!(!fields.is_empty(), ...)`.
#[test]
fn top_hits_docvalue_fields_glob_matches_zero_char_suffix() -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let name = schema_builder.add_f64_field("name", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer_with_num_threads(1, 15_000_000)?;
    let mut doc = TantivyDocument::default();
    doc.add_f64(name, 42.0);
    writer.add_document(doc)?;
    writer.commit()?;

    let agg: Aggregations = serde_json::from_value(json!({
        "top_hits_req": {
            "top_hits": {
                "size": 1,
                "sort": [
                    { "name": "asc" }
                ],
                "docvalue_fields": ["name*"]
            }
        }
    }))
    .unwrap();

    let collector = AggregationCollector::from_aggs(agg, Default::default());

    let reader = index.reader()?;
    let searcher = reader.searcher();
    let agg_res = searcher.search(&AllQuery, &collector)?;

    let res: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&agg_res)?).unwrap();

    let hits = res["top_hits_req"]["hits"]
        .as_array()
        .expect("hits should be an array");
    assert_eq!(hits.len(), 1, "single document should produce a single hit");
    let docvalue_fields = &hits[0]["docvalue_fields"];
    assert_eq!(
        docvalue_fields["name"],
        json!([42.0]),
        "glob `name*` must match field `name`"
    );
    Ok(())
}
