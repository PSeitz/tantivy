use common::TinySet;

use crate::postings::update_scored_bitset;
use crate::{DocId, Score};

/// Receives batches of matches and score contributions from a [`super::Scorer`].
///
/// The receiver owns the destination window and accumulation rule. Updates preserve existing
/// contributions and record matches even when their score is zero. Each submitted document must
/// belong to the receiver's window. Neither batch representation requires an allocation.
pub trait ScoredBlockOutput {
    /// Adds a strictly increasing slice of document IDs with one score per document.
    fn update_docs(&mut self, docs: &[DocId], scores: &[Score]);

    /// Adds matches represented by bitsets, with bit zero corresponding to `base_doc`.
    ///
    /// Scores are packed: one per set bit, ordered by bucket and then increasing bit position.
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
    fn update_docs(&mut self, docs: &[DocId], scores: &[Score]) {
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

    use super::{CombiningOutput, ScoredBlockOutput};
    use crate::query::{ScoreCombiner, SumCombiner};
    use crate::TERMINATED;

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
        output.update_docs(&[101, 164, 239], &[1.0, 2.0, 0.0]);
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
        output.update_docs(&[], &[]);
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
            output.update_docs(&[7, 70, 134], &[0.0, 2.0, 3.0]);
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
