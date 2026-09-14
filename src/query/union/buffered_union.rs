use std::ops::Range;

use common::TinySet;

use crate::docset::{DocSet, COLLECT_BLOCK_BUFFER_LEN, TERMINATED};
use crate::query::score_combiner::{DoNothingCombiner, ScoreCombiner};
use crate::query::scored_block_output::CombiningOutput;
use crate::query::size_hint::estimate_union;
use crate::query::{ScoredBatch, ScoredBlockOutput, Scorer, SCORE_BLOCK_LEN};
use crate::{DocId, Score};

// The buffered union looks ahead within a fixed-size sliding window
// of upcoming document IDs (the "horizon").
const HORIZON_NUM_TINYBITSETS: usize = HORIZON as usize / 64;
const HORIZON: u32 = 64u32 * 64u32;

/// Creates a `DocSet` that iterate through the union of two or more `DocSet`s.
pub struct BufferedUnionScorer<TScorer, TScoreCombiner = DoNothingCombiner> {
    /// Active scorers (already filtered of `TERMINATED`).
    docsets: Vec<TScorer>,
    /// Sliding window presence map for upcoming docs.
    ///
    /// There are `HORIZON_NUM_TINYBITSETS` buckets, each covering
    /// a span of 64 doc IDs. Bucket `i` represents the range
    /// `[window_start_doc + i*64, window_start_doc + (i+1)*64)`.
    bitsets: Box<[TinySet; HORIZON_NUM_TINYBITSETS]>,
    // Index of the current TinySet bucket within the sliding window.
    bucket_idx: usize,
    /// Per-doc score combiners for the current window.
    ///
    /// these accumulators merge contributions from all scorers that
    /// hit the same doc within the buffered window.
    scores: Box<[TScoreCombiner; HORIZON as usize]>,
    /// Start doc ID (inclusive) of the current sliding window.
    window_start_doc: DocId,
    /// Current doc ID of the union.
    doc: DocId,
    /// Combined score for current `doc` as produced by `TScoreCombiner`.
    score: Score,
    /// Number of documents in the segment.
    num_docs: u32,
}

fn clear_window<TScoreCombiner: ScoreCombiner>(
    bitsets: &mut [TinySet],
    score_combiners: &mut [TScoreCombiner],
) {
    debug_assert_eq!(bitsets.len() * 64, score_combiners.len());
    if !TScoreCombiner::REQUIRES_SCORING {
        bitsets.fill(TinySet::empty());
        return;
    }
    for (bucket_idx, matches) in bitsets.iter_mut().enumerate() {
        let score_start = bucket_idx * 64;
        for bit in *matches {
            score_combiners[score_start + bit as usize].clear();
        }
        matches.clear();
    }
}

fn drain_tinyset<TScoreCombiner: ScoreCombiner>(
    bits: TinySet,
    base_doc: DocId,
    docs: &mut [DocId],
    output_scores: &mut [Score],
    score_combiners: &mut [TScoreCombiner],
) {
    debug_assert_eq!(docs.len(), output_scores.len());
    debug_assert_eq!(docs.len(), bits.len() as usize);
    debug_assert_eq!(score_combiners.len(), 64);

    for ((output_doc, output_score), val) in docs.iter_mut().zip(output_scores).zip(bits) {
        let val = val as usize;
        *output_doc = base_doc + val as DocId;
        let score_combiner = &mut score_combiners[val];
        *output_score = score_combiner.score();
        score_combiner.clear();
    }
}

