//! Deduplicates aggregations into a shared execution plan and restores the requested result tree.

use rustc_hash::FxHashMap;

use super::agg_req::{Aggregation, AggregationVariants, Aggregations};
use super::bucket::{
    get_agg_name_and_property, Order, OrderTarget, TermsAggregation, TermsAggregationInternal,
};
use super::intermediate_agg_result::{
    IntermediateAggregationResult, IntermediateAggregationResults, IntermediateBucketResult,
    IntermediateMetricResult,
};
use super::metric::{
    IntermediateAverage, IntermediateCount, IntermediateMax, IntermediateMin, IntermediateSum,
    StatsAggregation,
};
use crate::TantivyError;

/// A physical aggregation request and the information needed to restore its logical result tree.
pub(crate) struct AggregationExecutionPlan {
    physical_aggs: Aggregations,
    rewrite: ScopeRewrite,
}

impl AggregationExecutionPlan {
    pub(crate) fn analyze(aggs: &Aggregations) -> Self {
        let (physical_aggs, rewrite) = ScopeRewrite::analyze(aggs);
        Self {
            physical_aggs,
            rewrite,
        }
    }

    pub(crate) fn physical_aggs(&self) -> &Aggregations {
        &self.physical_aggs
    }

    pub(crate) fn materialize_logical_results(
        &self,
        results: &mut IntermediateAggregationResults,
    ) -> crate::Result<()> {
        self.rewrite.apply(results)
    }
}

#[derive(Default)]
struct ScopeRewrite {
    children: FxHashMap<String, ScopeRewrite>,
    derivations: Vec<Derivation>,
}

enum Derivation {
    Duplicate {
        producer: String,
        output: String,
    },
    PlainTerms {
        producer: String,
        request: TermsAggregationInternal,
        outputs: Vec<(String, TermsAggregationInternal)>,
    },
    Stats {
        producer: String,
        outputs: Vec<(String, StatsOutput)>,
    },
}

#[derive(Clone, Copy)]
enum StatsOutput {
    Average,
    Count,
    Max,
    Min,
    Sum,
}

impl ScopeRewrite {
    fn analyze(aggs: &Aggregations) -> (Aggregations, Self) {
        let mut execution_aggs = aggs.clone();
        let mut rewrite = ScopeRewrite::default();
        let mut names: Vec<&String> = aggs.keys().collect();
        names.sort();

        // Analyze every child scope before removing redundant siblings at this scope.
        for name in &names {
            let agg = &aggs[*name];
            let (physical_children, child_rewrite) = Self::analyze(&agg.sub_aggregation);
            let physical_agg = execution_aggs.get_mut(*name).expect("aggregation cloned above");
            physical_agg.sub_aggregation = physical_children;
            if let AggregationVariants::Terms(terms) = &mut physical_agg.agg {
                if let Some(order) = terms.order.as_mut() {
                    if let OrderTarget::SubAggregation(target) = &mut order.target {
                        if let Some(physical_target) = child_rewrite.physical_order_target(target) {
                            *target = physical_target;
                        }
                    }
                }
            }
            if !child_rewrite.is_empty() {
                rewrite.children.insert((*name).clone(), child_rewrite);
            }
        }

        // Compare logical requests, including their complete child trees. Restore aliases after
        // the other derivations so they copy logical, rather than physical, results.
        let mut duplicates = Vec::new();
        for (index, name) in names.iter().enumerate() {
            if let Some(producer) = names[..index].iter().find(|candidate| {
                execution_aggs.contains_key(**candidate) && aggs[**candidate] == aggs[*name]
            }) {
                execution_aggs.remove(*name);
                rewrite.children.remove(*name);
                duplicates.push(Derivation::Duplicate {
                    producer: (*producer).clone(),
                    output: (*name).clone(),
                });
            }
        }
        names.retain(|name| execution_aggs.contains_key(*name));

        rewrite.derivations = share_terms_histograms(&mut execution_aggs, &names);

        // Single-value stats metrics use the same segment state. Replace compatible sibling
        // groups with one canonical stats aggregation and convert that state back to the requested
        // result types.
        for name in &names {
            if !execution_aggs.contains_key(*name) {
                continue;
            }
            let Some((field, missing, _)) = stats_metric(&aggs[*name]) else {
                continue;
            };
            let outputs: Vec<_> = names
                .iter()
                .filter_map(|candidate| {
                    let (candidate_field, candidate_missing, output) =
                        stats_metric(&aggs[*candidate])?;
                    (candidate_field == field && candidate_missing == missing)
                        .then(|| ((*candidate).clone(), output))
                })
                .collect();
            if outputs.len() < 2 {
                continue;
            }

            let producer = (*name).clone();
            let producer_agg = execution_aggs.get_mut(&producer).expect("group member exists");
            producer_agg.agg = AggregationVariants::Stats(StatsAggregation {
                field: field.to_string(),
                missing,
            });
            for (output, _) in &outputs {
                if output != &producer {
                    execution_aggs.remove(output);
                }
            }
            rewrite.derivations.push(Derivation::Stats { producer, outputs });
        }

        rewrite.derivations.extend(duplicates);
        (execution_aggs, rewrite)
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty() && self.derivations.is_empty()
    }

