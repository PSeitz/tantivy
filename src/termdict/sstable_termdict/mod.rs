use std::io;

mod inline_postings;
mod merger;

use std::iter::ExactSizeIterator;

use common::VInt;
use sstable::value::{ValueReader, ValueWriter};
use sstable::SSTable;
use tantivy_fst::automaton::AlwaysMatch;

use self::inline_postings::{invalid, read_usize};
pub use self::merger::TermMerger;
use crate::postings::TermInfo;

/// The term dictionary keeps terms sorted and stores their metadata in SSTable blocks.
pub type TermDictionary = sstable::Dictionary<TermSSTable>;

/// Builder for a term dictionary.
pub type TermDictionaryBuilder<W> = sstable::Writer<W, TermInfoBlock>;

/// `TermStreamer` acts as a cursor over a range of terms of a segment.
/// Terms are guaranteed to be sorted.
pub type TermStreamer<'a, A = AlwaysMatch, K = Vec<u8>> = sstable::Streamer<'a, TermSSTable, A, K>;

/// SSTable used to store TermInfo objects.
#[derive(Clone)]
pub struct TermSSTable;

pub type TermStreamerBuilder<'a, A = AlwaysMatch, K = Vec<u8>> =
    sstable::StreamerBuilder<'a, TermSSTable, A, K>;

impl SSTable for TermSSTable {
    type Value = TermInfo;
    type ValueReader = TermInfoBlock;
    type ValueWriter = TermInfoBlock;
}

/// Term metadata and inline postings for one dictionary block.
#[derive(Default)]
pub struct TermInfoBlock {
    term_infos: Vec<TermInfo>,
    pub(super) max_inline_docs: u32,
}

impl TermInfoBlock {
    pub(super) fn new(max_inline_docs: u32) -> io::Result<Self> {
        Ok(Self {
            term_infos: Vec::new(),
            max_inline_docs: super::validate_max_inline_docs(max_inline_docs)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?,
        })
    }

    fn read_term_infos(
        &mut self,
        data: &mut &[u8],
        num_terms: usize,
        has_inline_postings: bool,
    ) -> io::Result<usize> {
        let mut postings_start = read_usize(data)?;
        let mut positions_start = read_usize(data)?;
        let minimum_bytes = if has_inline_postings { 1 } else { 3 };
        if num_terms == 0 || num_terms > data.len() / minimum_bytes {
            return Err(invalid("invalid term count"));
        }
        let mut inline_term_count = 0;
        for _ in 0..num_terms {
            let doc_freq = u32::try_from(VInt::deserialize_u64(data)?)
                .map_err(|_| invalid("document count overflow"))?;
            let (postings_len, positions_len) = if has_inline_postings && doc_freq == 0 {
                inline_term_count += 1;
                (0, 0)
            } else {
                (read_usize(data)?, read_usize(data)?)
            };
            let postings_end = postings_start
                .checked_add(postings_len)
                .ok_or_else(|| invalid("postings offset overflow"))?;
            let positions_end = positions_start
                .checked_add(positions_len)
                .ok_or_else(|| invalid("positions offset overflow"))?;
            self.term_infos.push(TermInfo {
                doc_freq,
                postings_range: postings_start..postings_end,
                positions_range: positions_start..positions_end,
                inline_postings: None,
            });
            postings_start = postings_end;
            positions_start = positions_end;
        }
        Ok(inline_term_count)
    }
}

impl ValueReader for TermInfoBlock {
    type Value = TermInfo;

    fn configure(&mut self, max_inline_docs: u8) -> io::Result<()> {
        *self = Self::new(u32::from(max_inline_docs))?;
        Ok(())
    }

    #[inline(always)]
    fn value(&self, idx: usize) -> &TermInfo {
        &self.term_infos[idx]
    }

    fn load(&mut self, mut data: &[u8]) -> io::Result<usize> {
        let len_before = data.len();
        self.term_infos.clear();
        let num_terms = read_usize(&mut data)?;
        if num_terms != 0 {
            self.read_term_infos(&mut data, num_terms, false)?;
        } else if !data.is_empty() {
            inline_postings::read_block(&mut data, self)?;
        }
        Ok(len_before - data.len())
    }
}

impl ValueWriter for TermInfoBlock {
    type Value = TermInfo;

    fn write(&mut self, term_info: &TermInfo) {
        self.term_infos.push(term_info.clone());
    }

    fn serialize_block_with_keys(&self, keys: &[u8], buffer: &mut Vec<u8>) -> io::Result<bool> {
        if self
            .term_infos
            .iter()
            .all(|term| term.inline_postings.is_none())
        {
            return Ok(false);
        }
        inline_postings::write_block(self, keys, buffer)?;
        Ok(true)
    }

    fn serialize_block(&self, buffer: &mut Vec<u8>) {
        VInt(self.term_infos.len() as u64).serialize_into_vec(buffer);
        if self.term_infos.is_empty() {
            return;
        }
        VInt(self.term_infos[0].postings_range.start as u64).serialize_into_vec(buffer);
        VInt(self.term_infos[0].positions_range.start as u64).serialize_into_vec(buffer);
        for term_info in &self.term_infos {
            if term_info.inline_postings.is_some() {
                VInt(0).serialize_into_vec(buffer);
            } else {
                VInt(term_info.doc_freq as u64).serialize_into_vec(buffer);
                VInt(term_info.postings_range.len() as u64).serialize_into_vec(buffer);
                VInt(term_info.positions_range.len() as u64).serialize_into_vec(buffer);
            }
        }
    }

    fn clear(&mut self) {
        self.term_infos.clear();
    }
}

#[cfg(test)]
mod tests {
    use sstable::value::{ValueReader, ValueWriter};

    use super::{TermInfoBlock, VInt};
    use crate::postings::TermInfo;

    #[test]
    fn test_block_terminfos() {
        let expected = [
            (0, 17..45, 10..122),
            (10, 45..450, 122..1100),
            (129, 450..462, 1100..1302),
        ]
        .map(|(doc_freq, postings_range, positions_range)| TermInfo {
            doc_freq,
            postings_range,
            positions_range,
            inline_postings: None,
        });
        // Legacy metadata stores a count, base offsets, then document counts and range lengths.
        let mut legacy = Vec::new();
        for value in [3, 17, 10, 0, 28, 112, 10, 405, 978, 129, 12, 202] {
            VInt(value).serialize_into_vec(&mut legacy);
        }
        let mut writer = TermInfoBlock::default();
        for term in &expected {
            writer.write(term);
        }
        let mut buffer = Vec::new();
        writer.serialize_block(&mut buffer);
        assert_eq!(buffer, legacy);
        let mut reader = TermInfoBlock::default();
        assert_eq!(reader.load(&legacy).unwrap(), legacy.len());
        assert_eq!(reader.term_infos, expected);
    }
}
