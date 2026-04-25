// Regression test: `parse_into_milliseconds` (called from
// `DateHistogramAggregationReq::validate`) does `let val = number * unit_in_ms` without
// overflow check. A user-supplied `fixed_interval` like `"100000000000000d"` overflows
// `i64` in this multiplication. In a debug build (the default for `cargo test`) this
// panics with `attempt to multiply with overflow`. In release the value wraps and an
// arbitrary interval is accepted.

use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::{AggContextParams, AggregationCollector};
use tantivy::query::AllQuery;
use tantivy::schema::{Schema, FAST};
use tantivy::{Index, IndexWriter, TantivyDocument};

#[test]
fn date_histogram_huge_interval_returns_error_not_panic() -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let date_field = schema_builder.add_date_field("ts", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    {
        let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
        let mut doc = TantivyDocument::default();
        doc.add_date(date_field, common::DateTime::from_timestamp_secs(0));
        writer.add_document(doc)?;
        writer.commit()?;
    }

    let agg_json = r#"{
        "h": {
            "date_histogram": {
                "field": "ts",
                "fixed_interval": "100000000000000d"
            }
        }
    }"#;
    let aggs: Aggregations = serde_json::from_str(agg_json).unwrap();
    let collector = AggregationCollector::from_aggs(aggs, AggContextParams::default());

    let searcher = index.reader()?.searcher();
    // The misformed (overflowing) interval should turn into a normal `TantivyError` we
    // can surface to the caller — not an arithmetic panic that crashes the process.
    let res = searcher.search(&AllQuery, &collector);
    assert!(
        res.is_err(),
        "expected an error for an interval that overflows i64 milliseconds, got Ok"
    );
    Ok(())
}
