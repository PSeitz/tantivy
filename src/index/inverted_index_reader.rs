use std::io;

use common::json_path_writer::JSON_END_OF_PATH;
use common::{BinarySerializable, ByteCount};
#[cfg(feature = "quickwit")]
use common::{BitSet, HasLen, VInt};
#[cfg(feature = "quickwit")]
use futures_util::{FutureExt, StreamExt, TryStreamExt};
#[cfg(feature = "quickwit")]
use itertools::Itertools;
#[cfg(feature = "quickwit")]
use tantivy_fst::automaton::{AlwaysMatch, Automaton};

use crate::directory::FileSlice;
use crate::positions::PositionReader;
use crate::postings::{BlockSegmentPostings, SegmentPostings, TermInfo};
use crate::schema::{IndexRecordOption, Term, Type};
use crate::termdict::TermDictionary;
#[cfg(feature = "quickwit")]
use crate::Executor;

#[cfg(feature = "quickwit")]
// This is how many bytes we can hope to receive during a TTFB from S3 (~80MiB/s, 50ms).
const MERGE_HOLES_UNDER_BYTES: usize = (80 * 1024 * 1024 * 50) / 1000;

#[cfg(feature = "quickwit")]
fn send_coalesced_posting_ranges(
    posting_ranges: impl Iterator<Item = std::ops::Range<usize>>,
    sender: futures_channel::mpsc::UnboundedSender<std::ops::Range<usize>>,
) -> io::Result<()> {
    let posting_ranges = posting_ranges.filter(|range| !range.is_empty());
    let merged_posting_ranges = posting_ranges.coalesce(|range1, range2| {
        if range1.end + MERGE_HOLES_UNDER_BYTES >= range2.start {
            Ok(range1.start..range2.end)
        } else {
            Err((range1, range2))
        }
    });
    for posting_range in merged_posting_ranges {
        sender
            .unbounded_send(posting_range)
            .map_err(|_| io::Error::other("failed to send posting range back"))?;
    }
    Ok(())
}

/// The inverted index reader is in charge of accessing
/// the inverted index associated with a specific field.
///
/// # Note
///
/// It is safe to delete the segment associated with
/// an `InvertedIndexReader`. As long as it is open,
/// the [`FileSlice`] it is relying on should
/// stay available.
///
/// `InvertedIndexReader` are created by calling
/// [`SegmentReader::inverted_index()`](crate::SegmentReader::inverted_index).
pub struct InvertedIndexReader {
    termdict: TermDictionary,
    postings_file_slice: FileSlice,
    positions_file_slice: FileSlice,
    record_option: IndexRecordOption,
    total_num_tokens: u64,
}

/// Object that records the amount of space used by a field in an inverted index.
pub(crate) struct InvertedIndexFieldSpace {
    pub field_name: String,
    pub field_type: Type,
    pub postings_size: ByteCount,
    pub positions_size: ByteCount,
    pub num_terms: u64,
}

/// Returns None if the term is not a valid JSON path.
fn extract_field_name_and_field_type_from_json_path(term: &[u8]) -> Option<(String, Type)> {
    let index = term.iter().position(|&byte| byte == JSON_END_OF_PATH)?;
    let field_type_code = term.get(index + 1).copied()?;
    let field_type = Type::from_code(field_type_code)?;
    // Let's flush the current field.
    let field_name = String::from_utf8_lossy(&term[..index]).to_string();
    Some((field_name, field_type))
}

impl InvertedIndexFieldSpace {
    fn record(&mut self, term_info: &TermInfo) {
        self.postings_size += ByteCount::from(term_info.posting_num_bytes() as u64);
        self.positions_size += ByteCount::from(term_info.positions_num_bytes() as u64);
        self.num_terms += 1;
    }
}

impl InvertedIndexReader {
    pub(crate) fn new(
        termdict: TermDictionary,
        postings_file_slice: FileSlice,
        positions_file_slice: FileSlice,
        record_option: IndexRecordOption,
    ) -> io::Result<InvertedIndexReader> {
        let (total_num_tokens_slice, postings_body) = postings_file_slice.split(8);
        let total_num_tokens = u64::deserialize(&mut total_num_tokens_slice.read_bytes()?)?;
        Ok(InvertedIndexReader {
            termdict,
            postings_file_slice: postings_body,
            positions_file_slice,
            record_option,
            total_num_tokens,
        })
    }

