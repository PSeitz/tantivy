use std::ops::Range;

use common::TinySet;

use crate::docset::DocSet;
use crate::fieldnorm::FieldNormReader;
use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
use crate::postings::{BlockSegmentPostings, FreqReadingOption, Postings, SegmentPostings};
use crate::query::bm25::Bm25Weight;
use crate::query::{Explanation, ScoredBlockOutput, Scorer, SCORE_BLOCK_LEN};
use crate::{DocId, Score};

#[derive(Clone)]
pub struct TermScorer {
    postings: SegmentPostings,
    fieldnorm_reader: FieldNormReader,
    similarity_weight: Bm25Weight,
}

impl TermScorer {
    pub fn new(
        postings: SegmentPostings,
        fieldnorm_reader: FieldNormReader,
        similarity_weight: Bm25Weight,
    ) -> TermScorer {
        TermScorer {
            postings,
            fieldnorm_reader,
            similarity_weight,
        }
    }

    pub(crate) fn seek_block(&mut self, target_doc: DocId) {
        self.postings.block_cursor.seek_block(target_doc);
    }

    #[cfg(test)]
    pub fn create_for_test(
        doc_and_tfs: &[(DocId, u32)],
        fieldnorms: &[u32],
        similarity_weight: Bm25Weight,
    ) -> TermScorer {
        assert!(!doc_and_tfs.is_empty());
        assert!(
            doc_and_tfs
                .iter()
                .map(|(doc, _tf)| *doc)
                .max()
                .unwrap_or(0u32)
                < fieldnorms.len() as u32
        );
        let segment_postings =
            SegmentPostings::create_from_docs_and_tfs(doc_and_tfs, Some(fieldnorms));
        let fieldnorm_reader = FieldNormReader::for_test(fieldnorms);
        TermScorer::new(segment_postings, fieldnorm_reader, similarity_weight)
    }

    /// See `FreqReadingOption`.
    pub(crate) fn freq_reading_option(&self) -> FreqReadingOption {
        self.postings.block_cursor.freq_reading_option()
    }

    /// Returns the maximum score for the current block.
    ///
    /// In some rare case, the result may not be exact. In this case a lower value is returned,
    /// (and may lead us to return a lesser document).
    ///
    /// At index time, we store the (fieldnorm_id, term frequency) pair that maximizes the
    /// score assuming the average fieldnorm computed on this segment.
    ///
    /// Though extremely rare, it is theoretically possible that the actual average fieldnorm
    /// is different enough from the current segment average fieldnorm that the maximum over a
    /// specific is achieved on a different document.
    ///
    /// (The result is on the other hand guaranteed to be correct if there is only one segment).
    pub fn block_max_score(&mut self) -> Score {
        self.postings
            .block_cursor
            .block_max_score(&self.fieldnorm_reader, &self.similarity_weight)
    }

    pub fn term_freq(&self) -> u32 {
        self.postings.term_freq()
    }

    pub fn fieldnorm_id(&self) -> u8 {
        self.fieldnorm_reader.fieldnorm_id(self.doc())
    }

    pub fn explain(&self) -> Explanation {
        let fieldnorm_id = self.fieldnorm_id();
        let term_freq = self.term_freq();
        self.similarity_weight.explain(fieldnorm_id, term_freq)
    }

    pub fn max_score(&self) -> Score {
        self.similarity_weight.max_score()
    }

    pub fn last_doc_in_block(&self) -> DocId {
        self.postings.block_cursor.skip_reader().last_doc_in_block()
    }

    /// Returns a mutable reference to the underlying block cursor.
    pub(crate) fn block_cursor(&mut self) -> &mut BlockSegmentPostings {
        &mut self.postings.block_cursor
    }

    /// Returns a reference to the fieldnorm reader for batch lookups.
    pub(crate) fn fieldnorm_reader(&self) -> &FieldNormReader {
        &self.fieldnorm_reader
    }

    /// Returns a reference to the BM25 weight for batch score computation.
    pub(crate) fn bm25_weight(&self) -> &Bm25Weight {
        &self.similarity_weight
    }

