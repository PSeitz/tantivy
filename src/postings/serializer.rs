use std::cmp::Ordering;
use std::io::{self, Write};
use std::sync::Arc;

use common::{BinarySerializable, CountingWriter, VInt};

use super::{LoadedPostings, TermInfo};
use crate::directory::{CompositeWrite, WritePtr};
use crate::fieldnorm::FieldNormReader;
use crate::index::Segment;
use crate::positions::PositionSerializer;
use crate::postings::compression::{BlockEncoder, VIntEncoder, COMPRESSION_BLOCK_SIZE};
use crate::postings::skip::SkipSerializer;
use crate::query::Bm25Weight;
use crate::schema::{Field, FieldEntry, IndexRecordOption, Schema};
use crate::termdict::TermDictionaryBuilder;
use crate::{DocId, Score};

/// `InvertedIndexSerializer` is in charge of serializing
/// postings on disk, in the
/// * `.idx` (inverted index)
/// * `.pos` (positions file)
/// * `.term` (term dictionary)
///
/// `PostingsWriter` are in charge of pushing the data to the
/// serializer.
///
/// The serializer expects to receive the following calls
/// in this order :
/// * `set_field(...)`
/// * `new_term(...)`
/// * `write_doc(...)`
/// * `write_doc(...)`
/// * `write_doc(...)`
/// * ...
/// * `close_term()`
/// * `new_term(...)`
/// * `write_doc(...)`
/// * ...
/// * `close_term()`
/// * `set_field(...)`
/// * ...
/// * `close()`
///
/// Terms have to be pushed in a lexicographically-sorted order.
/// Within a term, documents have to be pushed in increasing order.
///
/// A description of the serialization format is
/// [available here](https://fulmicoton.gitbooks.io/tantivy-doc/content/inverted-index.html).
pub struct InvertedIndexSerializer {
    terms_write: CompositeWrite<WritePtr>,
    postings_write: CompositeWrite<WritePtr>,
    positions_write: CompositeWrite<WritePtr>,
    schema: Schema,
    max_inline_docs: u32,
}

impl InvertedIndexSerializer {
    /// Open a new `InvertedIndexSerializer` for the given segment
    pub fn open(segment: &Segment) -> crate::Result<InvertedIndexSerializer> {
        use crate::index::SegmentComponent::{Positions, Postings, Terms};
        segment.index().settings().validate_max_inline_docs()?;
        let inv_index_serializer = InvertedIndexSerializer {
            terms_write: CompositeWrite::wrap(segment.open_write(Terms)?),
            postings_write: CompositeWrite::wrap(segment.open_write(Postings)?),
            positions_write: CompositeWrite::wrap(segment.open_write(Positions)?),
            schema: segment.schema(),
            max_inline_docs: segment.index().settings().max_inline_docs,
        };
        Ok(inv_index_serializer)
    }

    /// Must be called before starting pushing terms of
    /// a given field.
    ///
    /// Loads the indexing options for the given field.
    pub fn new_field(
        &mut self,
        field: Field,
        total_num_tokens: u64,
        fieldnorm_reader: Option<FieldNormReader>,
    ) -> io::Result<FieldSerializer<'_>> {
        let field_entry: &FieldEntry = self.schema.get_field_entry(field);
        let term_dictionary_write = self.terms_write.for_field(field);
        let postings_write = self.postings_write.for_field(field);
        let positions_write = self.positions_write.for_field(field);
        let index_record_option = field_entry
            .field_type()
            .index_record_option()
            .unwrap_or(IndexRecordOption::Basic);
        FieldSerializer::create(
            index_record_option,
            total_num_tokens,
            term_dictionary_write,
            postings_write,
            positions_write,
            fieldnorm_reader,
            self.max_inline_docs,
        )
    }

    /// Closes the serializer.
    pub fn close(self) -> io::Result<()> {
        self.terms_write.close()?;
        self.postings_write.close()?;
        self.positions_write.close()?;
        Ok(())
    }
}