    fn physical_order_target(&self, logical_target: &str) -> Option<String> {
        // Resolve aliases before translating single-value metrics to stats properties.
        let (name, property) = get_agg_name_and_property(logical_target);
        for derivation in &self.derivations {
            if let Derivation::Duplicate { producer, output } = derivation {
                if name == output {
                    let target = if property.is_empty() {
                        producer.clone()
                    } else {
                        format!("{producer}.{property}")
                    };
                    return Some(self.physical_order_target(&target).unwrap_or(target));
                }
            }
        }
        self.derivations.iter().find_map(|derivation| {
            let Derivation::Stats { producer, outputs } = derivation else {
                return None;
            };
            outputs.iter().find_map(|(name, output)| {
                (name == logical_target).then(|| {
                    let property = match output {
                        StatsOutput::Average => "avg",
                        StatsOutput::Count => "count",
                        StatsOutput::Max => "max",
                        StatsOutput::Min => "min",
                        StatsOutput::Sum => "sum",
                    };
                    format!("{producer}.{property}")
                })
            })
        })
    }

    fn apply(&self, results: &mut IntermediateAggregationResults) -> crate::Result<()> {
        for (name, child_rewrite) in &self.children {
            if let Some(result) = results.aggs_res.get_mut(name) {
                apply_to_bucket_children(result, child_rewrite)?;
            }
        }

        for derivation in &self.derivations {
            match derivation {
                Derivation::Duplicate { producer, output } => {
                    let result = results.aggs_res.get(producer).cloned().ok_or_else(|| {
                        TantivyError::InternalError(format!(
                            "missing aggregation result {producer:?} for duplicate {output:?}"
                        ))
                    })?;
                    results.aggs_res.insert(output.clone(), result);
                }
                Derivation::PlainTerms { producer, request, outputs } => {
                    let result = results.aggs_res.get_mut(producer).ok_or_else(|| {
                        TantivyError::InternalError(format!(
                            "missing physical aggregation result {producer:?}"
                        ))
                    })?;
                    // Preserve the wider candidate set for every output before truncating.
                    let mut plain = result.clone();
                    trim_term_result(result, request)?;
                    strip_term_sub_aggregations(&mut plain)?;
                    for (name, request) in outputs {
                        let mut output = plain.clone();
                        trim_term_result(&mut output, request)?;
                        results.aggs_res.insert(name.clone(), output);
                    }
                }
                Derivation::Stats { producer, outputs } => {
                    let physical = results.aggs_res.remove(producer).ok_or_else(|| {
                        TantivyError::InternalError(format!(
                            "missing physical aggregation result {producer:?}"
                        ))
                    })?;
                    let IntermediateAggregationResult::Metric(IntermediateMetricResult::Stats(
                        stats,
                    )) = physical
                    else {
                        return Err(TantivyError::InternalError(format!(
                            "physical aggregation {producer:?} did not return stats"
                        )));
                    };
                    for (name, output) in outputs {
                        let metric = match output {
                            StatsOutput::Average => IntermediateMetricResult::Average(
                                IntermediateAverage::from_stats(stats),
                            ),
                            StatsOutput::Count => {
                                IntermediateMetricResult::Count(IntermediateCount::from_stats(stats))
                            }
                            StatsOutput::Max => {
                                IntermediateMetricResult::Max(IntermediateMax::from_stats(stats))
                            }
                            StatsOutput::Min => {
                                IntermediateMetricResult::Min(IntermediateMin::from_stats(stats))
                            }
                            StatsOutput::Sum => {
                                IntermediateMetricResult::Sum(IntermediateSum::from_stats(stats))
                            }
                        };
                        results
                            .aggs_res
                            .insert(name.clone(), IntermediateAggregationResult::Metric(metric));
                    }
                }
            }
        }
        Ok(())
    }
}

