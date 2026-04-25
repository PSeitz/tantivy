// Regression test: composite aggregations should count each document once per bucket key,
// not once per value occurrence. The composite collector iterates `values_for_doc(doc_id)`
// directly, so a doc with the same value indexed multiple times contributes multiple times
// to the same bucket.

use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::agg_result::AggregationResults;
use tantivy::aggregation::{AggContextParams, AggregationCollector};
use tantivy::query::AllQuery;
use tantivy::schema::{Schema, FAST};
use tantivy::{doc, Index, IndexWriter};

fn run_agg(agg_json: &str) -> tantivy::Result<serde_json::Value> {
    let mut schema_builder = Schema::builder();
    let v = schema_builder.add_u64_field("v", FAST);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    {
        let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
        // Single doc, multi-value column with the same value three times.
        writer.add_document(doc!(v => 10u64, v => 10u64, v => 10u64))?;
        writer.commit()?;
    }
    let aggs: Aggregations = serde_json::from_str(agg_json).unwrap();
    let collector = AggregationCollector::from_aggs(aggs, AggContextParams::default());
    let searcher = index.reader()?.searcher();
    let res: AggregationResults = searcher.search(&AllQuery, &collector)?;
    Ok(serde_json::from_str(&serde_json::to_string(&res)?)?)
}

#[test]
fn composite_terms_agg_should_dedup_multivalue() -> tantivy::Result<()> {
    let agg = r#"{
        "c": {
            "composite": {
                "size": 10,
                "sources": [
                    { "by_v": { "terms": { "field": "v" } } }
                ]
            }
        }
    }"#;
    let res = run_agg(agg)?;
    let buckets = res["c"]["buckets"].as_array().unwrap();
    assert_eq!(buckets.len(), 1, "expected exactly one bucket key v=10");
    let doc_count = buckets[0]["doc_count"].as_u64().unwrap();
    assert_eq!(
        doc_count, 1,
        "one doc indexed three times with v=10 should count once in the composite bucket; \
         got {doc_count}"
    );
    Ok(())
}

#[test]
fn composite_histogram_agg_should_dedup_multivalue() -> tantivy::Result<()> {
    let agg = r#"{
        "c": {
            "composite": {
                "size": 10,
                "sources": [
                    { "by_v": { "histogram": { "field": "v", "interval": 100.0 } } }
                ]
            }
        }
    }"#;
    let res = run_agg(agg)?;
    let buckets = res["c"]["buckets"].as_array().unwrap();
    assert_eq!(buckets.len(), 1, "expected exactly one bucket");
    let doc_count = buckets[0]["doc_count"].as_u64().unwrap();
    assert_eq!(
        doc_count, 1,
        "one doc indexed three times with v=10 should count once in the composite \
         histogram bucket; got {doc_count}"
    );
    Ok(())
}