/// The field serializer is in charge of
/// the serialization of a specific field.
pub struct FieldSerializer<'a, W: Write = WritePtr> {
    term_dictionary_builder: TermDictionaryBuilder<&'a mut CountingWriter<W>>,
    postings_serializer: PostingsSerializer,
    positions_serializer_opt: Option<PositionSerializer<&'a mut CountingWriter<W>>>,
    current_term_info: TermInfo,
    buffered_position_deltas: Vec<u32>,
    max_inline_docs: u32,
    term_open: bool,
    postings_write: &'a mut CountingWriter<W>,
    postings_start_offset: u64,
}

impl<'a, W: Write> FieldSerializer<'a, W> {
    /// Creates a new `FieldSerializer` for the given field type.
    /// Quickwit stores lists with at most `max_inline_docs` documents in the
    /// term dictionary. FST dictionaries inline only a single occurrence with
    /// a recorded frequency and position when `max_inline_docs` is nonzero.
    pub fn create(
        index_record_option: IndexRecordOption,
        total_num_tokens: u64,
        term_dictionary_write: &'a mut CountingWriter<W>,
        postings_write: &'a mut CountingWriter<W>,
        positions_write: &'a mut CountingWriter<W>,
        fieldnorm_reader: Option<FieldNormReader>,
        max_inline_docs: u32,
    ) -> io::Result<FieldSerializer<'a, W>> {
        let term_dictionary_builder = TermDictionaryBuilder::create_with_max_inline_docs(
            term_dictionary_write,
            max_inline_docs,
        )?;
        #[cfg(not(feature = "quickwit"))]
        let max_inline_docs = max_inline_docs.min(1);
        total_num_tokens.serialize(postings_write)?;
        let average_fieldnorm = fieldnorm_reader
            .as_ref()
            .map(|ff_reader| total_num_tokens as Score / ff_reader.num_docs() as Score)
            .unwrap_or(0.0);
        let postings_serializer =
            PostingsSerializer::new(average_fieldnorm, index_record_option, fieldnorm_reader);
        let positions_serializer_opt = if index_record_option.has_positions() {
            Some(PositionSerializer::new(positions_write))
        } else {
            None
        };

