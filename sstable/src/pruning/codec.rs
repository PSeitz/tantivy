//! Exact posting rows adapted from the experimental trigram posting codecs.
//! Low two tag bits select fixed IDs, deltas, runs, or bitmap. Bit 2 complements
//! membership. The enclosing directory supplies the frame length. No compression
//! feature is needed to read this format.

use std::io;

use common::OwnedBytes;

use super::invalid;

fn id_width(blocks: u32) -> usize {
    ((32 - blocks.saturating_sub(1).leading_zeros()).max(1) as usize).div_ceil(8)
}

fn var(mut value: u32, out: &mut Vec<u8>) {
    while value >= 128 {
        out.push(value as u8 | 128);
        value >>= 7;
    }
    out.push(value as u8);
}

fn unvar(data: &[u8], pos: &mut usize) -> io::Result<u32> {
    let mut value = 0;
    for shift in (0..35).step_by(7) {
        let byte = *data
            .get(*pos)
            .ok_or_else(|| invalid("truncated posting varint"))?;
        *pos += 1;
        if shift == 28 && byte > 15 {
            return Err(invalid("posting varint overflow"));
        }
        value |= u32::from(byte & 127) << shift;
        if byte < 128 {
            if shift != 0 && byte == 0 {
                return Err(invalid("noncanonical posting varint"));
            }
            return Ok(value);
        }
    }
    Err(invalid("posting varint overflow"))
}

fn payload(ids: &[u32], blocks: u32, codec: u8) -> Vec<u8> {
    let mut out = Vec::new();
    match codec {
        0 => {
            for id in ids {
                out.extend_from_slice(&id.to_le_bytes()[..id_width(blocks)]);
            }
        }
        1 => {
            let mut previous = 0;
            for &id in ids {
                var(id - previous, &mut out);
                previous = id;
            }
        }
        2 => {
            let mut previous_end = 0;
            let mut pos = 0;
            while pos < ids.len() {
                let start = ids[pos];
                let mut end = start + 1;
                pos += 1;
                while pos < ids.len() && ids[pos] == end {
                    end += 1;
                    pos += 1;
                }
                var(start - previous_end, &mut out);
                var(end - start, &mut out);
                previous_end = end;
            }
        }
        3 => {
            out.resize((blocks as usize).div_ceil(8), 0);
            for &id in ids {
                out[id as usize / 8] |= 1 << (id % 8);
            }
        }
        _ => unreachable!(),
    }
    out
}

pub(super) fn encode(ids: &[u32], blocks: u32) -> Vec<u8> {
    if ids.is_empty() {
        return vec![0];
    }
    if ids.len() == blocks as usize {
        return vec![4];
    }
    let mut best = vec![0];
    best.extend(payload(ids, blocks, 0));
    // A complement is only materialized while encoding one row.
    let mut pos = 0;
    let absent: Vec<u32> = (0..blocks)
        .filter(|&id| {
            if ids.get(pos) == Some(&id) {
                pos += 1;
                false
            } else {
                true
            }
        })
        .collect();
    for (values, complement) in [(ids, 0), (absent.as_slice(), 4)] {
        for codec in 0..4 {
            let data = payload(values, blocks, codec);
            if data.len() + 1 < best.len() {
                best.clear();
                best.push(codec | complement);
                best.extend(data);
            }
        }
    }
    best
}

#[derive(Debug)]
enum Membership {
    Ids(Vec<u32>),
    Runs(Vec<(u32, u32)>),
    Bitmap(OwnedBytes),
}

/// Decodes once at open. Runs and complemented rows stay compact in memory.
#[derive(Debug)]
pub(super) struct Posting {
    membership: Membership,
    complement: bool,
}