fn populate_window<TScorer: Scorer, TScoreCombiner: ScoreCombiner>(
    scorers: &mut Vec<TScorer>,
    bitsets: &mut [TinySet; HORIZON_NUM_TINYBITSETS],
    score_combiners: &mut [TScoreCombiner; HORIZON as usize],
    mut dense_scores: Option<&mut [Score; HORIZON as usize]>,
    window: Range<DocId>,
) -> DocId {
    debug_assert!(window.end.saturating_sub(window.start) <= HORIZON);

    let mut next = TERMINATED;
    for scorer in scorers.iter_mut() {
        if TScoreCombiner::REQUIRES_SCORING {
            if TScoreCombiner::ACCUMULATES_IN_DENSE_OUTPUT && dense_scores.is_some() {
                scorer.emit_scored_batches(
                    window.start,
                    window.end,
                    &mut CombiningOutput::new(
                        window.start,
                        bitsets,
                        &mut dense_scores.as_deref_mut().unwrap()[..],
                        TScoreCombiner::update_dense_output,
                    ),
                );
            } else {
                scorer.emit_scored_batches(
                    window.start,
                    window.end,
                    &mut CombiningOutput::new(
                        window.start,
                        bitsets,
                        score_combiners,
                        TScoreCombiner::update_score,
                    ),
                );
            }
        } else {
            scorer.fill_bitset_block(window.start, window.end, bitsets);
        }
        debug_assert!(
            scorer.doc() >= window.end || scorer.doc() == TERMINATED,
            "child fill did not reach the union horizon"
        );
        next = next.min(scorer.doc().max(window.end));
    }
    // Keep scorer order stable: it determines floating-point accumulation order.
    scorers.retain(|scorer| scorer.doc() != TERMINATED);
    next
}

impl<TScorer: Scorer, TScoreCombiner: ScoreCombiner> BufferedUnionScorer<TScorer, TScoreCombiner> {
    /// num_docs is the number of documents in the segment.
    pub(crate) fn build(
        docsets: Vec<TScorer>,
        score_combiner_fn: impl FnOnce() -> TScoreCombiner,
        num_docs: u32,
    ) -> BufferedUnionScorer<TScorer, TScoreCombiner> {
        let mut union = Self::new(docsets, score_combiner_fn, num_docs);
        if union.refill() {
            let advanced = union.advance_buffered();
            debug_assert!(advanced);
        } else {
            union.doc = TERMINATED;
        }
        union
    }

    fn new(
        docsets: Vec<TScorer>,
        score_combiner_fn: impl FnOnce() -> TScoreCombiner,
        num_docs: u32,
    ) -> BufferedUnionScorer<TScorer, TScoreCombiner> {
        let non_empty_docsets: Vec<TScorer> = docsets
            .into_iter()
            .filter(|docset| docset.doc() != TERMINATED)
            .collect();
        BufferedUnionScorer {
            docsets: non_empty_docsets,
            bitsets: Box::new([TinySet::empty(); HORIZON_NUM_TINYBITSETS]),
            scores: Box::new([score_combiner_fn(); HORIZON as usize]),
            bucket_idx: HORIZON_NUM_TINYBITSETS,
            window_start_doc: 0,
            doc: 0,
            score: 0.0,
            num_docs,
        }
    }

    /// Builds a union for eager match-window traversal.
    ///
    /// This constructor does not initialize scalar traversal. An instance is expected to use
    /// either this block API or the existing scalar API, never both.
    pub(crate) fn build_block(
        docsets: Vec<TScorer>,
        score_combiner_fn: impl FnOnce() -> TScoreCombiner,
        num_docs: u32,
    ) -> BufferedUnionScorer<TScorer, TScoreCombiner> {
        let mut union = Self::new(docsets, score_combiner_fn, num_docs);
        union.doc = union
            .docsets
            .iter()
            .map(DocSet::doc)
            .min()
            .unwrap_or(TERMINATED);
        union
    }

    /// Fills a scored window on a union created with [`Self::build_block`].
    ///
    /// Replaces both outputs, with bit and score offsets relative to `requested.start`.
    /// Returns the first remaining document at or beyond `requested.end`, or `TERMINATED`.
    pub(crate) fn fill_scored_window(
        &mut self,
        requested: Range<DocId>,
        matches: &mut [TinySet; HORIZON_NUM_TINYBITSETS],
        output_scores: &mut [Score; HORIZON as usize],
    ) -> DocId {
        assert!(requested.start <= requested.end && requested.end <= TERMINATED);
        assert!(requested.end - requested.start <= HORIZON);
        matches.fill(TinySet::empty());
        output_scores.fill(0.0);
        let next = populate_window(
            &mut self.docsets,
            matches,
            &mut self.scores,
            Some(output_scores),
            requested,
        );
        if !TScoreCombiner::ACCUMULATES_IN_DENSE_OUTPUT {
            for (bucket, bits) in matches.iter().enumerate() {
                for bit in *bits {
                    let offset = bucket * 64 + bit as usize;
                    output_scores[offset] = self.scores[offset].score();
                    self.scores[offset].clear();
                }
            }
        }
        self.doc = next;
        next
    }

