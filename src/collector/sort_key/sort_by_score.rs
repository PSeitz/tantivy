use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

use common::TinySet;

use crate::collector::sort_key::{Comparator, NaturalComparator};
use crate::collector::{SegmentSortKeyComputer, SortKeyComputer, TopNComputer};
use crate::{DocAddress, DocId, Order, Score};

#[inline]
fn scores_gt_tinyset(scores: &[Score; 64], threshold: Score) -> TinySet {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is part of the AArch64 baseline, and the function only reads the supplied
        // 64-element score array.
        return TinySet::from_bits(unsafe { scores_gt_mask_neon(scores, threshold) });
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        // SAFETY: this branch is compiled only when AVX2 is enabled for the target.
        return TinySet::from_bits(unsafe { scores_gt_mask_avx2(scores, threshold) });
    }

    #[allow(unreachable_code)]
    {
        let mut matches = TinySet::EMPTY;
        for (offset, &score) in scores.iter().enumerate() {
            if score > threshold {
                matches = matches.insert(offset as u32);
            }
        }
        matches
    }
}

#[inline]
fn scores_lt_tinyset(scores: &[Score; 64], threshold: Score) -> TinySet {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is part of the AArch64 baseline, and the function only reads the supplied
        // 64-element score array.
        return TinySet::from_bits(unsafe { scores_lt_mask_neon(scores, threshold) });
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        // SAFETY: this branch is compiled only when AVX2 is enabled for the target.
        return TinySet::from_bits(unsafe { scores_lt_mask_avx2(scores, threshold) });
    }

    #[allow(unreachable_code)]
    {
        let mut matches = TinySet::EMPTY;
        for (offset, &score) in scores.iter().enumerate() {
            if score < threshold {
                matches = matches.insert(offset as u32);
            }
        }
        matches
    }
}