    fn fill_compact_scored_buffer_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        let mut term_freqs = [0; SCORE_BLOCK_LEN];
        let len = self
            .postings
            .fill_doc_freqs_up_to(horizon, &mut docs[..], &mut term_freqs[..]);
        let mut fieldnorm_ids = [0; SCORE_BLOCK_LEN];
        self.fieldnorm_reader
            .fieldnorm_ids(&docs[..len], &mut fieldnorm_ids[..len]);
        self.similarity_weight.score_batch(
            &fieldnorm_ids[..len],
            &term_freqs[..len],
            &mut scores[..len],
        );
        len
    }

    fn fill_scored_blocks_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId],
        scores: &mut [Score],
    ) -> Range<usize> {
        assert_eq!(docs.len(), scores.len());
        assert_eq!(
            docs.len() % crate::postings::compression::COMPRESSION_BLOCK_SIZE,
            0
        );
        let mut term_freqs = [0; crate::postings::compression::COMPRESSION_BLOCK_SIZE];
        let mut fieldnorm_ids = [0; crate::postings::compression::COMPRESSION_BLOCK_SIZE];
        let mut range_start = None;
        let mut range_end = 0;

        for (block_ord, (docs_block, scores_block)) in docs
            .chunks_exact_mut(crate::postings::compression::COMPRESSION_BLOCK_SIZE)
            .zip(scores.chunks_exact_mut(crate::postings::compression::COMPRESSION_BLOCK_SIZE))
            .enumerate()
        {
            let docs_block: &mut [DocId; crate::postings::compression::COMPRESSION_BLOCK_SIZE] =
                docs_block.try_into().unwrap();
            let local_range =
                self.postings
                    .fill_doc_freq_block_up_to(horizon, docs_block, &mut term_freqs);
            if local_range.is_empty() {
                break;
            }
            self.fieldnorm_reader.fieldnorm_ids(
                &docs_block[local_range.clone()],
                &mut fieldnorm_ids[local_range.clone()],
            );
            self.similarity_weight.score_batch(
                &fieldnorm_ids[local_range.clone()],
                &term_freqs[local_range.clone()],
                &mut scores_block[local_range.clone()],
            );
            let block_offset = block_ord * crate::postings::compression::COMPRESSION_BLOCK_SIZE;
            range_start.get_or_insert(block_offset + local_range.start);
            range_end = block_offset + local_range.end;
            if local_range.end < crate::postings::compression::COMPRESSION_BLOCK_SIZE {
                break;
            }
        }

        range_start.unwrap_or(0)..range_end
    }
}

impl DocSet for TermScorer {
    #[inline]
    fn advance(&mut self) -> DocId {
        self.postings.advance()
    }

    #[inline]
    fn seek(&mut self, target: DocId) -> DocId {
        let current = self.postings.doc();
        if current >= target {
            return current;
        }
        self.postings.seek(target)
    }

    #[inline]
    fn doc(&self) -> DocId {
        self.postings.doc()
    }

    fn size_hint(&self) -> u32 {
        self.postings.size_hint()
    }

    #[inline]
    fn fill_bitset_block(&mut self, min_doc: DocId, horizon: DocId, mask: &mut [TinySet]) -> DocId {
        self.postings.fill_bitset_block(min_doc, horizon, mask)
    }
}

impl Scorer for TermScorer {
    #[inline]
    fn score(&mut self) -> Score {
        let fieldnorm_id = self.fieldnorm_id();
        let term_freq = self.term_freq();
        self.similarity_weight.score(fieldnorm_id, term_freq)
    }

