use std::str::CharIndices;

use super::{Token, TokenStream, Tokenizer};

/// Tokenize the text by splitting on whitespaces.
#[derive(Clone, Default)]
pub struct WhitespaceTokenizer {
    token: Token,
}

pub struct WhitespaceTokenStream<'a> {
    text: &'a str,
    chars: CharIndices<'a>,
    token: &'a mut Token,
}

impl Tokenizer for WhitespaceTokenizer {
    type TokenStream<'a> = WhitespaceTokenStream<'a>;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> WhitespaceTokenStream<'a> {
        self.token.reset();
        WhitespaceTokenStream {
            text,
            chars: text.char_indices(),
            token: &mut self.token,
        }
    }
}

impl WhitespaceTokenStream<'_> {
    // search for the end of the current token.
    fn search_token_end(&mut self) -> usize {
        (&mut self.chars)
            .filter(|(_, c)| c.is_ascii_whitespace())
            .map(|(offset, _)| offset)
            .next()
            .unwrap_or(self.text.len())
    }
}

impl TokenStream for WhitespaceTokenStream<'_> {
    fn advance(&mut self) -> bool {
        self.token.text.clear();
        self.token.position = self.token.position.wrapping_add(1);
        while let Some((offset_from, c)) = self.chars.next() {
            if !c.is_ascii_whitespace() {
                let offset_to = self.search_token_end();
                self.token.offset_from = offset_from;
                self.token.offset_to = offset_to;
                self.token.text.push_str(&self.text[offset_from..offset_to]);
                return true;
            }
        }
        false
    }

    fn token(&self) -> &Token {
        self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        self.token
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tests::assert_token;
    use crate::tokenizer::{TextAnalyzer, Token, WhitespaceTokenizer};

    #[test]
    fn test_whitespace_tokenizer() {
        let tokens = token_stream_helper("Hello, happy tax payer!");
        assert_eq!(tokens.len(), 4);
        assert_token(&tokens[0], 0, "Hello,", 0, 6);
        assert_token(&tokens[1], 1, "happy", 7, 12);
        assert_token(&tokens[2], 2, "tax", 13, 16);
        assert_token(&tokens[3], 3, "payer!", 17, 23);
    }

    fn token_stream_helper(text: &str) -> Vec<Token> {
        let mut a = TextAnalyzer::from(WhitespaceTokenizer::default());
        let mut token_stream = a.token_stream(text);
        let mut tokens: Vec<Token> = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_whitespace_tokenizer_splits_on_unicode_whitespace() {
        // Inputs that contain Unicode whitespace characters (NBSP and an em space).
        // The `WhitespaceTokenizer` is documented to "tokenize the text by splitting on
        // whitespaces", so callers naturally expect Unicode whitespace to be treated
        // the same as ASCII whitespace. Indexing text with a non-breaking space and then
        // searching for the post-whitespace word should match.
        let tokens = token_stream_helper("hello\u{00A0}world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");

        // Em space (U+2003) should also act as a separator.
        let tokens = token_stream_helper("foo\u{2003}bar");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "foo");
        assert_eq!(tokens[1].text, "bar");
    }

    #[test]
    fn test_whitespace_tokenizer_search_finds_term_split_by_nbsp() -> crate::Result<()> {
        use crate::collector::Count;
        use crate::query::TermQuery;
        use crate::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions};
        use crate::{Index, IndexWriter, Term};

        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("whitespace")
                .set_index_option(IndexRecordOption::Basic),
        );
        let body = schema_builder.add_text_field("body", text_options);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        {
            let mut index_writer: IndexWriter = index.writer_for_tests()?;
            // The two words are separated by a NO-BREAK SPACE (U+00A0).
            index_writer.add_document(doc!(body => "hello\u{00A0}world"))?;
            index_writer.commit()?;
        }
        let reader = index.reader()?;
        let searcher = reader.searcher();

        // The whitespace tokenizer is documented to split on whitespaces. NBSP is whitespace,
        // so `world` should match the only document.
        let term_query = TermQuery::new(
            Term::from_field_text(body, "world"),
            IndexRecordOption::Basic,
        );
        let count = searcher.search(&term_query, &Count)?;
        assert_eq!(count, 1);
        Ok(())
    }
}
