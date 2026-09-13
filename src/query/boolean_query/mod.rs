mod block_wand_intersection;
mod block_wand_union;
mod boolean_query;
mod boolean_weight;
mod occur_weights;

pub(crate) use self::block_wand_intersection::block_wand_intersection;
pub(crate) use self::block_wand_union::{block_wand, block_wand_single_scorer};
pub use self::boolean_query::BooleanQuery;
pub use self::boolean_weight::BooleanWeight;

#[cfg(test)]
mod tests {

    use std::ops::Bound;

    use super::*;
    use crate::collector::tests::TEST_COLLECTOR_WITH_SCORE;
    use crate::collector::{Count, TopDocs};
    use crate::query::score_combiner::DoNothingCombiner;
    use crate::query::term_query::TermScorer;
    use crate::query::union::SimpleUnion;
    use crate::query::{
        AllScorer, BufferedUnionScorer, EmptyScorer, EnableScoring, Intersection, Occur, Query,
        QueryParser, RangeQuery, RequiredOptionalScorer, ScoredBatch, ScoredBatchConsumer, Scorer,
        SumCombiner, TermQuery,
    };
    use crate::schema::*;
    use crate::{assert_nearly_equals, DocAddress, DocId, Index, IndexWriter, Score};

    #[derive(Default)]
    struct RecordingScoredBatchConsumer {
        compact_blocks: usize,
        windows: usize,
        docs: Vec<DocId>,
        scores: Vec<Score>,
    }

    impl ScoredBatchConsumer for RecordingScoredBatchConsumer {
        fn consume(&mut self, batch: ScoredBatch<'_>) {
            match batch {
                ScoredBatch::Sparse { docs, scores } => {
                    self.compact_blocks += 1;
                    self.docs.extend_from_slice(docs);
                    self.scores.extend_from_slice(scores);
                }
                ScoredBatch::Dense {
                    base_doc,
                    words,
                    scores,
                } => {
                    self.windows += 1;
                    for (bucket, bits) in words.iter().enumerate() {
                        for bit in *bits {
                            let offset = bucket * 64 + bit as usize;
                            self.docs.push(base_doc + offset as DocId);
                            self.scores.push(scores[offset]);
                        }
                    }
                }
            }
        }
    }