    // Keep refill out-of-line. When LLVM inlines it into `advance`, the full traversal path used
    // by combined collectors such as `(TopDocs, Count)` becomes sensitive to unrelated codegen
    // changes and regresses on large unions.
    #[inline(never)]
    fn refill(&mut self) -> bool {
        if let Some(min_doc) = self.docsets.iter().map(DocSet::doc).min() {
            // Reset the sliding window to start at the smallest doc
            // across all scorers and prebuffer within the horizon.
            self.window_start_doc = min_doc;
            self.bucket_idx = 0;
            self.doc = min_doc;
            populate_window(
                &mut self.docsets,
                &mut self.bitsets,
                &mut self.scores,
                None,
                min_doc..min_doc.saturating_add(HORIZON).min(TERMINATED),
            );
            true
        } else {
            false
        }
    }

    #[inline]
    fn advance_buffered(&mut self) -> bool {
        while self.bucket_idx < HORIZON_NUM_TINYBITSETS {
            if let Some(val) = self.bitsets[self.bucket_idx].pop_lowest() {
                let delta = val + (self.bucket_idx as u32) * 64;
                self.doc = self.window_start_doc + delta;
                let score_combiner = &mut self.scores[delta as usize];
                self.score = score_combiner.score();
                score_combiner.clear();
                return true;
            } else {
                self.bucket_idx += 1;
            }
        }
        false
    }

    fn fill_compact_scored_buffer(
        &mut self,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        if self.doc == TERMINATED {
            return 0;
        }

        docs[0] = self.doc;
        scores[0] = self.score;
        let mut count = 1;
        loop {
            while self.bucket_idx < HORIZON_NUM_TINYBITSETS {
                let chunk = self.bitsets[self.bucket_idx..].chunks(4).next().unwrap();
                if let &[bits0, bits1, bits2, bits3] = chunk {
                    let counts = [
                        bits0.len() as usize,
                        bits1.len() as usize,
                        bits2.len() as usize,
                        bits3.len() as usize,
                    ];
                    let group_count = counts.iter().sum::<usize>();
                    if group_count <= SCORE_BLOCK_LEN - count {
                        let mut output_start = count;
                        for (lane, (&bits, bucket_count)) in chunk.iter().zip(counts).enumerate() {
                            let output_end = output_start + bucket_count;
                            let score_start = (self.bucket_idx + lane) * 64;
                            drain_tinyset(
                                bits,
                                self.window_start_doc + score_start as DocId,
                                &mut docs[output_start..output_end],
                                &mut scores[output_start..output_end],
                                &mut self.scores[score_start..score_start + 64],
                            );
                            output_start = output_end;
                        }
                        self.bitsets[self.bucket_idx..self.bucket_idx + 4].fill(TinySet::empty());
                        self.bucket_idx += 4;
                        count += group_count;
                        continue;
                    }
                }

                let mut tinyset = self.bitsets[self.bucket_idx];
                while let Some(val) = tinyset.pop_lowest() {
                    let delta = val + self.bucket_idx as u32 * 64;
                    let doc = self.window_start_doc + delta;
                    let score_combiner = &mut self.scores[delta as usize];
                    let score = score_combiner.score();
                    score_combiner.clear();
                    if count == SCORE_BLOCK_LEN {
                        self.doc = doc;
                        self.score = score;
                        self.bitsets[self.bucket_idx] = tinyset;
                        return count;
                    }
                    docs[count] = doc;
                    scores[count] = score;
                    count += 1;
                }
                self.bitsets[self.bucket_idx] = TinySet::empty();
                self.bucket_idx += 1;
            }

            if !self.refill() {
                self.doc = TERMINATED;
                return count;
            }
        }
    }
}

