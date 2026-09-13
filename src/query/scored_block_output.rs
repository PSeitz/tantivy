use common::TinySet;

use crate::postings::update_scored_bitset;
use crate::{DocId, Score};

/// Borrowed scored matches in their native representation.
///
/// Storage is borrowed only for the callback. Its owner remains responsible for resetting or
/// reusing it afterwards; consumers must not clear the match bits needed to reset accumulators.
#[doc(hidden)]
pub enum ScoredBatch<'a> {
    /// Strictly increasing document IDs with one score per document.
    Sparse {
        docs: &'a [DocId],
        scores: &'a [Score],
    },
    /// A 4,096-document window, with bit zero corresponding to `base_doc`.
    ///
    /// Scores are indexed by document offset, not packed by set bit. Only scores for set bits
    /// are meaningful; unset bits leave an accumulation destination untouched.
    Dense {
        base_doc: DocId,
        words: &'a [TinySet; 64],
        scores: &'a [Score; 4096],
    },
}

/// Receives batches of matches and score contributions from a [`super::Scorer`].
///
/// The receiver owns the destination window and accumulation rule. Updates preserve existing
/// contributions and record matches even when their score is zero. Each submitted document must
/// belong to the receiver's window. Neither batch representation requires an allocation.
pub trait ScoredBlockOutput {
    /// Adds a borrowed batch without clearing existing contributions.
    fn update(&mut self, batch: ScoredBatch<'_>);

    /// Adds matches represented by bitsets, with bit zero corresponding to `base_doc`.
    ///
    /// Unlike [`ScoredBatch::Dense`], scores are packed: one per set bit, ordered by bucket and
    /// then increasing bit position.
    /// Unset bits have no score and leave the destination untouched. The batch's base need not
    /// align with the receiver's bitset buckets; only set bits must lie within its window.
    fn update_bitset(&mut self, base_doc: DocId, matches: &[TinySet], scores: &[Score]);
}

/// Adapts either per-document combiner state or a final dense score buffer to batch output.
/// The update function stays concrete inside the batch loop.
pub(crate) struct CombiningOutput<'a, A, F> {
    base_doc: DocId,
    matches: &'a mut [TinySet],
    accumulators: &'a mut [A],
    update_score: F,
}

impl<'a, A, F: FnMut(&mut A, Score)> CombiningOutput<'a, A, F> {
    pub(crate) fn new(
        base_doc: DocId,
        matches: &'a mut [TinySet],
        accumulators: &'a mut [A],
        update_score: F,
    ) -> Self {
        assert!(accumulators.len() <= matches.len() * 64);
        Self {
            base_doc,
            matches,
            accumulators,
            update_score,
        }
    }
}

impl<A, F: FnMut(&mut A, Score)> ScoredBlockOutput for CombiningOutput<'_, A, F> {
    fn update(&mut self, batch: ScoredBatch<'_>) {
        match batch {
            ScoredBatch::Sparse { docs, scores } => {
                assert_eq!(docs.len(), scores.len());
                update_scored_bitset(
                    docs,
                    scores,
                    self.matches,
                    self.accumulators,
                    self.base_doc,
                    &mut self.update_score,
                );
            }
            ScoredBatch::Dense {
                base_doc,
                words,
                scores,
            } => {
                for (bucket, bits) in words.iter().enumerate() {
                    for bit in *bits {
                        let source_offset = bucket * 64 + bit as usize;
                        let doc = u64::from(base_doc) + source_offset as u64;
                        let offset = doc.checked_sub(u64::from(self.base_doc)).unwrap();
                        assert!(offset < self.accumulators.len() as u64);
                        let offset = offset as usize;
                        self.matches[offset / 64].insert_mut((offset % 64) as u32);
                        (self.update_score)(&mut self.accumulators[offset], scores[source_offset]);
                    }
                }
            }
        }
    }

    fn update_bitset(&mut self, base_doc: DocId, matches: &[TinySet], scores: &[Score]) {
        let mut scores = scores.iter();
        for (bucket, bits) in matches.iter().enumerate() {
            for bit in *bits {
                let doc = u64::from(base_doc) + bucket as u64 * 64 + u64::from(bit);
                let offset = doc.checked_sub(u64::from(self.base_doc)).unwrap();
                assert!(offset < self.accumulators.len() as u64);
                let offset = offset as usize;
                let score = *scores.next().expect("missing score for set bit");
                self.matches[offset / 64].insert_mut((offset % 64) as u32);
                (self.update_score)(&mut self.accumulators[offset], score);
            }
        }
        assert!(scores.next().is_none(), "more scores than set bits");
    }
}

#[cfg(test)]
mod tests {
    use common::TinySet;

    use super::{CombiningOutput, ScoredBatch, ScoredBlockOutput};
    use crate::query::{ScoreCombiner, SumCombiner};
    use crate::{Score, TERMINATED};

