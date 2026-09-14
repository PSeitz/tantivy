use crate::docset::{DocSet, COLLECT_BLOCK_BUFFER_LEN, TERMINATED};
use crate::index::SegmentReader;
use crate::query::boost_query::BoostScorer;
use crate::query::explanation::does_not_match;
use crate::query::{
    EnableScoring, Explanation, Query, ScoredBatch, ScoredBlockOutput, Scorer, Weight,
    SCORE_BLOCK_LEN,
};
use crate::{DocId, Score};

/// Query that matches all of the documents.
///
/// All of the documents get the score 1.0.
#[derive(Clone, Debug)]
pub struct AllQuery;

impl Query for AllQuery {
    fn weight(&self, _: EnableScoring<'_>) -> crate::Result<Box<dyn Weight>> {
        Ok(Box::new(AllWeight))
    }
}

/// Weight associated with the `AllQuery` query.
pub struct AllWeight;

impl Weight for AllWeight {
    fn scorer(&self, reader: &SegmentReader, boost: Score) -> crate::Result<Box<dyn Scorer>> {
        let all_scorer = AllScorer::new(reader.max_doc());
        if boost != 1.0 {
            Ok(Box::new(BoostScorer::new(all_scorer, boost)))
        } else {
            Ok(Box::new(all_scorer))
        }
    }

    fn explain(&self, reader: &SegmentReader, doc: DocId) -> crate::Result<Explanation> {
        if doc >= reader.max_doc() {
            return Err(does_not_match(doc));
        }
        Ok(Explanation::new("AllQuery", 1.0))
    }
}

/// Scorer associated with the `AllQuery` query.
pub struct AllScorer {
    doc: DocId,
    max_doc: DocId,
}

impl AllScorer {
    /// Creates a new AllScorer with `max_doc` docs.
    pub fn new(max_doc: DocId) -> AllScorer {
        let doc = if max_doc == 0 { TERMINATED } else { 0 };
        AllScorer { doc, max_doc }
    }

    fn fill_compact_scored_buffer_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        if self.doc == TERMINATED || self.doc >= horizon {
            return 0;
        }
        let end = self
            .doc
            .saturating_add(SCORE_BLOCK_LEN as DocId)
            .min(self.max_doc)
            .min(horizon);
        let len = (end - self.doc) as usize;
        for (offset, output_doc) in docs[..len].iter_mut().enumerate() {
            *output_doc = self.doc + offset as DocId;
        }
        scores[..len].fill(1.0);
        self.doc = if end == self.max_doc { TERMINATED } else { end };
        len
    }
}

impl DocSet for AllScorer {
    #[inline(always)]
    fn advance(&mut self) -> DocId {
        if self.doc == TERMINATED || self.doc + 1 >= self.max_doc {
            self.doc = TERMINATED;
            return TERMINATED;
        }
        self.doc += 1;
        self.doc
    }

    fn seek(&mut self, target: DocId) -> DocId {
        debug_assert!(target >= self.doc);
        self.doc = target;
        if self.doc >= self.max_doc {
            self.doc = TERMINATED;
        }
        self.doc
    }

    fn fill_buffer(&mut self, buffer: &mut [DocId; COLLECT_BLOCK_BUFFER_LEN]) -> usize {
        if self.doc() == TERMINATED {
            return 0;
        }
        let is_safe_distance = self.doc() + (buffer.len() as u32) < self.max_doc;
        if is_safe_distance {
            let num_items = buffer.len();
            for buffer_val in buffer {
                *buffer_val = self.doc();
                self.doc += 1;
            }
            num_items
        } else {
            for (i, buffer_val) in buffer.iter_mut().enumerate() {
                *buffer_val = self.doc();
                if self.advance() == TERMINATED {
                    return i + 1;
                }
            }
            buffer.len()
        }
    }

    #[inline(always)]
    fn doc(&self) -> DocId {
        self.doc
    }

    fn size_hint(&self) -> u32 {
        self.max_doc
    }
}