impl<TScorer, TScoreCombiner> DocSet for BufferedUnionScorer<TScorer, TScoreCombiner>
where
    TScorer: Scorer,
    TScoreCombiner: ScoreCombiner,
{
    fn advance(&mut self) -> DocId {
        // A loop rather than two sequential advance_buffered calls
        // produces a single inlined copy
        loop {
            if self.advance_buffered() {
                return self.doc;
            }
            if !self.refill() {
                self.doc = TERMINATED;
                return TERMINATED;
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [DocId; COLLECT_BLOCK_BUFFER_LEN]) -> usize {
        if self.doc == TERMINATED {
            return 0;
        }
        // The current doc (self.doc) has already been popped from the bitsets,
        // so the loop below won't yield it. Emit it here first.
        buffer[0] = self.doc;
        let mut count = 1;

        loop {
            // Drain docs directly from the pre-computed bitsets.
            while self.bucket_idx < HORIZON_NUM_TINYBITSETS {
                // Move bitset to a local variable to avoid read/store on self.bitsets while
                // iterating through the bits.
                let mut tinyset: TinySet = self.bitsets[self.bucket_idx];

                while let Some(val) = tinyset.pop_lowest() {
                    let delta = val + (self.bucket_idx as u32) * 64;
                    let doc = self.window_start_doc + delta;

                    if count >= COLLECT_BLOCK_BUFFER_LEN {
                        // This popped document is the first unreturned one.
                        self.doc = doc;
                        self.bitsets[self.bucket_idx] = tinyset;
                        return COLLECT_BLOCK_BUFFER_LEN;
                    }
                    buffer[count] = doc;
                    count += 1;
                }
                self.bitsets[self.bucket_idx] = TinySet::empty();
                self.bucket_idx += 1;
            }

            // Current window exhausted, refill.
            if !self.refill() {
                self.doc = TERMINATED;
                return count;
            }
        }
    }

    fn seek(&mut self, target: DocId) -> DocId {
        if self.doc >= target {
            return self.doc;
        }
        let gap = target - self.window_start_doc;
        if gap < HORIZON {
            // Our value is within the buffered horizon.

            // Skipping to corresponding bucket.
            let new_bucket_idx = gap as usize / 64;
            for obsolete_tinyset in &mut self.bitsets[self.bucket_idx..new_bucket_idx] {
                obsolete_tinyset.clear();
            }
            for score_combiner in &mut self.scores[self.bucket_idx * 64..new_bucket_idx * 64] {
                score_combiner.clear();
            }
            self.bucket_idx = new_bucket_idx;

            // Advancing until we reach the end of the bucket
            // or we reach a doc greater or equal to the target.
            let mut doc = self.doc();
            while doc < target {
                doc = self.advance();
            }
            doc
        } else {
            // Clear the buffered info.
            clear_window(&mut self.bitsets[..], &mut self.scores[..]);

            // The target is outside of the buffered horizon.
            // advance all docsets to a doc >= to the target.
            for docset in &mut self.docsets {
                if docset.doc() < target {
                    docset.seek(target);
                }
            }
            self.docsets.retain(|docset| docset.doc() != TERMINATED);

            // at this point all of the docsets
            // are positioned on a doc >= to the target.
            if !self.refill() {
                self.doc = TERMINATED;
                return TERMINATED;
            }
            self.advance()
        }
    }

    // Use the default seek_danger: forwarding to children can leave them invalid
    // when another child matches, making an ordinary seek/refill unsafe (#3086).

    #[inline]
    fn doc(&self) -> DocId {
        self.doc
    }

    fn size_hint(&self) -> u32 {
        estimate_union(self.docsets.iter().map(DocSet::size_hint), self.num_docs)
    }

    fn cost(&self) -> u64 {
        self.docsets.iter().map(|docset| docset.cost()).sum()
    }

    // TODO Also implement `count` with deletes efficiently.
    fn count_including_deleted(&mut self) -> u32 {
        if self.doc == TERMINATED {
            return 0;
        }
        let mut count = self.bitsets[self.bucket_idx..HORIZON_NUM_TINYBITSETS]
            .iter()
            .map(|bitset| bitset.len())
            .sum::<u32>()
            + 1;
        clear_window(&mut self.bitsets[..], &mut self.scores[..]);
        while self.refill() {
            count += self.bitsets.iter().map(|bitset| bitset.len()).sum::<u32>();
            clear_window(&mut self.bitsets[..], &mut self.scores[..]);
        }
        self.bucket_idx = HORIZON_NUM_TINYBITSETS;
        self.doc = TERMINATED;
        count
    }
}

impl<TScorer, TScoreCombiner> Scorer for BufferedUnionScorer<TScorer, TScoreCombiner>
where
    TScoreCombiner: ScoreCombiner,
    TScorer: Scorer,
{
    #[inline]
    fn score(&mut self) -> Score {
        self.score
    }

    fn emit_scored_batches(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        cb: &mut dyn ScoredBlockOutput,
    ) -> DocId {
        assert!(min_doc <= horizon && horizon <= TERMINATED);
        if self.doc() < min_doc {
            self.seek(min_doc);
        }
        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];
        while self.doc() < horizon {
            let len = if horizon == TERMINATED {
                self.fill_compact_scored_buffer(&mut docs, &mut scores)
            } else {
                let mut len = 0;
                while len < SCORE_BLOCK_LEN && self.doc() < horizon {
                    docs[len] = self.doc();
                    scores[len] = self.score();
                    len += 1;
                    self.advance();
                }
                len
            };
            cb.update(ScoredBatch::Sparse {
                docs: &docs[..len],
                scores: &scores[..len],
            });
        }
        self.doc()
    }
}

#[cfg(test)]
mod tests {
    use common::TinySet;

    use super::BufferedUnionScorer;
    use crate::docset::{DocSet, COLLECT_BLOCK_BUFFER_LEN, TERMINATED};
    use crate::query::score_combiner::{
        DisjunctionMaxCombiner, DoNothingCombiner, ScoreCombiner, SumCombiner,
    };
    use crate::query::scorer::collect_scored_batches;
    use crate::query::{ConstScorer, ScoredBlockOutput, Scorer, VecDocSet};
    use crate::{DocId, Score};

    fn collect_scored<TScorer: Scorer>(mut scorer: TScorer) -> Vec<(DocId, Score)> {
        collect_scored_batches(&mut scorer, 0, TERMINATED)
    }

    fn assert_eager_block_scores(
        make_combiner: impl Fn() -> DisjunctionMaxCombiner,
        overlap_score: Score,
    ) {
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![0, 2, 4, 6]), 2.0),
            ConstScorer::new(VecDocSet::from(vec![1, 2, 5, 6]), 5.0),
        ];
        let mut union = BufferedUnionScorer::build_block(scorers, make_combiner, 8);
        let mut matches = [TinySet::EMPTY; 64];
        let mut scores = [Score::NAN; 4096];
        assert_eq!(union.fill_scored_window(0..4, &mut matches, &mut scores), 4);
        assert_eq!(&scores[..4], &[2.0, 5.0, overlap_score, 0.0]);
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(
            union.fill_scored_window(4..8, &mut matches, &mut scores),
            TERMINATED
        );
        assert_eq!(&scores[..4], &[2.0, 5.0, overlap_score, 0.0]);
        assert!(scores[4..].iter().all(|&score| score == 0.0));
    }

    #[test]
    fn test_eager_block_sum_scores_selected_docs() {
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![0, 2, 4, 6]), 2.0),
            ConstScorer::new(VecDocSet::from(vec![1, 2, 5, 6]), 5.0),
        ];
        let mut union = BufferedUnionScorer::build_block(scorers, SumCombiner::default, 8);
        let mut matches = [TinySet::EMPTY; 64];
        let mut scores = [Score::NAN; 4096];
        assert_eq!(
            union.fill_scored_window(0..8, &mut matches, &mut scores),
            TERMINATED
        );
        assert_eq!(
            matches[0].into_iter().collect::<Vec<_>>(),
            vec![0, 1, 2, 4, 5, 6]
        );
        assert_eq!(&scores[..8], &[2.0, 5.0, 7.0, 0.0, 2.0, 5.0, 7.0, 0.0]);
    }

    #[test]
    fn test_eager_block_dismax_scores_selected_docs_after_consecutive_fills() {
        assert_eager_block_scores(|| DisjunctionMaxCombiner::with_tie_breaker(0.3), 5.6);
    }

    #[test]
    fn test_eager_block_preserves_zero_scored_matches() {
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![1]), 0.0),
            ConstScorer::new(VecDocSet::from(vec![2]), 5.0),
        ];
        let mut union = BufferedUnionScorer::build_block(scorers, SumCombiner::default, 4);
        let mut matches = [TinySet::EMPTY; 64];
        let mut scores = [Score::NAN; 4096];
        assert_eq!(
            union.fill_scored_window(0..4, &mut matches, &mut scores),
            TERMINATED
        );
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(&scores[..4], &[0.0, 0.0, 5.0, 0.0]);
    }

    #[test]
    fn test_eager_block_dismax_dense_window() {
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![0, 2, 4, 6]), 2.0),
            ConstScorer::new(VecDocSet::from(vec![1, 2, 5, 6]), 5.0),
        ];
        let mut union = BufferedUnionScorer::build_block(
            scorers,
            || DisjunctionMaxCombiner::with_tie_breaker(0.3),
            8,
        );
        let mut matches = [TinySet::EMPTY; 64];
        let mut scores = [Score::NAN; 4096];
        assert_eq!(
            union.fill_scored_window(0..8, &mut matches, &mut scores),
            TERMINATED
        );
        assert_eq!(&scores[..8], &[2.0, 5.0, 5.6, 0.0, 2.0, 5.0, 5.6, 0.0]);
    }

    #[test]
    fn test_scored_windows_match_scalar_across_horizons() {
        fn check<T: ScoreCombiner>(make_combiner: impl Fn() -> T) {
            let make_scorers = || {
                vec![
                    ConstScorer::new(
                        VecDocSet::from((0..4_300).step_by(3).chain([9_999]).collect::<Vec<_>>()),
                        0.0,
                    ),
                    ConstScorer::new(
                        VecDocSet::from((0..4_300).step_by(5).chain([8_192]).collect::<Vec<_>>()),
                        2.5,
                    ),
                ]
            };
            let expected = collect_scored(BufferedUnionScorer::build(
                make_scorers(),
                &make_combiner,
                10_000,
            ));
            for window_len in [63, 2_048, 4_096] {
                let mut union =
                    BufferedUnionScorer::build_block(make_scorers(), &make_combiner, 10_000);
                let mut matches = [TinySet::full(); 64];
                let mut scores = [Score::NAN; 4096];
                let mut actual = Vec::new();
                let mut start = 0;
                while start < 10_000 {
                    let end = (start + window_len).min(10_000);
                    let next = union.fill_scored_window(start..end, &mut matches, &mut scores);
                    for (offset, &score) in scores.iter().enumerate() {
                        if matches[offset / 64].contains((offset % 64) as u32) {
                            let doc = start + offset as DocId;
                            assert!(doc < end);
                            actual.push((doc, score));
                        } else {
                            assert_eq!(score, 0.0);
                        }
                    }
                    assert!(next >= end);
                    assert_eq!(union.doc(), next);
                    start = next;
                }
                assert_eq!(actual, expected);
                assert_eq!(
                    union.fill_scored_window(10_000..10_000, &mut matches, &mut scores),
                    TERMINATED
                );
                assert!(matches.iter().all(|bits| bits.is_empty()));
                assert!(scores.iter().all(|&score| score == 0.0));
            }
        }
        check(SumCombiner::default);
        check(|| DisjunctionMaxCombiner::with_tie_breaker(0.3));
        check(DoNothingCombiner::default);
    }

    #[test]
    fn test_scored_window_near_terminated() {
        let scorer = ConstScorer::new(VecDocSet::from(vec![TERMINATED - 3, TERMINATED - 1]), 2.0);
        let mut union = BufferedUnionScorer::build_block(vec![scorer], SumCombiner::default, 2);
        let mut matches = [TinySet::EMPTY; 64];
        let mut scores = [Score::NAN; 4096];
        assert_eq!(
            union.fill_scored_window(TERMINATED - 4..TERMINATED - 1, &mut matches, &mut scores),
            TERMINATED - 1
        );
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), [1]);
        assert_eq!(&scores[..4], &[0.0, 2.0, 0.0, 0.0]);
        assert_eq!(
            union.fill_scored_window(TERMINATED - 1..TERMINATED, &mut matches, &mut scores),
            TERMINATED
        );
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), [0]);
        assert_eq!(&scores[..4], &[2.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_scored_fill_sum_across_blocks_and_horizons() {
        let even_docs = (0..9_000).step_by(2).collect::<Vec<DocId>>();
        let third_docs = (0..9_000).step_by(3).collect::<Vec<DocId>>();
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(even_docs), 0.25),
            ConstScorer::new(VecDocSet::from(third_docs), 1.5),
        ];
        let union = BufferedUnionScorer::build(scorers, SumCombiner::default, 9_000);
        let actual = collect_scored(union);

        let expected = (0..9_000)
            .filter_map(|doc| {
                let mut score = 0.0;
                if doc % 2 == 0 {
                    score += 0.25;
                }
                if doc % 3 == 0 {
                    score += 1.5;
                }
                (score != 0.0).then_some((doc, score))
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_scored_refill_preserves_child_accumulation_order() {
        let docs = vec![0, 1, 2];
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![0]), 9.0),
            ConstScorer::new(VecDocSet::from(docs.clone()), 1e20),
            ConstScorer::new(VecDocSet::from(docs.clone()), -1e20),
            ConstScorer::new(VecDocSet::from(docs), 1.0),
        ];
        let union = BufferedUnionScorer::build(scorers, SumCombiner::default, 3);
        assert_eq!(collect_scored(union), vec![(0, 1.0), (1, 1.0), (2, 1.0)]);
    }

    #[test]
    fn test_scored_fill_dismax_and_exclusive_horizon() {
        let scorers = vec![
            ConstScorer::new(VecDocSet::from(vec![1, 3, 5]), 1.0),
            ConstScorer::new(VecDocSet::from(vec![3, 4, 6]), 2.0),
        ];
        let mut union = BufferedUnionScorer::build(
            scorers,
            || DisjunctionMaxCombiner::with_tie_breaker(0.5),
            7,
        );
        assert_eq!(
            collect_scored_batches(&mut union, 0, 4),
            vec![(1, 1.0), (3, 2.5)]
        );
        assert_eq!(union.doc(), 4);
        assert_eq!(union.score(), 2.0);

        assert_eq!(
            collect_scored_batches(&mut union, 4, TERMINATED),
            vec![(4, 2.0), (5, 1.0), (6, 2.0)]
        );
        assert_eq!(union.doc(), TERMINATED);
    }

    #[test]
    fn test_scored_horizon_near_terminated() {
        let scorers = vec![ConstScorer::new(
            VecDocSet::from(vec![TERMINATED - 2, TERMINATED - 1]),
            3.0,
        )];
        let mut union = BufferedUnionScorer::build(scorers, SumCombiner::default, TERMINATED);
        assert_eq!(
            collect_scored_batches(&mut union, 0, TERMINATED - 1),
            vec![(TERMINATED - 2, 3.0)]
        );
        assert_eq!(union.doc(), TERMINATED - 1);
        assert_eq!(
            collect_scored_batches(&mut union, TERMINATED - 1, TERMINATED),
            vec![(TERMINATED - 1, 3.0)]
        );
    }

    struct ScorePanics {
        docs: Vec<DocId>,
        cursor: usize,
    }

    impl DocSet for ScorePanics {
        fn advance(&mut self) -> DocId {
            self.cursor += 1;
            self.doc()
        }

        fn doc(&self) -> DocId {
            self.docs.get(self.cursor).copied().unwrap_or(TERMINATED)
        }

        fn size_hint(&self) -> u32 {
            self.docs.len() as u32
        }
    }

    impl Scorer for ScorePanics {
        fn score(&mut self) -> Score {
            panic!("no-score union called score")
        }

        fn emit_scored_batches(
            &mut self,
            _min_doc: DocId,
            _horizon: DocId,
            _cb: &mut dyn ScoredBlockOutput,
        ) -> DocId {
            panic!("no-score union called scored fill")
        }
    }

    #[test]
    fn test_no_score_count_horizon_near_terminated() {
        let scorers = vec![ScorePanics {
            docs: vec![1, 5_000, TERMINATED - 1],
            cursor: 0,
        }];
        let mut union = BufferedUnionScorer::build(scorers, DoNothingCombiner::default, TERMINATED);
        assert_eq!(union.count_including_deleted(), 3);
        assert_eq!(union.doc(), TERMINATED);
    }

    #[test]
    fn test_no_score_refill_never_scores() {
        let scorers = vec![
            ScorePanics {
                docs: vec![1, 3, 5],
                cursor: 0,
            },
            ScorePanics {
                docs: vec![2, 3, 6],
                cursor: 0,
            },
        ];
        let mut union = BufferedUnionScorer::build(scorers, DoNothingCombiner::default, 7);
        let mut docs = [0; COLLECT_BLOCK_BUFFER_LEN];
        let len = union.fill_buffer(&mut docs);
        assert_eq!(&docs[..len], &[1, 2, 3, 5, 6]);
    }
}