    fn aux_test_helper() -> crate::Result<(Index, Field)> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        {
            // writing the segment
            let mut index_writer: IndexWriter = index.writer_for_tests()?;
            index_writer.add_document(doc!(text_field => "a b c"))?;
            index_writer.add_document(doc!(text_field => "a c"))?;
            index_writer.add_document(doc!(text_field => "b c"))?;
            index_writer.add_document(doc!(text_field => "a b c d"))?;
            index_writer.add_document(doc!(text_field => "d"))?;
            index_writer.commit()?;
        }
        Ok((index, text_field))
    }

    #[test]
    fn test_flat_term_union_emits_dense_scored_windows() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query = QueryParser::for_index(&index, vec![text_field]).parse_query("a b")?;
        let searcher = index.reader()?.searcher();
        let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
        let scorer = weight.scorer(searcher.segment_reader(0), 1.0)?;
        assert!(scorer.is::<BufferedUnionScorer<TermScorer, SumCombiner>>());
        let mut consumer = RecordingScoredBatchConsumer::default();
        weight.for_each_scored_batch(searcher.segment_reader(0), &mut consumer)?;

        assert_eq!(consumer.compact_blocks, 0);
        assert_eq!(consumer.windows, 1);
        assert_eq!(consumer.docs, vec![0, 1, 2, 3]);
        let mut scalar_scores = Vec::new();
        weight.for_each(searcher.segment_reader(0), &mut |_, score| {
            scalar_scores.push(score)
        })?;
        assert_eq!(consumer.scores, scalar_scores);
        Ok(())
    }

    #[test]
    fn test_sparse_flat_term_union_compact_matches_scalar_exactly() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let index = Index::create_in_ram(schema_builder.build());
        let mut index_writer: IndexWriter = index.writer_for_tests()?;
        for doc_id in 0..5_000 {
            let text = match doc_id {
                3 | 4_097 => "a",
                100 => "a b",
                2_049 => "b",
                4_999 => "c",
                _ => "x",
            };
            index_writer.add_document(doc!(text_field => text))?;
        }
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();
        let reader = searcher.segment_reader(0);
        let query = QueryParser::for_index(&index, vec![text_field]).parse_query("a b c")?;
        let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
        let scorer = weight.scorer(reader, 1.0)?;
        assert!(scorer.is::<SimpleUnion<TermScorer, SumCombiner>>());
        let mut scalar = Vec::new();
        weight.for_each(reader, &mut |doc, score| scalar.push((doc, score)))?;
        let mut blocked = Vec::new();
        weight.for_each_scored_block(reader, &mut |docs, scores| {
            blocked.extend(docs.iter().copied().zip(scores.iter().copied()));
        })?;

        assert_eq!(blocked, scalar);
        assert_eq!(
            blocked.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [3, 100, 2_049, 4_097, 4_999]
        );

        let mut consumer = RecordingScoredBatchConsumer::default();
        weight.for_each_scored_batch(reader, &mut consumer)?;
        assert!(consumer.compact_blocks > 0);
        assert_eq!(consumer.windows, 0);
        assert_eq!(consumer.docs, [3, 100, 2_049, 4_097, 4_999]);
        assert_eq!(
            consumer.scores,
            scalar.iter().map(|&(_, score)| score).collect::<Vec<_>>()
        );

        let weight = query.weight(EnableScoring::disabled_from_searcher(&searcher))?;
        let scorer = weight.scorer(reader, 1.0)?;
        assert!(scorer.is::<SimpleUnion<TermScorer, DoNothingCombiner>>());
        Ok(())
    }

    #[test]
    fn test_nested_sparse_term_union_is_simple_union() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let index = Index::create_in_ram(schema_builder.build());
        let mut index_writer: IndexWriter = index.writer_for_tests()?;
        for doc_id in 0..5_000 {
            let text = match doc_id {
                3 | 4_097 => "a",
                100 => "b rare",
                2_049 => "c",
                _ => "x",
            };
            index_writer.add_document(doc!(text_field => text))?;
        }
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();
        let query =
            QueryParser::for_index(&index, vec![text_field]).parse_query("+(a b c) +rare")?;
        let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
        let scorer = weight.scorer(searcher.segment_reader(0), 1.0)?;
        let intersection = scorer
            .downcast_ref::<Intersection<Box<dyn Scorer>>>()
            .expect("nested union should be a child of the intersection");
        assert!(intersection.has_child::<SimpleUnion<TermScorer, SumCombiner>>());
        assert!(!intersection.has_child::<BufferedUnionScorer<TermScorer, SumCombiner>>());
        Ok(())
    }

    #[test]
    fn test_nested_term_union_intersection_matches_scalar_exactly() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query_parser = QueryParser::for_index(&index, vec![text_field]);
        let searcher = index.reader()?.searcher();
        let reader = searcher.segment_reader(0);

        for (query_text, expected_docs) in
            [("+(a b) +c", vec![0, 1, 2, 3]), ("+(a b) +c +d", vec![3])]
        {
            let query = query_parser.parse_query(query_text)?;
            let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
            let mut scalar = Vec::new();
            weight.for_each(reader, &mut |doc, score| scalar.push((doc, score)))?;
            let mut blocked = Vec::new();
            weight.for_each_scored_block(reader, &mut |docs, scores| {
                blocked.extend(docs.iter().copied().zip(scores.iter().copied()));
            })?;

            assert_eq!(blocked, scalar);
            assert_eq!(
                blocked.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
                expected_docs
            );
        }
        Ok(())
    }

    #[test]
    pub fn test_boolean_non_all_term_disjunction() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query_parser = QueryParser::for_index(&index, vec![text_field]);
        let query = query_parser.parse_query("(+a +b) d")?;
        let searcher = index.reader()?.searcher();
        assert_eq!(query.count(&searcher)?, 3);
        Ok(())
    }

    #[test]
    pub fn test_boolean_single_must_clause() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query_parser = QueryParser::for_index(&index, vec![text_field]);
        let query = query_parser.parse_query("+a")?;
        let searcher = index.reader()?.searcher();
        let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
        let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0)?;
        assert!(scorer.is::<TermScorer>());
        Ok(())
    }

    #[test]
    pub fn test_boolean_termonly_intersection() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query_parser = QueryParser::for_index(&index, vec![text_field]);
        let searcher = index.reader()?.searcher();
        {
            let query = query_parser.parse_query("+a +b +c")?;
            let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0)?;
            assert!(scorer.is::<Intersection<TermScorer>>());
        }
        {
            let query = query_parser.parse_query("+a +(b c)")?;
            let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0)?;
            assert!(scorer.is::<Intersection<Box<dyn Scorer>>>());
        }
        Ok(())
    }

    #[test]
    pub fn test_boolean_reqopt() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;
        let query_parser = QueryParser::for_index(&index, vec![text_field]);
        let searcher = index.reader()?.searcher();
        {
            let query = query_parser.parse_query("+a b")?;
            let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0)?;
            assert!(scorer
                .is::<RequiredOptionalScorer<Box<dyn Scorer>, Box<dyn Scorer>, SumCombiner>>());
        }
        {
            let query = query_parser.parse_query("+a b")?;
            let weight = query.weight(EnableScoring::disabled_from_schema(searcher.schema()))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0)?;
            assert!(scorer.is::<TermScorer>());
        }
        Ok(())
    }

    #[test]
    pub fn test_boolean_query() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;

        let make_term_query = |text: &str| {
            let term_query = TermQuery::new(
                Term::from_field_text(text_field, text),
                IndexRecordOption::Basic,
            );
            let query: Box<dyn Query> = Box::new(term_query);
            query
        };

        let reader = index.reader()?;

        let matching_docs = |boolean_query: &dyn Query| {
            reader
                .searcher()
                .search(boolean_query, &TEST_COLLECTOR_WITH_SCORE)
                .unwrap()
                .docs()
                .iter()
                .cloned()
                .map(|doc| doc.doc_id)
                .collect::<Vec<DocId>>()
        };
        {
            let boolean_query = BooleanQuery::new(vec![(Occur::Must, make_term_query("a"))]);
            assert_eq!(matching_docs(&boolean_query), vec![0, 1, 3]);
        }
        {
            let boolean_query = BooleanQuery::new(vec![(Occur::Should, make_term_query("a"))]);
            assert_eq!(matching_docs(&boolean_query), vec![0, 1, 3]);
        }
        {
            let boolean_query = BooleanQuery::new(vec![
                (Occur::Should, make_term_query("a")),
                (Occur::Should, make_term_query("b")),
            ]);
            assert_eq!(matching_docs(&boolean_query), vec![0, 1, 2, 3]);
        }
        {
            let boolean_query = BooleanQuery::new(vec![
                (Occur::Must, make_term_query("a")),
                (Occur::Should, make_term_query("b")),
            ]);
            assert_eq!(matching_docs(&boolean_query), vec![0, 1, 3]);
        }
        {
            let boolean_query = BooleanQuery::new(vec![
                (Occur::Must, make_term_query("a")),
                (Occur::Should, make_term_query("b")),
                (Occur::MustNot, make_term_query("d")),
            ]);
            assert_eq!(matching_docs(&boolean_query), vec![0, 1]);
        }
        {
            let boolean_query = BooleanQuery::new(vec![(Occur::MustNot, make_term_query("d"))]);
            assert_eq!(matching_docs(&boolean_query), Vec::<u32>::new());
        }
        Ok(())
    }

    #[test]
    pub fn test_boolean_query_two_excluded() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;

        let make_term_query = |text: &str| {
            let term_query = TermQuery::new(
                Term::from_field_text(text_field, text),
                IndexRecordOption::Basic,
            );
            let query: Box<dyn Query> = Box::new(term_query);
            query
        };

        let reader = index.reader()?;

        let matching_topdocs = |query: &dyn Query| {
            reader
                .searcher()
                .search(query, &TopDocs::with_limit(3).order_by_score())
                .unwrap()
        };

        let score_doc_4: Score; // score of doc 4 should not be influenced by exclusion
        {
            let boolean_query_no_excluded =
                BooleanQuery::new(vec![(Occur::Must, make_term_query("d"))]);
            let topdocs_no_excluded = matching_topdocs(&boolean_query_no_excluded);
            assert_eq!(topdocs_no_excluded.len(), 2);
            let (top_score, top_doc) = topdocs_no_excluded[0];
            assert_eq!(top_doc, DocAddress::new(0, 4));
            assert_eq!(topdocs_no_excluded[1].1, DocAddress::new(0, 3)); // ignore score of doc 3.
            score_doc_4 = top_score;
        }

        {
            let boolean_query_two_excluded = BooleanQuery::new(vec![
                (Occur::Must, make_term_query("d")),
                (Occur::MustNot, make_term_query("a")),
                (Occur::MustNot, make_term_query("b")),
            ]);
            let topdocs_excluded = matching_topdocs(&boolean_query_two_excluded);
            assert_eq!(topdocs_excluded.len(), 1);
            let (top_score, top_doc) = topdocs_excluded[0];
            assert_eq!(top_doc, DocAddress::new(0, 4));
            assert_eq!(top_score, score_doc_4);
        }
        Ok(())
    }

    #[test]
    pub fn test_boolean_query_with_weight() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        {
            let mut index_writer: IndexWriter = index.writer_for_tests()?;
            index_writer.add_document(doc!(text_field => "a b c"))?;
            index_writer.add_document(doc!(text_field => "a c"))?;
            index_writer.add_document(doc!(text_field => "b c"))?;
            index_writer.commit()?;
        }
        let term_a: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "a"),
            IndexRecordOption::WithFreqs,
        ));
        let term_b: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "b"),
            IndexRecordOption::WithFreqs,
        ));
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let boolean_query =
            BooleanQuery::new(vec![(Occur::Should, term_a), (Occur::Should, term_b)]);
        let boolean_weight = boolean_query
            .weight(EnableScoring::enabled_from_searcher(&searcher))
            .unwrap();
        {
            let mut boolean_scorer = boolean_weight.scorer(searcher.segment_reader(0u32), 1.0)?;
            assert_eq!(boolean_scorer.doc(), 0u32);
            assert_nearly_equals!(boolean_scorer.score(), 0.84163445);
        }
        {
            let mut boolean_scorer = boolean_weight.scorer(searcher.segment_reader(0u32), 2.0)?;
            assert_eq!(boolean_scorer.doc(), 0u32);
            assert_nearly_equals!(boolean_scorer.score(), 1.6832689);
        }
        Ok(())
    }

    #[test]
    pub fn test_intersection_score() -> crate::Result<()> {
        let (index, text_field) = aux_test_helper()?;

        let make_term_query = |text: &str| {
            let term_query = TermQuery::new(
                Term::from_field_text(text_field, text),
                IndexRecordOption::Basic,
            );
            let query: Box<dyn Query> = Box::new(term_query);
            query
        };
        let reader = index.reader()?;
        let score_docs = |boolean_query: &dyn Query| {
            let fruit = reader
                .searcher()
                .search(boolean_query, &TEST_COLLECTOR_WITH_SCORE)
                .unwrap();
            fruit.scores().to_vec()
        };

        {
            let boolean_query = BooleanQuery::new(vec![
                (Occur::Must, make_term_query("a")),
                (Occur::Must, make_term_query("b")),
            ]);
            let scores = score_docs(&boolean_query);
            assert_nearly_equals!(scores[0], 0.977973);
            assert_nearly_equals!(scores[1], 0.84699446);
        }
        Ok(())
    }

    #[test]
    pub fn test_explain() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text = schema_builder.add_text_field("text", STRING);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;
        index_writer.add_document(doc!(text=>"a"))?;
        index_writer.add_document(doc!(text=>"b"))?;
        index_writer.commit()?;
        let searcher = index.reader()?.searcher();
        let term_a: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text, "a"),
            IndexRecordOption::Basic,
        ));
        let term_b: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text, "b"),
            IndexRecordOption::Basic,
        ));
        let query = BooleanQuery::from(vec![(Occur::Should, term_a), (Occur::Should, term_b)]);
        let explanation = query.explain(&searcher, DocAddress::new(0, 0u32))?;
        assert_nearly_equals!(explanation.value(), std::f32::consts::LN_2);
        Ok(())
    }

    #[test]
    pub fn test_boolean_weight_optimization() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;
        index_writer.add_document(doc!(text_field=>"hello"))?;
        index_writer.add_document(doc!(text_field=>"hello happy"))?;
        index_writer.commit()?;
        let searcher = index.reader()?.searcher();
        let term_match_all: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "hello"),
            IndexRecordOption::Basic,
        ));
        let term_match_some: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "happy"),
            IndexRecordOption::Basic,
        ));
        let term_match_none: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "tax"),
            IndexRecordOption::Basic,
        ));
        {
            let query = BooleanQuery::from(vec![
                (Occur::Must, term_match_all.box_clone()),
                (Occur::Must, term_match_some.box_clone()),
            ]);
            let weight = query.weight(EnableScoring::disabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0f32)?;
            assert!(scorer.is::<TermScorer>());
        }
        {
            let query = BooleanQuery::from(vec![
                (Occur::Must, term_match_all.box_clone()),
                (Occur::Must, term_match_some.box_clone()),
                (Occur::Must, term_match_none.box_clone()),
            ]);
            let weight = query.weight(EnableScoring::disabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0f32)?;
            assert!(scorer.is::<EmptyScorer>());
        }
        {
            let query = BooleanQuery::from(vec![
                (Occur::Should, term_match_all.box_clone()),
                (Occur::Should, term_match_none.box_clone()),
            ]);
            let weight = query.weight(EnableScoring::disabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0f32)?;
            assert!(scorer.is::<AllScorer>());
        }
        {
            let query = BooleanQuery::from(vec![
                (Occur::Should, term_match_some.box_clone()),
                (Occur::Should, term_match_none.box_clone()),
            ]);
            let weight = query.weight(EnableScoring::disabled_from_searcher(&searcher))?;
            let scorer = weight.scorer(searcher.segment_reader(0u32), 1.0f32)?;
            assert!(scorer.is::<TermScorer>());
        }
        Ok(())
    }

    #[test]
    pub fn test_min_should_match_with_all_query() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let num_field =
            schema_builder.add_i64_field("num", NumericOptions::default().set_fast().set_indexed());
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;

        index_writer.add_document(doc!(text_field => "apple", num_field => 10i64))?;
        index_writer.add_document(doc!(text_field => "banana", num_field => 20i64))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();

        let effective_all_match_query: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(num_field, 0)),
            Bound::Unbounded,
        ));
        let term_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "apple"),
            IndexRecordOption::Basic,
        ));

        // in some previous version, we would remove the 2 all_match, but then say we need *4*
        // matches out of the 3 term queries, which matches nothing.
        let mut bool_query = BooleanQuery::new(vec![
            (Occur::Should, effective_all_match_query.box_clone()),
            (Occur::Should, effective_all_match_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        bool_query.set_minimum_number_should_match(4);
        let count = searcher.search(&bool_query, &Count)?;
        assert_eq!(count, 1);

        Ok(())
    }

    // =========================================================================
    // AllScorer Preservation Regression Tests
    // =========================================================================
    //
    // These tests verify the fix for a bug where AllScorer instances (produced by
    // queries matching all documents, such as range queries covering all values)
    // were incorrectly removed from Boolean query processing, causing documents
    // to be unexpectedly excluded from results.
    //
    // The bug manifested in several scenarios:
    // 1. SHOULD + SHOULD where one clause is AllScorer
    // 2. MUST (AllScorer) + SHOULD
    // 3. Range queries in Boolean clauses when all documents match the range

    /// Regression test: SHOULD clause with AllScorer combined with other SHOULD clauses.
    ///
    /// When a SHOULD clause produces an AllScorer (e.g., from a range query matching
    /// all documents), the Boolean query should still match all documents.
    ///
    /// Bug before fix: AllScorer was removed during optimization, leaving only the
    /// other SHOULD clauses, which incorrectly excluded documents.
    #[test]
    pub fn test_should_with_all_scorer_regression() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let num_field =
            schema_builder.add_i64_field("num", NumericOptions::default().set_fast().set_indexed());
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;

        // All docs have num > 0, so range query will return AllScorer
        index_writer.add_document(doc!(text_field => "hello", num_field => 10i64))?;
        index_writer.add_document(doc!(text_field => "world", num_field => 20i64))?;
        index_writer.add_document(doc!(text_field => "hello world", num_field => 30i64))?;
        index_writer.add_document(doc!(text_field => "foo", num_field => 40i64))?;
        index_writer.add_document(doc!(text_field => "bar", num_field => 50i64))?;
        index_writer.add_document(doc!(text_field => "baz", num_field => 60i64))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();

        // Range query matching all docs (returns AllScorer)
        let all_match_query: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(num_field, 0)),
            Bound::Unbounded,
        ));
        let term_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "hello"),
            IndexRecordOption::Basic,
        ));

        // Verify range matches all 6 docs
        assert_eq!(searcher.search(all_match_query.as_ref(), &Count)?, 6);

        // RangeQuery(all) OR TermQuery should match all 6 docs
        let bool_query = BooleanQuery::new(vec![
            (Occur::Should, all_match_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        let count = searcher.search(&bool_query, &Count)?;
        assert_eq!(count, 6, "SHOULD with AllScorer should match all docs");

        // Order should not matter
        let bool_query_reversed = BooleanQuery::new(vec![
            (Occur::Should, term_query.box_clone()),
            (Occur::Should, all_match_query.box_clone()),
        ]);
        let count_reversed = searcher.search(&bool_query_reversed, &Count)?;
        assert_eq!(
            count_reversed, 6,
            "Order of SHOULD clauses should not matter"
        );

        Ok(())
    }

    /// Regression test: MUST clause with AllScorer combined with SHOULD clause.
    ///
    /// When MUST contains an AllScorer, all documents satisfy the MUST constraint.
    /// The SHOULD clause should only affect scoring, not filtering.
    ///
    /// Bug before fix: AllScorer was removed, leaving an empty must_scorers vector.
    /// intersect_scorers([]) incorrectly returned EmptyScorer, matching 0 documents.
    #[test]
    pub fn test_must_all_with_should_regression() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let num_field =
            schema_builder.add_i64_field("num", NumericOptions::default().set_fast().set_indexed());
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;

        // All docs have num > 0, so range query will return AllScorer
        index_writer.add_document(doc!(text_field => "apple", num_field => 10i64))?;
        index_writer.add_document(doc!(text_field => "banana", num_field => 20i64))?;
        index_writer.add_document(doc!(text_field => "cherry", num_field => 30i64))?;
        index_writer.add_document(doc!(text_field => "date", num_field => 40i64))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();

        // Range query matching all docs (returns AllScorer)
        let all_match_query: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(num_field, 0)),
            Bound::Unbounded,
        ));
        let term_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "apple"),
            IndexRecordOption::Basic,
        ));

        // Verify range matches all 4 docs
        assert_eq!(searcher.search(all_match_query.as_ref(), &Count)?, 4);

        // MUST(range matching all) AND SHOULD(term) should match all 4 docs
        let bool_query = BooleanQuery::new(vec![
            (Occur::Must, all_match_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        let count = searcher.search(&bool_query, &Count)?;
        assert_eq!(count, 4, "MUST AllScorer + SHOULD should match all docs");

        Ok(())
    }

    /// Regression test: Range queries in Boolean clauses when all documents match.
    ///
    /// Range queries can return AllScorer as an optimization when all indexed values
    /// fall within the range. This test ensures such queries work correctly in
    /// Boolean combinations.
    ///
    /// This is the most common real-world manifestation of the bug, occurring in
    /// queries like: (age > 50 OR name = 'Alice') AND status = 'active'
    /// when all documents have age > 50.
    #[test]
    pub fn test_range_query_all_match_in_boolean() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let name_field = schema_builder.add_text_field("name", TEXT);
        let age_field =
            schema_builder.add_i64_field("age", NumericOptions::default().set_fast().set_indexed());
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;

        // All documents have age > 50, so range query will return AllScorer
        index_writer.add_document(doc!(name_field => "alice", age_field => 55_i64))?;
        index_writer.add_document(doc!(name_field => "bob", age_field => 60_i64))?;
        index_writer.add_document(doc!(name_field => "charlie", age_field => 70_i64))?;
        index_writer.add_document(doc!(name_field => "diana", age_field => 80_i64))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();

        let range_query: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(age_field, 50)),
            Bound::Unbounded,
        ));
        let term_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(name_field, "alice"),
            IndexRecordOption::Basic,
        ));

        // Verify preconditions
        assert_eq!(searcher.search(range_query.as_ref(), &Count)?, 4);
        assert_eq!(searcher.search(term_query.as_ref(), &Count)?, 1);

        // SHOULD(range) OR SHOULD(term): range matches all, so result is 4
        let should_query = BooleanQuery::new(vec![
            (Occur::Should, range_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        assert_eq!(
            searcher.search(&should_query, &Count)?,
            4,
            "SHOULD range OR term should match all"
        );

        // MUST(range) AND SHOULD(term): range matches all, term is optional
        let must_should_query = BooleanQuery::new(vec![
            (Occur::Must, range_query.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        assert_eq!(
            searcher.search(&must_should_query, &Count)?,
            4,
            "MUST range + SHOULD term should match all"
        );

        Ok(())
    }

    /// Test multiple AllScorer instances in different clause types.
    ///
    /// Verifies correct behavior when AllScorers appear in multiple positions.
    #[test]
    pub fn test_multiple_all_scorers() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let num_field =
            schema_builder.add_i64_field("num", NumericOptions::default().set_fast().set_indexed());
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer: IndexWriter = index.writer_for_tests()?;

        // All docs have num > 0, so range queries will return AllScorer
        index_writer.add_document(doc!(text_field => "doc1", num_field => 10i64))?;
        index_writer.add_document(doc!(text_field => "doc2", num_field => 20i64))?;
        index_writer.add_document(doc!(text_field => "doc3", num_field => 30i64))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();

        // Two different range queries that both match all docs (return AllScorer)
        let all_query1: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(num_field, 0)),
            Bound::Unbounded,
        ));
        let all_query2: Box<dyn Query> = Box::new(RangeQuery::new(
            Bound::Excluded(Term::from_field_i64(num_field, 5)),
            Bound::Unbounded,
        ));
        let term_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(text_field, "doc1"),
            IndexRecordOption::Basic,
        ));

        // Multiple AllScorers in SHOULD
        let multi_all_should = BooleanQuery::new(vec![
            (Occur::Should, all_query1.box_clone()),
            (Occur::Should, all_query2.box_clone()),
            (Occur::Should, term_query.box_clone()),
        ]);
        assert_eq!(
            searcher.search(&multi_all_should, &Count)?,
            3,
            "Multiple AllScorers in SHOULD"
        );

        // AllScorer in both MUST and SHOULD
        let all_must_and_should = BooleanQuery::new(vec![
            (Occur::Must, all_query1.box_clone()),
            (Occur::Should, all_query2.box_clone()),
        ]);
        assert_eq!(
            searcher.search(&all_must_and_should, &Count)?,
            3,
            "AllScorer in both MUST and SHOULD"
        );

        Ok(())
    }
}

