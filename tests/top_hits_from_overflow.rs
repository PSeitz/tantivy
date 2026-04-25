use serde_json::json;
use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::AggregationCollector;
use tantivy::query::AllQuery;
use tantivy::schema::{Schema, FAST};
use tantivy::{Index, TantivyDocument};

/// Run a top_hits aggregation with `from` larger than the number of
/// matching documents. The user passed a valid request (paging past the
/// end of the matched window), so the search must not panic. The
/// expected result is an empty `hits` list.
///
/// On main `TopHitsTopNComputer::into_final_result` does
/// `hits.drain(..self.req.from.unwrap_or(0))` without bounding the
/// drain by `hits.len()`. When fewer docs match than the requested
/// `from`, `drain` panics with "range end index N out of range for
/// slice of length M".
#[test]
fn top_hits_from_greater_than_matches_must_not_panic() -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let price = schema_builder.add_f64_field("price", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer_with_num_threads(1, 15_000_000)?;
    let mut doc = TantivyDocument::default();
    doc.add_f64(price, 1.0);
    writer.add_document(doc)?;
    writer.commit()?;

    let agg: Aggregations = serde_json::from_value(json!({
        "top_hits_req": {
            "top_hits": {
                "size": 2,
                "from": 10,
                "sort": [
                    { "price": "asc" }
                ],
                "docvalue_fields": ["price"]
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

    assert_eq!(
        res,
        json!({
            "top_hits_req": {
                "hits": []
            }
        }),
        "from=10 with 1 matched doc should produce an empty hits list, not panic"
    );

    Ok(())
}