    /// Creates an empty `InvertedIndexReader` object, which
    /// contains no terms at all.
    pub fn empty(record_option: IndexRecordOption) -> InvertedIndexReader {
        InvertedIndexReader {
            termdict: TermDictionary::empty(),
            postings_file_slice: FileSlice::empty(),
            positions_file_slice: FileSlice::empty(),
            record_option,
            total_num_tokens: 0u64,
        }
    }

    /// Returns the term info associated with the term.
    pub fn get_term_info(&self, term: &Term) -> io::Result<Option<TermInfo>> {
        self.termdict.get(term.serialized_value_bytes())
    }

    /// Return the term dictionary datastructure.
    pub fn terms(&self) -> &TermDictionary {
        &self.termdict
    }

    /// Return the fields and types encoded in the dictionary in lexicographic order.
    /// Only valid on JSON fields.
    ///
    /// Notice: This requires a full scan and therefore **very expensive**.
    /// TODO: Move to sstable to use the index.
    pub(crate) fn list_encoded_json_fields(&self) -> io::Result<Vec<InvertedIndexFieldSpace>> {
        let mut stream = self.termdict.stream()?;
        let mut fields: Vec<InvertedIndexFieldSpace> = Vec::new();

        let mut current_field_opt: Option<InvertedIndexFieldSpace> = None;
        // Current field bytes, including the JSON_END_OF_PATH.
        let mut current_field_bytes: Vec<u8> = Vec::new();

        while let Some((term, term_info)) = stream.next() {
            if let Some(current_field) = &mut current_field_opt {
                if term.starts_with(&current_field_bytes) {
                    // We are still in the same field.
                    current_field.record(term_info);
                    continue;
                }
            }

            // This is a new field!
            // Let's flush the current field.
            fields.extend(current_field_opt.take());
            current_field_bytes.clear();

            // And create a new one.
            let Some((field_name, field_type)) =
                extract_field_name_and_field_type_from_json_path(term)
            else {
                error!(
                    "invalid term bytes encountered {term:?}. this only happens if the term \
                     dictionary is corrupted. please report"
                );
                continue;
            };
            let mut field_space = InvertedIndexFieldSpace {
                field_name,
                field_type,
                postings_size: ByteCount::default(),
                positions_size: ByteCount::default(),
                num_terms: 0u64,
            };
            field_space.record(term_info);

            // We include the json type and the json end of path to make sure the prefix check
            // is meaningful.
            current_field_bytes.extend_from_slice(&term[..field_space.field_name.len() + 2]);
            current_field_opt = Some(field_space);
        }

        // We need to flush the last field as well.
        fields.extend(current_field_opt.take());

        Ok(fields)
    }

    /// Resets the block reader to a term's inline or external postings.
    ///
    /// This is useful for enumerating through a list of terms,
    /// and consuming the associated posting lists while avoiding
    /// reallocating a [`BlockSegmentPostings`].
    ///
    /// # Warning
    ///
    /// This does not reset the positions list.
    pub fn reset_block_postings_from_terminfo(
        &self,
        term_info: &TermInfo,
        block_postings: &mut BlockSegmentPostings,
    ) -> io::Result<()> {
        if let Some(postings) = &term_info.inline_postings {
            block_postings.reset_inline_postings(postings);
            return Ok(());
        }
        let postings_slice = self
            .postings_file_slice
            .slice(term_info.postings_range.clone());
        let postings_bytes = postings_slice.read_bytes()?;
        block_postings.reset(term_info.doc_freq, postings_bytes)?;
        Ok(())
    }

    /// Returns a block postings given a `Term`.
    /// This method is for an advanced usage only.
    ///
    /// Most users should prefer using [`Self::read_postings()`] instead.
    pub fn read_block_postings(
        &self,
        term: &Term,
        option: IndexRecordOption,
    ) -> io::Result<Option<BlockSegmentPostings>> {
        self.get_term_info(term)?
            .map(move |term_info| self.read_block_postings_from_terminfo(&term_info, option))
            .transpose()
    }