/// A proptest which generates arbitrary permutations of a simple boolean AST, and then matches
/// the result against an index which contains all permutations of documents with N fields.
#[cfg(test)]
mod proptest_boolean_query {
    use std::collections::{BTreeMap, HashSet};
    use std::ops::{Bound, Range};

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::collector::{Collector, DocSetCollector, SegmentCollector};
    use crate::query::{AllQuery, BooleanQuery, Occur, Query, RangeQuery, TermQuery};
    use crate::schema::{Field, NumericOptions, OwnedValue, Schema, TEXT};
    use crate::{DocId, Index, Term};

    #[derive(Debug, Clone)]
    enum BooleanQueryAST {
        /// Matches all documents via AllQuery (wraps AllScorer in BoostScorer)
        All,
        /// Matches all documents via RangeQuery (returns bare AllScorer)
        /// This is the actual trigger for the AllScorer preservation bug
        RangeAll,
        /// Matches documents where the field has value "true"
        Leaf {
            field_idx: usize,
        },
        Union(Vec<BooleanQueryAST>),
        Intersection(Vec<BooleanQueryAST>),
    }

    impl BooleanQueryAST {
        fn matches(&self, doc_id: DocId) -> bool {
            match self {
                BooleanQueryAST::All => true,
                BooleanQueryAST::RangeAll => true,
                BooleanQueryAST::Leaf { field_idx } => Self::matches_field(doc_id, *field_idx),
                BooleanQueryAST::Union(children) => {
                    children.iter().any(|child| child.matches(doc_id))
                }
                BooleanQueryAST::Intersection(children) => {
                    children.iter().all(|child| child.matches(doc_id))
                }
            }
        }

