use std::cmp;
use std::io::{self, Read, Write};

use byteorder::{ByteOrder, LittleEndian};
use common::{BinarySerializable, FixedSize, HasLen};
use tantivy_bitpacker::{compute_num_bits, BitPacker};

use super::inline_postings::{self, invalid};
use crate::directory::{FileSlice, OwnedBytes};
use crate::postings::TermInfo;
use crate::termdict::TermOrdinal;

const BLOCK_LEN: usize = 256;

#[derive(Debug, Eq, PartialEq, Default)]
struct TermInfoBlockMeta {
    offset: u64,
    ref_term_info: TermInfo,
    doc_freq_nbits: u8,
    postings_offset_nbits: u8,
    positions_offset_nbits: u8,
}

impl BinarySerializable for TermInfoBlockMeta {
    fn serialize<W: Write + ?Sized>(&self, write: &mut W) -> io::Result<()> {
        self.offset.serialize(write)?;
        self.ref_term_info.serialize(write)?;
        write.write_all(&[
            self.doc_freq_nbits,
            self.postings_offset_nbits,
            self.positions_offset_nbits,
        ])?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self> {
        let offset = u64::deserialize(reader)?;
        let ref_term_info = TermInfo::deserialize(reader)?;
        let mut buffer = [0u8; 3];
        reader.read_exact(&mut buffer)?;
        Ok(TermInfoBlockMeta {
            offset,
            ref_term_info,
            doc_freq_nbits: buffer[0],
            postings_offset_nbits: buffer[1],
            positions_offset_nbits: buffer[2],
        })
    }
}

impl FixedSize for TermInfoBlockMeta {
    const SIZE_IN_BYTES: usize = u64::SIZE_IN_BYTES + TermInfo::SIZE_IN_BYTES + 3;
}

impl TermInfoBlockMeta {
    fn num_bits(&self) -> u8 {
        self.doc_freq_nbits + self.postings_offset_nbits + self.positions_offset_nbits
    }

    // Here inner_offset is the offset within the block, WITHOUT the first term_info.
    // In other word, term_info #1,#2,#3 gets inner_offset 0,1,2... While term_info #0
    // is encoded without bitpacking.
    fn deserialize_term_info(&self, data: &[u8], inner_offset: usize) -> TermInfo {
        assert!(inner_offset < BLOCK_LEN - 1);
        let num_bits = self.num_bits() as usize;

        let posting_start_addr = num_bits * inner_offset;
        // the posting_start is the posting_start of the next term info.
        let posting_end_addr = posting_start_addr + num_bits;
        let positions_start_addr = posting_start_addr + self.postings_offset_nbits as usize;
        // the position_end is the positions_start of the next term info.
        let positions_end_addr = positions_start_addr + num_bits;

        let doc_freq_addr = positions_start_addr + self.positions_offset_nbits as usize;

        let postings_start_offset = self.ref_term_info.postings_range.start
            + extract_bits(data, posting_start_addr, self.postings_offset_nbits) as usize;
        let postings_end_offset = self.ref_term_info.postings_range.start
            + extract_bits(data, posting_end_addr, self.postings_offset_nbits) as usize;

        let positions_start_offset = self.ref_term_info.positions_range.start
            + extract_bits(data, positions_start_addr, self.positions_offset_nbits) as usize;
        let positions_end_offset = self.ref_term_info.positions_range.start
            + extract_bits(data, positions_end_addr, self.positions_offset_nbits) as usize;

        let doc_freq = extract_bits(data, doc_freq_addr, self.doc_freq_nbits) as u32;

        TermInfo {
            doc_freq,
            postings_range: postings_start_offset..postings_end_offset,
            positions_range: positions_start_offset..positions_end_offset,
            inline_postings: None,
        }
    }
}

// Each bitmap bit selects one of two independently bitpacked groups.
#[derive(Default)]
struct InlineBlockMeta {
    bitmap: [u64; 4],
    doc_nbits: u8,
    position_nbits: u8,
    min_doc: u32,
    min_position: u32,
}

impl InlineBlockMeta {
    const SIZE: usize = 42;

    fn rank(&self, slot: usize) -> usize {
        let word = slot / 64;
        let bits = slot % 64;
        let preceding: u32 = self.bitmap[..word].iter().map(|v| v.count_ones()).sum();
        preceding as usize
            + if bits == 0 {
                0
            } else {
                (self.bitmap[word] & ((1u64 << bits) - 1)).count_ones() as usize
            }
    }

