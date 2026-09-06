use common::{HasLen, TinySet};

use crate::docset::{insert_docs_into_bitset, DocSet};
use crate::fastfield::AliveBitSet;
use crate::positions::PositionReader;
use crate::postings::compression::COMPRESSION_BLOCK_SIZE;
use crate::postings::{BlockSegmentPostings, Postings};
use crate::{DocId, Score, TERMINATED};

/// Scatters one sorted scored posting chunk into a bitset window and its accumulators.
#[inline]
pub(crate) fn update_scored_bitset<TAccumulator>(
    docs: &[DocId],
    scores: &[Score],
    bitsets: &mut [TinySet],
    accumulators: &mut [TAccumulator],
    min_doc: DocId,
    mut update_score: impl FnMut(&mut TAccumulator, Score),
) {
    debug_assert_eq!(docs.len(), scores.len());
    if docs.len() != COMPRESSION_BLOCK_SIZE {
        for (&doc, &score) in docs.iter().zip(scores) {
            let delta = doc - min_doc;
            bitsets[(delta / 64) as usize].insert_mut(delta % 64);
            update_score(&mut accumulators[delta as usize], score);
        }
        return;
    }

    let docs: &[DocId; COMPRESSION_BLOCK_SIZE] = docs.try_into().unwrap();
    let scores: &[Score; COMPRESSION_BLOCK_SIZE] = scores.try_into().unwrap();
    // Posting chunks are sorted. Validate the range once before the unchecked scatter.
    assert!(docs[0] >= min_doc);
    assert!(((docs[COMPRESSION_BLOCK_SIZE - 1] - min_doc) as usize) < accumulators.len());
    let first_delta = docs[0] - min_doc;
    let mut bucket = (first_delta / 64) as usize;
    let mut mask = TinySet::singleton(first_delta % 64);
    update_score(&mut accumulators[first_delta as usize], scores[0]);

    for (&doc, &score) in docs[1..].iter().zip(&scores[1..]) {
        let delta = doc - min_doc;
        let next_bucket = (delta / 64) as usize;
        if next_bucket == bucket {
            mask = mask.insert(delta % 64);
        } else {
            let output = &mut bitsets[bucket];
            *output = output.union(mask);
            bucket = next_bucket;
            mask = TinySet::singleton(delta % 64);
        }
        // SAFETY: the sorted chunk's range was checked above. A bounds check per posting
        // slowed buffered union TOP_100_COUNT by about 4%.
        update_score(
            unsafe { accumulators.get_unchecked_mut(delta as usize) },
            score,
        );
    }
    let output = &mut bitsets[bucket];
    *output = output.union(mask);
}

/// `SegmentPostings` represents the inverted list or postings associated with
/// a term in a `Segment`.
///
/// As we iterate through the `SegmentPostings`, the frequencies are optionally decoded.
/// Positions on the other hand, are optionally entirely decoded upfront.
#[derive(Clone)]
pub struct SegmentPostings {
    pub(crate) block_cursor: BlockSegmentPostings,
    cur: usize,
    position_reader: Option<PositionReader>,
}

impl SegmentPostings {
    /// Returns an empty segment postings object
    pub fn empty() -> Self {
        SegmentPostings {
            block_cursor: BlockSegmentPostings::empty(),
            cur: 0,
            position_reader: None,
        }
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

    /// Returns the overall number of documents in the block postings.
    /// It does not take in account whether documents are deleted or not.
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
        use crate::directory::FileSlice;
        use crate::postings::serializer::PostingsSerializer;
        use crate::schema::IndexRecordOption;
        let mut buffer = Vec::new();
        {
            let mut postings_serializer =
                PostingsSerializer::new(0.0, IndexRecordOption::Basic, None);
            postings_serializer.new_term(docs.len() as u32, false);
            for &doc in docs {
                postings_serializer.write_doc(doc, 1u32);
            }
            postings_serializer
                .close_term(docs.len() as u32, &mut buffer)
                .expect("In memory Serialization should never fail.");
        }
        let block_segment_postings = BlockSegmentPostings::open(
            docs.len() as u32,
            FileSlice::from(buffer),
            IndexRecordOption::Basic,
            IndexRecordOption::Basic,
        )
        .unwrap();
        SegmentPostings::from_block_postings(block_segment_postings, None)
    }