    /// Returns a block postings given a `term_info`.
    /// This method is for an advanced usage only.
    ///
    /// Most users should prefer using [`Self::read_postings()`] instead.
    pub fn read_block_postings_from_terminfo(
        &self,
        term_info: &TermInfo,
        requested_option: IndexRecordOption,
    ) -> io::Result<BlockSegmentPostings> {
        if let Some(postings) = &term_info.inline_postings {
            let mut block_postings =
                BlockSegmentPostings::new(self.record_option, requested_option);
            block_postings.reset_inline_postings(postings);
            return Ok(block_postings);
        }
        let postings_data = self
            .postings_file_slice
            .slice(term_info.postings_range.clone());
        BlockSegmentPostings::open(
            term_info.doc_freq,
            postings_data,
            self.record_option,
            requested_option,
        )
    }

    /// Returns a posting object given a `term_info`.
    /// This method is for an advanced usage only.
    ///
    /// Most users should prefer using [`Self::read_postings()`] instead.
    pub fn read_postings_from_terminfo(
        &self,
        term_info: &TermInfo,
        option: IndexRecordOption,
    ) -> io::Result<SegmentPostings> {
        let option = option.downgrade(self.record_option);

        let block_postings = self.read_block_postings_from_terminfo(term_info, option)?;
        let position_reader = {
            if let Some(postings) = &term_info.inline_postings {
                (option.has_positions() && !postings.position_deltas.is_empty())
                    .then(|| PositionReader::from_inline_postings(postings.clone()))
            } else if option.has_positions() {
                let positions_data = self
                    .positions_file_slice
                    .read_bytes_slice(term_info.positions_range.clone())?;
                let position_reader = PositionReader::open(positions_data)?;
                Some(position_reader)
            } else {
                None
            }
        };
        Ok(SegmentPostings::from_block_postings(
            block_postings,
            position_reader,
        ))
    }

    /// Returns the total number of tokens recorded for all documents
    /// (including deleted documents).
    pub fn total_num_tokens(&self) -> u64 {
        self.total_num_tokens
    }

    /// Returns the segment postings associated with the term, and with the given option,
    /// or `None` if the term has never been encountered and indexed.
    ///
    /// If the field was not indexed with the indexing options that cover
    /// the requested options, the returned [`SegmentPostings`] the method does not fail
    /// and returns a `SegmentPostings` with as much information as possible.
    ///
    /// For instance, requesting [`IndexRecordOption::WithFreqs`] for a
    /// [`TextOptions`](crate::schema::TextOptions) that does not index position
    /// will return a [`SegmentPostings`] with `DocId`s and frequencies.
    pub fn read_postings(
        &self,
        term: &Term,
        option: IndexRecordOption,
    ) -> io::Result<Option<SegmentPostings>> {
        self.get_term_info(term)?
            .map(move |term_info| self.read_postings_from_terminfo(&term_info, option))
            .transpose()
    }

    /// Returns the number of documents containing the term.
    pub fn doc_freq(&self, term: &Term) -> io::Result<u32> {
        Ok(self
            .get_term_info(term)?
            .map(|term_info| term_info.doc_freq)
            .unwrap_or(0u32))
    }
}

#[cfg(feature = "quickwit")]
impl InvertedIndexReader {
    pub(crate) async fn get_term_info_async(&self, term: &Term) -> io::Result<Option<TermInfo>> {
        self.termdict.get_async(term.serialized_value_bytes()).await
    }

    async fn get_term_range_async<'a, A: Automaton + 'a>(
        &'a self,
        terms: impl std::ops::RangeBounds<Term>,
        automaton: A,
        limit: Option<u64>,
        merge_holes_under_bytes: usize,
    ) -> io::Result<impl Iterator<Item = TermInfo> + 'a>
    where
        A::State: Clone,
    {
        use std::ops::Bound;
        // Matching terms are only used for their `TermInfo`.
        let range_builder = self.termdict.search(automaton).without_keys();
        let range_builder = match terms.start_bound() {
            Bound::Included(bound) => range_builder.ge(bound.serialized_value_bytes()),
            Bound::Excluded(bound) => range_builder.gt(bound.serialized_value_bytes()),
            Bound::Unbounded => range_builder,
        };
        let range_builder = match terms.end_bound() {
            Bound::Included(bound) => range_builder.le(bound.serialized_value_bytes()),
            Bound::Excluded(bound) => range_builder.lt(bound.serialized_value_bytes()),
            Bound::Unbounded => range_builder,
        };
        let range_builder = if let Some(limit) = limit {
            range_builder.limit(limit)
        } else {
            range_builder
        };

        let mut stream = range_builder
            .into_stream_async_merging_holes(merge_holes_under_bytes)
            .await?;

        let iter = std::iter::from_fn(move || stream.next_without_key().cloned());

        // limit on stream is only an optimization to load less data, the stream may still return
        // more than limit elements.
        let limit = limit.map(|limit| limit as usize).unwrap_or(usize::MAX);
        let iter = iter.take(limit);

        Ok(iter)
    }