    fn contains(&self, slot: usize) -> bool {
        self.bitmap[slot / 64] & (1 << (slot % 64)) != 0
    }

    fn serialize(&self, output: &mut Vec<u8>) -> io::Result<()> {
        for word in self.bitmap {
            word.serialize(output)?;
        }
        output.extend_from_slice(&[self.doc_nbits, self.position_nbits]);
        self.min_doc.serialize(output)?;
        self.min_position.serialize(output)?;
        Ok(())
    }

    fn deserialize(input: &mut &[u8]) -> io::Result<Self> {
        let mut meta = Self::default();
        for word in &mut meta.bitmap {
            *word = u64::deserialize(input)?;
        }
        meta.doc_nbits = u8::deserialize(input)?;
        meta.position_nbits = u8::deserialize(input)?;
        meta.min_doc = u32::deserialize(input)?;
        meta.min_position = u32::deserialize(input)?;
        if meta.doc_nbits > 31 || meta.position_nbits > 32 {
            return Err(invalid("invalid inline bit widths"));
        }
        Ok(meta)
    }
}

#[derive(Clone)]
pub struct TermInfoStore {
    num_terms: usize,
    block_meta_bytes: OwnedBytes,
    term_info_bytes: OwnedBytes,
    inline: bool,
}

fn extract_bits(data: &[u8], addr_bits: usize, num_bits: u8) -> u64 {
    assert!(num_bits <= 56);
    let addr_byte = addr_bits / 8;
    let bit_shift = (addr_bits % 8) as u64;
    let val_unshifted_unmasked: u64 = if data.len() >= addr_byte + 8 {
        LittleEndian::read_u64(&data[addr_byte..][..8])
    } else {
        // the buffer is not large enough.
        // Let's copy the few remaining bytes to a 8 byte buffer
        // padded with 0s.
        let mut buf = [0u8; 8];
        let data_to_copy = &data[addr_byte..];
        let nbytes = data_to_copy.len();
        buf[..nbytes].copy_from_slice(data_to_copy);
        LittleEndian::read_u64(&buf)
    };
    let val_shifted_unmasked = val_unshifted_unmasked >> bit_shift;
    let mask = (1u64 << u64::from(num_bits)) - 1;
    val_shifted_unmasked & mask
}

impl TermInfoStore {
    pub fn open(term_info_store_file: FileSlice, inline: bool) -> io::Result<TermInfoStore> {
        if term_info_store_file.len() < 16 {
            return Err(invalid("missing term-info header"));
        }
        let (len_slice, main_slice) = term_info_store_file.split(16);
        let mut bytes = len_slice.read_bytes()?;
        let len = u64::deserialize(&mut bytes)? as usize;
        let num_terms = u64::deserialize(&mut bytes)? as usize;
        let meta_size =
            TermInfoBlockMeta::SIZE_IN_BYTES + if inline { InlineBlockMeta::SIZE } else { 0 };
        if num_terms.div_ceil(BLOCK_LEN).checked_mul(meta_size) != Some(len)
            || len > main_slice.len()
        {
            return Err(invalid("invalid term-info index size"));
        }
        let (block_meta_file, term_info_file) = main_slice.split(len);
        let term_info_bytes = term_info_file.read_bytes()?;
        Ok(TermInfoStore {
            num_terms,
            block_meta_bytes: block_meta_file.read_bytes()?,
            term_info_bytes,
            inline,
        })
    }

