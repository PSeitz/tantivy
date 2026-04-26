use std::mem;

use super::{Token, TokenFilter, TokenStream, Tokenizer};

/// Token filter that lowercase terms.
#[derive(Clone)]
pub struct LowerCaser;

impl TokenFilter for LowerCaser {
    type Tokenizer<T: Tokenizer> = LowerCaserFilter<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        LowerCaserFilter {
            tokenizer,
            buffer: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct LowerCaserFilter<T> {
    tokenizer: T,
    buffer: String,
}

impl<T: Tokenizer> Tokenizer for LowerCaserFilter<T> {
    type TokenStream<'a> = LowerCaserTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.buffer.clear();
        LowerCaserTokenStream {
            tail: self.tokenizer.token_stream(text),
            buffer: &mut self.buffer,
        }
    }
}

pub struct LowerCaserTokenStream<'a, T> {
    buffer: &'a mut String,
    tail: T,
}

// writes a lowercased version of text into output.
fn to_lowercase_unicode(text: &str, output: &mut String) {
    output.clear();
    output.reserve(50);
    for c in text.chars() {
        // Contrary to the std, we do not take care of sigma special case.
        // This will have an normalizationo effect, which is ok for search.
        output.extend(c.to_lowercase());
    }
}

impl<T: TokenStream> TokenStream for LowerCaserTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        if self.token_mut().text.is_ascii() {
            // fast track for ascii.
            self.token_mut().text.make_ascii_lowercase();
        } else {
            to_lowercase_unicode(&self.tail.token().text, self.buffer);
            mem::swap(&mut self.tail.token_mut().text, self.buffer);
        }
        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tests::assert_token;
    use crate::tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer, Token};

    #[test]
    fn test_to_lower_case() {
        let tokens = token_stream_helper("Tree");
        assert_eq!(tokens.len(), 1);
        assert_token(&tokens[0], 0, "tree", 0, 4);

        let tokens = token_stream_helper("Русский текст");
        assert_eq!(tokens.len(), 2);
        assert_token(&tokens[0], 0, "русский", 0, 14);
        assert_token(&tokens[1], 1, "текст", 15, 25);
    }

    /// Regression test for "İ" (Turkish capital I with dot above, U+0130).
    ///
    /// Per Unicode, `'İ'.to_lowercase()` returns the two-character sequence
    /// `"i\u{307}"` (LATIN SMALL LETTER I + COMBINING DOT ABOVE). Tantivy's
    /// `LowerCaser` consumes this directly, which means an indexed token
    /// `"İstanbul"` becomes `"i\u{307}stanbul"`, while the lowercase of plain
    /// ASCII `"Istanbul"` becomes `"istanbul"`. The two no longer compare
    /// equal, so a TermQuery search for "istanbul" against a document
    /// containing "İstanbul" returns 0 hits even though both forms should
    /// canonicalize to the same searchable token.
    ///
    /// This end-to-end test indexes a document containing "İstanbul" and
    /// queries it with the lowercased ASCII form.
    #[test]
    fn test_lowercaser_turkish_capital_i_matches_ascii_i() -> crate::Result<()> {
        use crate::collector::Count;
        use crate::query::TermQuery;
        use crate::schema::{IndexRecordOption, Schema, Term, TEXT};
        use crate::Index;

        let mut schema_builder = Schema::builder();
        let title = schema_builder.add_text_field("title", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut index_writer = index.writer_for_tests()?;
        index_writer.add_document(crate::doc!(title => "İstanbul"))?;
        index_writer.commit()?;
        let searcher = index.reader()?.searcher();

        let term = Term::from_field_text(title, "istanbul");
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let count = searcher.search(&query, &Count)?;
        assert_eq!(
            count, 1,
            "expected 'istanbul' to match a document containing 'İstanbul' \
             after case-folding, but got {count} hits"
        );
        Ok(())
    }

    fn token_stream_helper(text: &str) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .build();

        let mut token_stream = token_stream.token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }
}
