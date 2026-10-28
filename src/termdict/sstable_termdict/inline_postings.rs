use std::io::{self, Read};
use std::sync::Arc;

use common::VInt;
use sstable::value::ValueWriter;
use tantivy_bitpacker::{BitPacker, BitUnpacker};

use crate::postings::LoadedPostings;

const POSITION_TAG: u32 = 1 << 31;
const VERSION: u64 = 2;

pub(super) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn read_usize(data: &mut &[u8]) -> io::Result<usize> {
    usize::try_from(VInt::deserialize_u64(data)?)
        .map_err(|_| invalid("term metadata integer overflow"))
}

fn count_and_positions_bits(max_inline_docs: u32) -> usize {
    (u32::BITS - max_inline_docs.saturating_sub(1).leading_zeros() + 1) as usize
}

fn write_lanes(values: &[u32], output: &mut Vec<u8>) -> io::Result<()> {
    if values.is_empty() {
        return Ok(());
    }
    for shift in [0, 8, 16, 24] {
        let lane: Vec<u8> = values.iter().map(|value| (value >> shift) as u8).collect();
        let compressed_lane = zstd::bulk::compress(&lane, 3)?;
        VInt(compressed_lane.len() as u64).serialize_into_vec(output);
        output.extend_from_slice(&compressed_lane);
    }
    Ok(())
}

fn read_lanes(data: &mut &[u8], count: usize) -> io::Result<Vec<u32>> {
    let mut values = Vec::new();
    if count == 0 {
        return Ok(values);
    }
    for shift in [0, 8, 16, 24] {
        let len = read_usize(data)?;
        let compressed_lane = data
            .get(..len)
            .ok_or_else(|| invalid("truncated byte lane"))?;
        *data = &data[len..];
        // Decompress before allocating the integer array from an untrusted count.
        let mut lane = Vec::new();
        zstd::stream::read::Decoder::new(compressed_lane)?
            .take(count as u64 + 1)
            .read_to_end(&mut lane)?;
        if lane.len() != count {
            return Err(invalid("byte lane length mismatch"));
        }
        values.resize(count, 0);
        for (value, byte) in values.iter_mut().zip(lane) {
            *value |= u32::from(byte) << shift;
        }
    }
    Ok(values)
}

pub(super) fn write_block(
    block: &super::TermInfoBlock,
    keys: &[u8],
    output: &mut Vec<u8>,
) -> io::Result<()> {
    let terms = &block.term_infos;
    let max_inline_docs = block.max_inline_docs;
    let count_and_positions_bits = count_and_positions_bits(max_inline_docs);
    let positions_flag = 1u8 << (count_and_positions_bits - 1);
    let mut metadata = Vec::new();
    // Zero marks inline postings. Legacy blocks with terms start with a positive count.
    VInt(0).serialize_into_vec(&mut metadata);
    VInt(VERSION).serialize_into_vec(&mut metadata);
    block.serialize_block(&mut metadata);
    let mut packer = BitPacker::new();
    let mut doc_deltas = Vec::new();
    let mut frequency_codes = Vec::new();
    let mut separate_position_deltas = Vec::new();
    for term in terms {
        if let Some(postings) = &term.inline_postings {
            if !(1..=max_inline_docs).contains(&term.doc_freq) {
                return Err(invalid(
                    "inline postings document count exceeds dictionary limit",
                ));
            }
            let has_positions = !postings.position_deltas.is_empty();
            let count_and_positions =
                (term.doc_freq as u8 - 1) | (u8::from(has_positions) * positions_flag);
            packer.write(
                u64::from(count_and_positions),
                count_and_positions_bits as u8,
                &mut metadata,
            )?;
            // Reset per list: the first ID is absolute, subsequent values are gaps.
            let mut previous_doc_id = 0;
            for &doc in &postings.doc_ids {
                doc_deltas.push(doc - previous_doc_id);
                previous_doc_id = doc;
            }
            if postings.term_freqs.is_empty() {
                // Zero marks implicit JSON frequencies. Text stores an explicit frequency of one.
                frequency_codes.resize(frequency_codes.len() + postings.doc_ids.len(), 0);
            }
            let mut deltas = postings.position_deltas.as_ref();
            for &freq in &postings.term_freqs {
                if freq >= POSITION_TAG || (has_positions && freq == 1 && deltas[0] >= POSITION_TAG)
                {
                    return Err(invalid("frequency or tagged position exceeds 31 bits"));
                }
                if has_positions && freq == 1 {
                    frequency_codes.push(POSITION_TAG | deltas[0]);
                } else {
                    frequency_codes.push(freq);
                    if has_positions {
                        separate_position_deltas.extend_from_slice(&deltas[..freq as usize]);
                    }
                }
                if has_positions {
                    deltas = &deltas[freq as usize..];
                }
            }
        } else if term.doc_freq == 0 {
            return Err(invalid("empty external postings list"));
        }
    }
    packer.flush(&mut metadata)?;
    let mut payload = Vec::new();
    for stream in [&doc_deltas, &frequency_codes, &separate_position_deltas] {
        write_lanes(stream, &mut payload)?;
    }
    sstable::encode_block_parts(&metadata, &payload, keys, output)
}

