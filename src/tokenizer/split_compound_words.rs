use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

use super::{Token, TokenFilter, TokenStream, Tokenizer};

/// A [`TokenFilter`] which splits compound words into their parts
/// based on a given dictionary.
///
/// Words only will be split if they can be fully decomposed into
/// consecutive matches into the given dictionary.
///
/// This is mostly useful to split [compound nouns][compound] common to many
/// Germanic languages into their constituents.
///
/// # Example
///
/// The quality of the dictionary determines the quality of the splits,
/// e.g. the missing stem "back" of "backen" implies that "brotbackautomat"
/// is not split in the following example.
///
/// ```rust
/// use tantivy::tokenizer::{SimpleTokenizer, SplitCompoundWords, TextAnalyzer};
///
/// let mut tokenizer =
///        TextAnalyzer::builder(SimpleTokenizer::default())
///        .filter(
///            SplitCompoundWords::from_dictionary([
///                 "dampf", "schiff", "fahrt", "brot", "backen", "automat",
///            ])
///            .unwrap()
///        )
///        .build();
/// {
///     let mut stream = tokenizer.token_stream("dampfschifffahrt");
///     assert_eq!(stream.next().unwrap().text, "dampf");
///     assert_eq!(stream.next().unwrap().text, "schiff");
///     assert_eq!(stream.next().unwrap().text, "fahrt");
///     assert_eq!(stream.next(), None);
/// }
/// let mut stream = tokenizer.token_stream("brotbackautomat");
/// assert_eq!(stream.next().unwrap().text, "brotbackautomat");
/// assert_eq!(stream.next(), None);
/// ```
///
/// [compound]: https://en.wikipedia.org/wiki/Compound_(linguistics)
#[derive(Clone)]
pub struct SplitCompoundWords {
    dict: AhoCorasick,
}

impl SplitCompoundWords {
    /// Create a filter from a given dictionary.
    ///
    /// The dictionary will be used to construct an [`AhoCorasick`] automaton
    /// with reasonable defaults. See [`from_automaton`][Self::from_automaton] if
    /// more control over its construction is required.
    pub fn from_dictionary<I, P>(dict: I) -> crate::Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        let dict = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .build(dict)
            .map_err(|err| {
                crate::TantivyError::InvalidArgument(format!(
                    "Failed to build Aho-Corasick automaton from dictionary: {err}"
                ))
            })?;

        Ok(Self::from_automaton(dict))
    }

    /// Create a filter from a given automaton.
    ///
    /// The automaton should use one of the leftmost-first match kinds
    /// and it should not be anchored.
    pub fn from_automaton(dict: AhoCorasick) -> Self {
        Self { dict }
    }
}

impl TokenFilter for SplitCompoundWords {
    type Tokenizer<T: Tokenizer> = SplitCompoundWordsFilter<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> SplitCompoundWordsFilter<T> {
        SplitCompoundWordsFilter {
            dict: self.dict,
            inner: tokenizer,
            cuts: Vec::new(),
            parts: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct SplitCompoundWordsFilter<T> {
    dict: AhoCorasick,
    inner: T,
    cuts: Vec<usize>,
    parts: Vec<Token>,
}

impl<T: Tokenizer> Tokenizer for SplitCompoundWordsFilter<T> {
    type TokenStream<'a> = SplitCompoundWordsTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.cuts.clear();
        self.parts.clear();
        SplitCompoundWordsTokenStream {
            dict: self.dict.clone(),
            tail: self.inner.token_stream(text),
            cuts: &mut self.cuts,
            parts: &mut self.parts,
            position_offset: 0,
        }
    }
}

pub struct SplitCompoundWordsTokenStream<'a, T> {
    dict: AhoCorasick,
    tail: T,
    cuts: &'a mut Vec<usize>,
    parts: &'a mut Vec<Token>,
    /// Number of extra positions consumed by previously-emitted splits.
    /// Each compound word that is split into N parts contributes (N - 1)
    /// to this offset, so that subsequent tokens from `tail` (and the
    /// parts of subsequent splits) appear at the correct, monotonically
    /// increasing position.
    position_offset: usize,
}

impl<T: TokenStream> SplitCompoundWordsTokenStream<'_, T> {
    // Will use `self.cuts` to fill `self.parts` if `self.tail.token()`
    // can fully be split into consecutive matches against `self.dict`.
    fn split(&mut self) {
        let token = self.tail.token();
        let base_position = token.position.wrapping_add(self.position_offset);
        let base_offset_from = token.offset_from;
        let mut text = token.text.as_str();

        self.cuts.clear();
        let mut pos = 0;

        for match_ in self.dict.find_iter(text) {
            if pos != match_.start() {
                break;
            }

            self.cuts.push(pos);
            pos = match_.end();
        }

        if pos == token.text.len() {
            let num_parts = self.cuts.len();
            // Fill `self.parts` in reverse order,
            // so that `self.parts.pop()` yields
            // the tokens in their original order.
            // The cuts encode the start byte of each part within `text`;
            // we walk them right-to-left, splitting `text` into (head, tail)
            // pairs so that each part receives its correct slice as well as
            // its correct position and byte offsets.
            let mut next_byte_end = text.len();
            for (rev_idx, cut) in self.cuts.iter().rev().enumerate() {
                let part_idx = num_parts - 1 - rev_idx; // 0-based index in document order
                let (head, tail) = text.split_at(*cut);

                text = head;
                self.parts.push(Token {
                    offset_from: base_offset_from + *cut,
                    offset_to: base_offset_from + next_byte_end,
                    position: base_position.wrapping_add(part_idx),
                    position_length: 1,
                    text: tail.to_owned(),
                });
                next_byte_end = *cut;
            }

            // Account for the (num_parts - 1) extra positions injected by this
            // split so that subsequent tokens line up correctly.
            if num_parts > 0 {
                self.position_offset = self.position_offset.wrapping_add(num_parts - 1);
            }
        } else {
            // No split: still apply the running position offset to the
            // tail token so its position remains consistent with previously
            // split tokens.
            if self.position_offset != 0 {
                let tok = self.tail.token_mut();
                tok.position = tok.position.wrapping_add(self.position_offset);
            }
        }
    }
}