    #[test]
    fn test_mixed_batches_accumulate_without_clearing() {
        let mut matches = [TinySet::singleton(10); 3];
        let mut accumulators = [(10.0, 0); 140];
        let mut output = CombiningOutput::new(
            100,
            &mut matches,
            &mut accumulators,
            |(sum, count), score| {
                *sum += score;
                *count += 1;
            },
        );
        output.update(ScoredBatch::Sparse {
            docs: &[101, 164, 239],
            scores: &[1.0, 2.0, 0.0],
        });
        // Batch buckets are unaligned with the destination buckets. Include an empty bucket.
        output.update_bitset(
            101,
            &[
                TinySet::singleton(0).insert(63),
                TinySet::EMPTY,
                TinySet::singleton(10),
            ],
            &[3.0, 4.0, 0.0],
        );
        output.update(ScoredBatch::Sparse {
            docs: &[],
            scores: &[],
        });
        output.update_bitset(0, &[], &[]);
        for (offset, &(sum, count)) in accumulators.iter().enumerate() {
            let expected = match offset {
                1 => (14.0, 2),
                64 => (16.0, 2),
                139 => (10.0, 2),
                _ => (10.0, 0),
            };
            assert_eq!((sum, count), expected);
            assert_eq!(
                matches[offset / 64].contains((offset % 64) as u32),
                count > 0 || offset % 64 == 10
            );
        }
    }

    #[test]
    fn test_combiner_and_dense_outputs_match() {
        let mut dense_matches = [TinySet::EMPTY; 2];
        let mut combiner_matches = dense_matches;
        let mut scores = [0.0; 128];
        let mut combiners = [SumCombiner::default(); 128];
        let mut dense = CombiningOutput::new(
            7,
            &mut dense_matches,
            &mut scores,
            SumCombiner::update_dense_output,
        );
        let mut combined = CombiningOutput::new(
            7,
            &mut combiner_matches,
            &mut combiners,
            SumCombiner::update_score,
        );
        for output in [&mut dense as &mut dyn ScoredBlockOutput, &mut combined] {
            output.update(ScoredBatch::Sparse {
                docs: &[7, 70, 134],
                scores: &[0.0, 2.0, 3.0],
            });
            output.update_bitset(70, &[TinySet::singleton(0).insert(63)], &[4.0, 5.0]);
        }
        assert_eq!(dense_matches, combiner_matches);
        assert_eq!(scores, combiners.map(|combiner| combiner.score()));
        assert_eq!(
            (scores[0], scores[63], scores[126], scores[127]),
            (0.0, 6.0, 5.0, 3.0)
        );
        assert!(dense_matches[0].contains(0));
    }

    #[test]
    fn test_dense_and_sparse_batches_match() {
        for base_doc in [100, TERMINATED - 140] {
            let mut sparse_matches = [TinySet::singleton(10); 3];
            let mut dense_matches = sparse_matches;
            let mut sparse_scores = [10.0; 140];
            let mut dense_scores = sparse_scores;
            let mut words = [TinySet::EMPTY; 64];
            // Unaligned source base, empty bucket, and a partial destination window.
            words[0] = TinySet::singleton(0).insert(63);
            words[2] = TinySet::singleton(10);
            let mut scores = [Score::NAN; 4096];
            for first_score in [1.0, 5.0] {
                scores[0] = first_score;
                scores[63] = 2.0;
                scores[138] = 0.0;
                CombiningOutput::new(
                    base_doc,
                    &mut sparse_matches,
                    &mut sparse_scores,
                    |sum, score| {
                        *sum += score;
                    },
                )
                .update(ScoredBatch::Sparse {
                    docs: &[base_doc + 1, base_doc + 64, base_doc + 139],
                    scores: &[first_score, 2.0, 0.0],
                });
                CombiningOutput::new(
                    base_doc,
                    &mut dense_matches,
                    &mut dense_scores,
                    |sum, score| {
                        *sum += score;
                    },
                )
                .update(ScoredBatch::Dense {
                    base_doc: base_doc + 1,
                    words: &words,
                    scores: &scores,
                });
                assert_eq!(sparse_matches, dense_matches);
                assert_eq!(sparse_scores, dense_scores);
                assert!(dense_matches[2].contains(11)); // Zero score is still a match.
            }
            words.fill(TinySet::EMPTY);
            CombiningOutput::new(
                base_doc,
                &mut dense_matches,
                &mut dense_scores,
                |sum, score| {
                    *sum += score;
                },
            )
            .update(ScoredBatch::Dense {
                base_doc: 0,
                words: &words,
                scores: &scores,
            });
            assert_eq!(sparse_matches, dense_matches);
            assert_eq!(sparse_scores, dense_scores);
        }
    }

    #[test]
    fn test_bitset_batch_base_before_window_near_terminated() {
        let mut matches = [TinySet::EMPTY];
        let mut scores = [0.0; 3];
        CombiningOutput::new(TERMINATED - 3, &mut matches, &mut scores, |sum, score| {
            *sum += score;
        })
        .update_bitset(
            TERMINATED - 5,
            &[TinySet::singleton(2).insert(4)],
            &[1.0, 2.0],
        );
        assert_eq!(matches, [TinySet::singleton(0).insert(2)]);
        assert_eq!(scores, [1.0, 0.0, 2.0]);
    }
}
