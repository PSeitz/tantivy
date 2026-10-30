//! Single-occurrence postings stored in the term-info block's inline group.
use std::io;
use std::sync::Arc;

use crate::postings::{LoadedPostings, TermInfo};
use crate::TERMINATED;

pub(super) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn values(term: &TermInfo) -> io::Result<(u32, u32)> {
    let postings = term.inline_postings.as_ref().unwrap();
    if term.doc_freq != 1
        || postings.doc_ids.len() != 1
        || postings.doc_ids[0] >= TERMINATED
        || postings.term_freqs.as_ref() != [1]
        || postings.position_deltas.len() != 1
        || !term.postings_range.is_empty()
        || !term.positions_range.is_empty()
    {
        return Err(invalid(
            "FST inline postings require one document and one position",
        ));
    }
    Ok((postings.doc_ids[0], postings.position_deltas[0]))
}

pub(super) fn term_info(doc: u32, position: u32) -> io::Result<TermInfo> {
    if doc >= TERMINATED {
        return Err(invalid("invalid inline document ID"));
    }
    Ok(TermInfo {
        doc_freq: 1,
        inline_postings: Some(Arc::new(LoadedPostings {
            doc_ids: Box::new([doc]),
            term_freqs: Box::new([1]),
            position_deltas: Box::new([position]),
        })),
        ..TermInfo::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_postings_values() -> io::Result<()> {
        for doc in [0, 42, TERMINATED - 1] {
            for position in [0, 17, u32::MAX] {
                let term = term_info(doc, position)?;
                assert_eq!(values(&term)?, (doc, position));
            }
        }
        assert!(term_info(TERMINATED, 0).is_err());
        let mut term = term_info(0, 0)?;
        term.doc_freq = 2;
        assert!(values(&term).is_err());
        Ok(())
    }
}