#[inline]
fn competitive_scores_tinyset(
    scores: &[Score; 64],
    threshold: Score,
    sort_order: Order,
) -> TinySet {
    match sort_order {
        Order::Desc => scores_gt_tinyset(scores, threshold),
        Order::Asc => scores_lt_tinyset(scores, threshold),
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn scores_gt_mask_neon(scores: &[Score; 64], threshold: Score) -> u64 {
    use std::arch::aarch64::{
        vaddvq_u32, vcgtq_f32, vdupq_n_f32, vld1q_f32, vld1q_u32, vmulq_u32, vshrq_n_u32,
    };

    let threshold = vdupq_n_f32(threshold);
    let bit_weights = vld1q_u32([1u32, 2, 4, 8].as_ptr());
    let mut mask = 0u64;
    for chunk in 0..8 {
        let offset = chunk * 8;
        let low_scores = vld1q_f32(scores.as_ptr().add(offset));
        let high_scores = vld1q_f32(scores.as_ptr().add(offset + 4));
        let low_bits = vshrq_n_u32::<31>(vcgtq_f32(low_scores, threshold));
        let high_bits = vshrq_n_u32::<31>(vcgtq_f32(high_scores, threshold));
        let low_mask = vaddvq_u32(vmulq_u32(low_bits, bit_weights));
        let high_mask = vaddvq_u32(vmulq_u32(high_bits, bit_weights));
        mask |= u64::from(low_mask | (high_mask << 4)) << offset;
    }
    mask
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn scores_lt_mask_neon(scores: &[Score; 64], threshold: Score) -> u64 {
    use std::arch::aarch64::{
        vaddvq_u32, vcgtq_f32, vdupq_n_f32, vld1q_f32, vld1q_u32, vmulq_u32, vshrq_n_u32,
    };

    let threshold = vdupq_n_f32(threshold);
    let bit_weights = vld1q_u32([1u32, 2, 4, 8].as_ptr());
    let mut mask = 0u64;
    for chunk in 0..8 {
        let offset = chunk * 8;
        let low_scores = vld1q_f32(scores.as_ptr().add(offset));
        let high_scores = vld1q_f32(scores.as_ptr().add(offset + 4));
        let low_bits = vshrq_n_u32::<31>(vcgtq_f32(threshold, low_scores));
        let high_bits = vshrq_n_u32::<31>(vcgtq_f32(threshold, high_scores));
        let low_mask = vaddvq_u32(vmulq_u32(low_bits, bit_weights));
        let high_mask = vaddvq_u32(vmulq_u32(high_bits, bit_weights));
        mask |= u64::from(low_mask | (high_mask << 4)) << offset;
    }
    mask
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn scores_gt_mask_avx2(scores: &[Score; 64], threshold: Score) -> u64 {
    use std::arch::x86_64::{
        _mm256_cmp_ps, _mm256_loadu_ps, _mm256_movemask_ps, _mm256_set1_ps, _CMP_GT_OQ,
    };

    let threshold = _mm256_set1_ps(threshold);
    let mut mask = 0u64;
    for chunk in 0..8 {
        let values = _mm256_loadu_ps(scores.as_ptr().add(chunk * 8));
        let comparisons = _mm256_cmp_ps::<_CMP_GT_OQ>(values, threshold);
        mask |= (_mm256_movemask_ps(comparisons) as u64) << (chunk * 8);
    }
    mask
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn scores_lt_mask_avx2(scores: &[Score; 64], threshold: Score) -> u64 {
    use std::arch::x86_64::{
        _mm256_cmp_ps, _mm256_loadu_ps, _mm256_movemask_ps, _mm256_set1_ps, _CMP_LT_OQ,
    };

    let threshold = _mm256_set1_ps(threshold);
    let mut mask = 0u64;
    for chunk in 0..8 {
        let values = _mm256_loadu_ps(scores.as_ptr().add(chunk * 8));
        let comparisons = _mm256_cmp_ps::<_CMP_LT_OQ>(values, threshold);
        mask |= (_mm256_movemask_ps(comparisons) as u64) << (chunk * 8);
    }
    mask
}

/// Sort by similarity score.
#[derive(Clone, Debug, Copy)]
pub struct SortBySimilarityScore;

impl SortKeyComputer for SortBySimilarityScore {
    type SortKey = Score;

    type Child = SortBySimilarityScore;

    type Comparator = NaturalComparator;

    fn requires_scoring(&self) -> bool {
        true
    }

    fn segment_sort_key_computer(
        &self,
        _segment_reader: &crate::SegmentReader,
    ) -> crate::Result<Self::Child> {
        Ok(SortBySimilarityScore)
    }

    // Sorting by score is special in that it allows for the Block-Wand optimization.
    //
    // We use a BinaryHeap (TopNHeap) instead of TopNComputer here so that the
    // threshold is always the exact K-th best score. TopNComputer only updates its
    // threshold every K docs (at truncation), giving Block-WAND a stale bound.
    fn collect_segment_top_k(
        &self,
        k: usize,
        weight: &dyn crate::query::Weight,
        reader: &crate::SegmentReader,
        segment_ord: u32,
    ) -> crate::Result<Vec<(Self::SortKey, DocAddress)>> {
        let mut top_n = TopNHeap::new(k);

        if let Some(alive_bitset) = reader.alive_bitset() {
            let mut threshold = Score::MIN;
            weight.for_each_pruning(Score::MIN, reader, &mut |doc, score| {
                if alive_bitset.is_deleted(doc) {
                    return threshold;
                }
                top_n.push(score, doc);
                threshold = top_n.threshold.unwrap_or(Score::MIN);
                threshold
            })?;
        } else {
            weight.for_each_pruning(Score::MIN, reader, &mut |doc, score| {
                top_n.push(score, doc);
                top_n.threshold.unwrap_or(Score::MIN)
            })?;
        }

        Ok(top_n
            .into_vec()
            .into_iter()
            .map(|(score, doc)| (score, DocAddress::new(segment_ord, doc)))
            .collect())
    }
}

impl SegmentSortKeyComputer for SortBySimilarityScore {
    type SortKey = Score;
    type SegmentSortKey = Score;
    type SegmentComparator = NaturalComparator;

    #[inline(always)]
    fn segment_sort_key(&mut self, _doc: DocId, score: Score) -> Score {
        score
    }

    fn collect_scored_block<C: Comparator<Score>>(
        &mut self,
        docs: &[DocId],
        scores: &[Score],
        top_n_computer: &mut TopNComputer<Score, DocId, C>,
    ) {
        assert_eq!(docs.len(), scores.len());
        let mut doc_chunks = docs.chunks_exact(64);
        let mut score_chunks = scores.chunks_exact(64);
        let sort_order = top_n_computer.sort_order();

        for (docs, scores) in doc_chunks.by_ref().zip(score_chunks.by_ref()) {
            if let Some(sort_order) = sort_order {
                if let Some(threshold) = top_n_computer.threshold {
                    let score_chunk: &[Score; 64] = scores.try_into().unwrap();
                    for offset in competitive_scores_tinyset(score_chunk, threshold, sort_order) {
                        let offset = offset as usize;
                        top_n_computer.append_doc(docs[offset], scores[offset]);
                    }
                } else {
                    for (&doc, &score) in docs.iter().zip(scores) {
                        top_n_computer.append_doc(doc, score);
                    }
                }
                continue;
            }

            for (&doc, &score) in docs.iter().zip(scores) {
                top_n_computer.push(score, doc);
            }
        }

        for (&doc, &score) in doc_chunks.remainder().iter().zip(score_chunks.remainder()) {
            top_n_computer.push(score, doc);
        }
    }

    fn collect_scored_window<C: Comparator<Score>>(
        &mut self,
        base_doc: DocId,
        match_tinysets: &[TinySet; 64],
        scores: &[Score; 4096],
        top_n_computer: &mut TopNComputer<Score, DocId, C>,
    ) {
        let sort_order = top_n_computer.sort_order();
        for (tinyset_idx, &tinyset) in match_tinysets.iter().enumerate() {
            let score_start = tinyset_idx * 64;
            let candidates = if let (Some(sort_order), Some(threshold)) =
                (sort_order, top_n_computer.threshold)
            {
                let score_chunk: &[Score; 64] =
                    scores[score_start..score_start + 64].try_into().unwrap();
                tinyset.intersect(competitive_scores_tinyset(
                    score_chunk,
                    threshold,
                    sort_order,
                ))
            } else {
                tinyset
            };

            for bit in candidates {
                let score_offset = score_start + bit as usize;
                let score = scores[score_offset];
                let doc = base_doc + score_offset as DocId;
                if sort_order.is_some() {
                    top_n_computer.append_doc(doc, score);
                } else {
                    top_n_computer.push(score, doc);
                }
            }
        }
    }

    fn convert_segment_sort_key(&self, score: Score) -> Score {
        score
    }
}

/// Min-heap entry: higher score = greater, lower doc wins ties.
struct ScoreHeapEntry {
    score: Score,
    doc: DocId,
}

impl Eq for ScoreHeapEntry {}

impl PartialEq for ScoreHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for ScoreHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoreHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.doc.cmp(&self.doc))
    }
}

/// Heap-based top-K for score collection. O(log K) per insert, but the threshold
/// is always tight, so Block-WAND prunes better than with [`TopNComputer`]'s
/// buffer/median approach.
///
/// Like [`TopNComputer`], items must arrive in ascending doc order, and equal
/// scores are rejected (strict `>`) so that lower doc IDs win ties.
///
/// [`TopNComputer`]: crate::collector::TopNComputer
struct TopNHeap {
    heap: BinaryHeap<Reverse<ScoreHeapEntry>>,
    top_n: usize,
    threshold: Option<Score>,
}

impl TopNHeap {
    fn new(top_n: usize) -> Self {
        TopNHeap {
            heap: BinaryHeap::with_capacity(top_n),
            top_n,
            threshold: None,
        }
    }

    #[inline]
    fn push(&mut self, score: Score, doc: DocId) {
        if self.heap.len() < self.top_n {
            self.heap.push(Reverse(ScoreHeapEntry { score, doc }));
            if self.heap.len() == self.top_n {
                self.threshold = self.heap.peek().map(|Reverse(entry)| entry.score);
            }
        } else if let Some(threshold) = self.threshold {
            if score > threshold {
                // peek_mut + assign is a single sift-down, vs pop + push = two sifts.
                if let Some(mut min) = self.heap.peek_mut() {
                    *min = Reverse(ScoreHeapEntry { score, doc });
                }
                self.threshold = self.heap.peek().map(|Reverse(entry)| entry.score);
            }
        }
    }

    fn into_vec(self) -> Vec<(Score, DocId)> {
        self.heap
            .into_vec()
            .into_iter()
            .map(|Reverse(entry)| (entry.score, entry.doc))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::collector::sort_key::NaturalComparator;
    use crate::collector::TopNComputer;

    #[test]
    fn test_scores_gt_tinyset() {
        let mut scores = [0.0; 64];
        for (offset, score) in scores.iter_mut().enumerate() {
            *score = offset as Score / 10.0;
        }
        scores[17] = Score::NAN;

        let threshold = 3.1;
        let expected = scores
            .iter()
            .enumerate()
            .fold(0u64, |mask, (offset, score)| {
                mask | (u64::from(*score > threshold) << offset)
            });
        assert_eq!(scores_gt_tinyset(&scores, threshold).bits(), expected);
    }

    #[test]
    fn test_scores_lt_tinyset() {
        let mut scores = [0.0; 64];
        for (offset, score) in scores.iter_mut().enumerate() {
            *score = offset as Score / 10.0;
        }
        scores[17] = Score::NAN;

        let threshold = 3.1;
        let expected = scores
            .iter()
            .enumerate()
            .fold(0u64, |mask, (offset, score)| {
                mask | (u64::from(*score < threshold) << offset)
            });
        assert_eq!(scores_lt_tinyset(&scores, threshold).bits(), expected);
    }

    #[test]
    fn test_top_n_heap_zero_capacity() {
        let mut heap = TopNHeap::new(0);
        heap.push(1.0, 0);
        heap.push(2.0, 1);
        assert!(heap.into_vec().is_empty());
    }

    #[test]
    fn test_top_n_heap_basic() {
        let mut heap = TopNHeap::new(2);
        heap.push(1.0, 0);
        heap.push(3.0, 1);
        heap.push(2.0, 2);

        let mut results = heap.into_vec();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then_with(|| a.1.cmp(&b.1)));
        assert_eq!(results, vec![(3.0, 1), (2.0, 2)]);
    }

    #[test]
    fn test_top_n_heap_threshold_always_accurate() {
        let mut heap = TopNHeap::new(2);
        assert_eq!(heap.threshold, None);

        heap.push(1.0, 0);
        assert_eq!(heap.threshold, None);

        heap.push(3.0, 1);
        assert_eq!(heap.threshold, Some(1.0));

        heap.push(2.0, 2); // evicts 1.0
        assert_eq!(heap.threshold, Some(2.0));

        heap.push(4.0, 3); // evicts 2.0
        assert_eq!(heap.threshold, Some(3.0));
    }

    #[test]
    fn test_top_n_heap_tiebreaking_lower_doc_wins() {
        let mut heap = TopNHeap::new(2);
        heap.push(5.0, 0);
        heap.push(5.0, 1);
        heap.push(5.0, 2); // rejected: not strictly > threshold

        let mut results = heap.into_vec();
        results.sort_by_key(|&(_, doc)| doc);
        assert_eq!(results, vec![(5.0, 0), (5.0, 1)]);
    }

    #[test]
    fn test_top_n_heap_single_element() {
        let mut heap = TopNHeap::new(1);
        heap.push(1.0, 0);
        assert_eq!(heap.threshold, Some(1.0));

        heap.push(0.5, 1); // rejected
        heap.push(2.0, 2); // accepted
        assert_eq!(heap.threshold, Some(2.0));

        let results = heap.into_vec();
        assert_eq!(results, vec![(2.0, 2)]);
    }

    #[test]
    fn test_top_n_heap_under_capacity() {
        let mut heap = TopNHeap::new(5);
        heap.push(3.0, 0);
        heap.push(1.0, 1);
        heap.push(2.0, 2);
        // Only 3 elements, capacity is 5 — all should be kept
        assert_eq!(heap.threshold, None);

        let mut results = heap.into_vec();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then_with(|| a.1.cmp(&b.1)));
        assert_eq!(results, vec![(3.0, 0), (2.0, 2), (1.0, 1)]);
    }

    proptest! {
        #[test]
        fn test_top_n_heap_matches_top_n_computer(
            limit in 0..20_usize,
            mut docs in proptest::collection::vec((0..1000_u32, 0..1000_u32), 0..200_usize),
        ) {
            // Both require ascending doc order.
            docs.sort_by_key(|(_, doc_id)| *doc_id);
            docs.dedup_by_key(|(_, doc_id)| *doc_id);

            let mut heap = TopNHeap::new(limit);
            let mut computer: TopNComputer<Score, DocId, NaturalComparator> =
                TopNComputer::new_with_comparator(limit, NaturalComparator);

            for &(score_u32, doc) in &docs {
                let score = score_u32 as Score;
                heap.push(score, doc);
                computer.push(score, doc);
            }

            let mut heap_results = heap.into_vec();
            heap_results.sort_by(|a, b| {
                b.0.partial_cmp(&a.0).unwrap().then_with(|| a.1.cmp(&b.1))
            });

            let computer_results: Vec<(Score, DocId)> = computer
                .into_sorted_vec()
                .into_iter()
                .map(|cd| (cd.sort_key, cd.doc))
                .collect();

            prop_assert_eq!(heap_results, computer_results);
        }
    }
}
