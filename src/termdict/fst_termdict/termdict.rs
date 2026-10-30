use std::io::{self, Write};
use std::sync::Arc;

use common::{BinarySerializable, CountingWriter, HasLen};
use once_cell::sync::Lazy;
use tantivy_fst::raw::Fst;
use tantivy_fst::Automaton;

use super::inline_postings::invalid;
use super::term_info_store::{TermInfoStore, TermInfoStoreWriter};
use super::{TermStreamer, TermStreamerBuilder};
use crate::directory::{FileSlice, OwnedBytes};
use crate::postings::TermInfo;
use crate::termdict::TermOrdinal;

fn convert_fst_error(e: tantivy_fst::Error) -> io::Error {
    io::Error::other(e)
}

const FST_VERSION: u32 = 1;
const FST_INLINE_VERSION: u32 = 2;

/// Builder for the new term dictionary.
///
/// Inserting must be done in the order of the `keys`.
pub struct TermDictionaryBuilder<W> {
    fst_builder: tantivy_fst::MapBuilder<W>,
    term_info_store_writer: TermInfoStoreWriter,
    term_ord: u64,
    max_inline_docs: u32,
}

impl<W> TermDictionaryBuilder<W>
where
    W: Write,
{
    /// Creates a new `TermDictionaryBuilder`
    pub fn create(w: W) -> io::Result<Self> {
        Self::create_with_max_inline_docs(w, 0)
    }

    pub fn create_with_max_inline_docs(w: W, max_inline_docs: u32) -> io::Result<Self> {
        crate::termdict::validate_max_inline_docs(max_inline_docs)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        let fst_builder = tantivy_fst::MapBuilder::new(w).map_err(convert_fst_error)?;
        Ok(TermDictionaryBuilder {
            fst_builder,
            term_info_store_writer: TermInfoStoreWriter::new(max_inline_docs != 0),
            term_ord: 0,
            max_inline_docs,
        })
    }

    /// Inserts a `(key, value)` pair in the term dictionary.
    ///
    /// *Keys have to be inserted in order.*
    pub fn insert<K: AsRef<[u8]>>(&mut self, key_ref: K, value: &TermInfo) -> io::Result<()> {
        let key = key_ref.as_ref();
        self.insert_key(key)?;
        self.insert_value(value)?;
        Ok(())
    }

    /// # Warning
    /// Horribly dangerous internal API
    ///
    /// If used, it must be used by systematically alternating calls
    /// to insert_key and insert_value.
    ///
    /// Prefer using `.insert(key, value)`
    pub fn insert_key(&mut self, key: &[u8]) -> io::Result<()> {
        self.fst_builder
            .insert(key, self.term_ord)
            .map_err(convert_fst_error)?;
        self.term_ord += 1;
        Ok(())
    }

    /// # Warning
    ///
    /// Horribly dangerous internal API. See `.insert_key(...)`.
    pub fn insert_value(&mut self, term_info: &TermInfo) -> io::Result<()> {
        self.term_info_store_writer.write_term_info(term_info)
    }

    /// Finalize writing the builder, and returns the underlying
    /// `Write` object.
    pub fn finish(mut self) -> io::Result<W> {
        let mut file = self.fst_builder.into_inner().map_err(convert_fst_error)?;
        {
            let mut counting_writer = CountingWriter::wrap(&mut file);
            self.term_info_store_writer
                .serialize(&mut counting_writer)?;
            let version = if self.max_inline_docs == 0 {
                FST_VERSION
            } else {
                FST_INLINE_VERSION
            };
            let footer_size = counting_writer.written_bytes();
            footer_size.serialize(&mut counting_writer)?;
            version.serialize(&mut counting_writer)?;
        }
        Ok(file)
    }
}

fn open_fst_index(fst_file: FileSlice) -> io::Result<tantivy_fst::Map<OwnedBytes>> {
    let bytes = fst_file.read_bytes()?;
    let fst = Fst::new(bytes).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Fst data is corrupted: {err:?}"),
        )
    })?;
    Ok(tantivy_fst::Map::from(fst))
}