    pub fn get(&self, term_ord: TermOrdinal) -> io::Result<TermInfo> {
        if term_ord >= self.num_terms as u64 {
            return Err(invalid("term ordinal out of bounds"));
        }
        let block_id = term_ord as usize / BLOCK_LEN;
        let meta_size = TermInfoBlockMeta::SIZE_IN_BYTES
            + if self.inline {
                InlineBlockMeta::SIZE
            } else {
                0
            };
        let mut block_data = &self.block_meta_bytes[block_id * meta_size..];
        let meta = TermInfoBlockMeta::deserialize(&mut block_data)?;
        if meta.doc_freq_nbits > 32
            || meta.postings_offset_nbits > 56
            || meta.positions_offset_nbits > 56
        {
            return Err(invalid("invalid external bit widths"));
        }
        let slot = term_ord as usize % BLOCK_LEN;
        let block_len = (self.num_terms - block_id * BLOCK_LEN).min(BLOCK_LEN);
        let mut external_count = block_len;
        let mut external_slot = slot;
        let inline_meta = if self.inline {
            let inline = InlineBlockMeta::deserialize(&mut block_data)?;
            let count = inline.rank(block_len);
            if count != inline.rank(BLOCK_LEN) {
                return Err(invalid("inline bitmap extends past block"));
            }
            external_count -= count;
            external_slot -= inline.rank(slot);
            Some(inline)
        } else {
            None
        };
        let external_bytes = (external_count.saturating_sub(1) * meta.num_bits() as usize
            + meta.postings_offset_nbits as usize
            + meta.positions_offset_nbits as usize)
            .div_ceil(8);
        let data = self
            .term_info_bytes
            .get(meta.offset as usize..)
            .ok_or_else(|| invalid("invalid term-info block offset"))?;
        let external = data
            .get(..external_bytes)
            .ok_or_else(|| invalid("truncated external term-info block"))?;
        if let Some(inline) = inline_meta {
            if inline.contains(slot) {
                let bits = inline.doc_nbits as usize + inline.position_nbits as usize;
                let inline_bytes = ((block_len - external_count) * bits).div_ceil(8);
                let data = data
                    .get(external_bytes..external_bytes + inline_bytes)
                    .ok_or_else(|| invalid("truncated inline term-info block"))?;
                let offset = inline.rank(slot) * bits;
                let doc = inline
                    .min_doc
                    .checked_add(extract_bits(data, offset, inline.doc_nbits) as u32)
                    .ok_or_else(|| invalid("inline document ID overflow"))?;
                let position = inline
                    .min_position
                    .checked_add(extract_bits(
                        data,
                        offset + inline.doc_nbits as usize,
                        inline.position_nbits,
                    ) as u32)
                    .ok_or_else(|| invalid("inline position overflow"))?;
                return inline_postings::term_info(doc, position);
            }
        }
        if external_slot == 0 {
            Ok(meta.ref_term_info)
        } else {
            Ok(meta.deserialize_term_info(external, external_slot - 1))
        }
    }

    pub fn num_terms(&self) -> usize {
        self.num_terms
    }
}

pub struct TermInfoStoreWriter {
    buffer_block_metas: Vec<u8>,
    buffer_term_infos: Vec<u8>,
    term_infos: Vec<TermInfo>,
    num_terms: u64,
    inline: bool,
}

fn bitpack_serialize<W: Write>(
    write: &mut W,
    bit_packer: &mut BitPacker,
    term_info_block_meta: &TermInfoBlockMeta,
    term_info: &TermInfo,
) -> io::Result<()> {
    bit_packer.write(
        term_info.postings_range.start as u64,
        term_info_block_meta.postings_offset_nbits,
        write,
    )?;
    bit_packer.write(
        term_info.positions_range.start as u64,
        term_info_block_meta.positions_offset_nbits,
        write,
    )?;
    bit_packer.write(
        u64::from(term_info.doc_freq),
        term_info_block_meta.doc_freq_nbits,
        write,
    )?;
    Ok(())
}

impl TermInfoStoreWriter {
    pub fn new(inline: bool) -> TermInfoStoreWriter {
        TermInfoStoreWriter {
            buffer_block_metas: Vec::new(),
            buffer_term_infos: Vec::new(),
            term_infos: Vec::with_capacity(BLOCK_LEN),
            num_terms: 0u64,
            inline,
        }
    }

