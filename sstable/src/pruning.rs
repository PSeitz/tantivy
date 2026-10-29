//! Optional literal-substring pruning with exact retained remainder postings.
//!
//! Format `BP01` (all integers little endian): magic, u32 block count, u32
//! trigram count; one (u32 length, complete common-prefix bytes) per block;
//! 256 u32 byte-row end offsets followed by their posting frames; then sorted
//! (u24 gram, u32 end offset) entries followed by trigram frames. Offsets are
//! relative to the corresponding frame area. A gram's numeric value is its
//! three bytes interpreted big endian. Missing trigram rows mean unknown.
//!
//! Posting frames use the adaptive codecs in `codec`. All byte rows are stored,
//! including empty ones. Prefixes and byte rows are mandatory. The budget covers
//! the entire section plus the enclosing footer's additional eight bytes.
//! Counts, lengths and frame offsets are u32, and sections are limited to u32::MAX
//! bytes. Scores and temporary builder postings are never serialized.

use std::collections::{HashMap, HashSet};
use std::io;

use common::OwnedBytes;

mod codec;
use codec::Posting;

const MAGIC: &[u8; 4] = b"BP01";
const FOOTER_BYTES: usize = 8;

/// Controls the additional serialized metadata used for literal block pruning.
/// No default budget or presence threshold is implied.
#[derive(Clone, Copy, Debug)]
pub struct PruningOptions {
    /// Maximum section size plus the additional eight-byte footer field.
    pub budget_bytes: usize,
    /// Maximum fraction of blocks containing a trigram in their term remainders,
    /// inclusive.
    pub max_block_presence: f64,
    /// Maximum blocks collected per trigram. Exceeding this count frees its
    /// block-ID vector and stops collection for that trigram. Zero skips collection
    /// for all trigrams; usize::MAX disables this cap. Skipped trigrams remain
    /// unknown during queries and may reduce pruning effectiveness.
    /// Vector capacity and other builder allocations also contribute to memory use.
    pub max_blocks_per_trigram: usize,
}

impl PruningOptions {
    pub(crate) fn validate(&self) -> io::Result<()> {
        if !self.max_block_presence.is_finite() || !(0.0..=1.0).contains(&self.max_block_presence) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "max_block_presence must be a finite fraction in 0..=1",
            ));
        }
        Ok(())
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn gram(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])
}

fn as_u32(value: usize) -> io::Result<u32> {
    u32::try_from(value).map_err(|_| invalid("pruning format exceeds u32 limit"))
}

/// Sums full-term trigram scores and collects block IDs for bytes and trigrams
/// in term remainders. Keys are buffered until their block is flushed.
/// Trigram block IDs are kept until finish or until their collection cap is exceeded.
pub(crate) struct PruningBuilder {
    options: PruningOptions,
    scores: HashMap<u32, u64>,
    keys: Vec<Vec<u8>>,
    prefixes: Vec<Vec<u8>>,
    bytes: Vec<Vec<u32>>,
    // None stops later blocks from recreating a discarded vector.
    block_ids_by_trigram: HashMap<u32, Option<Vec<u32>>>,
}