fn terms_compatible(left: &TermsAggregation, right: &TermsAggregation) -> bool {
    let mut left_req = TermsAggregationInternal::from_req(left);
    let right_req = TermsAggregationInternal::from_req(right);
    // Zero-count dictionary filling is collector-side and depends on the segment limit.
    // Keep those requests on the exact-match path.
    if left_req.min_doc_count != 0 && right_req.min_doc_count != 0 {
        left_req.size = right_req.size;
        left_req.segment_size = right_req.segment_size;
    }
    left_req == right_req && left.include == right.include && left.exclude == right.exclude
}

/// Share a terms + histogram collector with compatible plain terms siblings. Names are sorted
/// so producer selection is deterministic; each producer is widened once for all its outputs.
fn share_terms_histograms(aggs: &mut Aggregations, names: &[&String]) -> Vec<Derivation> {
    let mut rewrites = Vec::new();
    for name in names {
        let Some(producer) = aggs.get(*name) else {
            continue;
        };
        let AggregationVariants::Terms(terms) = &producer.agg else {
            continue;
        };
        let request = TermsAggregationInternal::from_req(terms);
        if producer.sub_aggregation.len() != 1
            || matches!(request.order.target, OrderTarget::SubAggregation(_))
            || !producer.sub_aggregation.values().all(|child| {
                child.sub_aggregation.is_empty()
                    && matches!(child.agg,
                        AggregationVariants::Histogram(_) | AggregationVariants::DateHistogram(_))
            })
        {
            continue;
        }
        let outputs: Vec<_> = aggs.iter().filter_map(|(name, agg)| {
            let AggregationVariants::Terms(plain) = &agg.agg else {
                return None;
            };
            (agg.sub_aggregation.is_empty() && terms_compatible(terms, plain))
                .then(|| (name.clone(), TermsAggregationInternal::from_req(plain)))
        }).collect();
        if outputs.is_empty() {
            continue;
        }
        let segment_size = outputs.iter().fold(request.segment_size,
            |limit, (_, output)| limit.max(output.segment_size));
        let mut shared = producer.clone();
        shared.agg = AggregationVariants::Terms(TermsAggregation {
            segment_size: Some(segment_size), ..terms.clone()
        });
        for (output, _) in &outputs {
            aggs.remove(output);
        }
        aggs.insert((*name).clone(), shared);
        rewrites.push(Derivation::PlainTerms { producer: (*name).clone(), request, outputs });
    }
    rewrites
}

fn stats_metric(agg: &Aggregation) -> Option<(&str, Option<f64>, StatsOutput)> {
    if !agg.sub_aggregation.is_empty() {
        return None;
    }
    match &agg.agg {
        AggregationVariants::Average(req) => {
            Some((&req.field, req.missing, StatsOutput::Average))
        }
        AggregationVariants::Count(req) => Some((&req.field, req.missing, StatsOutput::Count)),
        AggregationVariants::Max(req) => Some((&req.field, req.missing, StatsOutput::Max)),
        AggregationVariants::Min(req) => Some((&req.field, req.missing, StatsOutput::Min)),
        AggregationVariants::Sum(req) => Some((&req.field, req.missing, StatsOutput::Sum)),
        _ => None,
    }
}

fn trim_term_result(
    result: &mut IntermediateAggregationResult,
    request: &TermsAggregationInternal,
) -> crate::Result<()> {
    let IntermediateAggregationResult::Bucket(IntermediateBucketResult::Terms { buckets }) = result
    else {
        return Err(TantivyError::InternalError("expected terms result".to_string()));
    };
    let limit = request.segment_size as usize;
    if buckets.entries.len() <= limit {
        return Ok(());
    }
    buckets.entries.sort_unstable_by(|(left_key, left), (right_key, right)| {
        let ordering = match request.order.target {
            OrderTarget::Key => left_key.cmp(right_key),
            OrderTarget::Count => left.doc_count.cmp(&right.doc_count),
            OrderTarget::SubAggregation(_) => unreachable!("plain terms cannot order by a metric"),
        };
        let ordering = if request.order.order == Order::Desc {
            ordering.reverse()
        } else {
            ordering
        };
        ordering.then_with(|| left_key.cmp(right_key))
    });
    buckets.doc_count_error_upper_bound = buckets.entries[limit].1.doc_count;
    buckets.sum_other_doc_count += buckets.entries[limit..]
        .iter().map(|(_, entry)| entry.doc_count).sum::<u64>();
    buckets.entries.truncate(limit);
    buckets.key_to_idx_in_entries.clear();
    Ok(())
}