        fn matches_field(doc_id: DocId, field_idx: usize) -> bool {
            ((doc_id as usize) >> field_idx) & 1 == 1
        }

        fn to_query(&self, fields: &[Field], range_field: Field) -> Box<dyn Query> {
            match self {
                BooleanQueryAST::All => Box::new(AllQuery),
                BooleanQueryAST::RangeAll => {
                    // Range query that matches all docs (all have value >= 0)
                    // This returns bare AllScorer, triggering the bug we fixed
                    Box::new(RangeQuery::new(
                        Bound::Included(Term::from_field_i64(range_field, 0)),
                        Bound::Unbounded,
                    ))
                }
                BooleanQueryAST::Leaf { field_idx } => Box::new(TermQuery::new(
                    Term::from_field_text(fields[*field_idx], "true"),
                    crate::schema::IndexRecordOption::Basic,
                )),
                BooleanQueryAST::Union(children) => {
                    let sub_queries = children
                        .iter()
                        .map(|child| (Occur::Should, child.to_query(fields, range_field)))
                        .collect();
                    Box::new(BooleanQuery::new(sub_queries))
                }
                BooleanQueryAST::Intersection(children) => {
                    let sub_queries = children
                        .iter()
                        .map(|child| (Occur::Must, child.to_query(fields, range_field)))
                        .collect();
                    Box::new(BooleanQuery::new(sub_queries))
                }
            }
        }
    }