        let postings_start_offset = postings_write.written_bytes();
        Ok(FieldSerializer {
            term_dictionary_builder,
            postings_serializer,
            positions_serializer_opt,
            current_term_info: TermInfo::default(),
            buffered_position_deltas: Vec::new(),
            max_inline_docs,
            term_open: false,
            postings_write,
            postings_start_offset,
        })
    }

    fn postings_offset(&self) -> usize {
        (self.postings_write.written_bytes() - self.postings_start_offset) as usize
    }

    fn current_term_info(&self) -> TermInfo {
        let positions_start =
            if let Some(positions_serializer) = self.positions_serializer_opt.as_ref() {
                positions_serializer.written_bytes()
            } else {
                0u64
            } as usize;
        let addr = self.postings_offset();
        TermInfo {
            doc_freq: 0,
            postings_range: addr..addr,
            positions_range: positions_start..positions_start,
            inline_postings: None,
        }
    }

    /// Starts the postings for a new term.
    /// * term - the term. It needs to come after the previous term according to the lexicographical
    ///   order.
    /// * term_doc_freq - return the number of document containing the term.
    pub fn new_term(
        &mut self,
        term: &[u8],
        term_doc_freq: u32,
        record_term_freq: bool,
    ) -> io::Result<()> {
        assert!(
            !self.term_open,
            "Called new_term, while the previous term was not closed."
        );
        self.term_open = true;
        self.current_term_info = self.current_term_info();
        self.term_dictionary_builder.insert_key(term)?;
        self.postings_serializer
            .new_term(term_doc_freq, record_term_freq);
        Ok(())
    }

    /// Starts the postings for a new term without recording term frequencies.
    pub fn new_term_without_freq(&mut self, term: &[u8]) -> io::Result<()> {
        self.new_term(term, 0, false)
    }

    /// Serialize the information that a document contains for the current term:
    /// its term frequency, and the position deltas.
    ///
    /// At this point, the positions are already `delta-encoded`.
    /// For instance, if the positions are `2, 3, 17`,
    /// `position_deltas` is `2, 1, 14`
    ///
    /// Term frequencies and positions may be ignored by the serializer depending
    /// on the configuration of the field in the `Schema`.
    pub fn write_doc(&mut self, doc_id: DocId, term_freq: u32, position_deltas: &[u32]) {
        self.current_term_info.doc_freq += 1;
        self.postings_serializer.write_doc(doc_id, term_freq);
        if let Some(positions_serializer) = self.positions_serializer_opt.as_mut() {
            assert_eq!(term_freq as usize, position_deltas.len());
            if self.current_term_info.doc_freq <= self.max_inline_docs {
                self.buffered_position_deltas
                    .extend_from_slice(position_deltas);
            } else {
                positions_serializer.write_positions_delta(&self.buffered_position_deltas);
                self.buffered_position_deltas.clear();
                positions_serializer.write_positions_delta(position_deltas);
            }
        }
    }

    /// Finishes serializing the term's postings.
    ///
    /// Eligible lists within the inline document limit are stored in the term
    /// dictionary. External lists use `VInt` encoding for their final partial block.
    pub fn close_term(&mut self) -> io::Result<()> {
        crate::fail_point!("FieldSerializer::close_term", |msg: Option<String>| {
            Err(io::Error::other(format!("{msg:?}")))
        });

        if !self.term_open {
            return Ok(());
        }

        let inline = (1..=self.max_inline_docs).contains(&self.current_term_info.doc_freq);
        #[cfg(not(feature = "quickwit"))]
        let inline = inline
            && self.current_term_info.doc_freq == 1
            && self.postings_serializer.term_has_freq
            && self.postings_serializer.block.term_freqs() == [1]
            && self.buffered_position_deltas.len() == 1;
        if inline {
            let block = &self.postings_serializer.block;
            self.current_term_info.inline_postings = Some(Arc::new(LoadedPostings {
                doc_ids: block.doc_ids().into(),
                term_freqs: if self.postings_serializer.term_has_freq {
                    block.term_freqs().into()
                } else {
                    Box::default()
                },
                position_deltas: std::mem::take(&mut self.buffered_position_deltas)
                    .into_boxed_slice(),
            }));
        } else {
            self.postings_serializer
                .close_term(self.current_term_info.doc_freq, self.postings_write)?;
            self.current_term_info.postings_range.end = self.postings_offset();
            if let Some(positions_serializer) = self.positions_serializer_opt.as_mut() {
                positions_serializer.write_positions_delta(&self.buffered_position_deltas);
                self.buffered_position_deltas.clear();
                positions_serializer.close_term()?;
                self.current_term_info.positions_range.end =
                    positions_serializer.written_bytes() as usize;
            }
        }
        self.term_dictionary_builder
            .insert_value(&self.current_term_info)?;
        self.term_open = false;
        Ok(())
    }

    /// Closes the current field.
    pub fn close(mut self) -> io::Result<()> {
        self.close_term()?;
        if let Some(positions_serializer) = self.positions_serializer_opt {
            positions_serializer.close()?;
        }
        self.term_dictionary_builder.finish()?;
        Ok(())
    }
}

struct Block {
    doc_ids: [DocId; COMPRESSION_BLOCK_SIZE],
    term_freqs: [u32; COMPRESSION_BLOCK_SIZE],
    len: usize,
}

impl Block {
    fn new() -> Self {
        Block {
            doc_ids: [0u32; COMPRESSION_BLOCK_SIZE],
            term_freqs: [0u32; COMPRESSION_BLOCK_SIZE],
            len: 0,
        }
    }

    fn doc_ids(&self) -> &[DocId] {
        &self.doc_ids[..self.len]
    }

