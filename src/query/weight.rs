use std::cmp::Ordering;

use common::TinySet;
use downcast_rs::{impl_downcast, Downcast};

use super::{
    BufferedUnionScorer, ScoreCombiner, ScoredBatch, ScoredBlockOutput, Scorer, SCORE_BLOCK_LEN,
};
use crate::docset::{SeekDangerResult, COLLECT_BLOCK_BUFFER_LEN};
use crate::index::SegmentReader;
use crate::query::Explanation;
use crate::{DocId, DocSet, Score, TERMINATED};

/// Iterates through all of the documents and scores matched by the DocSet
/// `DocSet`.
pub(crate) fn for_each_scorer<TScorer: Scorer + ?Sized>(
    scorer: &mut TScorer,
    callback: &mut dyn FnMut(DocId, Score),
) {
    let mut doc = scorer.doc();
    while doc != TERMINATED {
        callback(doc, scorer.score());
        doc = scorer.advance();
    }
}

/// Iterates through aligned blocks of scored documents.
pub(crate) fn for_each_scorer_scored_block<TScorer: Scorer + ?Sized>(
    scorer: &mut TScorer,
    callback: &mut dyn FnMut(&[DocId], &[Score]),
) {
    struct CompactOutput<'a> {
        callback: &'a mut dyn FnMut(&[DocId], &[Score]),
        docs: Vec<DocId>,
        scores: Vec<Score>,
    }

    impl ScoredBlockOutput for CompactOutput<'_> {
        fn update(&mut self, batch: ScoredBatch<'_>) {
            match batch {
                ScoredBatch::Sparse { docs, scores } => (self.callback)(docs, scores),
                ScoredBatch::Dense {
                    base_doc,
                    words,
                    scores,
                } => {
                    self.docs.clear();
                    self.scores.clear();
                    for (bucket, bits) in words.iter().enumerate() {
                        for bit in *bits {
                            let offset = bucket * 64 + bit as usize;
                            self.docs.push(base_doc + offset as DocId);
                            self.scores.push(scores[offset]);
                        }
                    }
                    (self.callback)(&self.docs, &self.scores);
                }
            }
        }

        fn update_bitset(&mut self, _base_doc: DocId, _matches: &[TinySet], _scores: &[Score]) {
            unreachable!("scorers emit ScoredBatch values");
        }
    }

    scorer.emit_scored_batches(
        0,
        TERMINATED,
        &mut CompactOutput {
            callback,
            docs: Vec::new(),
            scores: Vec::new(),
        },
    );
}

/// Compacts buffered union windows into parallel document and score buffers.
pub(crate) fn for_each_buffered_scored_block<TScorer: Scorer, TScoreCombiner: ScoreCombiner>(
    scorer: &mut BufferedUnionScorer<TScorer, TScoreCombiner>,
    max_doc: DocId,
    callback: &mut dyn FnMut(&[DocId], &[Score]),
) {
    let mut matches = [TinySet::EMPTY; 64];
    let mut dense_scores = [0.0; 4096];
    let mut docs = Vec::with_capacity(SCORE_BLOCK_LEN);
    let mut scores = Vec::with_capacity(SCORE_BLOCK_LEN);
    let mut start = 0;
    while start < max_doc {
        let end = start.saturating_add(SCORE_BLOCK_LEN as DocId).min(max_doc);
        let next = scorer.fill_scored_window(start..end, &mut matches, &mut dense_scores);
        for (bucket, bits) in matches.iter().enumerate() {
            for bit in *bits {
                let offset = bucket * 64 + bit as usize;
                docs.push(start + offset as DocId);
                scores.push(dense_scores[offset]);
            }
        }
        if !docs.is_empty() {
            callback(&docs, &scores);
            docs.clear();
            scores.clear();
        }
        if next == TERMINATED {
            break;
        }
        start = next;
    }
}