    fn doc_ids(num_docs: usize, num_fields: usize) -> Range<DocId> {
        let permutations = 1 << num_fields;
        let copies = (num_docs as f32 / permutations as f32).ceil() as u32;
        0..(permutations * copies)
    }

    fn create_index_with_boolean_permutations(
        num_docs: usize,
        num_fields: usize,
    ) -> (Index, Vec<Field>, Field) {
        let mut schema_builder = Schema::builder();
        let fields: Vec<Field> = (0..num_fields)
            .map(|i| schema_builder.add_text_field(&format!("field_{}", i), TEXT))
            .collect();
        // Add a numeric field for RangeQuery tests - all docs have value = doc_id
        let range_field = schema_builder.add_i64_field(
            "range_field",
            NumericOptions::default().set_fast().set_indexed(),
        );
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_for_tests().unwrap();

        for doc_id in doc_ids(num_docs, num_fields) {
            let mut doc: BTreeMap<_, OwnedValue> = BTreeMap::default();
            for (field_idx, &field) in fields.iter().enumerate() {
                if (doc_id >> field_idx) & 1 == 1 {
                    doc.insert(field, "true".into());
                }
            }
            // All docs have non-negative values, so RangeQuery(>=0) matches all
            doc.insert(range_field, (doc_id as i64).into());
            writer.add_document(doc).unwrap();
        }
        writer.commit().unwrap();
        (index, fields, range_field)
    }