pub(super) fn read_block(data: &mut &[u8], block: &mut super::TermInfoBlock) -> io::Result<()> {
    let max_inline_docs = block.max_inline_docs;
    let count_and_positions_bits = count_and_positions_bits(max_inline_docs);
    let positions_flag = 1u8 << (count_and_positions_bits - 1);
    let version = VInt::deserialize_u64(data)?;
    if version != 1 && version != VERSION {
        return Err(invalid("unsupported inline postings version"));
    }
    let num_terms = read_usize(data)?;
    let inline_term_count = block.read_term_infos(data, num_terms, true)?;
    if inline_term_count == 0 {
        return Err(invalid("missing inline postings"));
    }
    let counts_and_positions_len = inline_term_count
        .checked_mul(count_and_positions_bits)
        .ok_or_else(|| invalid("inline counts and positions length overflow"))?
        .div_ceil(8);
    let counts_and_positions = data
        .get(..counts_and_positions_len)
        .ok_or_else(|| invalid("truncated inline counts and positions"))?;
    *data = &data[counts_and_positions_len..];
    let unpacker = BitUnpacker::new(count_and_positions_bits as u8);
    let inline_term_count =
        u32::try_from(inline_term_count).map_err(|_| invalid("inline term count overflow"))?;
    let doc_counts_and_positions = (0..inline_term_count).map(|i| {
        let count_and_positions = unpacker.get(i, counts_and_positions) as u8;
        (
            (count_and_positions & (positions_flag - 1)) as usize + 1,
            count_and_positions & positions_flag != 0,
        )
    });
    let doc_count = doc_counts_and_positions
        .clone()
        .try_fold(0usize, |total, (count, _)| {
            if count > max_inline_docs as usize {
                return Err(invalid(
                    "inline postings document count exceeds dictionary limit",
                ));
            }
            total
                .checked_add(count)
                .ok_or_else(|| invalid("document count overflow"))
        })?;
    let mut doc_codes = read_lanes(data, doc_count)?.into_iter();
    let frequency_codes = read_lanes(data, doc_count)?;
    let mut frequency_codes_iter = frequency_codes.iter();
    let mut position_count = 0usize;
    for (count, has_positions) in doc_counts_and_positions.clone() {
        for &freq in frequency_codes_iter.by_ref().take(count) {
            if freq & POSITION_TAG != 0 {
                if !has_positions {
                    return Err(invalid("embedded position without positions"));
                }
            } else if has_positions {
                if freq == 0 {
                    return Err(invalid("positions without frequencies"));
                }
                position_count = position_count
                    .checked_add(freq as usize)
                    .ok_or_else(|| invalid("position count overflow"))?;
            }
        }
    }
    let mut separate_position_deltas = read_lanes(data, position_count)?.into_iter();
    let mut frequency_codes = frequency_codes.into_iter();
    for (term, (count, has_positions)) in block
        .term_infos
        .iter_mut()
        .filter(|term| term.doc_freq == 0)
        .zip(doc_counts_and_positions)
    {
        let mut doc_ids: Box<[u32]> = doc_codes.by_ref().take(count).collect();
        let mut term_freqs = Vec::with_capacity(count);
        let mut position_deltas = Vec::new();
        if version == VERSION {
            let mut previous_doc_id = 0u32;
            for doc in &mut doc_ids {
                *doc += previous_doc_id;
                previous_doc_id = *doc;
            }
        }
        for freq in frequency_codes.by_ref().take(count) {
            if freq & POSITION_TAG != 0 {
                term_freqs.push(1);
                position_deltas.push(freq & !POSITION_TAG);
            } else if freq != 0 {
                term_freqs.push(freq);
                if has_positions {
                    position_deltas.extend(separate_position_deltas.by_ref().take(freq as usize));
                }
            }
        }
        term.doc_freq = count as u32;
        term.inline_postings = Some(Arc::new(LoadedPostings {
            doc_ids,
            term_freqs: term_freqs.into_boxed_slice(),
            position_deltas: position_deltas.into_boxed_slice(),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sstable::value::ValueReader;

    use super::super::TermInfoBlock;
    use super::*;
    use crate::postings::TermInfo;

    #[test]
    fn tagged_position_and_corrupt_blocks() -> io::Result<()> {
        let max_inline_docs = 3; // Leaves an unused count code to test invalid counts.
        let mut bytes = Vec::new();
        for value in [0, VERSION, 1, 0, 0, 0] {
            VInt(value).serialize_into_vec(&mut bytes);
        }
        bytes.push(1 << (count_and_positions_bits(max_inline_docs) - 1));
        write_lanes(&[u32::MAX - 1], &mut bytes)?;
        write_lanes(&[POSITION_TAG | 17], &mut bytes)?;
        let mut block = TermInfoBlock::new(max_inline_docs)?;
        assert_eq!(block.load(&bytes)?, bytes.len());
        let postings = block.term_infos[0].inline_postings.as_ref().unwrap();
        assert_eq!(&*postings.doc_ids, [u32::MAX - 1]);
        assert_eq!(&*postings.term_freqs, [1]);
        assert_eq!(&*postings.position_deltas, [17]);
        // A lone zero encodes an empty legacy block.
        for len in (0..bytes.len()).filter(|&len| len != 1) {
            assert!(TermInfoBlock::new(max_inline_docs)?
                .load(&bytes[..len])
                .is_err());
        }
        for (count_and_positions, message) in [
            (0, "embedded position without positions"),
            (1, "byte lane length mismatch"),
            (3, "inline postings document count exceeds dictionary limit"),
        ] {
            bytes[6] = count_and_positions;
            let error = TermInfoBlock::new(max_inline_docs)?
                .load(&bytes)
                .unwrap_err();
            assert_eq!(error.to_string(), message);
        }
        for version in [0, VERSION + 1] {
            let mut bytes = Vec::new();
            VInt(0).serialize_into_vec(&mut bytes);
            VInt(version).serialize_into_vec(&mut bytes);
            let error = TermInfoBlock::new(32)?.load(&bytes).unwrap_err();
            assert_eq!(error.to_string(), "unsupported inline postings version");
        }
        for (freq, position) in [(POSITION_TAG, 17), (1, POSITION_TAG)] {
            let postings = Arc::make_mut(block.term_infos[0].inline_postings.as_mut().unwrap());
            postings.term_freqs[0] = freq;
            postings.position_deltas[0] = position;
            let error = write_block(&block, b"", &mut Vec::new()).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
        Ok(())
    }

    #[test]
    fn block_streams_and_positions_roundtrip() -> io::Result<()> {
        use futures::executor::block_on;

        // Cover packed counts at every bit width and single-term block transitions.
        for (max_inline_docs, block_len) in [
            (1, usize::MAX),
            (2, usize::MAX),
            (3, usize::MAX),
            (8, usize::MAX),
            (16, usize::MAX),
            (32, usize::MAX),
            (32, 1),
        ] {
            let mut expected = Vec::new();
            for count in
                (1..=max_inline_docs).filter(|&count| count == 1 || count == max_inline_docs)
            {
                for with_positions in [false, true] {
                    let freqs: Vec<u32> =
                        (0..count).map(|i| if i % 2 == 0 { 1 } else { 3 }).collect();
                    let deltas = if with_positions {
                        freqs
                            .iter()
                            .flat_map(|&f| (0..f).map(|i| i * 65536))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    let term = TermInfo {
                        doc_freq: count,
                        postings_range: 5..5,
                        positions_range: 7..7,
                        inline_postings: Some(Arc::new(LoadedPostings {
                            doc_ids: (0..count).map(|i| i * 1_000_000).collect(),
                            term_freqs: freqs.into_boxed_slice(),
                            position_deltas: deltas.into_boxed_slice(),
                        })),
                    };
                    expected.push(term);
                }
            }
            let external = TermInfo {
                doc_freq: 129,
                postings_range: 5..90,
                positions_range: 7..400,
                inline_postings: None,
            };
            expected.push(external);
            // Exercise the public writer/reader path, including independent block frames.
            let mut table = sstable::Writer::with_value_writer(
                Vec::new(),
                TermInfoBlock::new(max_inline_docs)?,
            );
            table.set_block_len(block_len);
            for (i, term) in expected.iter().enumerate() {
                table.insert([b'a' + i as u8], term)?;
            }
            let bytes = table.finish()?;
            assert_eq!(bytes[4], 2);
            assert_eq!(&bytes[bytes.len() - 4..], &4u32.to_le_bytes());
            let dictionary = sstable::Dictionary::<super::super::TermSSTable>::from_bytes(
                common::OwnedBytes::new(bytes),
            )?
            .with_value_config(max_inline_docs as u8)?;
            for (i, term) in expected.iter().enumerate() {
                let key = [b'a' + i as u8];
                assert_eq!(dictionary.get(key)?.as_ref(), Some(term));
                assert_eq!(block_on(dictionary.get_async(key))?.as_ref(), Some(term));
                assert_eq!(
                    dictionary.term_info_from_ord(i as u64)?.as_ref(),
                    Some(term)
                );
            }
            for mut stream in [
                dictionary.stream()?,
                block_on(dictionary.range().into_stream_async())?,
            ] {
                let mut saved = Vec::new();
                while stream.advance() {
                    assert_eq!(stream.key(), [b'a' + saved.len() as u8]);
                    assert_eq!(stream.term_ord(), saved.len() as u64);
                    saved.push(stream.value().clone());
                }
                drop(stream);
                // Term infos must outlive both dictionary block changes and the stream.
                assert_eq!(saved, expected);
            }
            let automaton = tantivy_fst::Regex::new("a|e").unwrap();
            for gap in [None, Some(512 * 1024)] {
                let builder = dictionary.search(&automaton).without_keys();
                let mut stream = match gap {
                    None => builder.into_stream()?,
                    Some(gap) => block_on(builder.into_stream_async_merging_holes(gap))?,
                };
                for i in [0, 4].into_iter().filter(|&i| i < expected.len()) {
                    assert!(stream.advance());
                    assert_eq!(stream.term_ord(), i as u64);
                    assert_eq!(stream.value(), &expected[i]);
                }
                assert!(!stream.advance());
            }
        }
        Ok(())
    }

    #[test]
    fn document_versions_and_delta_reset() -> io::Result<()> {
        // Encode independently of the writer to distinguish absolute IDs from deltas.
        for (version, docs) in [
            (1, [100, 105, 115, 0, 3, 7]),
            (VERSION, [100, 5, 10, 0, 3, 7]),
        ] {
            let mut bytes = Vec::new();
            for value in [0, version, 3, 0, 0, 0, 0, 0] {
                VInt(value).serialize_into_vec(&mut bytes);
            }
            bytes.extend_from_slice(&[0x12, 0]); // Four-bit counts minus one: 2, 1, 0.
            write_lanes(&docs, &mut bytes)?;
            write_lanes(&[1; 6], &mut bytes)?;
            let mut block = TermInfoBlock::new(8)?;
            assert_eq!(block.load(&bytes)?, bytes.len());
            let docs: Vec<_> = block
                .term_infos
                .iter()
                .map(|term| term.inline_postings.as_ref().unwrap().doc_ids.as_ref())
                .collect();
            assert_eq!(docs, [&[100, 105, 115][..], &[0, 3], &[7]]);
        }
        Ok(())
    }
}