impl<T: TokenStream> TokenStream for SplitCompoundWordsTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        self.parts.pop();

        if !self.parts.is_empty() {
            return true;
        }

        if !self.tail.advance() {
            return false;
        }

        // Will yield either `self.parts.last()` or
        // `self.tail.token()` if it could not be split.
        self.split();
        true
    }

    fn token(&self) -> &Token {
        self.parts.last().unwrap_or_else(|| self.tail.token())
    }

    fn token_mut(&mut self) -> &mut Token {
        self.parts
            .last_mut()
            .unwrap_or_else(|| self.tail.token_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{SimpleTokenizer, TextAnalyzer};

    #[test]
    fn splitting_compound_words_works() {
        let mut tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(SplitCompoundWords::from_dictionary(["foo", "bar"]).unwrap())
            .build();

        {
            let mut stream = tokenizer.token_stream("");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foo bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobarbaz");
            assert_eq!(stream.next().unwrap().text, "foobarbaz");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("baz foobar qux");
            assert_eq!(stream.next().unwrap().text, "baz");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "qux");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobar foobar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobar foo bar foobar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobazbar foo bar foobar");
            assert_eq!(stream.next().unwrap().text, "foobazbar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("foobar qux foobar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "qux");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next(), None);
        }

        {
            let mut stream = tokenizer.token_stream("barfoo");
            assert_eq!(stream.next().unwrap().text, "bar");
            assert_eq!(stream.next().unwrap().text, "foo");
            assert_eq!(stream.next(), None);
        }
    }

    #[test]
    fn split_compound_words_assigns_distinct_positions_to_parts() {
        // The parts produced by splitting a compound word should each receive
        // a distinct position (relative to the parent token) so that downstream
        // consumers (e.g. PhraseQuery) treat them as adjacent rather than as
        // sitting on top of each other.
        let mut tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(SplitCompoundWords::from_dictionary(["foo", "bar"]).unwrap())
            .build();

        let mut stream = tokenizer.token_stream("foobar");
        let first = stream.next().unwrap().clone();
        assert_eq!(first.text, "foo");
        let second = stream.next().unwrap().clone();
        assert_eq!(second.text, "bar");
        assert_eq!(stream.next(), None);
        // The two parts of the compound word must have *different* positions,
        // otherwise a phrase query for "foo bar" cannot match the indexed
        // "foobar" form (both would share position 0).
        assert_ne!(
            first.position, second.position,
            "split parts share the same position (both = {})",
            first.position
        );
        assert!(
            first.position < second.position,
            "second part position should be greater than first (got {} vs {})",
            first.position,
            second.position
        );
    }

    #[test]
    fn phrase_query_matches_split_compound_word() -> crate::Result<()> {
        // Through the public API: a phrase query over the constituent parts
        // of a compound word should match documents whose text contains the
        // compound. With the bug the split parts share a single position,
        // so PhraseQuery (which requires position(t_i) + 1 == position(t_{i+1}))
        // returns zero hits.
        use crate::collector::Count;
        use crate::query::PhraseQuery;
        use crate::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions};
        use crate::{Index, IndexWriter, TantivyDocument, Term};

        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("compound")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        let text_field = schema_builder.add_text_field("text", text_options);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(SplitCompoundWords::from_dictionary(["foo", "bar"]).unwrap())
            .build();
        index.tokenizers().register("compound", analyzer);

        let mut writer: IndexWriter<TantivyDocument> = index.writer_for_tests()?;
        writer.add_document(doc!(text_field => "foobar"))?;
        writer.commit()?;

        let reader = index.reader()?;
        let searcher = reader.searcher();

        let phrase = PhraseQuery::new(vec![
            Term::from_field_text(text_field, "foo"),
            Term::from_field_text(text_field, "bar"),
        ]);
        let count = searcher.search(&phrase, &Count)?;
        assert_eq!(count, 1, "phrase 'foo bar' should match indexed 'foobar'");
        Ok(())
    }
}
