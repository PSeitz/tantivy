//! Postings module (also called inverted index)

mod block_search;

pub(crate) use self::block_search::search_block;

mod block_segment_postings;
pub(crate) mod compression;
mod indexing_context;
mod json_postings_writer;
mod loaded_postings;
mod merger;
mod per_field_postings_writer;
mod postings;
mod postings_writer;
mod recorder;
mod segment_postings;
/// Serializer module for the inverted index
pub mod serializer;
mod skip;
mod term_info;

pub use loaded_postings::LoadedPostings;
pub(crate) use loaded_postings::LoadedPostingsReader;
pub(crate) use merger::{next_mapped_doc, PostingsMerger};
pub(crate) use stacker::compute_table_memory_size;

pub use self::block_segment_postings::BlockSegmentPostings;
pub(crate) use self::indexing_context::IndexingContext;
pub(crate) use self::per_field_postings_writer::PerFieldPostingsWriter;
pub use self::postings::Postings;
pub(crate) use self::postings_writer::{serialize_postings, IndexingPosition, PostingsWriter};
pub use self::segment_postings::SegmentPostings;
pub use self::serializer::{FieldSerializer, InvertedIndexSerializer};
pub(crate) use self::skip::{BlockInfo, SkipReader};
pub use self::term_info::TermInfo;

#[expect(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub(crate) enum FreqReadingOption {
    NoFreq,
    SkipFreq,
    ReadFreq,
}

#[cfg(test)]
pub(crate) mod tests {
    use std::mem;

    use super::{InvertedIndexSerializer, LoadedPostingsReader, Postings};
    use crate::collector::{Count, TopDocs};
    use crate::docset::{DocSet, TERMINATED};
    use crate::fieldnorm::FieldNormReader;
    use crate::index::{Index, SegmentComponent, SegmentReader};
    use crate::indexer::operation::AddOperation;
    use crate::indexer::{NoMergePolicy, SegmentWriter};
    use crate::query::{Bm25Weight, PhraseQuery, RegexPhraseQuery, Scorer, TermQuery};
    use crate::schema::{
        Field, IndexRecordOption, Schema, Term, TextFieldIndexing, TextOptions, INDEXED, TEXT,
    };
    use crate::tokenizer::{SimpleTokenizer, MAX_TOKEN_LEN};
    use crate::{DocId, HasLen, IndexWriter, Score};

    #[test]
    fn test_inline_singleton_block_reader_reset() -> crate::Result<()> {
        let mut schema = Schema::builder();
        let field = schema.add_text_field("text", TEXT);
        let mut index = Index::create_in_ram(schema.build());
        index.settings_mut().max_inline_docs = 1;
        let mut writer = index.writer_for_tests()?;
        writer.add_document(doc!(field => "external external"))?;
        writer.add_document(doc!(field => "external external singleton external"))?;
        writer.commit()?;
        let searcher = index.reader()?.searcher();
        let inverted = searcher.segment_reader(0).inverted_index(field)?;
        let singleton = Term::from_field_text(field, "singleton");
        let external = Term::from_field_text(field, "external");
        assert!(inverted
            .get_term_info(&singleton)?
            .unwrap()
            .inline_postings
            .is_some());
        assert!(inverted
            .get_term_info(&external)?
            .unwrap()
            .inline_postings
            .is_none());
        let mut block = inverted
            .read_block_postings(&singleton, IndexRecordOption::WithFreqsAndPositions)?
            .unwrap();
        for (term, docs, freqs) in [
            (&singleton, &[1][..], &[1][..]),
            (&external, &[0, 1][..], &[2, 3][..]),
            (&singleton, &[1][..], &[1][..]),
        ] {
            let info = inverted.get_term_info(term)?.unwrap();
            inverted.reset_block_postings_from_terminfo(&info, &mut block)?;
            assert_eq!(block.docs(), docs);
            assert_eq!(block.freqs(), freqs);
            assert_eq!(block.doc_freq(), docs.len() as u32);
            assert!(block.reads_freqs());
            assert_eq!(block.position_offset(), 0);
            for (i, &freq) in freqs.iter().enumerate() {
                assert_eq!(block.freq(i), freq);
            }
            block.advance();
            assert!(block.docs().is_empty());
            assert_eq!(block.doc(0), TERMINATED);
        }
        Ok(())
    }