    /// Helper functions to create `SegmentPostings` for tests.
    #[cfg(test)]
    pub fn create_from_docs_and_tfs(
        doc_and_tfs: &[(u32, u32)],
        fieldnorms: Option<&[u32]>,
    ) -> SegmentPostings {
        use crate::directory::FileSlice;
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
        let block_segment_postings = BlockSegmentPostings::open(
            doc_and_tfs.len() as u32,
            FileSlice::from(buffer),
            IndexRecordOption::WithFreqs,
            IndexRecordOption::WithFreqs,
        )
        .unwrap();
        SegmentPostings::from_block_postings(block_segment_postings, None)
    }

    /// Reads a Segment postings from an &[u8]
    ///
    /// * `len` - number of document in the posting lists.
    /// * `data` - data array. The complete data is not necessarily used.
    /// * `freq_handler` - the freq handler is in charge of decoding frequencies and/or positions
    pub(crate) fn from_block_postings(
        segment_block_postings: BlockSegmentPostings,
        position_reader: Option<PositionReader>,
    ) -> SegmentPostings {
        SegmentPostings {
            block_cursor: segment_block_postings,
            cur: 0, // cursor within the block
            position_reader,
        }
    }

    /// Fills a bitset window directly from posting blocks.
    #[inline]
    pub(crate) fn fill_bitset_block(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        mask: &mut [TinySet],
    ) -> DocId {
        debug_assert!(horizon >= min_doc);
        debug_assert!((horizon - min_doc) as usize <= mask.len() * 64);
        if self.doc() < min_doc {
            self.seek(min_doc);
        }
        loop {
            // Process complete blocks
            if !self.block_cursor.block_is_loaded()
                && self.block_cursor.last_doc_in_block() < horizon
            {
                let block_docs = self.block_cursor.decode_docs();
                if block_docs.len() == COMPRESSION_BLOCK_SIZE {
                    let block_docs: &[DocId; COMPRESSION_BLOCK_SIZE] =
                        block_docs.try_into().unwrap();
                    insert_docs_into_bitset(block_docs, min_doc, mask);
                    self.cur = 0;
                    self.block_cursor.advance_without_loading();
                    continue;
                }
            }

            self.block_cursor.load_block();
            if self.doc() >= horizon {
                return self.doc();
            }

            let remaining_docs = &self.block_cursor.docs()[self.cur..];
            let num_docs = remaining_docs.partition_point(|&doc| doc < horizon);
            // Partial blocks are often sparse; avoid bucket-grouping overhead here.
            for &doc in &remaining_docs[..num_docs] {
                let delta = doc - min_doc;
                mask[(delta / 64) as usize].insert_mut(delta % 64);
            }
            self.cur += num_docs;

            if num_docs < remaining_docs.len() {
                return self.doc();
            }
            self.cur = 0;
            self.block_cursor.advance_without_loading();
        }
    }

    /// Visits scored posting chunks in `[min_doc, horizon)`.
    ///
    /// Seeks forward to `min_doc` if needed and leaves the cursor at the first remaining document
    /// at or beyond `horizon`. `score_batch` must fill every score in each supplied chunk;
    /// `consume` receives the documents and their scores, including zero-scored matches.
    pub(crate) fn for_each_scored_block(
        &mut self,
        min_doc: DocId,
        horizon: DocId,
        mut score_batch: impl FnMut(&[DocId], &[u32], &mut [Score]),
        mut consume: impl FnMut(&[DocId], &[Score]),
    ) -> DocId {
        assert!(min_doc <= horizon && horizon <= TERMINATED);
        if self.doc() < min_doc {
            self.seek(min_doc);
        }
        let mut docs = [0; COMPRESSION_BLOCK_SIZE];
        let mut term_freqs = [0; COMPRESSION_BLOCK_SIZE];
        let mut scores = [0.0; COMPRESSION_BLOCK_SIZE];
        self.for_each_doc_freq_block_up_to(
            horizon,
            &mut docs,
            &mut term_freqs,
            |docs, term_freqs| {
                let scores = &mut scores[..docs.len()];
                score_batch(docs, term_freqs, scores);
                consume(docs, scores);
            },
        );
        self.doc()
    }