    fn term_freqs(&self) -> &[u32] {
        &self.term_freqs[..self.len]
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn append_doc(&mut self, doc: DocId, term_freq: u32) {
        let len = self.len;
        self.doc_ids[len] = doc;
        self.term_freqs[len] = term_freq;
        self.len = len + 1;
    }

    fn is_full(&self) -> bool {
        self.len == COMPRESSION_BLOCK_SIZE
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn last_doc(&self) -> DocId {
        assert_eq!(self.len, COMPRESSION_BLOCK_SIZE);
        self.doc_ids[COMPRESSION_BLOCK_SIZE - 1]
    }
}

/// Serializer for postings lists.
pub struct PostingsSerializer {
    last_doc_id_encoded: u32,

    block_encoder: BlockEncoder,
    block: Box<Block>,

    postings_write: Vec<u8>,
    skip_write: SkipSerializer,

    mode: IndexRecordOption,
    fieldnorm_reader: Option<FieldNormReader>,

    bm25_weight: Option<Bm25Weight>,
    avg_fieldnorm: Score, /* Average number of term in the field for that segment.
                           * this value is used to compute the block wand information. */
    term_has_freq: bool,
}

impl PostingsSerializer {
    /// Creates a new `PostingsSerializer`.
    /// * avg_fieldnorm - average field norm for the field being serialized.
    /// * mode - indexing options for the field being serialized.
    pub fn new(
        avg_fieldnorm: Score,
        mode: IndexRecordOption,
        fieldnorm_reader: Option<FieldNormReader>,
    ) -> PostingsSerializer {
        PostingsSerializer {
            block_encoder: BlockEncoder::new(),
            block: Box::new(Block::new()),

            postings_write: Vec::new(),
            skip_write: SkipSerializer::new(),

            last_doc_id_encoded: 0u32,
            mode,

            fieldnorm_reader,
            bm25_weight: None,
            avg_fieldnorm,
            term_has_freq: false,
        }
    }

    /// Starts the serialization for a new term.
    /// * term_doc_freq - the number of documents containing the term.
    pub fn new_term(&mut self, term_doc_freq: u32, record_term_freq: bool) {
        self.block.clear();
        self.last_doc_id_encoded = 0;
        self.bm25_weight = None;

        self.term_has_freq = self.mode.has_freq() && record_term_freq;
        if !self.term_has_freq {
            return;
        }

        let num_docs_in_segment: u64 =
            if let Some(fieldnorm_reader) = self.fieldnorm_reader.as_ref() {
                fieldnorm_reader.num_docs() as u64
            } else {
                return;
            };

        if num_docs_in_segment == 0 {
            return;
        }

        self.bm25_weight = Some(Bm25Weight::for_one_term_without_explain(
            term_doc_freq as u64,
            num_docs_in_segment,
            self.avg_fieldnorm,
        ));
    }

    fn write_block(&mut self) {
        {
            // encode the doc ids
            let (num_bits, block_encoded): (u8, &[u8]) = self
                .block_encoder
                .compress_block_sorted(self.block.doc_ids(), self.last_doc_id_encoded);
            self.last_doc_id_encoded = self.block.last_doc();
            self.skip_write
                .write_doc(self.last_doc_id_encoded, num_bits);
            // last el block 0, offset block 1,
            self.postings_write.extend(block_encoded);
        }
        if self.term_has_freq {
            // encode the term frequencies
            let (num_bits, block_encoded): (u8, &[u8]) = self
                .block_encoder
                .compress_block_unsorted(self.block.term_freqs(), true);
            self.postings_write.extend(block_encoded);
            self.skip_write.write_term_freq(num_bits);
            if self.mode.has_positions() {
                // We serialize the sum of term freqs within the skip information
                // in order to navigate through positions.
                let sum_freq = self.block.term_freqs().iter().cloned().sum();
                self.skip_write.write_total_term_freq(sum_freq);
            }
            let mut blockwand_params = (0u8, 0u32);
            if let Some(bm25_weight) = self.bm25_weight.as_ref() {
                if let Some(fieldnorm_reader) = self.fieldnorm_reader.as_ref() {
                    let docs = self.block.doc_ids().iter().cloned();
                    let term_freqs = self.block.term_freqs().iter().cloned();
                    let fieldnorms = docs.map(|doc| fieldnorm_reader.fieldnorm_id(doc));
                    blockwand_params = fieldnorms
                        .zip(term_freqs)
                        .max_by(
                            |(left_fieldnorm_id, left_term_freq),
                             (right_fieldnorm_id, right_term_freq)| {
                                let left_score =
                                    bm25_weight.tf_factor(*left_fieldnorm_id, *left_term_freq);
                                let right_score =
                                    bm25_weight.tf_factor(*right_fieldnorm_id, *right_term_freq);
                                left_score
                                    .partial_cmp(&right_score)
                                    .unwrap_or(Ordering::Equal)
                            },
                        )
                        .unwrap();
                }
            }
            let (fieldnorm_id, term_freq) = blockwand_params;
            self.skip_write.write_blockwand_max(fieldnorm_id, term_freq);
        }
        self.block.clear();
    }

    /// Register that the given document contains the current term.
    /// * doc_id - the document id.
    /// * term_freq - the term frequency within the document.
    pub fn write_doc(&mut self, doc_id: DocId, term_freq: u32) {
        self.block.append_doc(doc_id, term_freq);
        if self.block.is_full() {
            self.write_block();
        }
    }

    /// Finish the serialization for this term.
    pub fn close_term(
        &mut self,
        doc_freq: u32,
        output_write: &mut impl std::io::Write,
    ) -> io::Result<()> {
        if !self.block.is_empty() {
            // we have doc ids waiting to be written
            // this happens when the number of doc ids is
            // not a perfect multiple of our block size.
            //
            // In that case, the remaining part is encoded
            // using variable int encoding.
            {
                let block_encoded = self
                    .block_encoder
                    .compress_vint_sorted(self.block.doc_ids(), self.last_doc_id_encoded);
                self.postings_write.write_all(block_encoded)?;
            }
            // ... Idem for term frequencies
            if self.term_has_freq {
                let block_encoded = self
                    .block_encoder
                    .compress_vint_unsorted(self.block.term_freqs());
                self.postings_write.write_all(block_encoded)?;
            }
            self.block.clear();
        }
        if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
            let skip_data = self.skip_write.data();
            VInt(skip_data.len() as u64).serialize(output_write)?;
            output_write.write_all(skip_data)?;
        }
        output_write.write_all(&self.postings_write[..])?;
        self.skip_write.clear();
        self.postings_write.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_postings_preserve_external_offsets_and_bytes() -> io::Result<()> {
        // Record options and frequency-free JSON terms are covered by the reader tests.
        let mode = IndexRecordOption::WithFreqsAndPositions;
        for max_inline_docs in [0, 32] {
            let mut terms = CountingWriter::wrap(Vec::new());
            let mut postings = CountingWriter::wrap(Vec::new());
            let mut positions = CountingWriter::wrap(Vec::new());
            let mut serializer = FieldSerializer::create(
                mode,
                0,
                &mut terms,
                &mut postings,
                &mut positions,
                None,
                max_inline_docs,
            )?;
            let mut expected_postings = Vec::new();
            let mut expected_positions = Vec::new();
            let mut position_serializer = PositionSerializer::new(&mut expected_positions);

            // End with a single-document list to check offsets and resetting after an external list.
            for (term, (count, first_freq)) in [
                (1, 1),
                (1, 2),
                (1, 130),
                (1, 1),
                (32, 1),
                (33, 1),
                (127, 1),
                (128, 1),
                (129, 1),
                (1, 1),
            ]
            .into_iter()
            .enumerate()
            {
                serializer.new_term(&term.to_be_bytes(), count, true)?;
                let fits_inline_limit = count <= max_inline_docs
                    && (cfg!(feature = "quickwit") || (count == 1 && first_freq == 1));
                let postings_start = expected_postings.len();
                let positions_start = position_serializer.written_bytes() as usize;
                let mut external_serializer = PostingsSerializer::new(0.0, mode, None);
                external_serializer.new_term(count, true);
                for doc in 0..count {
                    let doc_id = doc * 3;
                    // Include a full positions block among the first eight buffered documents.
                    let freq = if doc == 0 {
                        first_freq
                    } else if doc % 4 == 3 {
                        128
                    } else {
                        doc % 3 + 1
                    };
                    let deltas: Vec<u32> = (0..freq).map(|position| position % 5).collect();
                    serializer.write_doc(doc_id, freq, &deltas);
                    if !fits_inline_limit {
                        external_serializer.write_doc(doc_id, freq);
                        position_serializer.write_positions_delta(&deltas);
                    }
                }
                serializer.close_term()?;
                // Closing twice must not emit data or duplicate a dictionary value.
                serializer.close_term()?;
                let info = &serializer.current_term_info;
                assert_eq!(info.doc_freq, count);
                assert_eq!(info.inline_postings.is_some(), fits_inline_limit);
                if !fits_inline_limit {
                    external_serializer.close_term(count, &mut expected_postings)?;
                    position_serializer.close_term()?;
                }
                assert_eq!(info.postings_range, postings_start..expected_postings.len());
                assert_eq!(
                    info.positions_range,
                    positions_start..position_serializer.written_bytes() as usize
                );
            }
            serializer.close()?;
            position_serializer.close()?;
            assert_eq!(&postings.finish()[8..], expected_postings);
            assert_eq!(positions.finish(), expected_positions);
        }
        Ok(())
    }

    #[cfg(not(feature = "quickwit"))]
    #[test]
    fn test_fst_inline_requires_recorded_frequency_and_position() -> io::Result<()> {
        for mode in [
            IndexRecordOption::Basic,
            IndexRecordOption::WithFreqs,
            IndexRecordOption::WithFreqsAndPositions,
        ] {
            for record_freq in [false, true] {
                let mut terms = CountingWriter::wrap(Vec::new());
                let mut postings = CountingWriter::wrap(Vec::new());
                let mut positions = CountingWriter::wrap(Vec::new());
                let mut serializer = FieldSerializer::create(
                    mode,
                    0,
                    &mut terms,
                    &mut postings,
                    &mut positions,
                    None,
                    32,
                )?;
                assert_eq!(serializer.max_inline_docs, 1);
                serializer.new_term(b"term", 1, record_freq)?;
                serializer.write_doc(7, 1, &[4]);
                serializer.close_term()?;
                let inline = &serializer.current_term_info.inline_postings;
                assert_eq!(inline.is_some(), record_freq && mode.has_positions());
                if let Some(inline) = inline {
                    assert_eq!(&*inline.doc_ids, &[7]);
                    assert_eq!(&*inline.term_freqs, &[1]);
                    assert_eq!(&*inline.position_deltas, &[4]);
                }
                assert!(serializer.buffered_position_deltas.is_empty());
                serializer.close()?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_invalid_inline_cutoff_writes_nothing() {
        let mut terms = CountingWriter::wrap(Vec::new());
        let mut postings = CountingWriter::wrap(Vec::new());
        let mut positions = CountingWriter::wrap(Vec::new());
        let error = FieldSerializer::create(
            IndexRecordOption::WithFreqsAndPositions,
            0,
            &mut terms,
            &mut postings,
            &mut positions,
            None,
            33,
        )
        .err()
        .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(terms.written_bytes(), 0);
        assert_eq!(postings.written_bytes(), 0);
        assert_eq!(positions.written_bytes(), 0);

        let mut index = crate::Index::create_in_ram(Schema::builder().build());
        index.settings_mut().max_inline_docs = 33;
        let files_before = index.directory().list_managed_files();
        assert!(InvertedIndexSerializer::open(&index.new_segment()).is_err());
        assert_eq!(index.directory().list_managed_files(), files_before);
    }
}
