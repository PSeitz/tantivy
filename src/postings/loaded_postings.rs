use std::sync::Arc;

use crate::docset::{DocSet, TERMINATED};
use crate::postings::{Postings, SegmentPostings};
use crate::DocId;

/// Loaded postings arrays holding document IDs, term frequencies, and position deltas.
///
/// Shared by term dictionaries and in-memory postings readers. `SegmentPostings`
/// reads postings through block decoders instead.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct LoadedPostings {
    /// Strictly increasing segment-local document IDs.
    pub doc_ids: Box<[DocId]>,
    /// One frequency per document, or empty when frequencies are absent.
    pub term_freqs: Box<[u32]>,
    /// Position deltas in document order, or empty when positions are absent.
    pub position_deltas: Box<[u32]>,
}

/// Reader over shared loaded postings.
///
/// Used for sparse terms in `RegexPhraseQuery` to avoid retaining the block
/// decoder buffers of a `SegmentPostings` for every term.
pub struct LoadedPostingsReader {
    postings: Arc<LoadedPostings>,
    doc_index: usize,
    position_offset: usize,
}

impl LoadedPostingsReader {
    pub(crate) fn new(postings: Arc<LoadedPostings>) -> Self {
        Self {
            postings,
            doc_index: 0,
            position_offset: 0,
        }
    }

    pub(crate) fn doc_ids(&self) -> &[DocId] {
        &self.postings.doc_ids
    }

    /// Decodes the remaining documents, frequencies, and positions from segment postings.
    pub fn decode_remaining(segment_postings: &mut SegmentPostings) -> LoadedPostingsReader {
        let num_docs = segment_postings.doc_freq() as usize;
        let mut doc_ids = Vec::with_capacity(num_docs);
        let mut term_freqs = Vec::with_capacity(num_docs);
        let mut position_deltas = Vec::new();
        while segment_postings.doc() != TERMINATED {
            doc_ids.push(segment_postings.doc());
            term_freqs.push(segment_postings.term_freq());
            segment_postings.append_position_deltas(&mut position_deltas);
            segment_postings.advance();
        }
        Self::new(Arc::new(LoadedPostings {
            doc_ids: doc_ids.into_boxed_slice(),
            term_freqs: term_freqs.into_boxed_slice(),
            position_deltas: position_deltas.into_boxed_slice(),
        }))
    }
}

impl DocSet for LoadedPostingsReader {
    fn advance(&mut self) -> DocId {
        if self.doc_index < self.postings.doc_ids.len() {
            if !self.postings.position_deltas.is_empty() {
                self.position_offset += self.term_freq() as usize;
            }
            self.doc_index += 1;
        }
        self.doc()
    }

    fn doc(&self) -> DocId {
        self.postings
            .doc_ids
            .get(self.doc_index)
            .copied()
            .unwrap_or(TERMINATED)
    }

    fn size_hint(&self) -> u32 {
        self.postings.doc_ids.len() as u32
    }
}

impl Postings for LoadedPostingsReader {
    fn term_freq(&self) -> u32 {
        self.postings
            .term_freqs
            .get(self.doc_index)
            .copied()
            .unwrap_or(1)
    }

    fn append_positions_with_offset(&mut self, offset: u32, output: &mut Vec<u32>) {
        if self.postings.position_deltas.is_empty() || self.doc() == TERMINATED {
            return;
        }
        let end = self.position_offset + self.term_freq() as usize;
        let mut position = offset;
        for &delta in &self.postings.position_deltas[self.position_offset..end] {
            position += delta;
            output.push(position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_postings_independent_readers() {
        let postings = Arc::new(LoadedPostings {
            doc_ids: Box::new([1, 3, 8]),
            term_freqs: Box::new([2, 1, 3]),
            position_deltas: Box::new([0, 2, 5, 1, 2, 4]),
        });
        let mut first = LoadedPostingsReader::new(postings.clone());
        let mut second = LoadedPostingsReader::new(postings);
        assert!(Arc::ptr_eq(&first.postings, &second.postings));
        assert_eq!(first.seek(3), 3);
        let mut positions = vec![99];
        first.append_positions_with_offset(10, &mut positions);
        assert_eq!(positions, [99, 15]);
        assert_eq!(second.doc(), 1);
        assert_eq!(second.term_freq(), 2);
        second.positions(&mut positions);
        assert_eq!(positions, [0, 2]);
        assert_eq!(second.seek(7), 8);
        second.positions(&mut positions);
        assert_eq!(positions, [1, 3, 7]);
        assert_eq!(second.advance(), TERMINATED);
        assert_eq!(second.advance(), TERMINATED);
        assert_eq!(first.doc(), 3);
    }
}