    /// Visits posting chunks below an exclusive horizon.
    ///
    /// Complete blocks are decoded directly into the supplied scratch buffers. A block crossing
    /// the horizon stays in the posting decoder and is exposed as a borrowed slice, so its
    /// unconsumed suffix can be reused by the next call without decompressing the block again.
    pub(crate) fn for_each_doc_freq_block_up_to<F>(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; COMPRESSION_BLOCK_SIZE],
        term_freqs: &mut [u32; COMPRESSION_BLOCK_SIZE],
        mut consume: F,
    ) where
        F: FnMut(&[DocId], &[u32]),
    {
        loop {
            if !self.block_cursor.block_is_loaded() {
                if self.block_cursor.last_doc_in_block() < horizon {
                    let block_len = self
                        .block_cursor
                        .decode_docs_and_freqs_into(docs, term_freqs);
                    if block_len == 0 {
                        self.block_cursor.load_block();
                        return;
                    }
                    debug_assert_eq!(block_len, COMPRESSION_BLOCK_SIZE);
                    consume(&docs[..block_len], &term_freqs[..block_len]);
                    self.cur = 0;
                    self.block_cursor.advance_without_loading();
                    continue;
                }
                self.block_cursor.load_block();
            }

            if self.doc() >= horizon {
                return;
            }

            let start = self.cur;
            let block_len = self.block_cursor.docs().len();
            let end =
                start + self.block_cursor.docs()[start..].partition_point(|&doc| doc < horizon);
            let num_docs = end - start;
            let block_freqs = self.block_cursor.freq_output_array();
            if let Some(freqs) = block_freqs.get(start..end) {
                consume(&self.block_cursor.docs()[start..end], freqs);
            } else {
                term_freqs[..num_docs].fill(1);
                consume(
                    &self.block_cursor.docs()[start..end],
                    &term_freqs[..num_docs],
                );
            }
            self.cur = end;

            if end < block_len {
                return;
            }
            self.cur = 0;
            self.block_cursor.advance_without_loading();
        }
    }

    /// Decodes one posting block directly into parallel output buffers.
    pub(crate) fn fill_doc_freq_block_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId; COMPRESSION_BLOCK_SIZE],
        term_freqs: &mut [u32; COMPRESSION_BLOCK_SIZE],
    ) -> std::ops::Range<usize> {
        if self.block_cursor.block_is_loaded() && self.doc() >= horizon {
            return 0..0;
        }
        let start = self.cur;
        if self.block_cursor.block_is_loaded() {
            self.block_cursor.unload_block();
        }
        let block_len = self
            .block_cursor
            .decode_docs_and_freqs_into(docs, term_freqs);
        if block_len == 0 {
            self.block_cursor.load_block();
            return 0..0;
        }
        let end = if docs[block_len - 1] < horizon {
            block_len
        } else {
            start + docs[start..block_len].partition_point(|&doc| doc < horizon)
        };
        if end < block_len {
            self.block_cursor.load_block();
            self.cur = end;
        } else {
            self.cur = 0;
            self.block_cursor.advance_without_loading();
            if block_len < COMPRESSION_BLOCK_SIZE {
                self.block_cursor.load_block();
            }
        }
        start..end
    }

    /// Fills parallel document and term-frequency buffers through an exclusive horizon.
    ///
    /// Complete blocks are decoded directly into the output buffers. Already loaded block
    /// suffixes and horizon-crossing prefixes are copied so that the unconsumed postings remain
    /// available to the next call.
    pub(crate) fn fill_doc_freqs_up_to(
        &mut self,
        horizon: DocId,
        docs: &mut [DocId],
        term_freqs: &mut [u32],
    ) -> usize {
        assert_eq!(docs.len(), term_freqs.len());
        let mut num_filled = 0;

        while num_filled < docs.len() {
            if self.block_cursor.block_is_loaded() {
                if self.doc() >= horizon {
                    break;
                }
                let block_len = self.block_cursor.docs().len();
                let copied = self.block_cursor.copy_docs_and_term_freqs(
                    self.cur,
                    horizon,
                    &mut docs[num_filled..],
                    &mut term_freqs[num_filled..],
                );
                if copied == 0 {
                    break;
                }
                num_filled += copied;
                self.cur += copied;

                if self.cur < block_len {
                    break;
                }
                self.cur = 0;
                self.block_cursor.advance_without_loading();
                continue;
            }

            let remaining = docs.len() - num_filled;
            if remaining < COMPRESSION_BLOCK_SIZE
                || self.block_cursor.last_doc_in_block() >= horizon
            {
                self.block_cursor.load_block();
                continue;
            }

            let docs_block: &mut [DocId; COMPRESSION_BLOCK_SIZE] = (&mut docs
                [num_filled..num_filled + COMPRESSION_BLOCK_SIZE])
                .try_into()
                .unwrap();
            let term_freqs_block: &mut [u32; COMPRESSION_BLOCK_SIZE] = (&mut term_freqs
                [num_filled..num_filled + COMPRESSION_BLOCK_SIZE])
                .try_into()
                .unwrap();
            let block_len = self
                .block_cursor
                .decode_docs_and_freqs_into(docs_block, term_freqs_block);
            if block_len == 0 {
                self.block_cursor.load_block();
                break;
            }
            num_filled += block_len;
            self.block_cursor.advance_without_loading();
            if block_len < COMPRESSION_BLOCK_SIZE {
                self.block_cursor.load_block();
                break;
            }
        }

        // `DocSet::doc` must remain valid after a full output buffer ends exactly at a posting
        // block boundary.
        if !self.block_cursor.block_is_loaded() {
            self.block_cursor.load_block();
        }
        num_filled
    }
}