    async fn warm_postings_and_positions(
        &self,
        postings_range: std::ops::Range<usize>,
        positions_range: std::ops::Range<usize>,
        with_positions: bool,
    ) -> io::Result<()> {
        futures_util::future::try_join_all(
            [
                (&self.postings_file_slice, postings_range, true),
                (&self.positions_file_slice, positions_range, with_positions),
            ]
            .into_iter()
            .filter(|(_, range, enabled)| *enabled && !range.is_empty())
            .map(|(file, range, _)| file.read_bytes_slice_async(range)),
        )
        .await?;
        Ok(())
    }

    /// Warmup a block postings given a `Term`.
    /// This method is for an advanced usage only.
    ///
    /// returns a boolean, whether the term was found in the dictionary
    pub async fn warm_postings(&self, term: &Term, with_positions: bool) -> io::Result<bool> {
        let term_info_opt: Option<TermInfo> = self.get_term_info_async(term).await?;
        if let Some(term_info) = term_info_opt {
            self.warm_postings_and_positions(
                term_info.postings_range,
                term_info.positions_range,
                with_positions,
            )
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Warmup a block postings given a range of `Term`s.
    /// This method is for an advanced usage only.
    ///
    /// returns a boolean, whether a term matching the range was found in the dictionary
    pub async fn warm_postings_range(
        &self,
        terms: impl std::ops::RangeBounds<Term>,
        limit: Option<u64>,
        with_positions: bool,
    ) -> io::Result<bool> {
        let mut term_info = self
            .get_term_range_async(terms, AlwaysMatch, limit, 0)
            .await?;

        let Some(first_terminfo) = term_info.next() else {
            // no key matches, nothing more to load
            return Ok(false);
        };

        let last_terminfo = term_info.last().unwrap_or_else(|| first_terminfo.clone());

        let postings_range = first_terminfo.postings_range.start..last_terminfo.postings_range.end;
        let positions_range =
            first_terminfo.positions_range.start..last_terminfo.positions_range.end;

        self.warm_postings_and_positions(postings_range, positions_range, with_positions)
            .await?;
        Ok(true)
    }

    async fn download_posting_ranges(
        &self,
        posting_ranges: futures_channel::mpsc::UnboundedReceiver<std::ops::Range<usize>>,
    ) -> io::Result<bool> {
        let slices_downloaded = posting_ranges
            .map(|posting_range| {
                self.postings_file_slice
                    .read_bytes_slice_async(posting_range)
                    .map(|result| result.map(|_slice| ()))
            })
            .buffer_unordered(5)
            .try_collect::<Vec<()>>()
            .await?;
        Ok(!slices_downloaded.is_empty())
    }

    /// Warmup a block postings given a range of `Term`s.
    /// This method is for an advanced usage only.
    ///
    /// returns a boolean, whether a term matching the range was found in the dictionary
    pub async fn warm_postings_automaton<
        A: Automaton + Clone + Send + 'static,
        E: FnOnce(Box<dyn FnOnce() -> io::Result<()> + Send>) -> F,
        F: std::future::Future<Output = io::Result<()>>,
    >(
        &self,
        automaton: A,
        // with_positions: bool, at the moment we have no use for it, and supporting it would add
        // complexity to the coalesce
        executor: E,
    ) -> io::Result<bool>
    where
        A::State: Clone,
    {
        // We build a first iterator to download everything. Simply calling the function already
        // download everything we need from the sstable, but doesn't start iterating over it.
        let mut term_info_iter = self
            .get_term_range_async(.., automaton.clone(), None, MERGE_HOLES_UNDER_BYTES)
            .await?;
        // Inline-only matches also count, even though they require no external postings reads.
        let terms_found = term_info_iter.next().is_some();
        drop(term_info_iter);

        let (sender, posting_ranges_to_load_stream) = futures_channel::mpsc::unbounded();
        let termdict = self.termdict.clone();
        let cpu_bound_task = move || {
            // then we build a 2nd iterator, this one with no holes, so we don't go through blocks
            // we can't match.
            // This makes the assumption there is a caching layer below us, which gives sync read
            // for free after the initial async access. This might not always be true, but is in
            // Quickwit.
            // We build things from this closure otherwise we get into lifetime issues that can only
            // be solved with self referential strucs. Returning an io::Result from here is a bit
            // more leaky abstraction-wise, but a lot better than the alternative
            let mut stream = termdict.search(automaton).without_keys().into_stream()?;

            let posting_ranges = std::iter::from_fn(move || {
                stream
                    .next_without_key()
                    .map(|value| value.postings_range.clone())
            });
            send_coalesced_posting_ranges(posting_ranges, sender)
        };
        let task_handle = executor(Box::new(cpu_bound_task));

        futures_util::future::try_join(
            task_handle,
            self.download_posting_ranges(posting_ranges_to_load_stream),
        )
        .await?;
        Ok(terms_found)
    }

    /// Warms postings for one automaton and returns a bitset of matching document IDs.
    ///
    /// `max_doc` must be the segment's maximum document ID plus one, including deleted documents.
    /// The directory must cache asynchronous reads for subsequent synchronous reads.
    /// Term enumeration and postings decoding run on the executor. Use a thread-pool executor
    /// to keep this CPU work off the calling thread.
    pub async fn warm_postings_automaton_with_bitset<A: Automaton + Send + 'static>(
        &self,
        automaton: A,
        max_doc: u32,
        executor: &Executor,
    ) -> io::Result<BitSet>
    where
        A::State: Clone,
    {
        let term_info_stream = self
            .termdict
            .search(&automaton)
            .into_stream_async_merging_holes(MERGE_HOLES_UNDER_BYTES)
            .await?;
        drop(term_info_stream);

        let (posting_range_sender, posting_range_receiver) = futures_channel::mpsc::unbounded();
        let termdict = self.termdict.clone();
        let cpu_bound_task = move || {
            let mut stream = termdict.search(automaton).without_keys().into_stream()?;
            let mut encoded_postings = Vec::new();
            let mut bitset = BitSet::with_max_value(max_doc);
            let mut previous_end = 0;
            let posting_ranges = std::iter::from_fn(|| loop {
                let info = stream.next_without_key()?;
                if let Some(postings) = &info.inline_postings {
                    for &doc in &postings.doc_ids {
                        bitset.insert(doc);
                    }
                    continue;
                }
                let range = info.postings_range.clone();
                if range.is_empty() {
                    continue;
                }
                // Retain only what decoding postings needs, avoiding another dictionary scan.
                VInt((range.start - previous_end) as u64).serialize_into_vec(&mut encoded_postings);
                VInt(range.len() as u64).serialize_into_vec(&mut encoded_postings);
                VInt(u64::from(info.doc_freq)).serialize_into_vec(&mut encoded_postings);
                previous_end = range.end;
                return Some(range);
            });
            send_coalesced_posting_ranges(posting_ranges, posting_range_sender)?;
            Ok((encoded_postings, bitset))
        };
        let task_handle = executor.spawn_blocking(cpu_bound_task).map(|result| {
            result.map_err(|_| io::Error::other("automaton warmup task panicked"))?
        });
        let ((encoded_postings, mut bitset), _) = futures_util::future::try_join(
            task_handle,
            self.download_posting_ranges(posting_range_receiver),
        )
        .await?;

        let postings_file_slice = self.postings_file_slice.clone();
        let record_option = self.record_option;
        let cpu_bound_task = move || {
            let mut data = encoded_postings.as_slice();
            let mut previous_end = 0;
            while !data.is_empty() {
                let start = previous_end + VInt::deserialize_u64(&mut data)? as usize;
                let end = start + VInt::deserialize_u64(&mut data)? as usize;
                let doc_freq = VInt::deserialize_u64(&mut data)? as u32;
                previous_end = end;
                let mut postings = BlockSegmentPostings::open(
                    doc_freq,
                    postings_file_slice.slice(start..end),
                    record_option,
                    IndexRecordOption::Basic,
                )?;
                while !postings.docs().is_empty() {
                    for &doc in postings.docs() {
                        bitset.insert(doc);
                    }
                    postings.advance();
                }
            }
            Ok(bitset)
        };
        executor
            .spawn_blocking(cpu_bound_task)
            .await
            .map_err(|_| io::Error::other("automaton bitset task panicked"))?
    }

    /// Warmup the block postings for all terms.
    /// This method is for an advanced usage only.
    ///
    /// If you know which terms to pre-load, prefer using [`Self::warm_postings`] or
    /// [`Self::warm_postings`] instead.
    pub async fn warm_postings_full(&self, with_positions: bool) -> io::Result<()> {
        self.termdict.warm_up_dictionary().await?;
        self.warm_postings_and_positions(
            0..self.postings_file_slice.len(),
            0..self.positions_file_slice.len(),
            with_positions,
        )
        .await
    }

    /// Returns the number of documents containing the term asynchronously.
    pub async fn doc_freq_async(&self, term: &Term) -> io::Result<u32> {
        Ok(self
            .get_term_info_async(term)
            .await?
            .map(|term_info| term_info.doc_freq)
            .unwrap_or(0u32))
    }
}

#[cfg(all(test, feature = "quickwit"))]
mod tests {
    use std::ops::Range;
    use std::sync::Arc;

