use std::sync::Arc;

use crate::docset::{DocSet, TERMINATED};
use crate::postings::{Postings, SegmentPostings};
use crate::DocId;

/// Decoded postings shared by term dictionaries and in-memory postings readers.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct PostingsData {
    /// Strictly increasing segment-local document IDs.
    pub doc_ids: Box<[DocId]>,
    /// One frequency per document, or empty when frequencies are absent.
    pub term_freqs: Box<[u32]>,
    /// Position deltas in document order, or empty when positions are absent.
    pub position_deltas: Box<[u32]>,
}

/// An in-memory postings reader with a cursor over shared decoded data.
///
/// Used for sparse terms in `RegexPhraseQuery` to avoid retaining the block
/// decoder buffers of a `SegmentPostings` for every term.
pub struct LoadedPostings {
    data: Arc<PostingsData>,
    cursor: usize,
    position_offset: usize,
}

impl LoadedPostings {
    pub(crate) fn new(data: Arc<PostingsData>) -> Self {
        Self {
            data,
            cursor: 0,
            position_offset: 0,
        }
    }

    pub(crate) fn doc_ids(&self) -> &[DocId] {
        &self.data.doc_ids
    }

    /// Loads the remaining documents, frequencies, and available positions.
    pub fn load(segment_postings: &mut SegmentPostings) -> LoadedPostings {
        let num_docs = segment_postings.doc_freq() as usize;
        let mut doc_ids = Vec::with_capacity(num_docs);
        let mut term_freqs = Vec::with_capacity(num_docs);
        let mut position_deltas = Vec::new();
        while segment_postings.doc() != TERMINATED {
            doc_ids.push(segment_postings.doc());
            term_freqs.push(segment_postings.term_freq());
            let start = position_deltas.len();
            segment_postings.append_positions_with_offset(0, &mut position_deltas);
            let mut previous = 0;
            for position in &mut position_deltas[start..] {
                let absolute = *position;
                *position -= previous;
                previous = absolute;
            }
            segment_postings.advance();
        }
        Self::new(Arc::new(PostingsData {
            doc_ids: doc_ids.into_boxed_slice(),
            term_freqs: term_freqs.into_boxed_slice(),
            position_deltas: position_deltas.into_boxed_slice(),
        }))
    }
}

impl DocSet for LoadedPostings {
    fn advance(&mut self) -> DocId {
        if self.cursor < self.data.doc_ids.len() {
            if !self.data.position_deltas.is_empty() {
                self.position_offset += self.term_freq() as usize;
            }
            self.cursor += 1;
        }
        self.doc()
    }

    fn doc(&self) -> DocId {
        self.data
            .doc_ids
            .get(self.cursor)
            .copied()
            .unwrap_or(TERMINATED)
    }

    fn size_hint(&self) -> u32 {
        self.data.doc_ids.len() as u32
    }
}

impl Postings for LoadedPostings {
    fn term_freq(&self) -> u32 {
        self.data.term_freqs.get(self.cursor).copied().unwrap_or(1)
    }

    fn append_positions_with_offset(&mut self, offset: u32, output: &mut Vec<u32>) {
        if self.data.position_deltas.is_empty() || self.doc() == TERMINATED {
            return;
        }
        let end = self.position_offset + self.term_freq() as usize;
        let mut position = offset;
        for &delta in &self.data.position_deltas[self.position_offset..end] {
            position += delta;
            output.push(position);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub fn test_vec_postings() {
        let mut postings = LoadedPostings::new(Arc::new(PostingsData {
            doc_ids: (0..1024).map(|doc| doc * 3).collect(),
            ..Default::default()
        }));
        assert_eq!(postings.term_freq(), 1);
        assert_eq!(postings.doc(), 0u32);
        assert_eq!(postings.advance(), 3u32);
        assert_eq!(postings.seek(14u32), 15u32);
        assert_eq!(postings.seek(300u32), 300u32);
        assert_eq!(postings.seek(6000u32), TERMINATED);
    }

    #[test]
    fn test_shared_data_independent_cursors() {
        let data = Arc::new(PostingsData {
            doc_ids: Box::new([1, 3, 8]),
            term_freqs: Box::new([2, 1, 3]),
            position_deltas: Box::new([0, 2, 5, 1, 2, 4]),
        });
        let mut first = LoadedPostings::new(data.clone());
        let mut second = LoadedPostings::new(data);
        assert!(Arc::ptr_eq(&first.data, &second.data));
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

    #[test]
    fn test_empty_postings() {
        let mut postings = LoadedPostings::new(Arc::default());
        assert_eq!(postings.doc(), TERMINATED);
        assert_eq!(postings.advance(), TERMINATED);
        assert_eq!(postings.seek(TERMINATED), TERMINATED);
        let mut positions = Vec::new();
        postings.positions(&mut positions);
        assert!(positions.is_empty());
    }
}