/// Iterates through all of the documents matched by the DocSet
/// `DocSet`.
#[inline]
pub(crate) fn for_each_docset_buffered<T: DocSet + ?Sized>(
    docset: &mut T,
    buffer: &mut [DocId; COLLECT_BLOCK_BUFFER_LEN],
    mut callback: impl FnMut(&[DocId]),
) {
    loop {
        let num_items = docset.fill_buffer(buffer);
        callback(&buffer[..num_items]);
        if num_items != buffer.len() {
            break;
        }
    }
}

/// Calls `callback` with all of the `(doc, score)` for which score
/// is exceeding a given threshold.
///
/// This method is useful for the [`TopDocs`](crate::collector::TopDocs) collector.
/// For all docsets, the blanket implementation has the benefit
/// of prefiltering (doc, score) pairs, avoiding the
/// virtual dispatch cost.
///
/// More importantly, it makes it possible for scorers to implement
/// important optimization (e.g. BlockWAND for union).
pub(crate) fn for_each_pruning_scorer<TScorer: Scorer + ?Sized>(
    scorer: &mut TScorer,
    mut threshold: Score,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
) {
    let mut doc = scorer.doc();
    while doc != TERMINATED {
        let score = scorer.score();
        if score > threshold {
            threshold = callback(doc, score);
        }
        doc = scorer.advance();
    }
}

/// Receives scored matches in either their conventional compact form or as a dense
/// 4,096-document window.
///
/// This trait is an internal traversal contract. The callback borrows its input only for the
/// duration of the call. Producers retain ownership of buffer reset and reuse afterwards.
#[doc(hidden)]
pub trait ScoredBatchConsumer {
    fn consume(&mut self, batch: ScoredBatch<'_>);
}

pub(crate) fn for_each_dense_scored_windows<TScorer: Scorer, TScoreCombiner: ScoreCombiner>(
    scorer: &mut BufferedUnionScorer<TScorer, TScoreCombiner>,
    max_doc: DocId,
    consumer: &mut dyn ScoredBatchConsumer,
) {
    if max_doc == 0 {
        return;
    }

    let mut matches = [TinySet::EMPTY; 64];
    let mut dense_scores = [0.0; 4096];
    let mut start = 0;
    while start < max_doc {
        let end = start.saturating_add(4096).min(max_doc);
        let next = scorer.fill_scored_window(start..end, &mut matches, &mut dense_scores);
        if matches.iter().any(|tinyset| !tinyset.is_empty()) {
            consumer.consume(ScoredBatch::Dense {
                base_doc: start,
                words: &matches,
                scores: &dense_scores,
            });
        }
        if next == TERMINATED {
            break;
        }
        start = next;
    }
}

/// A Weight is the specialization of a `Query`
/// for a given set of segments.
///
/// See [`Query`](crate::query::Query).
pub trait Weight: Downcast + Send + Sync + 'static {
    /// Returns the scorer for the given segment.
    ///
    /// `boost` is a multiplier to apply to the score.
    ///
    /// See [`Query`](crate::query::Query).
    fn scorer(&self, reader: &SegmentReader, boost: Score) -> crate::Result<Box<dyn Scorer>>;

    /// !!!Dragons ahead!!!
    /// Returns a scorer initialized around `target` for use in an intersection.
    ///
    /// # Warning
    ///
    /// The result and the state of the returned scorer follow the semantics of
    /// [`DocSet::seek_danger`]. When this method returns
    /// [`SeekDangerResult::SeekLowerBound`], the scorer may be in an invalid state and must only
    /// receive further calls to [`DocSet::seek_danger`] until one returns
    /// [`SeekDangerResult::Found`].
    ///
    /// Some scorer types require substantial CPU work to locate their first matching document.
    /// During intersection initialization, this hook lets them start near the current candidate
    /// instead.
    fn scorer_danger(
        &self,
        reader: &SegmentReader,
        target: DocId,
        boost: Score,
    ) -> crate::Result<(SeekDangerResult, Box<dyn Scorer>)> {
        let mut scorer = self.scorer(reader, boost)?;
        let scorer_doc = scorer.doc();
        let seek_danger_result = match scorer_doc.cmp(&target) {
            Ordering::Less => scorer.seek_danger(target),
            Ordering::Equal => SeekDangerResult::Found,
            Ordering::Greater => SeekDangerResult::SeekLowerBound(scorer_doc),
        };
        Ok((seek_danger_result, scorer))
    }

