use std::ops::DerefMut;

use downcast_rs::impl_downcast;

use crate::docset::{DocSet, TERMINATED};
use crate::query::{ScoredBatch, ScoredBlockOutput};
use crate::{DocId, Score};

/// Maximum number of documents in one default sparse scored batch.
///
/// This is intentionally independent from the posting compression block size and boolean-query
/// window sizes.
pub const SCORE_BLOCK_LEN: usize = 2_048;

/// Scored set of documents matching a query within a specific segment.
///
/// See [`Query`](crate::query::Query).
pub trait Scorer: downcast_rs::Downcast + DocSet + 'static {
    /// Returns the score.
    ///
    /// This method will perform a bit of computation and is not cached.
    fn score(&mut self) -> Score;

    /// Emits scored batches for remaining documents in `[min_doc, horizon)`.
    ///
    /// Like [`DocSet::fill_bitset_block`], this seeks forward to `min_doc` if necessary.
    /// Submits each matching document exactly once, in increasing order, as document-ID or
    /// bitset batches. Zero-scored matches are included. The callback must cover the requested
    /// interval and owns initialization, combining, and clearing of its accumulators.
    ///
    /// Consumes the entire interval and returns the first remaining document at or beyond
    /// `horizon`, or [`TERMINATED`]. Scalar traversal can resume at that document.
    /// Boxed scorers forward this method to the underlying scorer's implementation.
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
            let mut len = 0;
            while len < SCORE_BLOCK_LEN && self.doc() < horizon {
                docs[len] = self.doc();
                scores[len] = self.score();
                len += 1;
                self.advance();
            }
            cb.update(ScoredBatch::Sparse {
                docs: &docs[..len],
                scores: &scores[..len],
            });
        }
        self.doc()
    }
}

impl_downcast!(Scorer);

impl Scorer for Box<dyn Scorer> {
    #[inline]
    fn score(&mut self) -> Score {
        self.deref_mut().score()
    }

    #[inline]
    fn emit_scored_batches(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        cb: &mut dyn ScoredBlockOutput,
    ) -> DocId {
        self.deref_mut().emit_scored_batches(min_doc, horizon, cb)
    }
}

#[cfg(test)]
pub(crate) fn collect_scored_batches(
    scorer: &mut dyn Scorer,
    min_doc: DocId,
    horizon: DocId,
) -> Vec<(DocId, Score)> {
    struct Collector(Vec<(DocId, Score)>);

    impl ScoredBlockOutput for Collector {
        fn update(&mut self, batch: ScoredBatch<'_>) {
            match batch {
                ScoredBatch::Sparse { docs, scores } => {
                    self.0
                        .extend(docs.iter().copied().zip(scores.iter().copied()));
                }
                ScoredBatch::Dense {
                    base_doc,
                    words,
                    scores,
                } => {
                    for (bucket, bits) in words.iter().enumerate() {
                        for bit in *bits {
                            let offset = bucket * 64 + bit as usize;
                            self.0.push((base_doc + offset as DocId, scores[offset]));
                        }
                    }
                }
            }
        }

        fn update_bitset(
            &mut self,
            _base_doc: DocId,
            _matches: &[common::TinySet],
            _scores: &[Score],
        ) {
            unreachable!("scorers emit ScoredBatch values");
        }
    }

    let mut collector = Collector(Vec::new());
    scorer.emit_scored_batches(min_doc, horizon, &mut collector);
    collector.0
}

#[cfg(test)]
mod tests {
    use common::TinySet;

    use super::{collect_scored_batches, Scorer, SCORE_BLOCK_LEN};
    use crate::query::scored_block_output::CombiningOutput;
    use crate::query::AllScorer;
    use crate::{DocId, DocSet, Score, TERMINATED};

    struct TestScorer {
        docs: Vec<DocId>,
        cursor: usize,
    }

    impl TestScorer {
        fn new(docs: Vec<DocId>) -> Self {
            Self { docs, cursor: 0 }
        }
    }

    impl DocSet for TestScorer {
        fn advance(&mut self) -> DocId {
            if self.cursor < self.docs.len() {
                self.cursor += 1;
            }
            self.doc()
        }

        fn doc(&self) -> DocId {
            self.docs.get(self.cursor).copied().unwrap_or(TERMINATED)
        }

        fn size_hint(&self) -> u32 {
            self.docs.len() as u32
        }
    }

    impl Scorer for TestScorer {
        fn score(&mut self) -> Score {
            self.doc() as Score + 0.5
        }
    }

    #[test]
    fn test_default_scored_bitset_fill_through_box() {
        let mut scorer: Box<dyn Scorer> = Box::new(AllScorer::new(6));
        let mut matches = [TinySet::EMPTY];
        let mut scores = [0.0; 4];
        assert_eq!(
            scorer.emit_scored_batches(
                0,
                4,
                &mut CombiningOutput::new(0, &mut matches, &mut scores, |sum, score| *sum += score),
            ),
            4
        );
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(scores, [1.0; 4]);
        assert_eq!(scorer.doc(), 4);
        assert_eq!(scorer.score(), 1.0);

        matches.fill(TinySet::EMPTY);
        scores.fill(0.0);
        assert_eq!(
            scorer.emit_scored_batches(
                4,
                8,
                &mut CombiningOutput::new(4, &mut matches, &mut scores, |sum, score| *sum += score),
            ),
            TERMINATED
        );
        assert_eq!(matches[0].into_iter().collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(scores, [1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_default_scored_batches_consume_and_align() {
        let expected_docs: Vec<DocId> = (0..SCORE_BLOCK_LEN as DocId + 3).collect();
        let mut scorer = TestScorer::new(expected_docs.clone());
        let scored = collect_scored_batches(&mut scorer, 0, TERMINATED);
        assert_eq!(
            scored,
            expected_docs
                .iter()
                .map(|&doc| (doc, doc as Score + 0.5))
                .collect::<Vec<_>>()
        );
        assert_eq!(scorer.doc(), TERMINATED);
    }

    #[test]
    fn test_default_scored_batches_horizon_is_exclusive() {
        let mut scorer = TestScorer::new(vec![1, 3, 5, 8]);
        assert_eq!(
            collect_scored_batches(&mut scorer, 0, 5),
            vec![(1, 1.5), (3, 3.5)]
        );
        assert_eq!(scorer.doc(), 5);
        assert!(collect_scored_batches(&mut scorer, 0, 5).is_empty());
        assert_eq!(
            collect_scored_batches(&mut scorer, 5, TERMINATED),
            vec![(5, 5.5), (8, 8.5)]
        );
    }

    #[test]
    fn test_scalar_and_batch_calls_can_be_interleaved_through_box() {
        let mut scorer: Box<dyn Scorer> = Box::new(TestScorer::new(vec![2, 4, 9]));
        assert_eq!(scorer.score(), 2.5);
        assert_eq!(scorer.advance(), 4);
        assert_eq!(
            collect_scored_batches(scorer.as_mut(), 0, TERMINATED),
            vec![(4, 4.5), (9, 9.5)]
        );
        assert_eq!(scorer.doc(), TERMINATED);
    }
}