    use common::HasLen;
    use tantivy_fst::automaton::AlwaysMatch;
    use tantivy_fst::Regex;

    use super::InvertedIndexReader;
    use crate::directory::{FileHandle, FileSlice, OwnedBytes};
    use crate::query::BitSetDocSet;
    use crate::schema::{Schema, STRING};
    use crate::{DocSet, Executor, Index, IndexWriter, TERMINATED};

    #[derive(Debug)]
    struct RejectEmptyReads(FileSlice);

    impl HasLen for RejectEmptyReads {
        fn len(&self) -> usize {
            self.0.len()
        }
    }

    impl FileHandle for RejectEmptyReads {
        fn read_bytes(&self, range: Range<usize>) -> std::io::Result<OwnedBytes> {
            assert!(!range.is_empty(), "empty external read");
            self.0.read_bytes_slice(range)
        }

        fn read_bytes_async<'life0, 'async_trait>(
            &'life0 self,
            range: Range<usize>,
        ) -> futures_util::future::BoxFuture<'async_trait, std::io::Result<OwnedBytes>>
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            // These test files are held in memory.
            Box::pin(async move { self.read_bytes(range) })
        }
    }

    fn reject_empty_reads(reader: &InvertedIndexReader) -> InvertedIndexReader {
        let wrap = |file: &FileSlice| FileSlice::new(Arc::new(RejectEmptyReads(file.clone())));
        InvertedIndexReader {
            termdict: reader.termdict.clone(),
            postings_file_slice: wrap(&reader.postings_file_slice),
            positions_file_slice: wrap(&reader.positions_file_slice),
            record_option: reader.record_option,
            total_num_tokens: reader.total_num_tokens,
        }
    }

