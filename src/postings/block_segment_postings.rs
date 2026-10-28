use std::io;

use common::VInt;

use crate::directory::OwnedBytes;
use crate::fieldnorm::FieldNormReader;
use crate::postings::compression::{BlockDecoder, VIntDecoder, COMPRESSION_BLOCK_SIZE};
use crate::postings::{BlockInfo, LoadedPostings, SkipReader};
use crate::query::Bm25Weight;
use crate::schema::IndexRecordOption;
use crate::{DocId, Score, TERMINATED};

/// `BlockSegmentPostings` is a cursor iterating over blocks
/// of documents.
///
/// # Warning
///
/// While it is useful for some very specific high-performance
/// use cases, you should prefer using `SegmentPostings` for most usage.
#[derive(Clone)]
pub struct BlockSegmentPostings {
    pub(crate) doc_decoder: BlockDecoder,
    block_loaded: bool,
    freq_decoder: BlockDecoder,
    reads_freqs: bool,
    record_option: IndexRecordOption,
    requested_freqs: bool,
    block_max_score_cache: Option<Score>,
    doc_freq: u32,
    data: OwnedBytes,
    skip_reader: SkipReader,
}

impl BlockSegmentPostings {
    pub(crate) fn reset_inline_postings(&mut self, postings: &LoadedPostings) {
        debug_assert!((1..=32).contains(&postings.doc_ids.len()));
        debug_assert!(
            postings.term_freqs.is_empty() || postings.doc_ids.len() == postings.term_freqs.len()
        );
        self.data = OwnedBytes::empty();
        self.doc_freq = postings.doc_ids.len() as u32;
        let freqs = if self.requested_freqs {
            postings.term_freqs.as_ref()
        } else {
            &[]
        };
        self.reads_freqs = !freqs.is_empty();
        self.skip_reader = SkipReader::new(OwnedBytes::empty(), self.doc_freq, self.record_option);
        self.doc_decoder.set_values(&postings.doc_ids, TERMINATED);
        self.freq_decoder.set_values(freqs, 1);
        self.block_max_score_cache = None;
        self.block_loaded = true;
    }

    /// Returns the block_max_score for the current block.
    /// It does not require the block to be loaded. For instance, it is ok to call this method
    /// after having called `.shallow_advance(..)`.
    ///
    /// See `TermScorer::block_max_score(..)` for more information.
    pub fn block_max_score(
        &mut self,
        fieldnorm_reader: &FieldNormReader,
        bm25_weight: &Bm25Weight,
    ) -> Score {
        if let Some(score) = self.block_max_score_cache {
            return score;
        }
        let score = if let (true, Some(score)) = (
            self.reads_freqs,
            self.skip_reader.block_max_score(bm25_weight),
        ) {
            score
        } else if self.block_is_loaded() {
            self.docs()
                .iter()
                .enumerate()
                .map(|(i, &doc)| {
                    bm25_weight.score(fieldnorm_reader.fieldnorm_id(doc), self.freq(i))
                })
                .reduce(Score::max)
                .unwrap_or(0.0)
        } else {
            // Leave the upper bound uncached so loading the block can refine it.
            return bm25_weight.max_score();
        };
        self.block_max_score_cache = Some(score);
        score
    }

    pub(crate) fn reads_freqs(&self) -> bool {
        self.reads_freqs
    }