impl PruningBuilder {
    pub(crate) fn new(options: PruningOptions) -> Self {
        Self {
            options,
            scores: HashMap::new(),
            keys: Vec::new(),
            prefixes: Vec::new(),
            bytes: vec![Vec::new(); 256],
            block_ids_by_trigram: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, key: &[u8], weight: u64) -> io::Result<()> {
        as_u32(key.len())?;
        let grams: HashSet<_> = key.windows(3).map(gram).collect();
        // Check before mutating so an overflowing term leaves the builder intact.
        for g in &grams {
            self.scores
                .get(g)
                .copied()
                .unwrap_or(0)
                .checked_add(weight)
                .ok_or_else(|| invalid("trigram score overflow"))?;
        }
        for g in grams {
            *self.scores.entry(g).or_default() += weight;
        }
        self.keys.push(key.to_vec());
        Ok(())
    }

    /// Collects the buffered keys when the writer flushes their block.
    /// Returns immediately when there are no buffered keys.
    pub(crate) fn flush_block(&mut self) -> io::Result<()> {
        let Some(first) = self.keys.first() else {
            return Ok(());
        };
        let block_id = as_u32(self.prefixes.len())?;
        block_id
            .checked_add(1)
            .ok_or_else(|| invalid("too many pruning blocks"))?;
        let mut prefix_len = first.len();
        for key in &self.keys[1..] {
            prefix_len = first[..prefix_len]
                .iter()
                .zip(key)
                .take_while(|(a, b)| a == b)
                .count();
        }
        let mut bytes = [false; 256];
        let mut remainder_trigrams = HashSet::new();
        for key in &self.keys {
            let remainder = &key[prefix_len..];
            for &byte in remainder {
                bytes[byte as usize] = true;
            }
            remainder_trigrams.extend(remainder.windows(3).map(gram));
        }
        for (byte, present) in bytes.into_iter().enumerate() {
            if present {
                self.bytes[byte].push(block_id);
            }
        }
        for trigram in remainder_trigrams {
            let retained_block_ids = self
                .block_ids_by_trigram
                .entry(trigram)
                .or_insert_with(|| Some(Vec::new()));
            if let Some(block_ids) = retained_block_ids {
                if block_ids.len() == self.options.max_blocks_per_trigram {
                    *retained_block_ids = None;
                } else {
                    block_ids.push(block_id);
                }
            }
        }
        self.prefixes.push(first[..prefix_len].to_vec());
        self.keys.clear();
        Ok(())
    }

    /// Flushes the final block and selects encoded block-ID lists by score and
    /// serialized cost.
    pub(crate) fn finish(mut self) -> io::Result<Vec<u8>> {
        self.options.validate()?;
        self.flush_block()?;
        let blocks = as_u32(self.prefixes.len())?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&blocks.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // selected row count
        // shortcut: store complete prefixes, derive them from index bounds if their cost limits
        // useful rows.
        for prefix in self.prefixes {
            out.extend_from_slice(&as_u32(prefix.len())?.to_le_bytes());
            out.extend(prefix);
        }
        let byte_frames: Vec<_> = self
            .bytes
            .iter()
            .map(|ids| codec::encode(ids, blocks))
            .collect();
        let mut offset = 0u32;
        for frame in &byte_frames {
            offset = offset
                .checked_add(as_u32(frame.len())?)
                .ok_or_else(|| invalid("pruning posting area too large"))?;
            out.extend_from_slice(&offset.to_le_bytes());
        }
        for frame in byte_frames {
            out.extend(frame);
        }
        let allowance = self
            .options
            .budget_bytes
            .saturating_sub(FOOTER_BYTES)
            .min(u32::MAX as usize);
        if self.options.budget_bytes < FOOTER_BYTES || out.len() > allowance {
            return Ok(Vec::new());
        }
        let mut eligible: Vec<_> = self
            .block_ids_by_trigram
            .into_iter()
            .filter_map(|(trigram, retained_block_ids)| {
                retained_block_ids.map(|block_ids| (trigram, block_ids))
            })
            .filter(|(_, ids)| {
                ids.len() as f64 <= self.options.max_block_presence * f64::from(blocks)
            })
            .collect();
        eligible
            .sort_unstable_by(|(a, _), (b, _)| self.scores[b].cmp(&self.scores[a]).then(a.cmp(b)));
        let mut selected = Vec::new();
        let mut remaining = allowance - out.len();
        for (key, ids) in eligible {
            // Every row needs a seven-byte directory entry and a codec tag.
            if remaining < 8 {
                break;
            }
            let frame = codec::encode(&ids, blocks);
            let Some(cost) = frame.len().checked_add(7) else {
                continue;
            };
            if cost <= remaining {
                remaining -= cost;
                selected.push((key, frame));
            }
        }
        selected.sort_unstable_by_key(|(key, _)| *key);
        out[8..12].copy_from_slice(&as_u32(selected.len())?.to_le_bytes());
        offset = 0;
        for (key, frame) in &selected {
            out.extend_from_slice(&key.to_le_bytes()[..3]);
            offset = offset
                .checked_add(as_u32(frame.len())?)
                .ok_or_else(|| invalid("pruning posting area too large"))?;
            out.extend_from_slice(&offset.to_le_bytes());
        }
        for (_, frame) in selected {
            out.extend(frame);
        }
        Ok(out)
    }
}

fn take<'a>(data: &'a [u8], pos: &mut usize, len: usize) -> io::Result<&'a [u8]> {
    let end = pos
        .checked_add(len)
        .ok_or_else(|| invalid("pruning length overflow"))?;
    let bytes = data
        .get(*pos..end)
        .ok_or_else(|| invalid("truncated pruning section"))?;
    *pos = end;
    Ok(bytes)
}