    #[test]
    fn test_inline_postings_reads_and_warmups_skip_external_files() -> crate::Result<()> {
        use crate::schema::{IndexRecordOption, Term, TEXT};

        let mut schema = Schema::builder();
        let field = schema.add_text_field("field", TEXT);
        let mut index = Index::create_in_ram(schema.build());
        index.settings_mut().max_inline_docs = 8;
        let mut writer: IndexWriter = index.writer_for_tests()?;
        writer.add_document(doc!(field => "apple apple banana"))?;
        writer.add_document(doc!(field => "banana berry"))?;
        writer.commit()?;
        let searcher = index.reader()?.searcher();
        let reader = searcher.segment_reader(0).inverted_index(field)?;
        let reader = reject_empty_reads(&reader);
        // Empty files plus the read guard reject every external read, including zero-byte reads.
        assert!(reader.postings_file_slice.is_empty());
        assert!(reader.positions_file_slice.is_empty());
        let apple = Term::from_field_text(field, "apple");
        let banana = Term::from_field_text(field, "banana");
        let info = reader.get_term_info(&apple)?.unwrap();
        assert!(info.inline_postings.is_some());
        let mut postings =
            reader.read_postings_from_terminfo(&info, IndexRecordOption::WithFreqsAndPositions)?;
        reader.reset_block_postings_from_terminfo(
            &reader.get_term_info(&banana)?.unwrap(),
            &mut postings.block_cursor,
        )?;
        let empty = reject_empty_reads(&InvertedIndexReader::empty(
            IndexRecordOption::WithFreqsAndPositions,
        ));
        for (reader, max_doc) in [(&reader, 2), (&empty, 0)] {
            futures::executor::block_on(async {
                let found = max_doc != 0;
                assert_eq!(reader.warm_postings(&apple, true).await?, found);
                assert_eq!(
                    reader
                        .warm_postings_range(apple.clone()..=banana.clone(), None, true)
                        .await?,
                    found
                );
                assert!(!reader.warm_postings_range(.., Some(0), true).await?);
                assert_eq!(
                    reader
                        .warm_postings_automaton(AlwaysMatch, |task| async move { task() })
                        .await?,
                    found
                );
                let bitset = reader
                    .warm_postings_automaton_with_bitset(
                        AlwaysMatch,
                        max_doc,
                        &Executor::single_thread(),
                    )
                    .await?;
                assert_eq!(bitset.len(), max_doc as usize);
                reader.warm_postings_full(true).await
            })?;
        }
        Ok(())
    }