    fn arb_boolean_query_ast(num_fields: usize) -> impl Strategy<Value = BooleanQueryAST> {
        // Leaf strategies: term queries, AllQuery, and RangeQuery matching all docs
        let leaf = prop_oneof![
            (0..num_fields).prop_map(|field_idx| BooleanQueryAST::Leaf { field_idx }),
            Just(BooleanQueryAST::All),
            Just(BooleanQueryAST::RangeAll),
        ];
        leaf.prop_recursive(
            8,   // 8 levels of recursion
            256, // 256 nodes max
            10,  // 10 items per collection
            |inner| {
                prop_oneof![
                    vec(inner.clone(), 1..10).prop_map(BooleanQueryAST::Union),
                    vec(inner, 1..10).prop_map(BooleanQueryAST::Intersection),
                ]
            },
        )
    }

    #[test]
    fn proptest_boolean_query() {
        // In the presence of optimizations around buffering, it can take large numbers of
        // documents to uncover some issues.
        let num_fields = 8;
        let num_docs = 1 << num_fields;
        let (index, fields, range_field) =
            create_index_with_boolean_permutations(num_docs, num_fields);
        let searcher = index.reader().unwrap().searcher();
        proptest!(|(ast in arb_boolean_query_ast(num_fields))| {
            let query = ast.to_query(&fields, range_field);

            let mut matching_docs = HashSet::new();
            for doc_id in doc_ids(num_docs, num_fields) {
                if ast.matches(doc_id as DocId) {
                    matching_docs.insert(doc_id as DocId);
                }
            }

            let doc_addresses = searcher.search(&*query, &DocSetCollector).unwrap();
            let result_docs: HashSet<DocId> =
                doc_addresses.into_iter().map(|doc_address| doc_address.doc_id).collect();
            prop_assert_eq!(result_docs, matching_docs);
        });
    }