impl DocSet for SegmentPostings {
    // goes to the next element.
    // next needs to be called a first time to point to the correct element.
    #[inline]
    fn advance(&mut self) -> DocId {
        debug_assert!(self.block_cursor.block_is_loaded());
        if self.cur == COMPRESSION_BLOCK_SIZE - 1 {
            self.cur = 0;
            self.block_cursor.advance();
        } else {
            self.cur += 1;
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
        self.cur = (self.cur + 1).min(COMPRESSION_BLOCK_SIZE - 1);
        if self.doc() >= target {
            return self.doc();
        }

        // Delegate block-local search to BlockSegmentPostings::seek, which returns
        // the in-block index of the first doc >= target.
        self.cur = self.block_cursor.seek(target);
        let doc = self.doc();
        debug_assert!(doc >= target);
        doc
    }

    /// Return the current document's `DocId`.
    #[inline]
    fn doc(&self) -> DocId {
        self.block_cursor.doc(self.cur)
    }

    fn size_hint(&self) -> u32 {
        self.len() as u32
    }

    fn fill_bitset_block(&mut self, min_doc: DocId, horizon: DocId, mask: &mut [TinySet]) -> DocId {
        SegmentPostings::fill_bitset_block(self, min_doc, horizon, mask)
    }
}

impl HasLen for SegmentPostings {
    fn len(&self) -> usize {
        self.block_cursor.doc_freq() as usize
    }
}

impl Postings for SegmentPostings {
    /// Returns the frequency associated with the current document.
    /// If the schema is set up so that no frequency have been encoded,
    /// this method should always return 1.
    ///
    /// # Panics
    ///
    /// Will panics if called without having called advance before.
    fn term_freq(&self) -> u32 {
        debug_assert!(
            // Here we do not use the len of `freqs()`
            // because it is actually ok to request for the freq of doc
            // even if no frequency were encoded for the field.
            //
            // In that case we hit the block just as if the frequency had been
            // decoded. The block is simply prefilled by the value 1.
            self.cur < COMPRESSION_BLOCK_SIZE,
            "Have you forgotten to call `.advance()` at least once before calling `.term_freq()`."
        );
        self.block_cursor.freq(self.cur)
    }