    #[test]
    fn test_warm_postings_automaton_with_bitset() -> crate::Result<()> {
        let mut schema_builder = Schema::builder();
        let field = schema_builder.add_text_field("field", STRING);
        let mut index = Index::create_in_ram(schema_builder.build());
        index.settings_mut().max_inline_docs = 8;
        let mut writer: IndexWriter = index.writer_for_tests()?;
        writer.add_document(doc!(field => "apple", field => "banana"))?;
        writer.add_document(doc!(field => "apricot"))?;
        writer.add_document(doc!(field => "banana", field => "berry"))?;
        writer.add_document(doc!(field => "carrot"))?;
        // Exercise full external blocks, inline lists, and duplicate matches.
        for _ in 0..260 {
            writer.add_document(doc!(field => "apple", field => "apricot"))?;
        }
        writer.commit()?;
        let searcher = index.reader()?.searcher();
        let segment_reader = searcher.segment_reader(0);
        let inverted_index = segment_reader.inverted_index(field)?;
        let inverted_index = reject_empty_reads(&inverted_index);
        let executor = Executor::multi_thread(1, "warmup-test-")?;
        for (pattern, expected) in [
            ("b.*", vec![0, 2]),
            ("z.*", vec![]),
            ("a.*", [vec![0, 1], (4..264).collect()].concat()),
            ("apple|berry", [vec![0, 2], (4..264).collect()].concat()),
            (".*", (0..264).collect()),
        ] {
            let bitset =
                futures::executor::block_on(inverted_index.warm_postings_automaton_with_bitset(
                    Regex::new(pattern).unwrap(),
                    segment_reader.max_doc(),
                    &executor,
                ))?;
            assert_eq!(bitset.len(), expected.len());
            let mut docset = BitSetDocSet::from(bitset);
            let mut docs = Vec::new();
            while docset.doc() != TERMINATED {
                docs.push(docset.doc());
                docset.advance();
            }
            assert_eq!(docs, expected);
        }
        futures::executor::block_on(async {
            assert!(
                inverted_index
                    .warm_postings_automaton(AlwaysMatch, |task| async move { task() })
                    .await?
            );
            assert!(inverted_index.warm_postings_range(.., None, true).await?);
            inverted_index.warm_postings_full(true).await
        })?;
        Ok(())
    }

    #[test]
    fn test_warm_postings_automaton_with_bitset_empty_index() -> crate::Result<()> {
        let inverted_index =
            super::InvertedIndexReader::empty(crate::schema::IndexRecordOption::Basic);
        let executor = Executor::single_thread();
        let bitset =
            futures::executor::block_on(inverted_index.warm_postings_automaton_with_bitset(
                Regex::new(".*").unwrap(),
                0,
                &executor,
            ))?;
        assert_eq!(bitset.len(), 0);
        Ok(())
    }
}