    fn flush_block(&mut self) -> io::Result<()> {
        let mut inline_meta = InlineBlockMeta::default();
        let mut inline_values = Vec::new();
        if self.inline {
            for (slot, term) in self.term_infos.iter().enumerate() {
                if term.inline_postings.is_some() {
                    let (doc, position) = inline_postings::values(term)?;
                    inline_meta.bitmap[slot / 64] |= 1 << (slot % 64);
                    inline_values.push((doc, position));
                }
            }
            inline_meta.min_doc = inline_values.iter().map(|&(doc, _)| doc).min().unwrap_or(0);
            inline_meta.min_position = inline_values.iter().map(|&(_, pos)| pos).min().unwrap_or(0);
            for (doc, position) in &mut inline_values {
                *doc -= inline_meta.min_doc;
                *position -= inline_meta.min_position;
                inline_meta.doc_nbits =
                    inline_meta.doc_nbits.max(compute_num_bits(u64::from(*doc)));
                inline_meta.position_nbits = inline_meta
                    .position_nbits
                    .max(compute_num_bits(u64::from(*position)));
            }
            self.term_infos
                .retain(|term| term.inline_postings.is_none());
        }
        let mut bit_packer = BitPacker::new();
        let ref_term_info = self.term_infos.first().cloned().unwrap_or_default();
        let last_term_info = self.term_infos.last().cloned().unwrap_or_default();
        let postings_end_offset =
            last_term_info.postings_range.end - ref_term_info.postings_range.start;
        let positions_end_offset =
            last_term_info.positions_range.end - ref_term_info.positions_range.start;
        for term_info in self.term_infos.iter_mut().skip(1) {
            term_info.postings_range.start -= ref_term_info.postings_range.start;
            term_info.positions_range.start -= ref_term_info.positions_range.start;
        }

        let mut max_doc_freq: u32 = 0u32;

        for term_info in self.term_infos.iter().skip(1) {
            max_doc_freq = cmp::max(max_doc_freq, term_info.doc_freq);
        }

        let max_doc_freq_nbits: u8 = compute_num_bits(u64::from(max_doc_freq));
        let max_postings_offset_nbits = compute_num_bits(postings_end_offset as u64);
        let max_positions_offset_nbits = compute_num_bits(positions_end_offset as u64);

        let term_info_block_meta = TermInfoBlockMeta {
            offset: self.buffer_term_infos.len() as u64,
            ref_term_info,
            doc_freq_nbits: max_doc_freq_nbits,
            postings_offset_nbits: max_postings_offset_nbits,
            positions_offset_nbits: max_positions_offset_nbits,
        };

        term_info_block_meta.serialize(&mut self.buffer_block_metas)?;
        if self.inline {
            inline_meta.serialize(&mut self.buffer_block_metas)?;
        }
        for term_info in self.term_infos.iter().skip(1) {
            bitpack_serialize(
                &mut self.buffer_term_infos,
                &mut bit_packer,
                &term_info_block_meta,
                term_info,
            )?;
        }

        // We still need to serialize the end offset for postings & positions.
        bit_packer.write(
            postings_end_offset as u64,
            term_info_block_meta.postings_offset_nbits,
            &mut self.buffer_term_infos,
        )?;
        bit_packer.write(
            positions_end_offset as u64,
            term_info_block_meta.positions_offset_nbits,
            &mut self.buffer_term_infos,
        )?;

        // Block need end up at the end of a byte.
        bit_packer.flush(&mut self.buffer_term_infos)?;
        for (doc, position) in inline_values {
            bit_packer.write(
                u64::from(doc),
                inline_meta.doc_nbits,
                &mut self.buffer_term_infos,
            )?;
            bit_packer.write(
                u64::from(position),
                inline_meta.position_nbits,
                &mut self.buffer_term_infos,
            )?;
        }
        bit_packer.flush(&mut self.buffer_term_infos)?;
        self.term_infos.clear();

        Ok(())
    }

    pub fn write_term_info(&mut self, term_info: &TermInfo) -> io::Result<()> {
        if term_info.inline_postings.is_some() {
            if !self.inline {
                return Err(invalid("inline postings are disabled"));
            }
            inline_postings::values(term_info)?;
        }
        self.num_terms += 1u64;
        self.term_infos.push(term_info.clone());
        if self.term_infos.len() >= BLOCK_LEN {
            self.flush_block()?;
        }
        Ok(())
    }

