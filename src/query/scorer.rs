use std::ops::{DerefMut, Range};

use downcast_rs::impl_downcast;

use crate::docset::{DocSet, TERMINATED};
use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
use crate::query::ScoredBlockOutput;
use crate::{DocId, Score};

/// Number of documents scored by one scored-buffer operation.
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

    /// Fills parallel document and score buffers, consuming the returned documents.
    ///
    /// The current document is included. Returned documents are strictly increasing and each
    /// score is aligned with the document at the same index. On return, [`DocSet::doc`] points to
    /// the first unreturned document, or [`TERMINATED`] if the scorer was exhausted.
    fn fill_scored_buffer(
        &mut self,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        self.fill_scored_buffer_up_to(TERMINATED, docs, scores)
    }

    /// Fills parallel document and score buffers with documents strictly below `horizon`.
    ///
    /// `horizon` is exclusive. A document greater than or equal to it is not consumed.
    fn fill_scored_buffer_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        if self.doc() == TERMINATED {
            return 0;
        }
        for i in 0..SCORE_BLOCK_LEN {
            let doc = self.doc();
            if doc >= horizon {
                return i;
            }
            docs[i] = doc;
            scores[i] = self.score();
            if self.advance() == TERMINATED {
                return i + 1;
            }
        }
        SCORE_BLOCK_LEN
    }

    /// Adds matches and score contributions for remaining documents in `[min_doc, horizon)`.
    ///
    /// Like [`DocSet::fill_bitset_block`], this seeks forward to `min_doc` if necessary.
    /// Submits each matching document exactly once, in increasing order, as document-ID or
    /// bitset batches. Zero-scored matches are included. The output must cover the requested
    /// interval and owns initialization, combining, and clearing of its accumulators.
    ///
    /// Consumes the entire interval and returns the first remaining document at or beyond
    /// `horizon`, or [`TERMINATED`]. Scalar traversal can resume at that document.
    /// Boxed scorers forward this method to the underlying scorer's implementation.
    fn fill_scored_bitset_block(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        output: &mut dyn ScoredBlockOutput,
    ) -> DocId {
        assert!(min_doc <= horizon && horizon <= TERMINATED);
        if self.doc() < min_doc {
            self.seek(min_doc);
        }
        let mut docs = [0; COMPRESSION_BLOCK_SIZE];
        let mut scores = [0.0; COMPRESSION_BLOCK_SIZE];
        while self.doc() < horizon {
            let range = self.fill_scored_buffer_range_up_to(horizon, &mut docs, &mut scores);
            output.update_docs(&docs[range.clone()], &scores[range]);
        }
        self.doc()
    }

    /// Fills arbitrary-sized document and score buffers through an exclusive horizon.
    ///
    /// The returned range identifies the consumed documents. Implementations may decode outside
    /// that range, but documents at or beyond `horizon` remain unconsumed.
    fn fill_scored_buffer_range_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId],
        scores: &mut [Score],
    ) -> Range<usize> {
        assert_eq!(docs.len(), scores.len());
        if self.doc() == TERMINATED {
            return 0..0;
        }
        for i in 0..docs.len() {
            let doc = self.doc();
            if doc >= horizon {
                return 0..i;
            }
            docs[i] = doc;
            scores[i] = self.score();
            if self.advance() == TERMINATED {
                return 0..i + 1;
            }
        }
        0..docs.len()
    }
}

impl_downcast!(Scorer);

impl Scorer for Box<dyn Scorer> {
    #[inline]
    fn score(&mut self) -> Score {
        self.deref_mut().score()
    }

    #[inline]
    fn fill_scored_buffer(
        &mut self,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        self.deref_mut().fill_scored_buffer(docs, scores)
    }

    #[inline]
    fn fill_scored_buffer_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        self.deref_mut()
            .fill_scored_buffer_up_to(horizon, docs, scores)
    }

    #[inline]
    fn fill_scored_bitset_block(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        output: &mut dyn ScoredBlockOutput,
    ) -> DocId {
        self.deref_mut()
            .fill_scored_bitset_block(min_doc, horizon, output)
    }

    #[inline]
    fn fill_scored_buffer_range_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId],
        scores: &mut [Score],
    ) -> Range<usize> {
        self.deref_mut()
            .fill_scored_buffer_range_up_to(horizon, docs, scores)
    }
}

#[cfg(test)]
mod tests {
    use common::TinySet;

    use super::{Scorer, SCORE_BLOCK_LEN};
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
            scorer.fill_scored_bitset_block(
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
            scorer.fill_scored_bitset_block(
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
    fn test_default_fill_scored_buffer_consumes_and_aligns() {
        let expected_docs: Vec<DocId> = (0..SCORE_BLOCK_LEN as DocId + 3).collect();
        let mut scorer = TestScorer::new(expected_docs.clone());
        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];

        assert_eq!(
            scorer.fill_scored_buffer(&mut docs, &mut scores),
            SCORE_BLOCK_LEN
        );
        assert_eq!(&docs[..], &expected_docs[..SCORE_BLOCK_LEN]);
        for (&doc, &score) in docs.iter().zip(&scores) {
            assert_eq!(score, doc as Score + 0.5);
        }
        assert_eq!(scorer.doc(), SCORE_BLOCK_LEN as DocId);

        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 3);
        assert_eq!(&docs[..3], &expected_docs[SCORE_BLOCK_LEN..]);
        assert_eq!(scorer.doc(), TERMINATED);
        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 0);
    }

    #[test]
    fn test_default_fill_scored_buffer_up_to_is_exclusive() {
        let mut scorer = TestScorer::new(vec![1, 3, 5, 8]);
        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];

        assert_eq!(
            scorer.fill_scored_buffer_up_to(5, &mut docs, &mut scores),
            2
        );
        assert_eq!(&docs[..2], &[1, 3]);
        assert_eq!(&scores[..2], &[1.5, 3.5]);
        assert_eq!(scorer.doc(), 5);
        assert_eq!(
            scorer.fill_scored_buffer_up_to(5, &mut docs, &mut scores),
            0
        );
        assert_eq!(scorer.doc(), 5);
        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 2);
        assert_eq!(&docs[..2], &[5, 8]);
        assert_eq!(scorer.doc(), TERMINATED);
    }

    #[test]
    fn test_scalar_and_block_calls_can_be_interleaved_through_box() {
        let mut scorer: Box<dyn Scorer> = Box::new(TestScorer::new(vec![2, 4, 9]));
        assert_eq!(scorer.score(), 2.5);
        assert_eq!(scorer.advance(), 4);

        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];
        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 2);
        assert_eq!(&docs[..2], &[4, 9]);
        assert_eq!(&scores[..2], &[4.5, 9.5]);
        assert_eq!(scorer.doc(), TERMINATED);
    }
}