    #[inline]
    fn fill_scored_buffer_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; SCORE_BLOCK_LEN],
        scores: &mut [Score; SCORE_BLOCK_LEN],
    ) -> usize {
        self.fill_compact_scored_buffer_up_to(horizon, docs, scores)
    }

    fn fill_scored_bitset_block(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        output: &mut dyn ScoredBlockOutput,
    ) -> DocId {
        let mut fieldnorm_ids = [0; COMPRESSION_BLOCK_SIZE];
        let fieldnorm_reader = &self.fieldnorm_reader;
        let similarity_weight = &self.similarity_weight;

        self.postings.for_each_scored_block(
            min_doc,
            horizon,
            |docs, term_freqs, scores| {
                let fieldnorm_ids = &mut fieldnorm_ids[..docs.len()];
                fieldnorm_reader.fieldnorm_ids(docs, fieldnorm_ids);
                similarity_weight.score_batch(fieldnorm_ids, term_freqs, scores);
            },
            |docs, scores| output.update_docs(docs, scores),
        )
    }

    #[inline]
    fn fill_scored_buffer_range_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId],
        scores: &mut [Score],
    ) -> Range<usize> {
        self.fill_scored_blocks_up_to(horizon, docs, scores)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::index::SegmentId;
    use crate::indexer::index_writer::MEMORY_BUDGET_NUM_BYTES_MIN;
    use crate::merge_policy::NoMergePolicy;
    use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
    use crate::query::term_query::TermScorer;
    use crate::query::{Bm25Weight, EnableScoring, Scorer, TermQuery, SCORE_BLOCK_LEN};
    use crate::schema::{IndexRecordOption, Schema, TEXT};
    use crate::{
        assert_nearly_equals, DocId, DocSet, Index, IndexWriter, Score, Searcher, Term, TERMINATED,
    };

    #[test]
    fn test_term_scorer_scored_buffers_match_scalar_scoring() {
        let num_docs = SCORE_BLOCK_LEN + 76;
        let doc_and_tfs: Vec<(DocId, u32)> = (0..num_docs as DocId)
            .map(|doc| (doc, doc % 5 + 1))
            .collect();
        let fieldnorms: Vec<u32> = (0..num_docs as u32).map(|doc| doc % 17 + 1).collect();
        let weight = Bm25Weight::for_one_term(num_docs as u64, 10_000, 9.0);
        let mut scorer = TermScorer::create_for_test(&doc_and_tfs, &fieldnorms, weight.clone());
        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];

        assert_eq!(
            scorer.fill_scored_buffer(&mut docs, &mut scores),
            SCORE_BLOCK_LEN
        );
        for i in 0..SCORE_BLOCK_LEN {
            assert_eq!(docs[i], i as DocId);
            assert_eq!(
                scores[i],
                weight.score(
                    crate::fieldnorm::FieldNormReader::fieldnorm_to_id(fieldnorms[i]),
                    doc_and_tfs[i].1
                )
            );
        }

        let first_unreturned = SCORE_BLOCK_LEN as DocId;
        assert_eq!(
            scorer.score(),
            weight.score(scorer.fieldnorm_id(), doc_and_tfs[SCORE_BLOCK_LEN].1)
        );
        assert_eq!(scorer.advance(), first_unreturned + 1);
        let horizon = first_unreturned + 36;
        assert_eq!(
            scorer.fill_scored_buffer_up_to(horizon, &mut docs, &mut scores),
            35
        );
        assert_eq!(
            &docs[..35],
            &(first_unreturned + 1..horizon).collect::<Vec<_>>()
        );
        assert_eq!(scorer.doc(), horizon);

        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 40);
        assert_eq!(
            &docs[..40],
            &(horizon..num_docs as DocId).collect::<Vec<_>>()
        );
        assert_eq!(scorer.doc(), TERMINATED);
        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 0);
    }

    #[test]
    fn test_term_scorer_scored_horizon_fill_decodes_all_blocks_at_once() {
        const HORIZON: usize = 64 * 64;
        const BUFFER_LEN: usize = HORIZON + COMPRESSION_BLOCK_SIZE;
        let doc_and_tfs: Vec<(DocId, u32)> = (0..4_300).map(|doc| (doc, doc % 5 + 1)).collect();
        let fieldnorms: Vec<u32> = (0..4_300).map(|doc| doc % 17 + 1).collect();
        let weight = Bm25Weight::for_one_term(4_300, 20_000, 9.0);
        let mut scorer = TermScorer::create_for_test(&doc_and_tfs, &fieldnorms, weight.clone());
        assert_eq!(scorer.seek(65), 65);
        let mut docs = [0; BUFFER_LEN];
        let mut scores = [0.0; BUFFER_LEN];

        let range =
            scorer.fill_scored_buffer_range_up_to(65 + HORIZON as DocId, &mut docs, &mut scores);
        assert_eq!(range, 65..65 + HORIZON);
        assert_eq!(
            &docs[range.clone()],
            &(65..65 + HORIZON as DocId).collect::<Vec<_>>()
        );
        for idx in range {
            let doc = docs[idx] as usize;
            assert_eq!(
                scores[idx],
                weight.score(
                    crate::fieldnorm::FieldNormReader::fieldnorm_to_id(fieldnorms[doc]),
                    doc_and_tfs[doc].1
                )
            );
        }
        assert_eq!(scorer.doc(), 65 + HORIZON as DocId);

        let range = scorer.fill_scored_buffer_range_up_to(TERMINATED, &mut docs, &mut scores);
        assert_eq!(range, 65..204);
        assert_eq!(
            &docs[range.clone()],
            &(65 + HORIZON as DocId..4_300).collect::<Vec<_>>()
        );
        for idx in range {
            let doc = docs[idx] as usize;
            assert_eq!(
                scores[idx],
                weight.score(
                    crate::fieldnorm::FieldNormReader::fieldnorm_to_id(fieldnorms[doc]),
                    doc_and_tfs[doc].1
                )
            );
        }
        assert_eq!(scorer.doc(), TERMINATED);
    }

    #[test]
    fn test_term_scorer_scored_buffer_with_basic_postings() {
        let postings = crate::postings::SegmentPostings::create_from_docs(&[1, 4, 7]);
        let fieldnorms = [2, 3, 4, 5, 6, 7, 8, 9];
        let fieldnorm_reader = crate::fieldnorm::FieldNormReader::for_test(&fieldnorms);
        let weight = Bm25Weight::for_one_term(3, 8, 5.5);
        let mut scorer = TermScorer::new(postings, fieldnorm_reader, weight.clone());
        let mut docs = [0; SCORE_BLOCK_LEN];
        let mut scores = [0.0; SCORE_BLOCK_LEN];

        assert_eq!(scorer.fill_scored_buffer(&mut docs, &mut scores), 3);
        assert_eq!(&docs[..3], &[1, 4, 7]);
        for i in 0..3 {
            let fieldnorm_id =
                crate::fieldnorm::FieldNormReader::fieldnorm_to_id(fieldnorms[docs[i] as usize]);
            assert_eq!(scores[i], weight.score(fieldnorm_id, 1));
        }
    }

    #[test]
    fn test_term_scorer_max_score() -> crate::Result<()> {
        let bm25_weight = Bm25Weight::for_one_term(3, 6, 10.0);
        let mut term_scorer = TermScorer::create_for_test(
            &[(2, 3), (3, 12), (7, 8)],
            &[0, 0, 10, 12, 0, 0, 0, 100],
            bm25_weight,
        );
        let max_scorer = term_scorer.max_score();
        crate::assert_nearly_equals!(max_scorer, 1.3990127);
        assert_eq!(term_scorer.doc(), 2);
        assert_eq!(term_scorer.term_freq(), 3);
        assert_nearly_equals!(term_scorer.block_max_score(), 1.3676447);
        assert_nearly_equals!(term_scorer.score(), 1.0892314);
        assert_eq!(term_scorer.advance(), 3);
        assert_eq!(term_scorer.doc(), 3);
        assert_eq!(term_scorer.term_freq(), 12);
        assert_nearly_equals!(term_scorer.score(), 1.3676447);
        assert_eq!(term_scorer.advance(), 7);
        assert_eq!(term_scorer.doc(), 7);
        assert_eq!(term_scorer.term_freq(), 8);
        assert_nearly_equals!(term_scorer.score(), 0.72015285);
        assert_eq!(term_scorer.advance(), TERMINATED);
        Ok(())
    }

    #[test]
    fn test_term_scorer_shallow_advance() -> crate::Result<()> {
        let bm25_weight = Bm25Weight::for_one_term(300, 1024, 10.0);
        let mut doc_and_tfs = vec![];
        for i in 0u32..300u32 {
            let doc = i * 10;
            doc_and_tfs.push((doc, 1u32 + doc % 3u32));
        }
        let fieldnorms: Vec<u32> = std::iter::repeat_n(10u32, 3_000).collect();
        let mut term_scorer = TermScorer::create_for_test(&doc_and_tfs, &fieldnorms, bm25_weight);
        assert_eq!(term_scorer.doc(), 0u32);
        term_scorer.seek_block(1289);
        assert_eq!(term_scorer.doc(), 0u32);
        term_scorer.seek(1289);
        assert_eq!(term_scorer.doc(), 1290);
        Ok(())
    }

    proptest! {
        #[test]
        fn test_term_scorer_block_max_score(term_freqs_fieldnorms in proptest::collection::vec((1u32..10u32, 0u32..100u32), 80..300)) {
        let term_doc_freq = term_freqs_fieldnorms.len();
         let doc_tfs: Vec<(u32, u32)> = term_freqs_fieldnorms.iter()
                   .cloned()
                  .enumerate()
                  .map(|(doc, (tf, _))| (doc as u32, tf))
                  .collect();

         let mut fieldnorms: Vec<u32> = vec![];
         for item in term_freqs_fieldnorms.iter().take(term_doc_freq) {
             let (tf, num_extra_terms) = item;
             fieldnorms.push(tf + num_extra_terms);
         }
         let average_fieldnorm = fieldnorms
             .iter()
             .cloned()
             .sum::<u32>() as Score / term_doc_freq as Score;
             // Average fieldnorm is over the entire index,
             // not necessarily the docs that are in the posting list.
             // For this reason we multiply by 1.1 to make a realistic value.
         let bm25_weight = Bm25Weight::for_one_term(term_doc_freq as u64,
            term_doc_freq as u64 * 10u64,
            average_fieldnorm);

         let mut term_scorer =
              TermScorer::create_for_test(&doc_tfs[..], &fieldnorms[..], bm25_weight);

         let docs: Vec<DocId> = (0..term_doc_freq).map(|doc| doc as DocId).collect();
         for block in docs.chunks(COMPRESSION_BLOCK_SIZE) {
             let block_max_score: Score = term_scorer.block_max_score();
             let mut block_max_score_computed: Score = 0.0;
             for &doc in block {
                assert_eq!(term_scorer.doc(), doc);
                block_max_score_computed = block_max_score_computed.max(term_scorer.score());
                term_scorer.advance();
             }
             assert_nearly_equals!(block_max_score_computed, block_max_score);
         }
        }
    }

    #[test]
    fn test_block_wand() {
        let mut doc_tfs: Vec<(u32, u32)> = vec![];
        for doc in 0u32..128u32 {
            doc_tfs.push((doc, 1u32));
        }
        for doc in 128u32..256u32 {
            doc_tfs.push((doc, if doc == 200 { 2u32 } else { 1u32 }));
        }
        doc_tfs.push((256, 1u32));
        doc_tfs.push((257, 3u32));
        doc_tfs.push((258, 1u32));

        let fieldnorms: Vec<u32> = std::iter::repeat_n(20u32, 300).collect();
        let bm25_weight = Bm25Weight::for_one_term(10, 129, 20.0);
        let mut docs = TermScorer::create_for_test(&doc_tfs[..], &fieldnorms[..], bm25_weight);
        assert_nearly_equals!(docs.block_max_score(), 2.5161593);
        docs.seek_block(135);
        assert_nearly_equals!(docs.block_max_score(), 3.4597192);
        docs.seek_block(256);
        // the block is not loaded yet.
        assert_nearly_equals!(docs.block_max_score(), 5.2971773);
        assert_eq!(256, docs.seek(256));
        assert_nearly_equals!(docs.block_max_score(), 3.9539647);
    }

    fn test_block_wand_aux(term_query: &TermQuery, searcher: &Searcher) -> crate::Result<()> {
        let term_weight =
            term_query.specialized_weight(EnableScoring::enabled_from_searcher(searcher))?;
        for reader in searcher.segment_readers() {
            let mut block_max_scores = vec![];
            let mut block_max_scores_b = vec![];
            let mut docs = vec![];
            {
                let mut term_scorer = term_weight.term_scorer_for_test(reader, 1.0)?.unwrap();
                while term_scorer.doc() != TERMINATED {
                    let mut score = term_scorer.score();
                    docs.push(term_scorer.doc());
                    for _ in 0..128 {
                        score = score.max(term_scorer.score());
                        if term_scorer.advance() == TERMINATED {
                            break;
                        }
                    }
                    block_max_scores.push(score);
                }
            }
            {
                let mut term_scorer = term_weight.term_scorer_for_test(reader, 1.0)?.unwrap();
                for d in docs {
                    term_scorer.seek_block(d);
                    block_max_scores_b.push(term_scorer.block_max_score());
                }
            }
            for (l, r) in block_max_scores
                .iter()
                .cloned()
                .zip(block_max_scores_b.iter().cloned())
            {
                assert_nearly_equals!(l, r);
            }
        }
        Ok(())
    }

    #[ignore]
    #[test]
    fn test_block_wand_long_test() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer: IndexWriter =
            index.writer_with_num_threads(3, 3 * MEMORY_BUDGET_NUM_BYTES_MIN)?;
        use rand::Rng;
        let mut rng = rand::rng();
        writer.set_merge_policy(Box::new(NoMergePolicy));
        for _ in 0..3_000 {
            let term_freq = rng.random_range(1..10000);
            let words: Vec<&str> = std::iter::repeat_n("bbbb", term_freq).collect();
            let text = words.join(" ");
            writer.add_document(doc!(text_field=>text))?;
        }
        writer.commit()?;
        let term_query = TermQuery::new(
            Term::from_field_text(text_field, "bbbb"),
            IndexRecordOption::WithFreqs,
        );
        let segment_ids: Vec<SegmentId>;
        let reader = index.reader()?;
        {
            let searcher = reader.searcher();
            segment_ids = searcher
                .segment_readers()
                .iter()
                .map(|segment| segment.segment_id())
                .collect();
            test_block_wand_aux(&term_query, &searcher)?;
        }
        writer.merge(&segment_ids[..]).wait().unwrap();
        {
            reader.reload()?;
            let searcher = reader.searcher();
            assert_eq!(searcher.segment_readers().len(), 1);
            test_block_wand_aux(&term_query, &searcher)?;
        }
        Ok(())
    }
}