    pub fn serialize<W: io::Write + ?Sized>(&mut self, write: &mut W) -> io::Result<()> {
        if !self.term_infos.is_empty() {
            self.flush_block()?;
        }
        let len = self.buffer_block_metas.len() as u64;
        len.serialize(write)?;
        self.num_terms.serialize(write)?;
        write.write_all(&self.buffer_block_metas)?;
        write.write_all(&self.buffer_term_infos)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use common::BinarySerializable;
    use tantivy_bitpacker::{compute_num_bits, BitPacker};

    use super::{extract_bits, TermInfoBlockMeta, TermInfoStore, TermInfoStoreWriter};
    use crate::directory::FileSlice;
    use crate::postings::TermInfo;

    #[test]
    fn test_inline_groups_and_corruption() -> crate::Result<()> {
        use super::{inline_postings, InlineBlockMeta};
        use crate::TERMINATED;
        use common::FixedSize;

        for pattern in 0..4 {
            let mut writer = TermInfoStoreWriter::new(true);
            let mut expected = Vec::new();
            let mut offset = 0;
            // Include full/partial blocks, every bitmap word boundary, and empty groups.
            for slot in 0..777 {
                let inline = match pattern {
                    0 => false,
                    1 => true,
                    2 => slot % 2 == 0,
                    _ => slot % 256 == 0 || slot % 256 == 255,
                };
                let term = if inline {
                    let (doc, position) = if slot % 3 == 0 {
                        (0, 0)
                    } else {
                        (TERMINATED - 1, u32::MAX)
                    };
                    inline_postings::term_info(doc, position)?
                } else {
                    let term = TermInfo {
                        doc_freq: 2,
                        postings_range: offset..offset + 3,
                        positions_range: offset * 2..(offset + 3) * 2,
                        inline_postings: None,
                    };
                    offset += 3;
                    term
                };
                writer.write_term_info(&term)?;
                expected.push(term);
            }
            let mut bytes = Vec::new();
            writer.serialize(&mut bytes)?;
            let store = TermInfoStore::open(FileSlice::from(bytes.clone()), true)?;
            for (ordinal, term) in expected.iter().enumerate() {
                assert_eq!(&store.get(ordinal as u64)?, term);
            }
            assert!(store.get(777).is_err());
            let mut header = &bytes[16..];
            let external = TermInfoBlockMeta::deserialize(&mut header)?;
            let inline = InlineBlockMeta::deserialize(&mut header)?;
            assert!(external.postings_offset_nbits <= 10);
            assert!(external.positions_offset_nbits <= 11);
            if pattern == 1 || pattern == 2 {
                assert_eq!((inline.doc_nbits, inline.position_nbits), (31, 32));
            }
            let mut truncated = bytes.clone();
            truncated.pop();
            let store = TermInfoStore::open(FileSlice::from(truncated), true)?;
            assert!(store.get(776).is_err());

            let mut invalid_width = bytes.clone();
            invalid_width[16 + TermInfoBlockMeta::SIZE_IN_BYTES + 32] = 32;
            assert!(TermInfoStore::open(FileSlice::from(invalid_width), true)?
                .get(0)
                .is_err());
            // The final block has nine terms. Bit 63 must remain clear.
            let last_header = 16 + 3 * (TermInfoBlockMeta::SIZE_IN_BYTES + InlineBlockMeta::SIZE);
            bytes[last_header + TermInfoBlockMeta::SIZE_IN_BYTES + 7] |= 128;
            assert!(TermInfoStore::open(FileSlice::from(bytes), true)?
                .get(768)
                .is_err());
        }
        // Every singleton needs zero payload bits, even at the largest values.
        for (doc, position) in [(0, 0), (42, 17), (TERMINATED - 1, u32::MAX)] {
            let mut writer = TermInfoStoreWriter::new(true);
            writer.write_term_info(&inline_postings::term_info(doc, position)?)?;
            let mut bytes = Vec::new();
            writer.serialize(&mut bytes)?;
            assert_eq!(
                bytes.len(),
                16 + TermInfoBlockMeta::SIZE_IN_BYTES + InlineBlockMeta::SIZE
            );
            assert_eq!(
                TermInfoStore::open(FileSlice::from(bytes), true)?.get(0)?,
                inline_postings::term_info(doc, position)?
            );
        }
        for length in 0..16 {
            assert!(TermInfoStore::open(FileSlice::from(vec![0; length]), true).is_err());
        }
        Ok(())
    }

    #[test]
    fn test_inline_reference_values() -> crate::Result<()> {
        use super::{inline_postings, InlineBlockMeta};
        use crate::TERMINATED;
        use common::FixedSize;

        let cases = [
            (
                vec![(1005, 5000), (1000, 5007), (1007, 5004)],
                (1000, 5000, 3, 3),
            ),
            (
                vec![(TERMINATED - 1, u32::MAX - 1), (TERMINATED - 8, u32::MAX)],
                (TERMINATED - 8, u32::MAX - 1, 3, 1),
            ),
            (
                vec![(0, u32::MAX), (0, u32::MAX - 2)],
                (0, u32::MAX - 2, 0, 2),
            ),
            (
                vec![(TERMINATED - 1, 0), (TERMINATED - 2, 0)],
                (TERMINATED - 2, 0, 1, 0),
            ),
            (
                vec![(TERMINATED - 1, u32::MAX); 256],
                (TERMINATED - 1, u32::MAX, 0, 0),
            ),
        ];
        for (values, expected) in cases {
            let mut writer = TermInfoStoreWriter::new(true);
            for &(doc, position) in &values {
                writer.write_term_info(&inline_postings::term_info(doc, position)?)?;
            }
            let mut bytes = Vec::new();
            writer.serialize(&mut bytes)?;
            let mut header = &bytes[16 + TermInfoBlockMeta::SIZE_IN_BYTES..];
            let meta = InlineBlockMeta::deserialize(&mut header)?;
            assert_eq!(
                (
                    meta.min_doc,
                    meta.min_position,
                    meta.doc_nbits,
                    meta.position_nbits
                ),
                expected
            );
            assert_eq!(
                bytes.len(),
                16 + TermInfoBlockMeta::SIZE_IN_BYTES
                    + InlineBlockMeta::SIZE
                    + (values.len() * (meta.doc_nbits + meta.position_nbits) as usize).div_ceil(8)
            );
            let store = TermInfoStore::open(FileSlice::from(bytes), true)?;
            for (ordinal, (doc, position)) in values.into_iter().enumerate() {
                assert_eq!(
                    store.get(ordinal as u64)?,
                    inline_postings::term_info(doc, position)?
                );
            }
        }

        let mut writer = TermInfoStoreWriter::new(true);
        for (doc, position) in [(10, 20), (11, 21)] {
            writer.write_term_info(&inline_postings::term_info(doc, position)?)?;
        }
        let mut bytes = Vec::new();
        writer.serialize(&mut bytes)?;
        let min_doc_offset = 16 + TermInfoBlockMeta::SIZE_IN_BYTES + 34;
        for (offset, base) in [
            (min_doc_offset, u32::MAX),
            (min_doc_offset + 4, u32::MAX),
            (min_doc_offset, TERMINATED - 1),
        ] {
            let mut corrupt = bytes.clone();
            corrupt[offset..offset + 4].copy_from_slice(&base.to_le_bytes());
            let store = TermInfoStore::open(FileSlice::from(corrupt), true)?;
            assert_eq!(
                store.get(1).unwrap_err().kind(),
                std::io::ErrorKind::InvalidData
            );
        }
        Ok(())
    }

    #[test]
    fn test_term_info_block() {
        crate::tests::fixed_size_test::<TermInfoBlockMeta>();
    }

    #[test]
    fn test_bitpacked() {
        let mut buffer = Vec::new();
        let mut bitpack = BitPacker::new();
        bitpack.write(321u64, 9, &mut buffer).unwrap();
        assert_eq!(compute_num_bits(321u64), 9);
        bitpack.write(2u64, 2, &mut buffer).unwrap();
        assert_eq!(compute_num_bits(2u64), 2);
        bitpack.write(51, 6, &mut buffer).unwrap();
        assert_eq!(compute_num_bits(51), 6);
        bitpack.close(&mut buffer).unwrap();
        assert_eq!(buffer.len(), 3);
        assert_eq!(extract_bits(&buffer[..], 0, 9), 321u64);
        assert_eq!(extract_bits(&buffer[..], 9, 2), 2u64);
        assert_eq!(extract_bits(&buffer[..], 11, 6), 51u64);
    }

    #[test]
    fn test_term_info_block_meta_serialization() {
        let term_info_block_meta = TermInfoBlockMeta {
            offset: 2009u64,
            ref_term_info: TermInfo {
                doc_freq: 512,
                postings_range: 51..57,
                positions_range: 110..134,
                inline_postings: None,
            },
            doc_freq_nbits: 10,
            postings_offset_nbits: 5,
            positions_offset_nbits: 8,
        };
        let mut buffer: Vec<u8> = Vec::new();
        term_info_block_meta.serialize(&mut buffer).unwrap();
        let mut cursor: &[u8] = &buffer[..];
        let term_info_block_meta_serde = TermInfoBlockMeta::deserialize(&mut cursor).unwrap();
        assert_eq!(term_info_block_meta_serde, term_info_block_meta);
    }

    #[test]
    fn test_pack() -> crate::Result<()> {
        let mut store_writer = TermInfoStoreWriter::new(false);
        let mut term_infos = vec![];
        let offset = |i| i * 13 + i * i;
        for i in 0usize..1000usize {
            let term_info = TermInfo {
                doc_freq: i as u32,
                postings_range: offset(i)..offset(i + 1),
                positions_range: offset(i) * 3..offset(i + 1) * 3,
                inline_postings: None,
            };
            store_writer.write_term_info(&term_info)?;
            term_infos.push(term_info);
        }
        let mut buffer = Vec::new();
        store_writer.serialize(&mut buffer)?;
        let term_info_store = TermInfoStore::open(FileSlice::from(buffer), false)?;
        for i in 0..1000 {
            assert_eq!(
                term_info_store.get(i as u64)?,
                term_infos[i],
                "term info {i}"
            );
        }
        Ok(())
    }
}