    struct ScoredDocSetCollector;

    struct ScoredDocSetSegmentCollector {
        segment_ord: u32,
        docs: HashSet<DocId>,
    }

    impl Collector for ScoredDocSetCollector {
        type Fruit = HashSet<crate::DocAddress>;
        type Child = ScoredDocSetSegmentCollector;

        fn for_segment(
            &self,
            segment_ord: crate::SegmentOrdinal,
            _reader: &crate::SegmentReader,
        ) -> crate::Result<Self::Child> {
            Ok(ScoredDocSetSegmentCollector {
                segment_ord,
                docs: HashSet::new(),
            })
        }

        fn requires_scoring(&self) -> bool {
            true
        }

        fn merge_fruits(
            &self,
            segment_fruits: Vec<(u32, HashSet<DocId>)>,
        ) -> crate::Result<Self::Fruit> {
            Ok(segment_fruits
                .into_iter()
                .flat_map(|(segment_ord, docs)| {
                    docs.into_iter()
                        .map(move |doc| crate::DocAddress::new(segment_ord, doc))
                })
                .collect())
        }
    }

    impl SegmentCollector for ScoredDocSetSegmentCollector {
        type Fruit = (u32, HashSet<DocId>);

        fn collect(&mut self, doc: DocId, _score: crate::Score) {
            self.docs.insert(doc);
        }

