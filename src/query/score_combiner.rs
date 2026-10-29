use crate::query::Scorer;
use crate::Score;

/// The `ScoreCombiner` trait defines how to compute
/// an overall score given a list of scores.
pub trait ScoreCombiner: Default + Clone + Send + Copy + 'static {
    /// Whether Block-WAND may drive this combiner.
    ///
    /// Block-WAND prunes blocks by comparing the *sum* of the per-term block
    /// maxima against the threshold, and scores the surviving documents the
    /// same way. That is only the right answer for a combiner that sums. A
    /// combiner aggregating any other way (dis_max takes the maximum) has to
    /// see every matching term, so it must not be pruned this way.
    const SUPPORTS_BLOCK_WAND: bool = false;

    /// Whether this combiner needs child scores while merging matches.
    const REQUIRES_SCORING: bool;

    /// Whether child scores can be combined directly in the dense output score buffer.
    ///
    /// Combining scores must require only a `Score` value per document.
    #[doc(hidden)]
    const ACCUMULATES_IN_DENSE_OUTPUT: bool = false;

    /// Combines a child score with the score in the dense output buffer.
    ///
    /// Callers may invoke this only when [`Self::ACCUMULATES_IN_DENSE_OUTPUT`] is true.
    #[doc(hidden)]
    fn update_dense_output(_output: &mut Score, _score: Score) {
        unreachable!("combiner does not accumulate directly in dense output")
    }

    /// Aggregates a precomputed score into this combiner.
    fn update_score(&mut self, score: Score);

    /// Aggregates the score combiner with the given scorer.
    ///
    /// The `ScoreCombiner` may decide to call `.scorer.score()`
    /// or not.
    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer);

    /// Clears the score combiner state back to its initial state.
    fn clear(&mut self);

    /// Returns the aggregate score.
    fn score(&self) -> Score;
}

/// Just ignores scores. The `DoNothingCombiner` does not
/// even call the scorers `.score()` function.
///
/// It is useful to optimize the case when scoring is disabled.
#[derive(Default, Clone, Copy)] //< these should not be too much work :)
pub struct DoNothingCombiner;

impl ScoreCombiner for DoNothingCombiner {
    const REQUIRES_SCORING: bool = false;

    fn update_score(&mut self, _score: Score) {}

    fn update<TScorer: Scorer>(&mut self, _scorer: &mut TScorer) {}

    fn clear(&mut self) {}

    #[inline]
    fn score(&self) -> Score {
        1.0
    }
}

/// Sums the score of different scorers.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct SumCombiner {
    score: Score,
}

impl ScoreCombiner for SumCombiner {
    const SUPPORTS_BLOCK_WAND: bool = true;
    const REQUIRES_SCORING: bool = true;
    const ACCUMULATES_IN_DENSE_OUTPUT: bool = true;

    #[inline]
    fn update_dense_output(output: &mut Score, score: Score) {
        *output += score;
    }

    fn update_score(&mut self, score: Score) {
        self.score += score;
    }

    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer) {
        self.update_score(scorer.score());
    }

    fn clear(&mut self) {
        self.score = 0.0;
    }

    #[inline]
    fn score(&self) -> Score {
        self.score
    }
}

/// Take max score of different scorers
/// and optionally sum it with other matches multiplied by `tie_breaker`
#[derive(Default, Clone, Copy)]
pub struct DisjunctionMaxCombiner {
    max: Score,
    sum: Score,
    tie_breaker: Score,
}

impl DisjunctionMaxCombiner {
    /// Creates `DisjunctionMaxCombiner` with tie breaker
    pub fn with_tie_breaker(tie_breaker: Score) -> DisjunctionMaxCombiner {
        DisjunctionMaxCombiner {
            max: 0.0,
            sum: 0.0,
            tie_breaker,
        }
    }
}

impl ScoreCombiner for DisjunctionMaxCombiner {
    const REQUIRES_SCORING: bool = true;

    fn update_score(&mut self, score: Score) {
        self.max = Score::max(score, self.max);
        self.sum += score;
    }

    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer) {
        self.update_score(scorer.score());
    }

    fn clear(&mut self) {
        self.max = 0.0;
        self.sum = 0.0;
    }

    #[inline]
    fn score(&self) -> Score {
        self.max + (self.sum - self.max) * self.tie_breaker
    }
}

#[cfg(test)]
mod tests {
    use super::{DisjunctionMaxCombiner, DoNothingCombiner, ScoreCombiner, SumCombiner};
    use crate::query::Scorer;
    use crate::{DocId, DocSet, Score, TERMINATED};

    struct FixedScorer(Score);

    impl DocSet for FixedScorer {
        fn advance(&mut self) -> DocId {
            TERMINATED
        }

        fn doc(&self) -> DocId {
            0
        }

        fn size_hint(&self) -> u32 {
            1
        }
    }

    impl Scorer for FixedScorer {
        fn score(&mut self) -> Score {
            self.0
        }
    }

    struct PanickingScorer;

    impl DocSet for PanickingScorer {
        fn advance(&mut self) -> DocId {
            TERMINATED
        }

        fn doc(&self) -> DocId {
            0
        }

        fn size_hint(&self) -> u32 {
            1
        }
    }

    impl Scorer for PanickingScorer {
        fn score(&mut self) -> Score {
            panic!("DoNothingCombiner must not score")
        }
    }

    #[test]
    fn test_sum_update_score_matches_scorer_update() {
        let scores = [1.0, 0.25, 3.5, 2.0];
        let mut from_scores = SumCombiner::default();
        let mut from_scorers = SumCombiner::default();
        for score in scores {
            from_scores.update_score(score);
            from_scorers.update(&mut FixedScorer(score));
        }
        assert_eq!(from_scores.score(), from_scorers.score());
    }

    #[test]
    fn test_dismax_update_score_matches_scorer_update() {
        let scores = [1.0, 0.25, 3.5, 2.0];
        let mut from_scores = DisjunctionMaxCombiner::with_tie_breaker(0.3);
        let mut from_scorers = DisjunctionMaxCombiner::with_tie_breaker(0.3);
        for score in scores {
            from_scores.update_score(score);
            from_scorers.update(&mut FixedScorer(score));
        }
        assert_eq!(from_scores.score(), from_scorers.score());
    }

    #[test]
    fn test_do_nothing_update_never_scores() {
        let mut combiner = DoNothingCombiner;
        combiner.update(&mut PanickingScorer);
        combiner.update_score(42.0);
        assert_eq!(combiner.score(), 1.0);
    }
}