    #[test]
    fn test_inline_postings_record_options_at_inline_boundaries() -> crate::Result<()> {
        let options = [
            IndexRecordOption::Basic,
            IndexRecordOption::WithFreqs,
            IndexRecordOption::WithFreqsAndPositions,
        ];
        // Include a document whose positions span a compression block.
        let term_freqs = [1, 2, 130];
        // Codec tests cover each bit width. Serializer tests cover disabled inlining.
        for max_inline_docs in [1, 32] {
            // Larger lists come first so each term's position depends only on its order and
            // frequency.
            let mut counts = vec![max_inline_docs + 1, max_inline_docs, 1];
            counts.dedup();
            for indexed_option in options {
                let mut schema = Schema::builder();
                let field = schema.add_text_field(
                    "text",
                    TextOptions::default().set_indexing_options(
                        TextFieldIndexing::default().set_index_option(indexed_option),
                    ),
                );
                let mut index = Index::create_in_ram(schema.build());
                index.settings_mut().max_inline_docs = max_inline_docs;
                let mut writer = index.writer_for_tests()?;
                let largest_count = max_inline_docs + 1;
                // Gaps distinguish absolute document ids from posting ordinals.
                for doc in 0..largest_count * 2 + 1 {
                    let mut tokens = Vec::new();
                    if doc % 2 == 1 {
                        let ordinal = doc / 2;
                        for &count in counts.iter().filter(|&&count| count > ordinal) {
                            tokens.extend(std::iter::repeat_n(
                                format!("term{count}"),
                                term_freqs[ordinal as usize % 3] as usize,
                            ));
                        }
                    }
                    writer.add_document(doc!(field => tokens.join(" ")))?;
                }
                writer.commit()?;
                let searcher = index.reader()?.searcher();
                assert_eq!(searcher.segment_readers().len(), 1);
                let inverted = searcher.segment_reader(0).inverted_index(field)?;
                let norms = FieldNormReader::for_test(&vec![10; (largest_count * 2 + 1) as usize]);
                for (term_ordinal, &count) in counts.iter().enumerate() {
                    let term = Term::from_field_text(field, &format!("term{count}"));
                    let info = inverted.get_term_info(&term)?.unwrap();
                    assert_eq!(info.doc_freq, count);
                    assert_eq!(
                        info.inline_postings.is_some(),
                        count <= max_inline_docs
                            && (cfg!(feature = "quickwit")
                                || (count == 1
                                    && indexed_option.has_positions()
                                    && term_freqs[0] == 1)),
                        "limit={max_inline_docs}, count={count}, option={indexed_option:?}"
                    );
                    for requested_option in options {
                        let option = indexed_option.downgrade(requested_option);
                        let mut postings =
                            inverted.read_postings(&term, requested_option)?.unwrap();
                        let mut cursor =
                            LoadedPostingsReader::decode_remaining(&mut postings.clone());
                        assert_eq!(cursor.size_hint(), count);
                        for postings in [&mut postings as &mut dyn Postings, &mut cursor] {
                            for ordinal in 0..count {
                                assert_eq!(postings.doc(), ordinal * 2 + 1);
                                let freq = if option.has_freq() {
                                    term_freqs[ordinal as usize % 3]
                                } else {
                                    1
                                };
                                assert_eq!(postings.term_freq(), freq);
                                let mut positions = Vec::new();
                                postings.positions(&mut positions);
                                if option.has_positions() {
                                    let start = term_ordinal as u32 * freq;
                                    assert_eq!(
                                        positions,
                                        (start..start + freq).collect::<Vec<_>>()
                                    );
                                } else {
                                    assert!(positions.is_empty());
                                }
                                postings.advance();
                            }
                            assert_eq!(postings.doc(), TERMINATED);
                        }
                        let mut postings =
                            inverted.read_postings(&term, requested_option)?.unwrap();
                        let block = &mut postings.block_cursor;
                        assert_eq!(block.rank(count * 2 - 1), count - 1);
                        assert_eq!(block.rank(count * 2), count);
                        let weight =
                            Bm25Weight::for_one_term(count as u64, searcher.num_docs(), 10.0);
                        let max_freq = if option.has_freq() {
                            *term_freqs[..(count as usize).min(3)].iter().max().unwrap()
                        } else {
                            1
                        };
                        assert_eq!(
                            block.block_max_score(&norms, &weight),
                            weight.score(norms.fieldnorm_id(0), max_freq)
                        );
                        assert_eq!(postings.seek((count * 2 - 2).max(1)), count * 2 - 1);
                        assert_eq!(postings.seek(count * 2), TERMINATED);
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_inline_postings_phrase_matches_and_bm25_scores() -> crate::Result<()> {
        let mut expected_hits = None;
        for cutoff in [0, 2, 8] {
            let mut schema = Schema::builder();
            let field = schema.add_text_field("text", TEXT);
            let mut index = Index::create_in_ram(schema.build());
            index.settings_mut().max_inline_docs = cutoff;
            let mut writer = index.writer_for_tests()?;
            for text in [
                "alpha beta filler filler",
                "alpha beta alpha beta",
                "beta alpha filler filler",
            ] {
                writer.add_document(doc!(field => text))?;
            }
            writer.commit()?;
            let searcher = index.reader()?.searcher();
            let alpha = Term::from_field_text(field, "alpha");
            let beta = Term::from_field_text(field, "beta");
            let phrase = PhraseQuery::new(vec![alpha.clone(), beta]);
            let regex = RegexPhraseQuery::new(field, vec!["alpha".into(), "beta".into()]);
            let top_docs = TopDocs::with_limit(3).order_by_score();
            let hits = searcher.search(&phrase, &top_docs)?;
            assert_eq!(hits.len(), 2);
            assert_eq!(searcher.search(&regex, &top_docs)?, hits);
            // With Quickwit at cutoff 2, this mixes inline "filler" with external terms.
            let regex = RegexPhraseQuery::new(field, vec![".*".into(), ".*".into()]);
            assert_eq!(searcher.search(&regex, &Count)?, 3);
            let query = TermQuery::new(alpha, IndexRecordOption::WithFreqs);
            let hits = searcher.search(&query, &top_docs)?;
            assert_eq!(hits.len(), 3);
            assert_eq!(hits[0].1.doc_id, 1);
            // Inline and mixed dictionaries must preserve external-postings scores and ordering.
            assert_eq!(&hits, expected_hits.get_or_insert_with(|| hits.clone()));
        }
        Ok(())
    }

    #[test]
    fn test_inline_single_occurrence_merge_and_delete_conversion() -> crate::Result<()> {
        let mut schema = Schema::builder();
        let field = schema.add_text_field("text", TEXT);
        let mut index = Index::create_in_ram(schema.build());
        index.settings_mut().max_inline_docs = 1;
        let mut writer = index.writer_for_tests()?;
        writer.set_merge_policy(Box::new(NoMergePolicy));
        writer.add_document(doc!(field => "shrink remove"))?;
        writer.add_document(doc!(field => "prefix shrink grow"))?;
        writer.commit()?;
        writer.add_document(doc!(field => "prefix grow"))?;
        writer.commit()?;
        let reader = index.reader()?;
        let shrink = Term::from_field_text(field, "shrink");
        let grow = Term::from_field_text(field, "grow");
        let searcher = reader.searcher();
        assert_eq!(searcher.segment_readers().len(), 2);
        for segment in searcher.segment_readers() {
            let inverted = segment.inverted_index(field)?;
            let info = inverted.get_term_info(&grow)?.unwrap();
            assert_eq!(info.doc_freq, 1);
            assert!(info.inline_postings.is_some());
            if let Some(info) = inverted.get_term_info(&shrink)? {
                assert_eq!(info.doc_freq, 2);
                assert!(info.inline_postings.is_none());
            }
        }
        writer.delete_term(Term::from_field_text(field, "remove"));
        writer.commit()?;
        writer.merge(&index.searchable_segment_ids()?).wait()?;
        reader.reload()?;
        let searcher = reader.searcher();
        assert_eq!(searcher.segment_readers().len(), 1);
        assert_eq!(searcher.num_docs(), 2);
        let inverted = searcher.segment_reader(0).inverted_index(field)?;
        // Find the surviving first-segment document through "shrink" so position
        // assertions are independent of the merge's segment ordering.
        let shrink_doc = inverted
            .read_postings(&shrink, IndexRecordOption::Basic)?
            .unwrap()
            .doc();
        // Deletion turns external "shrink" into inline postings. Merging
        // the two inline "grow" lists turns them into one external list.
        for (term, count, inline) in [(&shrink, 1, true), (&grow, 2, false)] {
            let info = inverted.get_term_info(term)?.unwrap();
            assert_eq!(info.doc_freq, count);
            assert_eq!(info.inline_postings.is_some(), inline);
            let mut postings = inverted
                .read_postings(term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            for _ in 0..count {
                assert!(postings.doc() < 2);
                assert_eq!(postings.term_freq(), 1);
                let mut positions = Vec::new();
                postings.positions(&mut positions);
                let position = if term == &grow && postings.doc() == shrink_doc {
                    2
                } else {
                    1
                };
                assert_eq!(positions, vec![position]);
                postings.advance();
            }
            assert_eq!(postings.doc(), TERMINATED);
        }
        assert!(inverted
            .get_term_info(&Term::from_field_text(field, "remove"))?
            .is_none());
        assert_eq!(
            searcher.search(&PhraseQuery::new(vec![shrink, grow]), &Count)?,
            1
        );
        Ok(())
    }

    #[test]
    pub fn test_position_write() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let segment = index.new_segment();
        let mut posting_serializer = InvertedIndexSerializer::open(&segment)?;
        let mut field_serializer = posting_serializer.new_field(text_field, 120 * 4, None)?;
        field_serializer.new_term("abc".as_bytes(), 12u32, true)?;
        for doc_id in 0u32..120u32 {
            let delta_positions = vec![1, 2, 3, 2];
            field_serializer.write_doc(doc_id, 4, &delta_positions);
        }
        field_serializer.close_term()?;
        mem::drop(field_serializer);
        posting_serializer.close()?;
        let read = segment.open_read(SegmentComponent::Positions)?;
        assert_eq!(read.len(), 207);
        Ok(())
    }

    #[test]
    pub fn test_skip_positions() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let title = schema_builder.add_text_field("title", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer = index.writer_for_tests()?;
        index_writer.add_document(doc!(title => r#"abc abc abc"#))?;
        index_writer.add_document(doc!(title => r#"abc be be be be abc"#))?;
        for _ in 0..1_000 {
            index_writer.add_document(doc!(title => r#"abc abc abc"#))?;
        }
        index_writer.add_document(doc!(title => r#"abc be be be be abc"#))?;
        index_writer.commit()?;

        let searcher = index.reader()?.searcher();
        let inverted_index = searcher.segment_reader(0u32).inverted_index(title)?;
        let term = Term::from_field_text(title, "abc");
        let mut positions = Vec::new();
        {
            let mut postings = inverted_index
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            assert_eq!(postings.doc(), 0);
            postings.positions(&mut positions);
            assert_eq!(&[0, 1, 2], &positions[..]);
            postings.positions(&mut positions);
            assert_eq!(&[0, 1, 2], &positions[..]);
            assert_eq!(postings.advance(), 1);
            assert_eq!(postings.doc(), 1);
            postings.positions(&mut positions);
            assert_eq!(&[0, 5], &positions[..]);
        }
        {
            let mut postings = inverted_index
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            assert_eq!(postings.doc(), 0);
            assert_eq!(postings.advance(), 1);
            postings.positions(&mut positions);
            assert_eq!(&[0, 5], &positions[..]);
        }
        {
            let mut postings = inverted_index
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            assert_eq!(postings.seek(1), 1);
            assert_eq!(postings.doc(), 1);
            postings.positions(&mut positions);
            assert_eq!(&[0, 5], &positions[..]);
        }
        {
            let mut postings = inverted_index
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            assert_eq!(postings.seek(1002), 1002);
            assert_eq!(postings.doc(), 1002);
            postings.positions(&mut positions);
            assert_eq!(&[0, 5], &positions[..]);
        }
        {
            let mut postings = inverted_index
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            assert_eq!(postings.seek(100), 100);
            assert_eq!(postings.seek(1002), 1002);
            assert_eq!(postings.doc(), 1002);
            postings.positions(&mut positions);
            assert_eq!(&[0, 5], &positions[..]);
        }
        Ok(())
    }

    #[test]
    pub fn test_index_max_length_token() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_index_option(IndexRecordOption::WithFreqsAndPositions)
                .set_tokenizer("simple_no_truncation"),
        );
        let text_field = schema_builder.add_text_field("text", text_options);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        index
            .tokenizers()
            .register("simple_no_truncation", SimpleTokenizer::default());
        let reader = index.reader()?;
        let mut index_writer = index.writer_for_tests()?;

        let ok_token_text: String = "A".repeat(MAX_TOKEN_LEN);
        index_writer.add_document(doc!(text_field=>ok_token_text.clone()))?;
        index_writer.commit()?;
        reader.reload()?;
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0u32);
        let inverted_index = segment_reader.inverted_index(text_field)?;
        assert_eq!(inverted_index.terms().num_terms(), 1);
        let mut bytes = vec![];
        assert!(inverted_index.terms().ord_to_term(0, &mut bytes)?);
        assert_eq!(&bytes[..], ok_token_text.as_bytes());

        Ok(())
    }

    #[test]
    pub fn test_drop_token_that_are_too_long() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_index_option(IndexRecordOption::WithFreqsAndPositions)
                .set_tokenizer("simple_no_truncation"),
        );
        let text_field = schema_builder.add_text_field("text", text_options);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        index
            .tokenizers()
            .register("simple_no_truncation", SimpleTokenizer::default());
        let reader = index.reader()?;
        let mut index_writer = index.writer_for_tests()?;

        let mut exceeding_token_text: String = "A".repeat(MAX_TOKEN_LEN + 1);
        exceeding_token_text.push_str(" hello");
        index_writer.add_document(doc!(text_field=>exceeding_token_text))?;
        index_writer.commit()?;
        reader.reload()?;
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0u32);
        let inverted_index = segment_reader.inverted_index(text_field)?;
        assert_eq!(inverted_index.terms().num_terms(), 1);
        let mut bytes = vec![];
        assert!(inverted_index.terms().ord_to_term(0, &mut bytes)?);
        assert_eq!(&bytes, b"hello");

        Ok(())
    }

    #[test]
    pub fn test_position_and_fieldnorm1() -> crate::Result<()> {
        let mut positions = Vec::new();
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let segment = index.new_segment();

        {
            let mut segment_writer =
                SegmentWriter::for_segment(15_000_000, segment.clone()).unwrap();
            {
                // checking that position works if the field has two values
                let op = AddOperation {
                    opstamp: 0u64,
                    document: doc!(
                       text_field => "a b a c a d a a.",
                       text_field => "d d d d a"
                    ),
                };
                segment_writer.add_document(op)?;
            }
            {
                let op = AddOperation {
                    opstamp: 1u64,
                    document: doc!(text_field => "b a"),
                };
                segment_writer.add_document(op).unwrap();
            }
            for i in 2..1000 {
                let mut text: String = "e ".repeat(i);
                text.push_str(" a");
                let op = AddOperation {
                    opstamp: 2u64,
                    document: doc!(text_field => text),
                };
                segment_writer.add_document(op).unwrap();
            }
            segment_writer.finalize()?;
        }
        {
            let segment_reader = SegmentReader::open(&segment)?;
            {
                let fieldnorm_reader = segment_reader.get_fieldnorms_reader(text_field)?;
                assert_eq!(fieldnorm_reader.fieldnorm(0), 8 + 5);
                assert_eq!(fieldnorm_reader.fieldnorm(1), 2);
                for i in 2..1000 {
                    assert_eq!(
                        fieldnorm_reader.fieldnorm_id(i),
                        FieldNormReader::fieldnorm_to_id(i + 1)
                    );
                }
            }
            {
                let term_a = Term::from_field_text(text_field, "abcdef");
                assert!(segment_reader
                    .inverted_index(term_a.field())?
                    .read_postings(&term_a, IndexRecordOption::WithFreqsAndPositions)?
                    .is_none());
            }
            {
                let term_a = Term::from_field_text(text_field, "a");
                let mut postings_a = segment_reader
                    .inverted_index(term_a.field())?
                    .read_postings(&term_a, IndexRecordOption::WithFreqsAndPositions)?
                    .unwrap();
                assert_eq!(postings_a.len(), 1000);
                assert_eq!(postings_a.doc(), 0);
                assert_eq!(postings_a.term_freq(), 6);
                postings_a.positions(&mut positions);
                assert_eq!(&positions[..], [0, 2, 4, 6, 7, 13]);
                assert_eq!(postings_a.advance(), 1u32);
                assert_eq!(postings_a.doc(), 1u32);
                assert_eq!(postings_a.term_freq(), 1);
                for i in 2u32..1000u32 {
                    assert_eq!(postings_a.advance(), i);
                    assert_eq!(postings_a.term_freq(), 1);
                    postings_a.positions(&mut positions);
                    assert_eq!(&positions[..], [i]);
                    assert_eq!(postings_a.doc(), i);
                }
                assert_eq!(postings_a.advance(), TERMINATED);
            }
            {
                let term_e = Term::from_field_text(text_field, "e");
                let mut postings_e = segment_reader
                    .inverted_index(term_e.field())?
                    .read_postings(&term_e, IndexRecordOption::WithFreqsAndPositions)?
                    .unwrap();
                assert_eq!(postings_e.len(), 1000 - 2);
                for i in 2u32..1000u32 {
                    assert_eq!(postings_e.term_freq(), i);
                    postings_e.positions(&mut positions);
                    assert_eq!(positions.len(), i as usize);
                    for j in 0..positions.len() {
                        assert_eq!(positions[j], (j as u32));
                    }
                    assert_eq!(postings_e.doc(), i);
                    postings_e.advance();
                }
                assert_eq!(postings_e.doc(), TERMINATED);
            }
        }
        Ok(())
    }

    #[test]
    pub fn test_position_and_fieldnorm2() -> crate::Result<()> {
        let mut positions: Vec<u32> = Vec::new();
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        {
            let mut index_writer = index.writer_for_tests()?;
            index_writer.add_document(doc!(text_field => "g b b d c g c"))?;
            index_writer.add_document(doc!(text_field => "g a b b a d c g c"))?;
            index_writer.commit()?;
        }
        let term_a = Term::from_field_text(text_field, "a");
        let searcher = index.reader()?.searcher();
        let segment_reader = searcher.segment_reader(0);
        let mut postings = segment_reader
            .inverted_index(text_field)?
            .read_postings(&term_a, IndexRecordOption::WithFreqsAndPositions)?
            .unwrap();
        assert_eq!(postings.doc(), 1u32);
        postings.positions(&mut positions);
        assert_eq!(&positions[..], &[1u32, 4]);
        Ok(())
    }

    #[test]
    fn test_skip_next() -> crate::Result<()> {
        let term_0 = Term::from_field_u64(Field::from_field_id(0), 0);
        let term_1 = Term::from_field_u64(Field::from_field_id(0), 1);
        let term_2 = Term::from_field_u64(Field::from_field_id(0), 2);

        let num_docs = 300u32;

        let index = {
            let mut schema_builder = Schema::builder();
            let value_field = schema_builder.add_u64_field("value", INDEXED);
            let schema = schema_builder.build();
            let index = Index::create_in_ram(schema);
            {
                let mut index_writer = index.writer_for_tests()?;
                for i in 0u64..num_docs as u64 {
                    let doc = doc!(value_field => 2u64, value_field => i % 2u64);
                    index_writer.add_document(doc)?;
                }
                assert!(index_writer.commit().is_ok());
            }
            index
        };
        let searcher = index.reader()?.searcher();
        let segment_reader = searcher.segment_reader(0);

        // check that the basic usage works
        for i in 0..num_docs - 1 {
            for j in i + 1..num_docs {
                let mut segment_postings = segment_reader
                    .inverted_index(term_2.field())?
                    .read_postings(&term_2, IndexRecordOption::Basic)?
                    .unwrap();
                assert_eq!(segment_postings.seek(i), i);
                assert_eq!(segment_postings.doc(), i);

                assert_eq!(segment_postings.seek(j), j);
                assert_eq!(segment_postings.doc(), j);
            }
        }

        {
            let mut segment_postings = segment_reader
                .inverted_index(term_2.field())?
                .read_postings(&term_2, IndexRecordOption::Basic)?
                .unwrap();

            // check that `skip_next` advances the iterator
            assert_eq!(segment_postings.doc(), 0);

            assert_eq!(segment_postings.seek(1), 1);
            assert_eq!(segment_postings.doc(), 1);

            assert_eq!(segment_postings.seek(1), 1);
            assert_eq!(segment_postings.doc(), 1);

            // check that going beyond the end is handled
            assert_eq!(segment_postings.seek(num_docs), TERMINATED);
        }

        // check that filtering works
        {
            let mut segment_postings = segment_reader
                .inverted_index(term_0.field())?
                .read_postings(&term_0, IndexRecordOption::Basic)?
                .unwrap();

            for i in 0..num_docs / 2 {
                assert_eq!(segment_postings.seek(i * 2), i * 2);
                assert_eq!(segment_postings.doc(), i * 2);
            }

            let mut segment_postings = segment_reader
                .inverted_index(term_0.field())?
                .read_postings(&term_0, IndexRecordOption::Basic)?
                .unwrap();

            for i in 0..num_docs / 2 - 1 {
                assert!(segment_postings.seek(i * 2 + 1) > i * 2);
                assert_eq!(segment_postings.doc(), (i + 1) * 2);
            }
        }

        // delete some of the documents
        {
            let mut index_writer: IndexWriter = index.writer_for_tests()?;
            index_writer.delete_term(term_0);
            assert!(index_writer.commit().is_ok());
        }
        let searcher = index.reader()?.searcher();
        assert_eq!(searcher.segment_readers().len(), 1);
        let segment_reader = searcher.segment_reader(0);

        // make sure seeking still works
        for i in 0..num_docs {
            let mut segment_postings = segment_reader
                .inverted_index(term_2.field())?
                .read_postings(&term_2, IndexRecordOption::Basic)?
                .unwrap();

            assert_eq!(segment_postings.seek(i), i);
            assert_eq!(segment_postings.doc(), i);
            if i % 2 == 0 {
                assert!(segment_reader.is_deleted(i));
            }
        }

        // now try with a longer sequence
        {
            let mut segment_postings = segment_reader
                .inverted_index(term_2.field())?
                .read_postings(&term_2, IndexRecordOption::Basic)?
                .unwrap();

            let mut last = 2; // start from 5 to avoid seeking to 3 twice
            let mut cur = 3;
            loop {
                let seek = segment_postings.seek(cur);
                if seek == TERMINATED {
                    break;
                }
                assert_eq!(seek, segment_postings.doc());
                if seek == cur {
                    assert_eq!(segment_postings.doc(), cur);
                } else {
                    assert_eq!(segment_postings.doc(), cur + 1);
                }
                let next = cur + last;
                last = cur;
                cur = next;
            }
            assert_eq!(cur, 377);
        }

        // delete everything else
        {
            let mut index_writer: IndexWriter = index.writer_for_tests()?;
            index_writer.delete_term(term_1);
            assert!(index_writer.commit().is_ok());
        }
        let searcher = index.reader()?.searcher();

        // finally, check that it's empty
        {
            let searchable_segment_ids = index.searchable_segment_ids()?;
            assert!(searchable_segment_ids.is_empty());
            assert_eq!(searcher.num_docs(), 0);
        }
        Ok(())
    }

    /// Wraps a given docset, and forward all call but the
    /// `.skip_next(...)`. This is useful to test that a specialized
    /// implementation of `.skip_next(...)` is consistent
    /// with the default implementation.
    pub(crate) struct UnoptimizedDocSet<TDocSet: DocSet>(TDocSet);

    impl<TDocSet: DocSet> UnoptimizedDocSet<TDocSet> {
        pub fn wrap(docset: TDocSet) -> UnoptimizedDocSet<TDocSet> {
            UnoptimizedDocSet(docset)
        }
    }

    impl<TDocSet: DocSet> DocSet for UnoptimizedDocSet<TDocSet> {
        fn advance(&mut self) -> DocId {
            self.0.advance()
        }

        fn doc(&self) -> DocId {
            self.0.doc()
        }

        fn size_hint(&self) -> u32 {
            self.0.size_hint()
        }
    }

    impl<TScorer: Scorer> Scorer for UnoptimizedDocSet<TScorer> {
        #[inline]
        fn score(&mut self) -> Score {
            self.0.score()
        }
    }

    pub fn test_skip_against_unoptimized<F: Fn() -> Box<dyn DocSet>>(
        postings_factory: F,
        targets: Vec<u32>,
    ) {
        for target in targets {
            let mut postings_opt = postings_factory();
            if target < postings_opt.doc() {
                continue;
            }
            let mut postings_unopt = UnoptimizedDocSet::wrap(postings_factory());
            let skip_result_opt = postings_opt.seek(target);
            let skip_result_unopt = postings_unopt.seek(target);
            assert_eq!(
                skip_result_unopt, skip_result_opt,
                "Failed while skipping to {target}"
            );
            assert!(skip_result_opt >= target);
            assert_eq!(skip_result_opt, postings_opt.doc());
            if skip_result_opt == TERMINATED {
                return;
            }
            while postings_opt.doc() != TERMINATED {
                assert_eq!(postings_opt.doc(), postings_unopt.doc());
                assert_eq!(postings_opt.advance(), postings_unopt.advance());
            }
        }
    }
}

#[cfg(all(test, feature = "unstable"))]
mod bench {
    use once_cell::sync::Lazy;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use test::{self, Bencher};

    use crate::docset::TERMINATED;
    use crate::query::Intersection;
    use crate::schema::{Field, IndexRecordOption, Schema, TantivyDocument, Term, STRING};
    use crate::{tests, DocSet, Index, IndexWriter};

    pub static TERM_A: Lazy<Term> = Lazy::new(|| {
        let field = Field::from_field_id(0);
        Term::from_field_text(field, "a")
    });
    pub static TERM_B: Lazy<Term> = Lazy::new(|| {
        let field = Field::from_field_id(0);
        Term::from_field_text(field, "b")
    });
    pub static TERM_C: Lazy<Term> = Lazy::new(|| {
        let field = Field::from_field_id(0);
        Term::from_field_text(field, "c")
    });
    pub static TERM_D: Lazy<Term> = Lazy::new(|| {
        let field = Field::from_field_id(0);
        Term::from_field_text(field, "d")
    });

    pub static INDEX: Lazy<Index> = Lazy::new(|| {
        let mut schema_builder = Schema::builder();
        let text_field = schema_builder.add_text_field("text", STRING);
        let schema = schema_builder.build();

        let mut rng: StdRng = StdRng::from_seed([1u8; 32]);

        let index = Index::create_in_ram(schema);
        let posting_list_size = 1_000_000;
        {
            let mut index_writer: IndexWriter = index.writer_for_tests().unwrap();
            for _ in 0..posting_list_size {
                let mut doc = TantivyDocument::default();
                if rng.random_bool(1f64 / 15f64) {
                    doc.add_text(text_field, "a");
                }
                if rng.random_bool(1f64 / 10f64) {
                    doc.add_text(text_field, "b");
                }
                if rng.random_bool(1f64 / 5f64) {
                    doc.add_text(text_field, "c");
                }
                doc.add_text(text_field, "d");
                index_writer.add_document(doc).unwrap();
            }
            assert!(index_writer.commit().is_ok());
        }
        index
    });

    #[bench]
    fn bench_segment_postings(b: &mut Bencher) {
        let reader = INDEX.reader().unwrap();
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0);

        b.iter(|| {
            let mut segment_postings = segment_reader
                .inverted_index(TERM_A.field())
                .unwrap()
                .read_postings(&TERM_A, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            while segment_postings.advance() != TERMINATED {}
        });
    }

    #[bench]
    fn bench_segment_intersection(b: &mut Bencher) {
        let reader = INDEX.reader().unwrap();
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0);
        b.iter(|| {
            let segment_postings_a = segment_reader
                .inverted_index(TERM_A.field())
                .unwrap()
                .read_postings(&TERM_A, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            let segment_postings_b = segment_reader
                .inverted_index(TERM_B.field())
                .unwrap()
                .read_postings(&TERM_B, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            let segment_postings_c = segment_reader
                .inverted_index(TERM_C.field())
                .unwrap()
                .read_postings(&TERM_C, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            let segment_postings_d = segment_reader
                .inverted_index(TERM_D.field())
                .unwrap()
                .read_postings(&TERM_D, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            let mut intersection = Intersection::new(
                vec![
                    segment_postings_a,
                    segment_postings_b,
                    segment_postings_c,
                    segment_postings_d,
                ],
                reader.searcher().num_docs() as u32,
            );
            while intersection.advance() != TERMINATED {}
        });
    }

    fn bench_skip_next(p: f64, b: &mut Bencher) {
        let reader = INDEX.reader().unwrap();
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0);
        let docs = tests::sample(segment_reader.num_docs(), p);

        let mut segment_postings = segment_reader
            .inverted_index(TERM_A.field())
            .unwrap()
            .read_postings(&TERM_A, IndexRecordOption::Basic)
            .unwrap()
            .unwrap();

        let mut existing_docs = Vec::new();
        for doc in &docs {
            if *doc >= segment_postings.doc() {
                existing_docs.push(*doc);
                if segment_postings.seek(*doc) == TERMINATED {
                    break;
                }
            }
        }

        b.iter(|| {
            let mut segment_postings = segment_reader
                .inverted_index(TERM_A.field())
                .unwrap()
                .read_postings(&TERM_A, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            for doc in &existing_docs {
                if segment_postings.seek(*doc) == TERMINATED {
                    break;
                }
            }
        });
    }

    #[bench]
    fn bench_skip_next_p01(b: &mut Bencher) {
        bench_skip_next(0.001, b);
    }

    #[bench]
    fn bench_skip_next_p1(b: &mut Bencher) {
        bench_skip_next(0.01, b);
    }

    #[bench]
    fn bench_skip_next_p10(b: &mut Bencher) {
        bench_skip_next(0.1, b);
    }

    #[bench]
    fn bench_skip_next_p90(b: &mut Bencher) {
        bench_skip_next(0.9, b);
    }

    #[bench]
    fn bench_iterate_segment_postings(b: &mut Bencher) {
        let reader = INDEX.reader().unwrap();
        let searcher = reader.searcher();
        let segment_reader = searcher.segment_reader(0);
        b.iter(|| {
            let n: u32 = test::black_box(17);
            let mut segment_postings = segment_reader
                .inverted_index(TERM_A.field())
                .unwrap()
                .read_postings(&TERM_A, IndexRecordOption::Basic)
                .unwrap()
                .unwrap();
            let mut s = 0u32;
            while segment_postings.doc() != TERMINATED {
                s += (segment_postings.doc() & n) % 1024;
                segment_postings.advance();
            }
            s
        });
    }
}
