use common::HasLen;

use crate::docset::DocSet;
use crate::fastfield::AliveBitSet;
use crate::positions::PositionReader;
use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
use crate::postings::{BlockSegmentPostings, Postings};
use crate::{DocId, TERMINATED};

/// `SegmentPostings` represents the inverted list or postings associated with
/// a term in a `Segment`.
///
/// External postings and positions are decoded by block as they are read.
/// Inline postings are decoded when their dictionary block is loaded.
#[derive(Clone)]
pub struct SegmentPostings {
    pub(crate) block_cursor: BlockSegmentPostings,
    doc_index: usize,
    position_reader: Option<PositionReader>,
}

impl SegmentPostings {
    /// Returns an empty postings list.
    pub fn empty() -> Self {
        Self::from_block_postings(BlockSegmentPostings::empty(), None)
    }

    /// Compute the number of non-deleted documents.
    ///
    /// This method will clone and scan through the posting lists.
    /// (this is a rather expensive operation).
    pub fn doc_freq_given_deletes(&self, alive_bitset: &AliveBitSet) -> u32 {
        let mut docset = self.clone();
        let mut doc_freq = 0;
        loop {
            let doc = docset.doc();
            if doc == TERMINATED {
                return doc_freq;
            }
            if alive_bitset.is_alive(doc) {
                doc_freq += 1u32;
            }
            docset.advance();
        }
    }

    /// Returns the number of documents in the postings list, including deleted documents.
    pub fn doc_freq(&self) -> u32 {
        self.block_cursor.doc_freq()
    }

    /// Creates a segment postings object with the given documents
    /// and no frequency encoded.
    ///
    /// This method is mostly useful for unit tests.
    ///
    /// It serializes the doc ids using tantivy's codec
    /// and returns a `SegmentPostings` object that embeds a
    /// buffer with the serialized data.
    #[cfg(test)]
    pub fn create_from_docs(docs: &[u32]) -> SegmentPostings {
        let doc_and_tfs: Vec<_> = docs.iter().map(|&doc| (doc, 1)).collect();
        Self::create_from_docs_and_tfs(&doc_and_tfs, None)
    }

    /// Helper functions to create `SegmentPostings` for tests.
    #[cfg(test)]
    pub fn create_from_docs_and_tfs(
        doc_and_tfs: &[(u32, u32)],
        fieldnorms: Option<&[u32]>,
    ) -> SegmentPostings {
        use crate::directory::OwnedBytes;
        use crate::fieldnorm::FieldNormReader;
        use crate::postings::serializer::PostingsSerializer;
        use crate::schema::IndexRecordOption;
        use crate::Score;
        let mut buffer: Vec<u8> = Vec::new();
        let fieldnorm_reader = fieldnorms.map(FieldNormReader::for_test);
        let average_field_norm = fieldnorms
            .map(|fieldnorms| {
                if fieldnorms.is_empty() {
                    return 0.0;
                }
                let total_num_tokens: u64 = fieldnorms
                    .iter()
                    .map(|&fieldnorm| fieldnorm as u64)
                    .sum::<u64>();
                total_num_tokens as Score / fieldnorms.len() as Score
            })
            .unwrap_or(0.0);
        let mut postings_serializer = PostingsSerializer::new(
            average_field_norm,
            IndexRecordOption::WithFreqs,
            fieldnorm_reader,
        );
        postings_serializer.new_term(doc_and_tfs.len() as u32, true);
        for &(doc, tf) in doc_and_tfs {
            postings_serializer.write_doc(doc, tf);
        }
        postings_serializer
            .close_term(doc_and_tfs.len() as u32, &mut buffer)
            .unwrap();
        let mut block_segment_postings =
            BlockSegmentPostings::new(IndexRecordOption::WithFreqs, IndexRecordOption::WithFreqs);
        block_segment_postings
            .reset(doc_and_tfs.len() as u32, OwnedBytes::new(buffer))
            .unwrap();
        SegmentPostings::from_block_postings(block_segment_postings, None)
    }

    pub(crate) fn append_position_deltas(&mut self, output: &mut Vec<u32>) {
        let term_freq = self.term_freq();
        if let Some(position_reader) = self.position_reader.as_mut() {
            debug_assert!(
                !self.block_cursor.freqs().is_empty(),
                "No positions available"
            );
            let read_offset = self.block_cursor.position_offset()
                + (self.block_cursor.freqs()[..self.doc_index]
                    .iter()
                    .cloned()
                    .sum::<u32>() as u64);
            let output_len = output.len();
            // TODO: use MaybeUninit to avoid zeroing the output before copying deltas.
            output.resize(output_len + term_freq as usize, 0u32);
            position_reader.read(read_offset, &mut output[output_len..]);
        }
    }