    fn append_positions_with_offset(&mut self, offset: u32, output: &mut Vec<u32>) {
        let term_freq = self.term_freq();
        let prev_len = output.len();
        if let Some(position_reader) = self.position_reader.as_mut() {
            debug_assert!(
                !self.block_cursor.freqs().is_empty(),
                "No positions available"
            );
            let read_offset = self.block_cursor.position_offset()
                + (self.block_cursor.freqs()[..self.cur]
                    .iter()
                    .cloned()
                    .sum::<u32>() as u64);
            // TODO: instead of zeroing the output, we could use MaybeUninit or similar.
            output.resize(prev_len + term_freq as usize, 0u32);
            position_reader.read(read_offset, &mut output[prev_len..]);
            let mut cum = offset;
            for output_mut in output[prev_len..].iter_mut() {
                cum += *output_mut;
                *output_mut = cum;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use common::{HasLen, TinySet};

    use super::{update_scored_bitset, SegmentPostings};
    use crate::docset::{DocSet, TERMINATED};
    use crate::fastfield::AliveBitSet;
    use crate::postings::postings::Postings;

    #[test]
    fn test_empty_segment_postings() {
        let mut postings = SegmentPostings::empty();
        assert_eq!(postings.advance(), TERMINATED);
        assert_eq!(postings.advance(), TERMINATED);
        assert_eq!(postings.len(), 0);
    }

    #[test]
    fn test_empty_postings_doc_returns_terminated() {
        let mut postings = SegmentPostings::empty();
        assert_eq!(postings.doc(), TERMINATED);
        assert_eq!(postings.advance(), TERMINATED);
    }

    #[test]
    fn test_empty_postings_doc_term_freq_returns_0() {
        let postings = SegmentPostings::empty();
        assert_eq!(postings.term_freq(), 1);
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

    #[test]
    fn test_fill_bitset_block_with_different_window_sizes() {
        let expected: Vec<u32> = (0..260).map(|i| i * 100).collect();

        for num_buckets in [16, 64] {
            let mut postings = SegmentPostings::create_from_docs(&expected);
            let mut actual = Vec::new();
            let mut min_doc = postings.doc();
            while min_doc != TERMINATED {
                let mut mask = vec![TinySet::EMPTY; num_buckets];
                let horizon = min_doc.saturating_add(num_buckets as u32 * 64);
                let next_doc = postings.fill_bitset_block(min_doc, horizon, &mut mask);
                for (bucket, bits) in mask.into_iter().enumerate() {
                    actual.extend(
                        bits.into_iter()
                            .map(|bit| min_doc + bucket as u32 * 64 + bit),
                    );
                }
                min_doc = next_doc;
            }
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_for_each_scored_block() {
        let docs: Vec<u32> = (0..300).map(|doc| doc * 3).collect();
        let doc_freqs: Vec<_> = docs.iter().map(|&doc| (doc, doc % 7 + 1)).collect();
        for with_freqs in [false, true] {
            for window_len in [64, 512, 2048] {
                let mut postings = if with_freqs {
                    SegmentPostings::create_from_docs_and_tfs(&doc_freqs, None)
                } else {
                    SegmentPostings::create_from_docs(&docs)
                };
                let mut min_doc = 5;
                while min_doc <= *docs.last().unwrap() {
                    let horizon = min_doc + window_len;
                    let mut mask = vec![TinySet::singleton(1); window_len as usize / 64];
                    let mut accumulators = vec![(10.0, 0); window_len as usize];
                    let next_doc = postings.for_each_scored_block(
                        min_doc,
                        horizon,
                        |docs, freqs, scores| {
                            assert_eq!(docs.len(), freqs.len());
                            for (score, &freq) in scores.iter_mut().zip(freqs) {
                                *score = (freq - 1) as f32;
                            }
                        },
                        |docs, scores| {
                            update_scored_bitset(
                                docs,
                                scores,
                                &mut mask,
                                &mut accumulators,
                                min_doc,
                                |(sum, count), score| {
                                    *sum += score;
                                    *count += 1;
                                },
                            );
                        },
                    );
                    for (delta, &(sum, count)) in accumulators.iter().enumerate() {
                        let doc = min_doc + delta as u32;
                        let matched = docs.binary_search(&doc).is_ok();
                        assert_eq!(
                            mask[delta / 64].contains((delta % 64) as u32),
                            matched || delta % 64 == 1
                        );
                        assert_eq!(count, usize::from(matched));
                        let score = if matched && with_freqs {
                            (doc % 7) as f32
                        } else {
                            0.0
                        };
                        assert_eq!(sum, 10.0 + score);
                    }
                    assert_eq!(
                        next_doc,
                        docs.iter()
                            .copied()
                            .find(|&doc| doc >= horizon)
                            .unwrap_or(TERMINATED)
                    );
                    assert_eq!(postings.doc(), next_doc);
                    // Filling an already consumed interval must leave both outputs untouched.
                    assert_eq!(
                        postings.for_each_scored_block(
                            min_doc,
                            horizon,
                            |_, _, _| panic!("interval already consumed"),
                            |_, _| panic!("interval already consumed"),
                        ),
                        next_doc
                    );
                    min_doc = horizon;
                }
                assert_eq!(postings.advance(), TERMINATED);
            }
        }
    }

    #[test]
    fn test_for_each_doc_freq_block_up_to_reuses_horizon_crossing_blocks() {
        let expected: Vec<(u32, u32)> = (0..260).map(|i| (i * 100, i % 7 + 1)).collect();
        let mut postings = SegmentPostings::create_from_docs_and_tfs(&expected, None);
        let mut docs = [u32::MAX; crate::postings::compression::COMPRESSION_BLOCK_SIZE];
        let mut term_freqs = [0; crate::postings::compression::COMPRESSION_BLOCK_SIZE];
        let mut actual = Vec::new();

        let mut horizon = 4_096;
        while postings.doc() != TERMINATED {
            postings.for_each_doc_freq_block_up_to(
                horizon,
                &mut docs,
                &mut term_freqs,
                |chunk_docs, chunk_term_freqs| {
                    actual.extend(
                        chunk_docs
                            .iter()
                            .copied()
                            .zip(chunk_term_freqs.iter().copied()),
                    );
                },
            );
            assert!(postings.doc() >= horizon || postings.doc() == TERMINATED);

            let num_docs_before_repeated_fill = actual.len();
            postings.for_each_doc_freq_block_up_to(
                horizon,
                &mut docs,
                &mut term_freqs,
                |chunk_docs, _| actual.extend(chunk_docs.iter().map(|&doc| (doc, 0))),
            );
            assert_eq!(actual.len(), num_docs_before_repeated_fill);
            horizon = horizon.saturating_add(4_096);
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fill_doc_freq_block_up_to_uses_implicit_frequency() {
        let mut postings = SegmentPostings::create_from_docs(&(0..260).collect::<Vec<_>>());
        let mut docs = [u32::MAX; crate::postings::compression::COMPRESSION_BLOCK_SIZE];
        let mut term_freqs = [0; crate::postings::compression::COMPRESSION_BLOCK_SIZE];

        assert_eq!(
            postings.fill_doc_freq_block_up_to(100, &mut docs, &mut term_freqs),
            0..100
        );
        assert_eq!(&docs[..100], &(0..100).collect::<Vec<_>>());
        assert_eq!(&term_freqs[..100], &[1; 100]);
        assert_eq!(postings.doc(), 100);
    }

    #[test]
    fn test_fill_doc_freqs_up_to_crosses_blocks() {
        let doc_and_tfs: Vec<(u32, u32)> = (0..260).map(|doc| (doc * 2, doc % 7 + 1)).collect();
        let mut postings = SegmentPostings::create_from_docs_and_tfs(&doc_and_tfs, None);
        postings.seek(250);

        let mut docs = [0; 10];
        let mut term_freqs = [0; 10];
        assert_eq!(
            postings.fill_doc_freqs_up_to(268, &mut docs, &mut term_freqs),
            9
        );
        assert_eq!(&docs[..9], &[250, 252, 254, 256, 258, 260, 262, 264, 266]);
        let expected_freqs: Vec<u32> = (125..134).map(|doc| doc % 7 + 1).collect();
        assert_eq!(&term_freqs[..9], expected_freqs);
        assert_eq!(postings.doc(), 268);
    }

    #[test]
    fn test_fill_doc_freqs_up_to_decodes_complete_blocks_into_output() {
        let doc_and_tfs: Vec<(u32, u32)> = (0..600).map(|doc| (doc * 2, doc % 7 + 1)).collect();
        let mut postings = SegmentPostings::create_from_docs_and_tfs(&doc_and_tfs, None);
        postings.seek(250);

        let mut docs = [0; 400];
        let mut term_freqs = [0; 400];
        assert_eq!(
            postings.fill_doc_freqs_up_to(TERMINATED, &mut docs, &mut term_freqs),
            400
        );
        assert_eq!(
            &docs[..],
            &(125..525).map(|doc| doc * 2).collect::<Vec<_>>()
        );
        assert_eq!(
            &term_freqs[..],
            &(125..525).map(|doc| doc % 7 + 1).collect::<Vec<_>>()
        );
        assert_eq!(postings.doc(), 1_050);
    }

    #[test]
    fn test_fill_doc_freqs_uses_implicit_freq_for_basic_postings() {
        let mut postings = SegmentPostings::create_from_docs(&[1, 4, 9]);
        let mut docs = [0; 8];
        let mut term_freqs = [0; 8];

        assert_eq!(
            postings.fill_doc_freqs_up_to(TERMINATED, &mut docs, &mut term_freqs),
            3
        );
        assert_eq!(&docs[..3], &[1, 4, 9]);
        assert_eq!(&term_freqs[..3], &[1, 1, 1]);
        assert_eq!(postings.doc(), TERMINATED);
    }
}