    // Resets the block segment postings on another position
    // in the postings file.
    //
    // This is useful for enumerating through a list of terms,
    // and consuming the associated posting lists while avoiding
    // reallocating a `BlockSegmentPostings`.
    //
    // # Warning
    //
    // This does not reset the positions list.
    pub(crate) fn reset(&mut self, doc_freq: u32, mut postings_data: OwnedBytes) -> io::Result<()> {
        let skip_len = if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
            VInt::deserialize_u64(&mut postings_data)? as usize
        } else {
            0
        };
        let (skip_data, postings_data) = postings_data.split(skip_len);
        self.data = postings_data;
        self.block_max_score_cache = None;
        self.block_loaded = false;
        let mut record_option = self.record_option;
        // JSON numeric terms omit frequencies even when the field records them.
        if skip_data.len() < 8 * (doc_freq as usize / COMPRESSION_BLOCK_SIZE) {
            record_option = IndexRecordOption::Basic;
        }
        self.reads_freqs = record_option.has_freq() && self.requested_freqs;
        self.skip_reader = SkipReader::new(skip_data, doc_freq, record_option);
        self.doc_freq = doc_freq;
        self.load_block();
        Ok(())
    }

    /// Returns the number of documents in the postings list, including deleted documents.
    pub fn doc_freq(&self) -> u32 {
        self.doc_freq
    }

    /// Returns the array of docs in the current block.
    ///
    /// Opening or resetting postings loads the first block.
    #[inline]
    pub fn docs(&self) -> &[DocId] {
        debug_assert!(self.block_is_loaded());
        self.doc_decoder.output_array()
    }

    /// Return the document at index `idx` of the block.
    #[inline]
    pub fn doc(&self, idx: usize) -> u32 {
        self.doc_decoder.output(idx)
    }

    /// Return the array of `term freq` in the block.
    #[inline]
    pub fn freqs(&self) -> &[u32] {
        debug_assert!(self.block_is_loaded());
        self.freq_decoder.output_array()
    }

    /// Return the frequency at index `idx` of the block.
    #[inline]
    pub fn freq(&self, idx: usize) -> u32 {
        debug_assert!(self.block_is_loaded());
        self.freq_decoder.output(idx)
    }

    /// Returns the number of documents in the decoded block.
    /// Blocks contain up to `COMPRESSION_BLOCK_SIZE` documents.
    #[inline]
    pub fn block_len(&self) -> usize {
        self.docs().len()
    }

    /// Position on a block that may contains `target_doc`.
    ///
    /// If all docs are smaller than target, the block loaded may be empty,
    /// or be the last an incomplete VInt block.
    pub fn seek(&mut self, target_doc: DocId) -> usize {
        // Move to the block that might contain our document.
        self.seek_block(target_doc);
        self.load_block();

        // At this point we are on the block that might contain our document.
        let doc = self.doc_decoder.seek_within_block(target_doc);

        // The final partial block is padded with TERMINATED, ensuring a document ID
        // or padding value is >= target_doc.
        debug_assert!(doc < COMPRESSION_BLOCK_SIZE);

        // `doc` is now the first element >= `target_doc`.
        // If all docs are smaller than target, the current block is incomplete and padded
        // with TERMINATED. After the search, the cursor points to the first TERMINATED.
        doc
    }

    /// Returns the number of documents with a doc id strictly smaller than `target`
    /// (i.e. the *rank* of `target` in this posting list).
    ///
    /// This jumps to the block that may contain `target` through the skip list, so no
    /// skipped block is decoded; a single block is then decoded to locate `target`
    /// within it. The cost is therefore `O(number_of_skip_list_entries)` plus one block
    /// decode, rather than `O(doc_freq)`.
    ///
    /// Like [`Self::seek`], the underlying cursor only ever moves forward. This method
    /// must be called with **non-decreasing** `target` values (galloping); calling it
    /// with a `target` smaller than a previous one yields an incorrect result. `target`
    /// must be a valid doc id (i.e. `target <= TERMINATED`), exactly as for `seek`.
    ///
    /// Edge cases: returns `0` when `target` is smaller than every doc id, and
    /// `doc_freq()` when `target` is larger than every doc id.
    pub fn rank(&mut self, target: DocId) -> u32 {
        // `within` = number of docs in the landed block with a doc id < target.
        let within = self.seek(target);
        // `remaining_docs` counts the landed block and everything after it, so the
        // difference is the number of docs in all blocks strictly before it.
        let docs_before_block = self.doc_freq - self.skip_reader.remaining_docs();
        docs_before_block + within as u32
    }

    pub(crate) fn position_offset(&self) -> u64 {
        self.skip_reader.position_offset()
    }

    /// Seeks the skip list to the block that may contain `target_doc`.
    /// Call `load_block()` before reading its documents or frequencies.
    pub(crate) fn seek_block(&mut self, target_doc: DocId) {
        if self.skip_reader.seek(target_doc) {
            self.block_max_score_cache = None;
            self.block_loaded = false;
        }
    }

    #[inline]
    pub(crate) fn has_remaining_docs(&self) -> bool {
        self.skip_reader.has_remaining_docs()
    }

    pub(crate) fn block_is_loaded(&self) -> bool {
        self.block_loaded
    }

    pub(crate) fn load_block(&mut self) {
        if self.block_is_loaded() {
            return;
        }
        let offset = self.skip_reader.byte_offset();
        let doc_offset = self.skip_reader.last_doc_in_previous_block;
        match self.skip_reader.block_info() {
            BlockInfo::BitPacked {
                doc_num_bits,
                strict_delta_encoded,
                tf_num_bits,
                ..
            } => {
                let data = &self.data[offset..];
                let consumed = self.doc_decoder.uncompress_block_sorted(
                    data,
                    doc_offset,
                    doc_num_bits,
                    strict_delta_encoded,
                );
                if self.reads_freqs {
                    self.freq_decoder.uncompress_block_unsorted(
                        &data[consumed..],
                        tf_num_bits,
                        strict_delta_encoded,
                    );
                }
            }
            BlockInfo::VInt { num_docs } => {
                let data = if num_docs == 0 {
                    &[]
                } else {
                    &self.data[offset..]
                };
                let consumed = self.doc_decoder.uncompress_vint_sorted(
                    data,
                    doc_offset,
                    num_docs as usize,
                    TERMINATED,
                );
                // JSON numeric terms can omit frequencies in a frequency-enabled field.
                self.reads_freqs &= data.len() > consumed;
                if self.reads_freqs {
                    self.freq_decoder.uncompress_vint_unsorted(
                        &data[consumed..],
                        num_docs as usize,
                        TERMINATED,
                    );
                }
            }
        }
        if !self.reads_freqs {
            self.freq_decoder.set_values(&[], 1);
        }
        self.block_loaded = true;
    }

    /// Advance to the next block.
    pub fn advance(&mut self) {
        self.skip_reader.advance();
        self.block_loaded = false;
        self.block_max_score_cache = None;
        self.load_block();
    }

    /// Returns an empty postings list.
    pub fn empty() -> BlockSegmentPostings {
        Self::new(IndexRecordOption::Basic, IndexRecordOption::Basic)
    }

    pub(crate) fn new(
        record_option: IndexRecordOption,
        requested_option: IndexRecordOption,
    ) -> Self {
        BlockSegmentPostings {
            doc_decoder: BlockDecoder::with_val(TERMINATED),
            block_loaded: true,
            freq_decoder: BlockDecoder::with_val(1),
            reads_freqs: false,
            record_option,
            requested_freqs: record_option.downgrade(requested_option).has_freq(),
            block_max_score_cache: None,
            doc_freq: 0,
            data: OwnedBytes::empty(),
            skip_reader: SkipReader::new(OwnedBytes::empty(), 0, IndexRecordOption::Basic),
        }
    }

    pub(crate) fn skip_reader(&self) -> &SkipReader {
        &self.skip_reader
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockSegmentPostings, OwnedBytes};
    use crate::docset::{DocSet, TERMINATED};
    use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
    use crate::postings::serializer::PostingsSerializer;
    use crate::postings::SegmentPostings;
    use crate::schema::IndexRecordOption;
    use crate::DocId;

    #[test]
    fn test_block_segment_postings_reset_advance_and_rank() {
        let mut block_postings = BlockSegmentPostings::empty();
        // Reset between VInt-only lists and lists spanning multiple skip blocks.
        for len in [3, 1000, 3] {
            let docs: Vec<DocId> = (0..len).map(|i| i * 3).collect();
            let mut serializer = PostingsSerializer::new(0.0, IndexRecordOption::Basic, None);
            serializer.new_term(len, false);
            for &doc in &docs {
                serializer.write_doc(doc, 1);
            }
            let mut data = Vec::new();
            serializer.close_term(len, &mut data).unwrap();
            block_postings.reset(len, OwnedBytes::new(data)).unwrap();
            assert_eq!(block_postings.doc_freq(), docs.len() as u32);
            let mut blocks = block_postings.clone();
            let mut offset = 0;
            while !blocks.docs().is_empty() {
                let end = offset + blocks.block_len();
                assert_eq!(blocks.docs(), &docs[offset..end]);
                offset = end;
                blocks.advance();
            }
            assert_eq!(offset, docs.len());
            // Seek through gaps on both sides of a block boundary, then advance.
            for ordinal in [0, 1, 127, 128, 129, docs.len() - 1] {
                if let Some(&doc) = docs.get(ordinal) {
                    let mut postings =
                        SegmentPostings::from_block_postings(block_postings.clone(), None);
                    assert_eq!(postings.seek(doc.saturating_sub(1)), doc);
                    assert_eq!(postings.seek(doc), doc);
                    assert_eq!(
                        postings.advance(),
                        docs.get(ordinal + 1).copied().unwrap_or(TERMINATED)
                    );
                    assert_eq!(postings.seek(TERMINATED), TERMINATED);
                }
            }
            // Rank counts docs strictly below each target. Targets must be non-decreasing.
            for target in [0, 1, 2, 3, 4, 299, 300, 301, 1500, 2996, 2997, 3000, 10_000] {
                let expected = docs.iter().filter(|&&d| d < target).count() as u32;
                assert_eq!(
                    block_postings.rank(target),
                    expected,
                    "rank({target}) mismatch"
                );
            }
            assert_eq!(block_postings.doc(COMPRESSION_BLOCK_SIZE - 1), TERMINATED);
        }
    }
}
