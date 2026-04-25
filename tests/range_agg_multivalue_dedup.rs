// Regression test: range and histogram aggregations should count each document once per
// bucket, not once per value occurrence. Term aggregation was fixed in #2854 by feeding
// the segment collector through `fetch_block_with_missing_unique_per_doc`. The range and
// histogram aggregations still call the non-deduplicated `fetch_block`, so a doc with
// the same value indexed multiple times in the same bucket inflates `doc_count`.

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
fn range_agg_should_dedup_multivalue_within_bucket() -> tantivy::Result<()> {
    let agg = r#"{
        "r": {
            "range": {
                "field": "v",
                "ranges": [{ "from": 0.0, "to": 100.0 }]
            }
        }
    }"#;
    let res = run_agg(agg)?;
    let doc_count = res["r"]["buckets"][0]["doc_count"].as_u64().unwrap();
    assert_eq!(
        doc_count, 1,
        "one doc indexed three times with v=10 should count once in the [0, 100) bucket; \
         got {doc_count}"
    );
    Ok(())
}

#[test]
fn histogram_agg_should_dedup_multivalue_within_bucket() -> tantivy::Result<()> {
    let agg = r#"{
        "h": {
            "histogram": {
                "field": "v",
                "interval": 100.0
            }
        }
    }"#;
    let res = run_agg(agg)?;
    let doc_count = res["h"]["buckets"][0]["doc_count"].as_u64().unwrap();
    assert_eq!(
        doc_count, 1,
        "one doc indexed three times with v=10 should count once in the histogram bucket; \
         got {doc_count}"
    );
    Ok(())
}