fn strip_term_sub_aggregations(result: &mut IntermediateAggregationResult) -> crate::Result<()> {
    let IntermediateAggregationResult::Bucket(IntermediateBucketResult::Terms { buckets }) = result
    else {
        return Err(TantivyError::InternalError(
            "terms producer did not return terms buckets".to_string(),
        ));
    };
    for (_, entry) in &mut buckets.entries {
        entry.sub_aggregation = IntermediateAggregationResults::default();
    }
    Ok(())
}

fn apply_to_bucket_children(
    result: &mut IntermediateAggregationResult,
    rewrite: &ScopeRewrite,
) -> crate::Result<()> {
    let IntermediateAggregationResult::Bucket(bucket) = result else {
        return Ok(());
    };
    match bucket {
        IntermediateBucketResult::Range(range) => {
            for entry in range.buckets.values_mut() {
                rewrite.apply(&mut entry.sub_aggregation_res)?;
            }
        }
        IntermediateBucketResult::Histogram { buckets, .. } => {
            for entry in buckets {
                rewrite.apply(&mut entry.sub_aggregation)?;
            }
        }
        IntermediateBucketResult::Terms { buckets } => {
            for (_, entry) in &mut buckets.entries {
                rewrite.apply(&mut entry.sub_aggregation)?;
            }
        }
        IntermediateBucketResult::Filter { sub_aggregations, .. } => {
            rewrite.apply(sub_aggregations)?;
        }
        IntermediateBucketResult::Composite { buckets } => {
            for entry in buckets.entries.values_mut() {
                rewrite.apply(&mut entry.sub_aggregation)?;
            }
        }
        IntermediateBucketResult::MultiTerms { buckets } => {
            for entry in buckets.entries.values_mut() {
                rewrite.apply(&mut entry.sub_aggregation)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_ordering_preserves_stats_property() {
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "a": { "stats": { "field": "score" } },
            "b": { "stats": { "field": "score" } }
        })).unwrap();
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.rewrite.physical_order_target("b.avg"), Some("a.avg".into()));
        assert_eq!(plan.rewrite.physical_order_target("other.avg"), None);
    }

    #[test]
    fn deduplicates_identical_date_histograms() {
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "__b0_timestamp": { "date_histogram": { "field": "timestamp", "fixed_interval": "300000ms" } },
            "__b1_timestamp": { "date_histogram": { "field": "timestamp", "fixed_interval": "300000ms" } },
            "__b2_timestamp": { "date_histogram": { "field": "timestamp", "fixed_interval": "300000ms" } },
            "__b3_timestamp": { "date_histogram": { "field": "timestamp", "fixed_interval": "300000ms" } },
            "different": { "date_histogram": { "field": "timestamp", "fixed_interval": "600000ms" } }
        })).unwrap();
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.physical_aggs.len(), 2);
        assert!(plan.physical_aggs.contains_key("__b0_timestamp"));
        assert!(plan.physical_aggs.contains_key("different"));
    }

    #[test]
    fn terms_histogram_sharing_respects_different_sizes() {
        let mut aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "__b1_service": {
                "terms": { "field": "service", "size": 50 }
            },
            "__b2_status": {
                "terms": { "field": "status", "size": 10 },
                "aggs": {
                    "time:3600000": {
                        "date_histogram": {
                            "field": "timestamp", "fixed_interval": "3600000ms"
                        }
                    }
                }
            },
            "__b3_status": {
                "terms": { "field": "status", "size": 50 }
            }
        })).unwrap();

        // Widen the histogram producer to retain enough candidates for both outputs.
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.physical_aggs.len(), 2);
        let AggregationVariants::Terms(terms) = &plan.physical_aggs["__b2_status"].agg else {
            unreachable!();
        };
        assert_eq!(terms.segment_size, Some(500));
        assert!(!plan.physical_aggs.contains_key("__b3_status"));

        // With matching limits, the status requests share; the unrelated service request stays.
        let AggregationVariants::Terms(terms) =
            &mut aggs.get_mut("__b2_status").unwrap().agg
        else {
            unreachable!();
        };
        terms.size = Some(50);
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.physical_aggs.len(), 2);
        assert!(plan.physical_aggs.contains_key("__b1_service"));
        assert!(plan.physical_aggs.contains_key("__b2_status"));
        assert!(!plan.physical_aggs.contains_key("__b3_status"));

        // Dictionary filling for zero-count buckets still requires matching limits.
        for name in ["__b2_status", "__b3_status"] {
            let AggregationVariants::Terms(terms) = &mut aggs.get_mut(name).unwrap().agg else {
                unreachable!();
            };
            terms.min_doc_count = Some(0);
        }
        let AggregationVariants::Terms(terms) = &mut aggs.get_mut("__b2_status").unwrap().agg else {
            unreachable!();
        };
        terms.size = Some(10);
        assert_eq!(AggregationExecutionPlan::analyze(&aggs).physical_aggs.len(), 3);
    }

    #[test]
    fn shared_terms_with_different_limits_match_independent_requests() -> crate::Result<()> {
        use crate::aggregation::tests::{exec_request, get_test_index_from_values_and_terms};

        // Unique counts exercise actual segment truncation without ambiguous cutoff ties.
        let values: Vec<_> = (0..8).flat_map(|term| {
            (0..term + 1).map(move |_| (term as f64, format!("term{term}")))
        }).collect();
        let index = get_test_index_from_values_and_terms(false, &[values.clone(), values])?;
        for order in ["_count", "_key"] {
            for direction in ["asc", "desc"] {
                for (hist_limit, plain_limit) in [(2, 5), (5, 2)] {
                    let aggs: Aggregations = serde_json::from_value(serde_json::json!({
                        "hist": {
                            "terms": { "field": "string_id", "size": hist_limit,
                                "segment_size": hist_limit, "order": { (order): direction } },
                            "aggs": { "time": { "histogram": {
                                "field": "score_f64", "interval": 2.0
                            } } }
                        },
                        "plain": { "terms": { "field": "string_id", "size": plain_limit,
                            "segment_size": plain_limit, "order": { (order): direction } } },
                        "wider": { "terms": { "field": "string_id", "size": 7,
                            "segment_size": 7, "order": { (order): direction } } }
                    }))?;
                    assert_eq!(AggregationExecutionPlan::analyze(&aggs).physical_aggs.len(), 1);
                    let shared = exec_request(aggs.clone(), &index)?;
                    for (name, agg) in aggs {
                        let independent = exec_request([(name.clone(), agg)].into_iter().collect(), &index)?;
                        assert_eq!(shared[&name], independent[&name], "{order} {direction} {hist_limit}/{plain_limit}");
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn duplicates_restore_logical_children_and_metric_ordering() -> crate::Result<()> {
        use crate::aggregation::tests::{exec_request, get_test_index_from_values_and_terms};

        let index = get_test_index_from_values_and_terms(false, &[
            vec![(2.0, "a".to_string()), (4.0, "a".to_string())],
            vec![(9.0, "b".to_string())],
        ])?;
        let subtree = serde_json::json!({
            "terms": { "field": "string_id", "order": { "z_sum": "asc" } },
            "aggs": {
                "a_sum": { "sum": { "field": "score_f64" } },
                "avg": { "avg": { "field": "score_f64" } },
                "z_sum": { "sum": { "field": "score_f64" } }
            }
        });
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "a": subtree, "b": subtree
        }))?;
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.physical_aggs.len(), 1);
        assert_eq!(plan.physical_aggs["a"].sub_aggregation.len(), 1);
        let result = exec_request(aggs, &index)?;
        assert_eq!(result["a"], result["b"]);
        assert_eq!(result["b"]["buckets"][0]["key"], "a");
        assert_eq!(result["b"]["buckets"][0]["z_sum"]["value"], 6.0);
        assert_eq!(result["b"]["buckets"][0]["avg"]["value"], 3.0);

        let histogram = serde_json::json!({
            "histogram": { "field": "score_f64", "interval": 2.0 }
        });
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "h0": histogram, "h1": histogram, "h2": histogram, "h3": histogram
        }))?;
        assert_eq!(AggregationExecutionPlan::analyze(&aggs).physical_aggs.len(), 1);
        let result = exec_request(aggs, &index)?;
        for name in ["h1", "h2", "h3"] {
            assert_eq!(result["h0"], result[name]);
        }
        assert_eq!(result["h0"]["buckets"][0]["doc_count"], 1);
        Ok(())
    }

    #[test]
    fn analyzes_terms_histogram_and_sum_average_recursively() {
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "by_term_with_time": {
                "terms": { "field": "text" },
                "aggs": {
                    "time": { "histogram": { "field": "time", "interval": 1.0 } },
                    "sum": { "sum": { "field": "score" } },
                    "avg": { "avg": { "field": "score" } }
                }
            },
            "plain": { "terms": { "field": "text" } }
        }))
        .unwrap();

        // The terms rule deliberately requires the histogram to be the sole child.
        let plan = AggregationExecutionPlan::analyze(&aggs);
        assert_eq!(plan.physical_aggs.len(), 2);
        assert_eq!(
            plan.physical_aggs["by_term_with_time"].sub_aggregation.len(),
            2
        );

        let terms_aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "with_time": {
                "terms": { "field": "text" },
                "aggs": { "time": { "histogram": { "field": "time", "interval": 1.0 } } }
            },
            "plain": { "terms": { "field": "text" } }
        }))
        .unwrap();
        let plan = AggregationExecutionPlan::analyze(&terms_aggs);
        assert_eq!(plan.physical_aggs.len(), 1);
        assert!(plan.physical_aggs.contains_key("with_time"));
    }

    #[test]
    fn shared_collectors_materialize_equivalent_results() -> crate::Result<()> {
        use crate::aggregation::tests::{
            exec_request, get_test_index_from_values_and_terms,
        };

        let index = get_test_index_from_values_and_terms(
            false,
            &[
                vec![(2.0, "a".to_string()), (4.0, "a".to_string())],
                vec![(9.0, "b".to_string())],
            ],
        )?;
        let aggs: Aggregations = serde_json::from_value(serde_json::json!({
            "with_histogram": {
                "terms": { "field": "string_id", "order": { "_key": "asc" } },
                "aggs": {
                    "histogram": { "histogram": { "field": "score_f64", "interval": 2.0 } }
                }
            },
            "plain": {
                "terms": { "field": "string_id", "order": { "_key": "asc" } }
            },
            "sum": { "sum": { "field": "score_f64" } },
            "average": { "avg": { "field": "score_f64" } },
            "count": { "value_count": { "field": "score_f64" } },
            "max": { "max": { "field": "score_f64" } },
            "min": { "min": { "field": "score_f64" } },
            "ordered": {
                "terms": {
                    "field": "string_id",
                    "order": { "nested_sum": "asc" }
                },
                "aggs": {
                    "nested_sum": { "sum": { "field": "score_f64" } },
                    "nested_average": { "avg": { "field": "score_f64" } }
                }
            }
        }))?;

        let result = exec_request(aggs, &index)?;
        assert_eq!(result["sum"]["value"], 15.0);
        assert_eq!(result["average"]["value"], 5.0);
        assert_eq!(result["count"]["value"], 3.0);
        assert_eq!(result["max"]["value"], 9.0);
        assert_eq!(result["min"]["value"], 2.0);
        assert_eq!(
            result["plain"]["buckets"][0]["doc_count"],
            result["with_histogram"]["buckets"][0]["doc_count"]
        );
        assert_eq!(
            result["plain"]["buckets"][1]["doc_count"],
            result["with_histogram"]["buckets"][1]["doc_count"]
        );
        assert!(result["plain"]["buckets"][0].get("histogram").is_none());
        assert!(result["with_histogram"]["buckets"][0]
            .get("histogram")
            .is_some());
        assert_eq!(result["ordered"]["buckets"][0]["key"], "a");
        assert_eq!(result["ordered"]["buckets"][0]["nested_sum"]["value"], 6.0);
        assert_eq!(
            result["ordered"]["buckets"][0]["nested_average"]["value"],
            3.0
        );
        Ok(())
    }
}