impl Posting {
    pub(super) fn open(frame: OwnedBytes, blocks: u32) -> io::Result<Self> {
        let &tag = frame
            .first()
            .ok_or_else(|| invalid("empty posting frame"))?;
        if tag & !7 != 0 {
            return Err(invalid("unknown posting codec"));
        }
        let data = frame.slice(1..frame.len());
        let membership = match tag & 3 {
            0 | 1 => {
                let mut ids = Vec::new();
                if tag & 3 == 0 {
                    let width = id_width(blocks);
                    if !data.len().is_multiple_of(width) {
                        return Err(invalid("truncated fixed posting"));
                    }
                    for chunk in data.chunks_exact(width) {
                        let mut bytes = [0; 4];
                        bytes[..width].copy_from_slice(chunk);
                        ids.push(u32::from_le_bytes(bytes));
                    }
                } else {
                    let mut pos = 0;
                    let mut previous = 0u32;
                    while pos < data.len() {
                        previous = previous
                            .checked_add(unvar(&data, &mut pos)?)
                            .ok_or_else(|| invalid("posting delta overflow"))?;
                        ids.push(previous);
                    }
                }
                if ids.last().is_some_and(|&id| id >= blocks)
                    || ids.windows(2).any(|pair| pair[0] >= pair[1])
                {
                    return Err(invalid("posting IDs out of order or range"));
                }
                Membership::Ids(ids)
            }
            2 => {
                let mut runs = Vec::new();
                let mut pos = 0;
                let mut previous_end = 0u32;
                while pos < data.len() {
                    let gap = unvar(&data, &mut pos)?;
                    let length = unvar(&data, &mut pos)?;
                    let start = previous_end
                        .checked_add(gap)
                        .ok_or_else(|| invalid("posting run overflow"))?;
                    let end = start
                        .checked_add(length)
                        .ok_or_else(|| invalid("posting run overflow"))?;
                    if length == 0 || end > blocks || (!runs.is_empty() && gap == 0) {
                        return Err(invalid("invalid posting run"));
                    }
                    runs.push((start, end));
                    previous_end = end;
                }
                Membership::Runs(runs)
            }
            3 => {
                if data.len() != (blocks as usize).div_ceil(8)
                    || (!blocks.is_multiple_of(8)
                        && data.last().is_some_and(|byte| byte >> (blocks % 8) != 0))
                {
                    return Err(invalid("invalid posting bitmap"));
                }
                Membership::Bitmap(data)
            }
            _ => unreachable!(),
        };
        Ok(Self {
            membership,
            complement: tag & 4 != 0,
        })
    }

    pub(super) fn contains(&self, block: u32) -> bool {
        let present = match &self.membership {
            Membership::Ids(ids) => ids.binary_search(&block).is_ok(),
            Membership::Runs(runs) => {
                let pos = runs.partition_point(|&(start, _)| start <= block);
                pos != 0 && block < runs[pos - 1].1
            }
            Membership::Bitmap(data) => data[block as usize / 8] & (1 << (block % 8)) != 0,
        };
        present != self.complement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codecs_and_complements_round_trip() {
        for blocks in [0, 1, 7, 8, 9, 255, 256, 257, 65537] {
            for divisor in [1, 2, 7, 1000] {
                let ids: Vec<_> = (0..blocks).filter(|id| id % divisor == 0).collect();
                for codec in 0..4 {
                    for complement in [false, true] {
                        let mut frame = vec![codec | if complement { 4 } else { 0 }];
                        frame.extend(payload(&ids, blocks, codec));
                        let posting = Posting::open(OwnedBytes::new(frame), blocks).unwrap();
                        for id in 0..blocks {
                            assert_eq!(posting.contains(id), (id % divisor == 0) != complement);
                        }
                    }
                }
                let frame = encode(&ids, blocks);
                let posting = Posting::open(OwnedBytes::new(frame), blocks).unwrap();
                for id in 0..blocks {
                    assert_eq!(posting.contains(id), id % divisor == 0);
                }
            }
        }
    }

    #[test]
    fn malformed_frames() {
        for frame in [
            vec![],
            vec![8],
            vec![255],
            vec![0, 10],
            vec![0, 1, 1],
            vec![0, 2, 1],
            vec![1, 128],
            vec![1, 128, 0],
            vec![1, 255, 255, 255, 255, 16],
            vec![1, 1, 0],
            vec![2, 0],
            vec![2, 0, 0],
            vec![2, 9, 2],
            vec![2, 0, 1, 0, 1],
            vec![3, 0],
            vec![3, 0, 4],
        ] {
            assert!(
                Posting::open(OwnedBytes::new(frame.clone()), 10).is_err(),
                "{frame:?}"
            );
        }
        assert!(Posting::open(OwnedBytes::new(vec![0, 1]), 257).is_err());
    }
}