    /// Creates a postings cursor over blocks and optional positions.
    pub(crate) fn from_block_postings(
        segment_block_postings: BlockSegmentPostings,
        position_reader: Option<PositionReader>,
    ) -> SegmentPostings {
        SegmentPostings {
            block_cursor: segment_block_postings,
            doc_index: 0,
            position_reader,
        }
    }
}

impl DocSet for SegmentPostings {
    // Move to the next document.
    #[inline]
    fn advance(&mut self) -> DocId {
        debug_assert!(self.block_cursor.block_is_loaded());
        if self.doc_index == COMPRESSION_BLOCK_SIZE - 1 {
            self.doc_index = 0;
            self.block_cursor.advance();
        } else {
            self.doc_index += 1;
        }
        self.doc()
    }

    #[inline]
    fn seek(&mut self, target: DocId) -> DocId {
        debug_assert!(self.doc() <= target);
        if self.doc() >= target {
            return self.doc();
        }

        // As an optimization, if the block is already loaded, we can
        // cheaply check the next doc.
        self.doc_index = (self.doc_index + 1).min(COMPRESSION_BLOCK_SIZE - 1);
        if self.doc() >= target {
            return self.doc();
        }

        // Delegate block-local search to BlockSegmentPostings::seek, which returns
        // the in-block index of the first doc >= target.
        self.doc_index = self.block_cursor.seek(target);
        let doc = self.doc();
        debug_assert!(doc >= target);
        doc
    }

    /// Return the current document's `DocId`.
    #[inline]
    fn doc(&self) -> DocId {
        self.block_cursor.doc(self.doc_index)
    }

    fn size_hint(&self) -> u32 {
        self.len() as u32
    }
}

impl HasLen for SegmentPostings {
    fn len(&self) -> usize {
        self.block_cursor.doc_freq() as usize
    }
}

impl Postings for SegmentPostings {
    /// Returns the current document's term frequency, or 1 when frequencies are omitted.
    fn term_freq(&self) -> u32 {
        debug_assert!(
            // The decoder fills omitted frequencies with 1.
            self.doc_index < COMPRESSION_BLOCK_SIZE,
            "document index exceeds the compression block"
        );
        self.block_cursor.freq(self.doc_index)
    }

    fn append_positions_with_offset(&mut self, offset: u32, output: &mut Vec<u32>) {
        let output_len = output.len();
        self.append_position_deltas(output);
        let mut position = offset;
        for delta in &mut output[output_len..] {
            position += *delta;
            *delta = position;
        }
    }
}

#[cfg(test)]
mod tests {

    use common::HasLen;

    use super::SegmentPostings;
    use crate::docset::TERMINATED;
    use crate::fastfield::AliveBitSet;
    use crate::postings::postings::Postings;

    #[test]
    fn test_empty_segment_postings() {
        let mut postings = SegmentPostings::empty();
        assert_eq!(postings.doc_freq(), 0);
        assert_eq!(postings.len(), 0);
        assert!(postings.block_cursor.docs().is_empty());
        for target in [0, 42, TERMINATED] {
            assert_eq!(postings.block_cursor.rank(target), 0);
        }
        postings.block_cursor.advance();
        assert!(postings.block_cursor.docs().is_empty());
        assert_eq!(postings.block_cursor.rank(TERMINATED), 0);
        let mut cursor = crate::postings::LoadedPostingsReader::decode_remaining(&mut postings);
        for postings in [&mut postings as &mut dyn Postings, &mut cursor] {
            assert_eq!(postings.doc(), TERMINATED);
            assert_eq!(postings.size_hint(), 0);
            assert_eq!(postings.term_freq(), 1);
            assert_eq!(postings.advance(), TERMINATED);
            assert_eq!(postings.advance(), TERMINATED);
            assert_eq!(postings.seek(TERMINATED), TERMINATED);
            let mut positions = Vec::new();
            postings.positions(&mut positions);
            assert!(positions.is_empty());
        }
    }

    #[test]
    fn test_doc_freq() {
        let docs = SegmentPostings::create_from_docs(&[0, 2, 10]);
        assert_eq!(docs.doc_freq(), 3);
        let alive_bitset = AliveBitSet::for_test_from_deleted_docs(&[2], 12);
        assert_eq!(docs.doc_freq_given_deletes(&alive_bitset), 2);
        let all_deleted =
            AliveBitSet::for_test_from_deleted_docs(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], 12);
        assert_eq!(docs.doc_freq_given_deletes(&all_deleted), 0);
    }
}