impl Scorer for AllScorer {
    #[inline]
    fn score(&mut self) -> Score {
        1.0
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
            let len = self.fill_compact_scored_buffer_up_to(horizon, &mut docs, &mut scores);
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
    use super::AllQuery;
    use crate::docset::{DocSet, COLLECT_BLOCK_BUFFER_LEN, TERMINATED};
    use crate::query::scorer::collect_scored_batches;
    use crate::query::{AllScorer, EnableScoring, Query, Scorer, SCORE_BLOCK_LEN};
    use crate::schema::{Schema, TEXT};
    use crate::{DocId, Index, IndexWriter};

    fn create_test_index() -> crate::Result<Index> {
        let mut schema_builder = Schema::builder();
        let field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;
        index_writer.add_document(doc!(field=>"aaa"))?;
        index_writer.add_document(doc!(field=>"bbb"))?;
        index_writer.commit()?;
        index_writer.add_document(doc!(field=>"ccc"))?;
        index_writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_all_query() -> crate::Result<()> {
        let index = create_test_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let weight = AllQuery.weight(EnableScoring::disabled_from_schema(&index.schema()))?;
        {
            let reader = searcher.segment_reader(0);
            let mut scorer = weight.scorer(reader, 1.0)?;
            assert_eq!(scorer.doc(), 0u32);
            assert_eq!(scorer.advance(), 1u32);
            assert_eq!(scorer.doc(), 1u32);
            assert_eq!(scorer.advance(), TERMINATED);
        }
        {
            let reader = searcher.segment_reader(1);
            let mut scorer = weight.scorer(reader, 1.0)?;
            assert_eq!(scorer.doc(), 0u32);
            assert_eq!(scorer.advance(), TERMINATED);
        }
        Ok(())
    }

    #[test]
    fn test_all_query_with_boost() -> crate::Result<()> {
        let index = create_test_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let weight = AllQuery.weight(EnableScoring::disabled_from_schema(searcher.schema()))?;
        let reader = searcher.segment_reader(0);
        {
            let mut scorer = weight.scorer(reader, 2.0)?;
            assert_eq!(scorer.doc(), 0u32);
            assert_eq!(scorer.score(), 2.0);
        }
        {
            let mut scorer = weight.scorer(reader, 1.5)?;
            assert_eq!(scorer.doc(), 0u32);
            assert_eq!(scorer.score(), 1.5);
        }
        Ok(())
    }

    #[test]
    fn test_scored_batches_with_horizon_and_partial_block() {
        let mut scorer = AllScorer::new(SCORE_BLOCK_LEN as u32 + 6);
        assert_eq!(
            collect_scored_batches(&mut scorer, 0, 3),
            vec![(0, 1.0), (1, 1.0), (2, 1.0)]
        );
        assert_eq!(scorer.doc(), 3);
        let remaining = collect_scored_batches(&mut scorer, 3, TERMINATED);
        assert_eq!(remaining.len(), SCORE_BLOCK_LEN + 3);
        assert_eq!(remaining.first(), Some(&(3, 1.0)));
        assert_eq!(remaining.last(), Some(&(SCORE_BLOCK_LEN as DocId + 5, 1.0)));
        assert_eq!(scorer.doc(), TERMINATED);

        let mut empty = AllScorer::new(0);
        assert!(collect_scored_batches(&mut empty, 0, TERMINATED).is_empty());
    }

    #[test]
    pub fn test_fill_buffer() {
        let mut postings = AllScorer {
            doc: 0u32,
            max_doc: COLLECT_BLOCK_BUFFER_LEN as u32 * 2 + 9,
        };
        let mut buffer = [0u32; COLLECT_BLOCK_BUFFER_LEN];
        assert_eq!(postings.fill_buffer(&mut buffer), COLLECT_BLOCK_BUFFER_LEN);
        for i in 0u32..COLLECT_BLOCK_BUFFER_LEN as u32 {
            assert_eq!(buffer[i as usize], i);
        }
        assert_eq!(postings.fill_buffer(&mut buffer), COLLECT_BLOCK_BUFFER_LEN);
        for i in 0u32..COLLECT_BLOCK_BUFFER_LEN as u32 {
            assert_eq!(buffer[i as usize], i + COLLECT_BLOCK_BUFFER_LEN as u32);
        }
        assert_eq!(postings.fill_buffer(&mut buffer), 9);
    }
}
