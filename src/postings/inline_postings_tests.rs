use crate::collector::{Count, TopDocs};
use crate::fieldnorm::FieldNormReader;
use crate::indexer::NoMergePolicy;
use crate::postings::{LoadedPostingsReader, Postings};
use crate::query::{Bm25Weight, PhraseQuery, RegexPhraseQuery, TermQuery};
use crate::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions, TEXT};
use crate::{DocSet, Index, Term, TERMINATED};

#[test]
fn test_inline_postings_mixed_legacy_merge() -> crate::Result<()> {
    for destination_cutoff in [1, 0] {
        let mut schema = Schema::builder();
        let field = schema.add_text_field("text", TEXT);
        let mut index = Index::create_in_ram(schema.build());
        let mut segments = Vec::new();
        for (cutoff, text, token) in [
            (0, "legacy shared shared tail", "legacy"),
            (1, "modern shared tail", "modern"),
        ] {
            index.settings_mut().max_inline_docs = cutoff;
            let mut writer = index.writer_for_tests()?;
            writer.set_merge_policy(Box::new(NoMergePolicy));
            writer.add_document(doc!(field => text))?;
            writer.commit()?;
            writer.wait_merging_threads()?;
            let id = index
                .searchable_segment_ids()?
                .into_iter()
                .find(|id| !segments.contains(id))
                .unwrap();
            segments.push(id);
            let searcher = index.reader()?.searcher();
            let segment = searcher
                .segment_readers()
                .iter()
                .find(|segment| segment.segment_id() == id)
                .unwrap();
            let info = segment
                .inverted_index(field)?
                .get_term_info(&Term::from_field_text(field, token))?
                .unwrap();
            assert_eq!(info.inline_postings.is_some(), cutoff == 1);
        }
        index.settings_mut().max_inline_docs = destination_cutoff;
        let mut writer = index.writer_for_tests::<crate::TantivyDocument>()?;
        writer.set_merge_policy(Box::new(NoMergePolicy));
        writer.merge(&segments).wait()?;
        let searcher = index.reader()?.searcher();
        assert_eq!(searcher.segment_readers().len(), 1);
        assert_eq!(searcher.num_docs(), 2);
        let inverted = searcher.segment_reader(0).inverted_index(field)?;
        let legacy_doc = inverted
            .read_postings(
                &Term::from_field_text(field, "legacy"),
                IndexRecordOption::Basic,
            )?
            .unwrap()
            .doc();
        for (token, count) in [("legacy", 1), ("modern", 1), ("shared", 2), ("tail", 2)] {
            let term = Term::from_field_text(field, token);
            let info = inverted.get_term_info(&term)?.unwrap();
            assert_eq!(info.doc_freq, count);
            assert_eq!(
                info.inline_postings.is_some(),
                destination_cutoff == 1 && count == 1
            );
            let mut postings = inverted
                .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                .unwrap();
            for ordinal in 0..count {
                let doc = match token {
                    "legacy" => legacy_doc,
                    "modern" => 1 - legacy_doc,
                    _ => ordinal,
                };
                assert_eq!(postings.doc(), doc);
                let expected = match token {
                    "shared" if doc == legacy_doc => vec![1, 2],
                    "shared" => vec![1],
                    "tail" if doc == legacy_doc => vec![3],
                    "tail" => vec![2],
                    _ => vec![0],
                };
                assert_eq!(postings.term_freq(), expected.len() as u32);
                let mut positions = Vec::new();
                postings.positions(&mut positions);
                assert_eq!(positions, expected);
                postings.advance();
            }
            assert_eq!(postings.doc(), TERMINATED);
            assert_eq!(
                searcher.search(&TermQuery::new(term, IndexRecordOption::WithFreqs), &Count)?,
                count as usize
            );
        }
        let phrase = PhraseQuery::new(vec![
            Term::from_field_text(field, "shared"),
            Term::from_field_text(field, "tail"),
        ]);
        assert_eq!(searcher.search(&phrase, &Count)?, 2);
        let regex = RegexPhraseQuery::new(field, vec!["shared".into(), "tail".into()]);
        assert_eq!(searcher.search(&regex, &Count)?, 2);
    }
    Ok(())
}

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
fn test_inline_postings_json_top_score_and_numeric_frequency_resets() -> crate::Result<()> {
    use crate::query::QueryParser;

    let mut schema = Schema::builder();
    let json = schema.add_json_field("json", TEXT);
    let mut index = Index::create_in_ram(schema.build());
    index.settings_mut().max_inline_docs = 8;
    let mut writer = index.writer_for_tests()?;
    for doc in 0..260 {
        let mut value = serde_json::json!({"n": 2, "long": "word word word word"});
        if doc < 9 {
            value["external"] = serde_json::json!(4);
        }
        if doc < 2 {
            value["s"] = serde_json::json!(if doc == 0 {
                "text text"
            } else {
                "text text text text text text text text text text"
            });
            value["inline"] = serde_json::json!(3);
        }
        writer.add_document(doc!(json => value))?;
    }
    writer.commit()?;
    let searcher = index.reader()?.searcher();
    let query =
        QueryParser::for_index(&index, vec![json]).parse_query("json.s:text OR json.inline:3")?;
    let top_one = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
    let top_two = searcher.search(&query, &TopDocs::with_limit(2).order_by_score())?;
    assert_eq!(top_two[0].1.doc_id, 1);
    assert_eq!(top_one[0], top_two[0]);

    let inverted = searcher.segment_reader(0).inverted_index(json)?;
    let mut numeric = Term::from_field_json_path(json, "n", false);
    numeric.append_type_and_fast_value(2i64);
    let mut inline_numeric = Term::from_field_json_path(json, "inline", false);
    inline_numeric.append_type_and_fast_value(3i64);
    let mut text = Term::from_field_json_path(json, "s", false);
    text.append_type_and_str("text");
    let mut long_text = Term::from_field_json_path(json, "long", false);
    long_text.append_type_and_str("word");
    let mut external_numeric = Term::from_field_json_path(json, "external", false);
    external_numeric.append_type_and_fast_value(4i64);
    let mut block = inverted
        .read_block_postings(&numeric, IndexRecordOption::WithFreqsAndPositions)?
        .unwrap();
    for (term, freqs) in [
        (&text, &[2, 10][..]),
        (&long_text, &[4; 128][..]),
        (&inline_numeric, &[][..]),
        (&numeric, &[][..]),
        (&text, &[2, 10][..]),
        (&external_numeric, &[][..]),
    ] {
        let info = inverted.get_term_info(term)?.unwrap();
        inverted.reset_block_postings_from_terminfo(&info, &mut block)?;
        assert_eq!(block.doc(0), 0);
        assert_eq!(block.doc_freq(), info.doc_freq);
        assert_eq!(block.position_offset(), 0);
        assert_eq!(block.freqs(), freqs);
        assert_eq!(block.freq(0), freqs.first().copied().unwrap_or(1));
        assert_eq!(block.reads_freqs(), !freqs.is_empty());
        if info.doc_freq > 8 {
            let i = block.seek(info.doc_freq - 1);
            assert_eq!(block.doc(i), info.doc_freq - 1);
            assert_eq!(block.freq(i), freqs.last().copied().unwrap_or(1));
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
        // Larger lists come first so each term's position depends only on its order and frequency.
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
                    let mut postings = inverted.read_postings(&term, requested_option)?.unwrap();
                    let mut cursor = LoadedPostingsReader::decode_remaining(&mut postings.clone());
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
                                assert_eq!(positions, (start..start + freq).collect::<Vec<_>>());
                            } else {
                                assert!(positions.is_empty());
                            }
                            postings.advance();
                        }
                        assert_eq!(postings.doc(), TERMINATED);
                    }
                    let mut postings = inverted.read_postings(&term, requested_option)?.unwrap();
                    let block = &mut postings.block_cursor;
                    assert_eq!(block.rank(count * 2 - 1), count - 1);
                    assert_eq!(block.rank(count * 2), count);
                    let weight = Bm25Weight::for_one_term(count as u64, searcher.num_docs(), 10.0);
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
fn test_inline_single_occurrence_and_fallbacks() -> crate::Result<()> {
    for option in [
        IndexRecordOption::Basic,
        IndexRecordOption::WithFreqs,
        IndexRecordOption::WithFreqsAndPositions,
    ] {
        for cutoff in [0, 8] {
            let mut schema = Schema::builder();
            let field = schema.add_text_field(
                "text",
                TextOptions::default()
                    .set_indexing_options(TextFieldIndexing::default().set_index_option(option)),
            );
            let mut index = Index::create_in_ram(schema.build());
            index.settings_mut().max_inline_docs = cutoff;
            let mut writer = index.writer_for_tests()?;
            writer.add_document(doc!(field => "filler"))?;
            writer.add_document(doc!(field => "prefix once repeated repeated suffix"))?;
            writer.commit()?;
            let searcher = index.reader()?.searcher();
            let inverted = searcher.segment_reader(0).inverted_index(field)?;
            // Read in dictionary order to cover external lists between inline lists.
            for (token, expected_positions) in [
                ("once", vec![1]),
                ("prefix", vec![0]),
                ("repeated", vec![2, 3]),
                ("suffix", vec![4]),
            ] {
                let term = Term::from_field_text(field, token);
                let info = inverted.get_term_info(&term)?.unwrap();
                assert_eq!(info.doc_freq, 1);
                assert_eq!(
                    info.inline_postings.is_some(),
                    cutoff > 0
                        && (cfg!(feature = "quickwit")
                            || (option.has_positions() && expected_positions.len() == 1)),
                    "term={token}, option={option:?}, cutoff={cutoff}"
                );
                let mut postings = inverted
                    .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)?
                    .unwrap();
                assert_eq!(postings.doc(), 1);
                assert_eq!(
                    postings.term_freq(),
                    if option.has_freq() {
                        expected_positions.len() as u32
                    } else {
                        1
                    }
                );
                let mut positions = Vec::new();
                postings.positions(&mut positions);
                if option.has_positions() {
                    assert_eq!(positions, expected_positions);
                } else {
                    assert!(positions.is_empty());
                }
                assert_eq!(postings.advance(), TERMINATED);
            }
        }
    }
    Ok(())
}