static EMPTY_TERM_DICT_FILE: Lazy<FileSlice> = Lazy::new(|| {
    let term_dictionary_data: Vec<u8> = TermDictionaryBuilder::create(Vec::<u8>::new())
        .expect("Creating a TermDictionaryBuilder in a Vec<u8> should never fail")
        .finish()
        .expect("Writing in a Vec<u8> should never fail");
    FileSlice::from(term_dictionary_data)
});

/// The term dictionary contains all of the terms in
/// `tantivy index` in a sorted manner.
///
/// FST values are sorted term ordinals indexing `TermInfoStore`.
#[derive(Clone)]
pub struct TermDictionary {
    fst_index: Arc<tantivy_fst::Map<OwnedBytes>>,
    term_info_store: TermInfoStore,
}

impl TermDictionary {
    /// Opens a `TermDictionary`.
    pub fn open(file: FileSlice) -> io::Result<Self> {
        if file.len() < 12 {
            return Err(invalid("missing FST footer"));
        }
        let (main_slice, footer_len_slice) = file.split_from_end(12);
        let mut footer_len_bytes = footer_len_slice.read_bytes()?;
        let footer_size = u64::deserialize(&mut footer_len_bytes)?;
        let version = u32::deserialize(&mut footer_len_bytes)?;
        if version != FST_VERSION && version != FST_INLINE_VERSION {
            return Err(io::Error::other(format!(
                "Unsupported fst version, found {version}",
            )));
        }
        if footer_size > main_slice.len() as u64 {
            return Err(invalid("invalid FST footer size"));
        }

        let (fst_file_slice, values_file_slice) = main_slice.split_from_end(footer_size as usize);
        let fst_index = open_fst_index(fst_file_slice)?;
        let term_info_store =
            TermInfoStore::open(values_file_slice, version == FST_INLINE_VERSION)?;
        Ok(TermDictionary {
            fst_index: Arc::new(fst_index),
            term_info_store,
        })
    }

    /// Creates an empty term dictionary which contains no terms.
    pub fn empty() -> Self {
        TermDictionary::open(EMPTY_TERM_DICT_FILE.clone()).unwrap()
    }

    /// Returns the number of terms in the dictionary.
    /// Term ordinals range from 0 to `num_terms() - 1`.
    pub fn num_terms(&self) -> usize {
        self.term_info_store.num_terms()
    }

    /// Returns the ordinal associated with a given term.
    pub fn term_ord<K: AsRef<[u8]>>(&self, key: K) -> io::Result<Option<TermOrdinal>> {
        Ok(self.fst_index.get(key))
    }

    /// Stores the term associated with a given term ordinal in
    /// a `bytes` buffer.
    ///
    /// Term ordinals are defined as the position of the term in
    /// the sorted list of terms.
    ///
    /// Returns true if and only if the term has been found.
    ///
    /// Regardless of whether the term is found or not,
    /// the buffer may be modified.
    pub fn ord_to_term(&self, mut ord: TermOrdinal, bytes: &mut Vec<u8>) -> io::Result<bool> {
        bytes.clear();
        let fst = self.fst_index.as_fst();
        let mut node = fst.root();
        while ord != 0 || !node.is_final() {
            if let Some(transition) = node
                .transitions()
                .take_while(|transition| transition.out.value() <= ord)
                .last()
            {
                ord -= transition.out.value();
                bytes.push(transition.inp);
                let new_node_addr = transition.addr;
                node = fst.node(new_node_addr);
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Returns the number of terms in the dictionary.
    pub fn term_info_from_ord(&self, term_ord: TermOrdinal) -> TermInfo {
        self.term_info_store
            .get(term_ord)
            .expect("Corrupted term-info record")
    }

    /// Lookups the value corresponding to the key.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> io::Result<Option<TermInfo>> {
        self.fst_index
            .get(key)
            .map(|ordinal| self.term_info_store.get(ordinal))
            .transpose()
    }

    /// Returns a range builder, to stream all of the terms
    /// within an interval.
    pub fn range(&self) -> TermStreamerBuilder<'_> {
        TermStreamerBuilder::new(self, self.fst_index.range())
    }

    /// A stream of all the sorted terms.
    pub fn stream(&self) -> io::Result<TermStreamer<'_>> {
        self.range().into_stream()
    }