fn read_u32(data: &[u8], pos: &mut usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(take(data, pos, 4)?.try_into().unwrap()))
}

fn open_frames(
    data: &OwnedBytes,
    pos: &mut usize,
    ends: &[u32],
    blocks: u32,
) -> io::Result<Vec<Posting>> {
    let start = *pos;
    let length = ends.last().copied().unwrap_or(0) as usize;
    take(data, pos, length)?;
    let mut previous = 0;
    let mut rows = Vec::new();
    for &end in ends {
        if end <= previous || end as usize > length {
            return Err(invalid("invalid pruning posting offsets"));
        }
        rows.push(Posting::open(
            data.slice(start + previous as usize..start + end as usize),
            blocks,
        )?);
        previous = end;
    }
    Ok(rows)
}

/// Separately loaded, validated pruning metadata. Missing trigram rows are unknown.
#[derive(Debug)]
pub(crate) struct BlockPruning {
    prefixes: Vec<OwnedBytes>,
    bytes: Vec<Posting>,
    keys: Vec<u32>,
    rows: Vec<Posting>,
}

impl BlockPruning {
    pub(crate) fn open(data: OwnedBytes, expected_blocks: usize) -> io::Result<Self> {
        if data.len() > u32::MAX as usize {
            return Err(invalid("pruning section too large"));
        }
        let mut pos = 0;
        if take(&data, &mut pos, 4)? != MAGIC {
            return Err(invalid("unknown pruning format"));
        }
        let blocks = read_u32(&data, &mut pos)?;
        let count = read_u32(&data, &mut pos)? as usize;
        if blocks as usize != expected_blocks || count > 1 << 24 {
            return Err(invalid("invalid pruning counts"));
        }
        // Bound allocation/work by the supplied bytes before trusting counts.
        if blocks as usize > (data.len() - pos) / 4 {
            return Err(invalid("truncated pruning prefixes"));
        }
        let mut prefixes = Vec::new();
        for _ in 0..blocks {
            let length = read_u32(&data, &mut pos)? as usize;
            let start = pos;
            take(&data, &mut pos, length)?;
            prefixes.push(data.slice(start..pos));
        }
        let mut ends = Vec::with_capacity(256);
        for _ in 0..256 {
            ends.push(read_u32(&data, &mut pos)?);
        }
        let bytes = open_frames(&data, &mut pos, &ends, blocks)?;
        let directory_len = count
            .checked_mul(7)
            .ok_or_else(|| invalid("pruning directory overflow"))?;
        let directory = take(&data, &mut pos, directory_len)?;
        let mut keys = Vec::new();
        ends.clear();
        for entry in directory.as_chunks::<7>().0 {
            let key = u32::from_le_bytes([entry[0], entry[1], entry[2], 0]);
            if keys.last().is_some_and(|&previous| previous >= key) {
                return Err(invalid("pruning keys out of order"));
            }
            keys.push(key);
            ends.push(u32::from_le_bytes(entry[3..].try_into().unwrap()));
        }
        let rows = open_frames(&data, &mut pos, &ends, blocks)?;
        if pos != data.len() {
            return Err(invalid("trailing pruning bytes"));
        }
        Ok(Self {
            prefixes,
            bytes,
            keys,
            rows,
        })
    }