#[test]
fn test_inline_single_occurrence_merge_and_delete_conversion() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let field = schema.add_text_field("text", TEXT);
    let mut index = Index::create_in_ram(schema.build());
    index.settings_mut().max_inline_docs = 8;
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
            assert_eq!(info.inline_postings.is_some(), cfg!(feature = "quickwit"));
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
    // Deletion turns the FST's external "shrink" into inline postings. Merging
    // the two inline "grow" lists turns them into one external list.
    for (term, count, inline) in [(&shrink, 1, true), (&grow, 2, cfg!(feature = "quickwit"))] {
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
fn test_inline_postings_merge_and_delete_remapping() -> crate::Result<()> {
    let mut schema = Schema::builder();
    let field = schema.add_text_field("text", TEXT);
    let basic = schema.add_text_field("basic", crate::schema::STRING);
    let mut index = Index::create_in_ram(schema.build());
    index.settings_mut().max_inline_docs = 8;
    let mut writer = index.writer_for_tests()?;
    writer.set_merge_policy(Box::new(NoMergePolicy));
    writer.add_document(doc!(field => "shrink remove"))?;
    for _ in 0..8 {
        writer.add_document(
            doc!(field => "shrink grow grow", basic => "present", basic => "inline"),
        )?;
    }
    writer.commit()?;
    let first_segment = index.searchable_segment_ids()?[0];
    writer.wait_merging_threads()?;
    index.settings_mut().max_inline_docs = 32;
    let mut writer = index.writer_for_tests()?;
    writer.set_merge_policy(Box::new(NoMergePolicy));
    writer.add_document(doc!(field => "grow", basic => "present"))?;
    writer.commit()?;
    let second_segment = index
        .searchable_segment_ids()?
        .into_iter()
        .find(|id| *id != first_segment)
        .unwrap();
    let shrink = Term::from_field_text(field, "shrink");
    let grow = Term::from_field_text(field, "grow");
    let reader = index.reader()?;
    writer.delete_term(Term::from_field_text(field, "remove"));
    writer.commit()?;
    writer.wait_merging_threads()?;
    index.settings_mut().max_inline_docs = 8;
    let mut writer = index.writer_for_tests::<crate::TantivyDocument>()?;
    writer.set_merge_policy(Box::new(NoMergePolicy));
    writer.merge(&[first_segment, second_segment]).wait()?;
    reader.reload()?;
    let searcher = reader.searcher();
    assert_eq!(searcher.segment_readers().len(), 1);
    assert_eq!(searcher.num_docs(), 9);
    let inverted = searcher.segment_reader(0).inverted_index(field)?;
    for (field, token, count) in [
        (field, "shrink", 8),
        (field, "grow", 9),
        (basic, "present", 9),
        (basic, "inline", 8),
    ] {
        let inverted = searcher.segment_reader(0).inverted_index(field)?;
        let term = Term::from_field_text(field, token);
        let info = inverted.get_term_info(&term)?.unwrap();
        assert_eq!(info.doc_freq, count);
        assert_eq!(
            info.inline_postings.is_some(),
            cfg!(feature = "quickwit") && count == 8
        );
        let mut postings = inverted
            .read_postings_from_terminfo(&info, IndexRecordOption::WithFreqsAndPositions)?;
        if field == basic {
            if let Some(inline) = &info.inline_postings {
                assert!(inline.term_freqs.is_empty());
                assert!(inline.position_deltas.is_empty());
            }
            assert!(postings.block_cursor.freqs().is_empty());
        }
        for doc in 0..count {
            assert_eq!(postings.doc(), doc);
            let expected = match token {
                "shrink" => vec![0],
                "grow" if doc < 8 => vec![1, 2],
                "grow" => vec![0],
                _ => vec![],
            };
            assert_eq!(postings.term_freq(), expected.len().max(1) as u32);
            let mut positions = Vec::new();
            postings.positions(&mut positions);
            assert_eq!(positions, expected);
            postings.advance();
        }
        assert_eq!(postings.doc(), TERMINATED);
    }
    assert!(inverted
        .get_term_info(&Term::from_field_text(field, "remove"))?
        .is_none());
    assert_eq!(
        searcher.search(&PhraseQuery::new(vec![shrink, grow]), &Count)?,
        8
    );
    Ok(())
}