    /// Returns an [`Explanation`] for the given document.
    fn explain(&self, reader: &SegmentReader, doc: DocId) -> crate::Result<Explanation>;

    /// Returns the number documents within the given [`SegmentReader`].
    fn count(&self, reader: &SegmentReader) -> crate::Result<u32> {
        let mut scorer = self.scorer(reader, 1.0)?;
        if let Some(alive_bitset) = reader.alive_bitset() {
            Ok(scorer.count(alive_bitset))
        } else {
            Ok(scorer.count_including_deleted())
        }
    }

    /// Iterates through all of the document matched by the DocSet
    /// `DocSet` and push the scored documents to the collector.
    fn for_each(
        &self,
        reader: &SegmentReader,
        callback: &mut dyn FnMut(DocId, Score),
    ) -> crate::Result<()> {
        let mut scorer = self.scorer(reader, 1.0)?;
        for_each_scorer(scorer.as_mut(), callback);
        Ok(())
    }

    /// Iterates through all of the document matched by the DocSet
    /// `DocSet` and push the scored documents to the collector.
    fn for_each_scored_block(
        &self,
        reader: &SegmentReader,
        callback: &mut dyn FnMut(&[DocId], &[Score]),
    ) -> crate::Result<()> {
        let mut scorer = self.scorer(reader, 1.0)?;
        for_each_scorer_scored_block(scorer.as_mut(), callback);
        Ok(())
    }

    /// Iterates through scored matches using the representation naturally produced by the query.
    ///
    /// The default implementation emits compact blocks. Queries backed by a dense window may
    /// override this method without changing the collector-facing traversal API.
    #[doc(hidden)]
    fn for_each_scored_batch(
        &self,
        reader: &SegmentReader,
        consumer: &mut dyn ScoredBatchConsumer,
    ) -> crate::Result<()> {
        self.for_each_scored_block(reader, &mut |docs, scores| {
            consumer.consume(ScoredBatch::Sparse { docs, scores });
        })
    }

    /// Iterates through blocks of matching document IDs without computing scores.
    fn for_each_no_score(
        &self,
        reader: &SegmentReader,
        callback: &mut dyn FnMut(&[DocId]),
    ) -> crate::Result<()> {
        let mut docset = self.scorer(reader, 1.0)?;

        let mut buffer = [0u32; COLLECT_BLOCK_BUFFER_LEN];
        for_each_docset_buffered(&mut docset, &mut buffer, callback);
        Ok(())
    }

    /// Calls `callback` with all of the `(doc, score)` for which score
    /// is exceeding a given threshold.
    ///
    /// This method is useful for the [`TopDocs`](crate::collector::TopDocs) collector.
    /// For all docsets, the blanket implementation has the benefit
    /// of prefiltering (doc, score) pairs, avoiding the
    /// virtual dispatch cost.
    ///
    /// More importantly, it makes it possible for scorers to implement
    /// important optimization (e.g. BlockWAND for union).
    fn for_each_pruning(
        &self,
        threshold: Score,
        reader: &SegmentReader,
        callback: &mut dyn FnMut(DocId, Score) -> Score,
    ) -> crate::Result<()> {
        let mut scorer = self.scorer(reader, 1.0)?;
        for_each_pruning_scorer(scorer.as_mut(), threshold, callback);
        Ok(())
    }
}

impl_downcast!(Weight);