    /// Resolves each query gram once. Offsets are preserved for prefix crossings.
    pub(crate) fn prepare<'a>(&'a self, literal: &'a [u8]) -> PreparedLiteral<'a> {
        let rows = literal
            .windows(3)
            .map(|bytes| {
                self.keys
                    .binary_search(&gram(bytes))
                    .ok()
                    .map(|id| &self.rows[id])
            })
            .collect();
        PreparedLiteral {
            pruning: self,
            literal,
            rows,
        }
    }
}

pub(crate) struct PreparedLiteral<'a> {
    pruning: &'a BlockPruning,
    literal: &'a [u8],
    rows: Vec<Option<&'a Posting>>,
}

impl PreparedLiteral<'_> {
    fn remainder_permits(&self, start: usize, block: u32) -> bool {
        self.literal[start..]
            .iter()
            .all(|&byte| self.pruning.bytes[byte as usize].contains(block))
            && self
                .rows
                .get(start..)
                .unwrap_or_default()
                .iter()
                .all(|row| row.is_none_or(|row| row.contains(block)))
    }

    pub(crate) fn permits(&self, block_id: u64) -> bool {
        let Ok(block) = usize::try_from(block_id) else {
            return true;
        };
        let Some(prefix) = self.pruning.prefixes.get(block) else {
            return true;
        };
        if self.literal.is_empty()
            || prefix
                .windows(self.literal.len())
                .any(|part| part == self.literal)
        {
            return true;
        }
        if self.remainder_permits(0, block as u32) {
            return true;
        }
        // Only grams wholly inside this continuation constrain a crossing match.
        (1..=prefix.len().min(self.literal.len() - 1)).any(|consumed| {
            prefix.ends_with(&self.literal[..consumed])
                && self.remainder_permits(consumed, block as u32)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(budget_bytes: usize, max_block_presence: f64) -> PruningOptions {
        PruningOptions {
            budget_bytes,
            max_block_presence,
            max_blocks_per_trigram: usize::MAX,
        }
    }

    fn build(blocks: &[Vec<Vec<u8>>], options: PruningOptions) -> Vec<u8> {
        let mut builder = PruningBuilder::new(options);
        for block in blocks {
            for key in block {
                builder.insert(key, 1).unwrap();
            }
            builder.flush_block().unwrap();
        }
        builder.finish().unwrap()
    }

    fn open(bytes: Vec<u8>, blocks: usize) -> BlockPruning {
        BlockPruning::open(OwnedBytes::new(bytes), blocks).unwrap()
    }

    #[test]
    fn options_and_distinct_full_term_scores() {
        for cap in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
            assert!(options(1000, cap).validate().is_err());
            assert!(PruningBuilder::new(options(1000, cap)).finish().is_err());
        }
        for cap in [0.0, 0.5, 1.0] {
            assert!(options(0, cap).validate().is_ok());
        }
        let mut builder = PruningBuilder::new(options(usize::MAX, 1.0));
        builder.insert(b"aaaaab", 7).unwrap();
        builder.insert(b"aaaac", 11).unwrap();
        assert_eq!(builder.scores[&gram(b"aaa")], 18);
        assert_eq!(builder.scores[&gram(b"aab")], 7);
        builder.flush_block().unwrap();
        assert_eq!(builder.prefixes, vec![b"aaaa".to_vec()]);
        assert!(builder.block_ids_by_trigram.is_empty());
        assert_eq!(builder.bytes[b'a' as usize], vec![0]);
        builder.flush_block().unwrap();
        builder.insert(b"aaa", u64::MAX).unwrap_err();
        assert_eq!(builder.scores[&gram(b"aaa")], 18);
        assert!(builder.keys.is_empty());
        let mut builder = PruningBuilder::new(options(usize::MAX, 1.0));
        builder.insert(b"aaaa", u64::MAX).unwrap();
        builder.insert(b"aaa", 0).unwrap();
        assert!(builder.insert(b"aaa", 1).is_err());
        // finish flushes the pending block, including its complete prefix.
        assert_eq!(
            open(builder.finish().unwrap(), 1).prefixes[0].as_ref(),
            b"aaa"
        );
    }

    #[test]
    fn exact_budget_cap_ties_and_unknown_rows() {
        let blocks = vec![
            vec![b"abc".to_vec(), b"def".to_vec()],
            vec![b"abc".to_vec(), b"xyz".to_vec()],
        ];
        let mandatory = build(&blocks, options(usize::MAX, 0.0));
        let cost = mandatory.len() + FOOTER_BYTES;
        assert!(build(&blocks, options(cost - 1, 0.0)).is_empty());
        assert_eq!(build(&blocks, options(cost, 0.0)), mandatory);
        assert!(open(mandatory, 2).keys.is_empty());
        // All-block abc is capped out. Equal-score def precedes xyz numerically.
        let row_cost = 7 + codec::encode(&[0], 2).len();
        let bytes = build(&blocks, options(cost + row_cost, 0.5));
        assert_eq!(bytes.len() + FOOTER_BYTES, cost + row_cost);
        let pruning = open(bytes, 2);
        assert_eq!(pruning.keys, vec![gram(b"def")]);
        let all = open(build(&blocks, options(usize::MAX, 1.0)), 2);
        assert_eq!(all.keys, vec![gram(b"abc"), gram(b"def"), gram(b"xyz")]);
        for (id, key) in all.keys.iter().enumerate() {
            for (block, terms) in blocks.iter().enumerate() {
                let expected = terms
                    .iter()
                    .any(|term| term.windows(3).any(|w| gram(w) == *key));
                assert_eq!(all.rows[id].contains(block as u32), expected);
            }
        }
        // Absent and budget-omitted trigrams remain unknown when bytes permit.
        assert!(pruning.prepare(b"cba").permits(0));
        assert!(pruning.prepare(b"abc").permits(0));
        assert!(!pruning.prepare(b"def").permits(1));
        assert!(pruning.prepare(b"abc").permits(u64::MAX));
        assert_eq!(
            build(&blocks, options(cost + row_cost, 0.5)),
            build(&blocks, options(cost + row_cost, 0.5))
        );
    }

    #[test]
    fn collection_cap_stops_collecting_block_ids() {
        for cap in [0, 1, 2, 4, usize::MAX] {
            let mut options = options(usize::MAX, 1.0);
            options.max_blocks_per_trigram = cap;
            let mut builder = PruningBuilder::new(options);
            for block in 0..4 {
                builder.insert(b"", 0).unwrap(); // Keep the common prefix empty.
                builder.insert(b"abcabc", 1).unwrap(); // Repeated trigrams count once per block.
                if block == 0 {
                    builder.insert(b"xyz", 1).unwrap();
                }
                builder.flush_block().unwrap();
                let retained_block_ids = &builder.block_ids_by_trigram[&gram(b"abc")];
                if block < cap {
                    assert_eq!(
                        retained_block_ids.as_ref().unwrap(),
                        &(0..=block as u32).collect::<Vec<_>>()
                    );
                } else {
                    assert!(retained_block_ids.is_none());
                }
            }
            let pruning = open(builder.finish().unwrap(), 4);
            assert_eq!(pruning.keys.contains(&gram(b"abc")), cap >= 4);
            assert_eq!(pruning.keys.contains(&gram(b"xyz")), cap >= 1);
            for block in 0..4 {
                assert!(pruning.prepare(b"abcabc").permits(block));
            }
            assert!(pruning.prepare(b"xyz").permits(0));
            // Byte membership permits matches after a trigram's block IDs are discarded.
            assert!(pruning.prepare(b"abc").permits(3));
            if cap == 0 {
                assert!(pruning.keys.is_empty());
            }
        }
    }

    #[test]
    fn ranks_by_full_term_weight_before_numeric_key() {
        let blocks = vec![vec![b"abc".to_vec(), b"xyz".to_vec()]];
        let mandatory = build(&blocks, options(usize::MAX, 0.0)).len();
        let row_cost = 7 + codec::encode(&[0], 1).len();
        let mut builder = PruningBuilder::new(options(mandatory + FOOTER_BYTES + row_cost, 1.0));
        builder.insert(b"abc", 1).unwrap();
        builder.insert(b"xyz", 2).unwrap();
        assert_eq!(open(builder.finish().unwrap(), 1).keys, vec![gram(b"xyz")]);
    }

    #[test]
    fn pins_prefix_byte_directory_and_frame_layout() {
        let blocks = vec![vec![vec![0, 255]]];
        let actual = build(&blocks, options(usize::MAX, 1.0));
        let mut expected = b"BP01".to_vec();
        expected.extend(1u32.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        expected.extend(2u32.to_le_bytes());
        expected.extend([0, 255]);
        for end in 1u32..=256 {
            expected.extend(end.to_le_bytes());
        }
        expected.extend([0; 256]); // empty fixed-ID row for every remainder byte
        assert_eq!(actual, expected);
        assert_eq!(actual.len() + FOOTER_BYTES, 1306);
    }

    #[test]
    fn skips_nonfitting_higher_score_row() {
        let mut builder = PruningBuilder::new(options(usize::MAX, 1.0));
        for block in 0..64 {
            builder.insert(b"", 0).unwrap();
            if block % 2 == 0 {
                builder.insert(b"abc", 100).unwrap();
            }
            if block == 63 {
                builder.insert(b"xyz", 1).unwrap();
            }
            builder.flush_block().unwrap();
        }
        // Reconstruct mandatory size independently from the format fields.
        let mandatory = 12
            + builder.prefixes.iter().map(|p| 4 + p.len()).sum::<usize>()
            + 256 * 4
            + builder
                .bytes
                .iter()
                .map(|ids| codec::encode(ids, 64).len())
                .sum::<usize>();
        let small = 7 + codec::encode(&[63], 64).len();
        assert!(
            codec::encode(
                builder.block_ids_by_trigram[&gram(b"abc")]
                    .as_ref()
                    .unwrap(),
                64
            )
            .len()
                + 7
                > small
        );
        builder.options.budget_bytes = mandatory + FOOTER_BYTES + small;
        let bytes = builder.finish().unwrap();
        assert_eq!(bytes.len(), mandatory + small);
        assert_eq!(open(bytes, 64).keys, vec![gram(b"xyz")]);
    }

    #[test]
    fn prefix_crossings_short_continuations_and_raw_bytes() {
        let blocks = vec![
            vec![b"abcde".to_vec(), b"abcxy".to_vec()],
            vec![b"0abc".to_vec(), b"1cde".to_vec()],
            vec![b"aaaaab".to_vec(), b"aaaaac".to_vec()],
            vec![vec![0, 255, 128]],
            vec![Vec::new()],
        ];
        let pruning = open(build(&blocks, options(usize::MAX, 1.0)), blocks.len());
        assert_eq!(pruning.prefixes[0].as_ref(), b"abc");
        // abc/cde rows exist but exclude block 0. A whole-query AND would fail.
        for literal in [b"abc".as_slice(), b"bcd", b"bcde", b"cde", b"de", b"e", b""] {
            assert!(pruning.prepare(literal).permits(0), "{literal:?}");
        }
        assert!(pruning.prepare(b"aaaab").permits(2));
        assert!(pruning.prepare(&[255, 128]).permits(3));
        assert!(!pruning.prepare(b"ABC").permits(0));
        assert!(!pruning.prepare(b"a").permits(4));
        assert!(!pruning.prepare(b"z").permits(0));
    }

    fn words(max_len: usize) -> Vec<Vec<u8>> {
        let mut words = vec![Vec::new()];
        for len in 1..=max_len {
            for bits in 0..1usize << len {
                words.push(
                    (0..len)
                        .map(|bit| if bits & (1 << bit) == 0 { 0 } else { 255 })
                        .collect(),
                );
            }
        }
        words
    }

    #[test]
    fn exhaustive_no_false_negatives_and_partial_containment() {
        let terms = words(4);
        let blocks: Vec<_> = terms
            .iter()
            .enumerate()
            .flat_map(|(i, a)| terms[i..].iter().map(move |b| vec![a.clone(), b.clone()]))
            .collect();
        let fallback_bytes = build(&blocks, options(usize::MAX, 0.0));
        let budget = fallback_bytes.len() + FOOTER_BYTES + 20;
        let fallback = open(fallback_bytes, blocks.len());
        let partial = open(build(&blocks, options(budget, 0.75)), blocks.len());
        let full = open(build(&blocks, options(usize::MAX, 1.0)), blocks.len());
        for literal in words(5) {
            let full_query = full.prepare(&literal);
            let partial_query = partial.prepare(&literal);
            let byte_query = fallback.prepare(&literal);
            for (id, terms) in blocks.iter().enumerate() {
                let truth = literal.is_empty()
                    || terms
                        .iter()
                        .any(|term| term.windows(literal.len()).any(|w| w == literal));
                let full = full_query.permits(id as u64);
                let partial = partial_query.permits(id as u64);
                assert!(!truth || full, "{terms:?} {literal:?}");
                assert!(!full || partial);
                assert!(!partial || byte_query.permits(id as u64));
            }
        }
    }

    #[test]
    fn malformed_sections_and_empty_dictionary() {
        let blocks = vec![vec![b"abc".to_vec(), b"def".to_vec()]];
        let bytes = build(&blocks, options(usize::MAX, 1.0));
        for length in 0..bytes.len() {
            assert!(
                BlockPruning::open(OwnedBytes::new(bytes[..length].to_vec()), 1).is_err(),
                "{length}"
            );
        }
        // Byte mutations may be valid alternative summaries, but must never panic.
        for offset in 0..bytes.len() {
            let mut mutated = bytes.clone();
            mutated[offset] ^= 255;
            let _ = BlockPruning::open(OwnedBytes::new(mutated), 1);
        }
        assert!(BlockPruning::open(OwnedBytes::new(bytes.clone()), 2).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(BlockPruning::open(OwnedBytes::new(trailing), 1).is_err());
        for (offset, replacement) in [
            (0, 0u32),
            (8, u32::MAX),
            (12, u32::MAX),
            (16, 0),
            (16, u32::MAX),
        ] {
            let mut corrupt = bytes.clone();
            corrupt[offset..offset + 4].copy_from_slice(&replacement.to_le_bytes());
            assert!(BlockPruning::open(OwnedBytes::new(corrupt), 1).is_err());
        }
        // Locate trigram directory from the byte directory's final end offset.
        let last_end =
            u32::from_le_bytes(bytes[16 + 255 * 4..16 + 256 * 4].try_into().unwrap()) as usize;
        let directory = 16 + 256 * 4 + last_end;
        let mut duplicate = bytes.clone();
        duplicate.copy_within(directory..directory + 3, directory + 7);
        assert!(BlockPruning::open(OwnedBytes::new(duplicate), 1).is_err());
        let empty = open(build(&[], options(usize::MAX, 1.0)), 0);
        assert!(empty.prefixes.is_empty());
        assert!(build(&blocks, options(0, 1.0)).is_empty());
    }
}