        fn harvest(self) -> Self::Fruit {
            (self.segment_ord, self.docs)
        }
    }

    const UNION_NUM_DOCS: DocId = 10_000;
    const UNION_TOKEN_NAMES: [&str; 6] = [
        "dense",
        "frequent",
        "periodic",
        "sparsealpha",
        "sparsebeta",
        "sparsetail",
    ];

    #[derive(Clone, Debug)]
    enum UnionQueryAST {
        Leaf(usize),
        Union(Vec<UnionQueryAST>),
    }

    impl UnionQueryAST {
        fn matches(&self, doc: DocId) -> bool {
            match self {
                UnionQueryAST::Leaf(token) => token_matches(*token, doc),
                UnionQueryAST::Union(children) => children.iter().any(|child| child.matches(doc)),
            }
        }

        fn to_query(&self, field: Field) -> Box<dyn Query> {
            match self {
                UnionQueryAST::Leaf(token) => Box::new(TermQuery::new(
                    Term::from_field_text(field, UNION_TOKEN_NAMES[*token]),
                    crate::schema::IndexRecordOption::Basic,
                )),
                UnionQueryAST::Union(children) => Box::new(BooleanQuery::new(
                    children
                        .iter()
                        .map(|child| (Occur::Should, child.to_query(field)))
                        .collect(),
                )),
            }
        }
    }

    fn token_matches(token: usize, doc: DocId) -> bool {
        match token {
            0 => doc % 3 == 0,
            1 => doc % 5 <= 1,
            2 => doc % 257 == 17,
            // These gaps are deliberately larger than the buffered union's 4,096-doc window.
            3 => matches!(doc, 10 | 9_001),
            4 => matches!(doc, 63 | 4_700 | 9_990),
            5 => matches!(doc, 8_500 | 9_999),
            _ => unreachable!(),
        }
    }

    fn create_sparse_union_index() -> (Index, Field) {
        let mut schema_builder = Schema::builder();
        let field = schema_builder.add_text_field("tokens", TEXT);
        let index = Index::create_in_ram(schema_builder.build());
        let mut writer = index.writer_for_tests().unwrap();

        for doc in 0..UNION_NUM_DOCS {
            let text = UNION_TOKEN_NAMES
                .iter()
                .enumerate()
                .filter_map(|(token, name)| token_matches(token, doc).then_some(*name))
                .collect::<Vec<_>>()
                .join(" ");
            let mut document: BTreeMap<_, OwnedValue> = BTreeMap::default();
            document.insert(field, text.into());
            writer.add_document(document).unwrap();
        }
        writer.commit().unwrap();
        (index, field)
    }

    fn arb_union_query_ast() -> impl Strategy<Value = UnionQueryAST> {
        (0..UNION_TOKEN_NAMES.len())
            .prop_map(UnionQueryAST::Leaf)
            .prop_recursive(4, 32, 6, |inner| {
                vec(inner, 2..6).prop_map(UnionQueryAST::Union)
            })
    }

    #[test]
    fn proptest_real_index_buffered_union() {
        let (index, field) = create_sparse_union_index();
        let searcher = index.reader().unwrap().searcher();

        proptest!(ProptestConfig::with_cases(32), |(generated in arb_union_query_ast())| {
            // Keeping these two leaves in every generated tree makes a later refill see the
            // sparse scorer already positioned beyond the next union window. `generated` makes
            // the resulting unions flat or nested and adds other posting distributions.
            let ast = UnionQueryAST::Union(vec![
                UnionQueryAST::Leaf(0),
                UnionQueryAST::Leaf(3),
                generated,
            ]);
            let query = ast.to_query(field);
            let expected: HashSet<DocId> = (0..UNION_NUM_DOCS)
                .filter(|&doc| ast.matches(doc))
                .collect();

            let unscored_docs = searcher.search(&*query, &DocSetCollector).unwrap();
            let unscored_result: HashSet<DocId> = unscored_docs
                .into_iter()
                .map(|address| address.doc_id)
                .collect();
            prop_assert_eq!(&unscored_result, &expected);

            // This collector requests scores without enabling TopDocs' Block-WAND path, so it
            // exercises the scored buffered-union refill.
            let scored_docs = searcher.search(&*query, &ScoredDocSetCollector).unwrap();
            let scored_result: HashSet<DocId> = scored_docs
                .into_iter()
                .map(|address| address.doc_id)
                .collect();
            prop_assert_eq!(scored_result, expected);
        });
    }
}