    /// Returns a search builder, to stream all of the terms
    /// within the Automaton
    pub fn search<'a, A: Automaton + 'a>(&'a self, automaton: A) -> TermStreamerBuilder<'a, A> {
        let stream_builder = self.fst_index.search(automaton);
        TermStreamerBuilder::<A>::new(self, stream_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::super::inline_postings;
    use super::*;
    use crate::postings::LoadedPostings;

    #[test]
    fn test_inline_postings_dictionary_blocks_and_streams() -> io::Result<()> {
        for maximum in [0, 1, 2, 3, 4, 8, 16, 17, 32] {
            let mut builder =
                TermDictionaryBuilder::create_with_max_inline_docs(Vec::new(), maximum)?;
            let mut terms = Vec::new();
            let mut offset = 0;
            // Cross bitmap and metadata block boundaries.
            for ordinal in 0..800 {
                let inline = maximum != 0 && ordinal % 3 != 2;
                let count = if inline { 1 } else { 40 };
                let end = offset + if inline { 0 } else { 100 };
                let info = TermInfo {
                    doc_freq: count,
                    postings_range: if inline { 0..0 } else { offset..end },
                    positions_range: if inline { 0..0 } else { offset * 2..end * 2 },
                    inline_postings: inline.then(|| {
                        Arc::new(LoadedPostings {
                            doc_ids: Box::new([799 - ordinal]),
                            term_freqs: Box::new([1]),
                            position_deltas: Box::new([ordinal * 3]),
                        })
                    }),
                };
                let key = format!("term{ordinal:04}");
                if ordinal % 2 == 0 {
                    builder.insert(&key, &info)?;
                } else {
                    builder.insert_key(key.as_bytes())?;
                    builder.insert_value(&info)?;
                }
                terms.push((key, info));
                offset = end;
            }
            let bytes = builder.finish()?;
            let version = u32::from_le_bytes(bytes[bytes.len() - 4..].try_into().unwrap());
            assert_eq!(
                version,
                if maximum == 0 {
                    FST_VERSION
                } else {
                    FST_INLINE_VERSION
                }
            );
            let mut ordinal_builder = TermDictionaryBuilder::create(Vec::new())?;
            for (key, _) in &terms {
                ordinal_builder.insert(key, &TermInfo::default())?;
            }
            let ordinal_bytes = ordinal_builder.finish()?;
            let fst_length = |data: &[u8]| {
                data.len()
                    - 12
                    - u64::from_le_bytes(data[data.len() - 12..data.len() - 4].try_into().unwrap())
                        as usize
            };
            assert_eq!(
                &bytes[..fst_length(&bytes)],
                &ordinal_bytes[..fst_length(&ordinal_bytes)]
            );
            let dictionary = TermDictionary::open(FileSlice::from(bytes))?;
            assert_eq!(dictionary.num_terms(), terms.len());
            for (ordinal, (key, info)) in terms.iter().enumerate() {
                assert_eq!(dictionary.get(key)?.as_ref(), Some(info));
                assert_eq!(dictionary.term_info_from_ord(ordinal as u64), *info);
                assert_eq!(dictionary.term_ord(key)?, Some(ordinal as u64));
                let mut decoded_key = Vec::new();
                assert!(dictionary.ord_to_term(ordinal as u64, &mut decoded_key)?);
                assert_eq!(decoded_key, key.as_bytes());
            }
            let clone = dictionary.clone();
            let mut stream = clone.stream()?;
            for (key, info) in &terms {
                let (actual_key, actual_info) = stream.next().unwrap();
                assert_eq!(actual_key, key.as_bytes());
                assert_eq!(actual_info, info);
            }
            assert!(stream.next().is_none());
            let mut stream = dictionary
                .range()
                .ge(b"term0255")
                .lt(b"term0258")
                .into_stream()?;
            for (key, info) in &terms[255..258] {
                assert_eq!(stream.next(), Some((key.as_bytes(), info)));
            }
            assert!(stream.next().is_none());
            let regex = tantivy_fst::Regex::new("term025[567]").unwrap();
            let mut stream = dictionary.search(regex).into_stream()?;
            for (key, info) in &terms[255..258] {
                assert_eq!(stream.next(), Some((key.as_bytes(), info)));
            }
            assert!(stream.next().is_none());
        }
        Ok(())
    }

    #[test]
    fn test_inline_postings_prefixes_and_reverse_streams() -> io::Result<()> {
        let keys: &[&[u8]] = &[b"", b"a", b"ab", b"abc", b"b"];
        let mut builder = TermDictionaryBuilder::create_with_max_inline_docs(Vec::new(), 1)?;
        for (ordinal, key) in keys.iter().enumerate() {
            let info = inline_postings::term_info(4 - ordinal as u32, u32::MAX - ordinal as u32)?;
            builder.insert(key, &info)?;
        }
        let dictionary = TermDictionary::open(FileSlice::from(builder.finish()?))?;
        assert_eq!(dictionary.term_info_store.num_terms(), keys.len());
        assert_eq!(dictionary.num_terms(), keys.len());
        for backward in [false, true] {
            let mut builder = dictionary.range().ge(b"a").le(b"abc").without_keys();
            if backward {
                builder = builder.backward();
            }
            let mut stream = builder.into_stream()?;
            let ordinals = if backward {
                vec![3, 2, 1]
            } else {
                vec![1, 2, 3]
            };
            for ordinal in ordinals {
                assert!(stream.advance());
                assert_eq!(stream.term_ord(), ordinal as u64);
                assert_eq!(stream.value(), &dictionary.get(keys[ordinal])?.unwrap());
            }
            assert!(!stream.advance());
        }
        let mut stream = dictionary.range().backward().into_stream()?;
        for (ordinal, key) in keys.iter().enumerate().rev() {
            assert!(stream.advance());
            assert_eq!(stream.key(), *key);
            assert_eq!(stream.term_ord(), ordinal as u64);
            assert_eq!(stream.value(), &dictionary.get(key)?.unwrap());
        }
        assert!(!stream.advance());
        assert_eq!(dictionary.term_ord(b"missing")?, None);
        assert!(dictionary.get(b"missing")?.is_none());
        assert!(!dictionary.ord_to_term(5, &mut Vec::new())?);
        Ok(())
    }

    #[test]
    fn test_inline_postings_dictionary_versions_and_footers() -> io::Result<()> {
        for maximum in [0, 8, 32] {
            let bytes = TermDictionaryBuilder::create_with_max_inline_docs(Vec::new(), maximum)?
                .finish()?;
            let dictionary = TermDictionary::open(FileSlice::from(bytes))?;
            assert_eq!(dictionary.num_terms(), 0);
            assert!(dictionary.stream()?.next().is_none());
        }
        assert!(TermDictionaryBuilder::create_with_max_inline_docs(Vec::new(), 33).is_err());
        let term = TermInfo {
            doc_freq: 1,
            inline_postings: Some(Arc::new(LoadedPostings {
                doc_ids: Box::new([42]),
                term_freqs: Box::new([1]),
                position_deltas: Box::new([17]),
            })),
            ..TermInfo::default()
        };
        assert!(TermDictionaryBuilder::create(Vec::new())?
            .insert(b"term", &term)
            .is_err());
        let mut builder = TermDictionaryBuilder::create_with_max_inline_docs(Vec::new(), 8)?;
        builder.insert(b"term", &term)?;
        let bytes = builder.finish()?;
        assert_eq!(
            TermDictionary::open(FileSlice::from(bytes.clone()))?.get(b"term")?,
            Some(term)
        );
        let len = bytes.len();
        for range in [len - 12..len - 4, len - 4..len] {
            let mut invalid = bytes.clone();
            invalid[range].fill(255);
            assert!(TermDictionary::open(FileSlice::from(invalid)).is_err());
        }
        for version in [0, FST_INLINE_VERSION + 1, u32::MAX] {
            let mut unsupported = bytes.clone();
            unsupported[len - 4..].copy_from_slice(&version.to_le_bytes());
            assert!(TermDictionary::open(FileSlice::from(unsupported)).is_err());
        }
        for len in 0..12 {
            assert!(TermDictionary::open(FileSlice::from(vec![0; len])).is_err());
        }
        Ok(())
    }
}
