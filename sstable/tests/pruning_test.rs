use std::future::Future;
use std::io;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use common::file_slice::{FileHandle, FileSlice};
use common::{HasLen, OwnedBytes};
use tantivy_fst::Regex;
use tantivy_sstable::{Dictionary, MonotonicU64SSTable, PruningOptions};

type Dict = Dictionary<MonotonicU64SSTable>;

fn build(terms: &[Vec<u8>], block_len: usize, budget: Option<usize>) -> Vec<u8> {
    build_with_cap(terms, block_len, budget, usize::MAX)
}

fn build_with_cap(
    terms: &[Vec<u8>],
    block_len: usize,
    budget: Option<usize>,
    max_blocks_per_trigram: usize,
) -> Vec<u8> {
    let mut writer = Dict::builder(Vec::new()).unwrap();
    writer.set_block_len(block_len);
    if let Some(budget_bytes) = budget {
        writer
            .enable_pruning(
                PruningOptions {
                    budget_bytes,
                    max_block_presence: 1.0,
                    max_blocks_per_trigram,
                },
                |v| *v,
            )
            .unwrap();
    }
    for (ord, term) in terms.iter().enumerate() {
        // Exercise the split insertion API used by the postings serializer and merge.
        writer.insert_key(term).unwrap();
        writer.insert_value(&(ord as u64)).unwrap();
    }
    writer.finish().unwrap()
}

fn matches(dict: &Dict, literal: &str, hint: bool) -> Vec<(Vec<u8>, u64)> {
    let mut builder = dict.search(Regex::new(&format!(".*{literal}.*")).unwrap());
    if hint {
        builder = builder.prune_with_substring(literal);
    }
    let mut stream = builder.into_stream().unwrap();
    let mut result = Vec::new();
    while stream.advance() {
        assert_eq!(*stream.value(), stream.term_ord());
        result.push((stream.key().to_vec(), stream.term_ord()));
    }
    result
}

// In-memory FileSlice futures complete on their first poll.
fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("in-memory read unexpectedly pending"),
    }
}

#[derive(Debug)]
struct RecordingFile {
    bytes: OwnedBytes,
    reads: Mutex<Vec<Range<usize>>>,
}
impl HasLen for RecordingFile {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}
impl FileHandle for RecordingFile {
    fn read_bytes(&self, range: Range<usize>) -> io::Result<OwnedBytes> {
        self.reads.lock().unwrap().push(range.clone());
        Ok(self.bytes.slice(range))
    }
}
impl RecordingFile {
    fn take_reads(&self) -> Vec<Range<usize>> {
        std::mem::take(&mut *self.reads.lock().unwrap())
    }
}

#[test]
fn lazy_loading_prunes_reads_and_preserves_ordinals() {
    let terms: Vec<_> = (0..100)
        .flat_map(|group| {
            (0..20).map(move |term| {
                let word = if (group == 20 || group == 80) && term == 10 {
                    "needle"
                } else {
                    "haystack"
                };
                format!("{group:03}-{term:03}-{word}").into_bytes()
            })
        })
        .collect();
    let ordinary = build(&terms, 100, None);
    let bytes = build(&terms, 100, Some(100_000));
    let pruning_start = u64::from_le_bytes(
        bytes[bytes.len() - 28..bytes.len() - 20]
            .try_into()
            .unwrap(),
    ) as usize;
    let pruning_end = bytes.len() - 28;
    let handle = Arc::new(RecordingFile {
        bytes: OwnedBytes::new(bytes.clone()),
        reads: Mutex::new(Vec::new()),
    });
    let mut dict = Dict::open(FileSlice::new(handle.clone())).unwrap();
    assert!(
        handle
            .take_reads()
            .iter()
            .all(|r| r.end <= pruning_start || r.start >= pruning_end)
    );
    let expected = matches(&dict, "needle", false);
    let unpruned_bytes: usize = handle.take_reads().iter().map(|r| r.len()).sum();
    assert_eq!(expected.len(), 2);
    assert_eq!(matches(&dict, "needle", true), expected);
    assert_eq!(
        handle.take_reads().iter().map(|r| r.len()).sum::<usize>(),
        unpruned_bytes
    );
    assert!(dict.load_pruning().unwrap());
    assert_eq!(handle.take_reads(), vec![pruning_start..pruning_end]);
    assert!(dict.load_pruning().unwrap());
    assert!(handle.take_reads().is_empty());
    assert_eq!(matches(&dict, "needle", true), expected);
    let pruned_bytes: usize = handle.take_reads().iter().map(|r| r.len()).sum();
    assert!(
        pruned_bytes < unpruned_bytes,
        "{pruned_bytes} >= {unpruned_bytes}"
    );
    assert_eq!(dict.num_bytes().get_bytes() as usize, bytes.len());
    assert!(bytes.len() - ordinary.len() <= 100_000);
    assert_eq!(matches(&dict, "needle", false), expected);
    handle.take_reads();
    let mut stream = dict.range().limit(1).into_stream().unwrap();
    let mut limited_count = 0;
    while stream.advance() {
        limited_count += 1;
    }
    let limited_reads = handle.take_reads();
    let mut stream = dict
        .range()
        .limit(1)
        .prune_with_substring("")
        .into_stream()
        .unwrap();
    let mut hinted_count = 0;
    while stream.advance() {
        hinted_count += 1;
    }
    assert_eq!(hinted_count, limited_count);
    assert_eq!(handle.take_reads(), limited_reads);
    assert!(limited_count < terms.len());

    let mut dict = Dict::from_bytes(OwnedBytes::new(bytes)).unwrap();
    dict.load_pruning().unwrap();
    let mut stream = ready(
        dict.range()
            .limit(1)
            .prune_with_substring("")
            .into_stream_async(),
    )
    .unwrap();
    let mut async_count = 0;
    while stream.advance() {
        async_count += 1;
    }
    assert_eq!(async_count, limited_count);
    for hole in [0, 1_000_000] {
        let mut stream = ready(
            dict.search(Regex::new(".*needle.*").unwrap())
                .ge("010")
                .lt("090")
                .limit(1)
                .prune_with_substring("needle")
                .without_keys()
                .into_stream_async_merging_holes(hole),
        )
        .unwrap();
        let mut ords = Vec::new();
        while stream.advance() {
            assert_eq!(*stream.value(), stream.term_ord());
            ords.push(stream.term_ord());
        }
        assert_eq!(
            ords,
            expected.iter().map(|(_, ord)| *ord).collect::<Vec<_>>()
        );
    }
    let mut stream = dict
        .search(Regex::new(".*needle.*").unwrap())
        .gt("030")
        .le("090")
        .prune_with_substring("needle")
        .without_keys()
        .into_stream()
        .unwrap();
    assert!(stream.advance());
    assert_eq!(stream.term_ord(), expected[1].1);
    assert!(!stream.advance());
}

#[test]
fn compatibility_empty_single_block_final_flush_and_budget() {
    for terms in [
        vec![],
        vec![b"abc".to_vec()],
        vec![b"abcde".to_vec(), b"abcxy".to_vec()],
    ] {
        let ordinary = build(&terms, 4000, None);
        assert_eq!(build(&terms, 4000, Some(0)), ordinary);
        let mut old = Dict::from_bytes(OwnedBytes::new(ordinary.clone())).unwrap();
        assert!(!old.load_pruning().unwrap());
        let enabled = build(&terms, 4000, Some(100_000));
        assert_eq!(&enabled[enabled.len() - 4..], &4u32.to_le_bytes());
        let added = enabled.len() - ordinary.len();
        assert_eq!(build(&terms, 4000, Some(added)), enabled);
        // No trigram rows exist here, so one byte less cannot fit mandatory data.
        assert_eq!(build(&terms, 4000, Some(added - 1)), ordinary);
        let mut dict = Dict::from_bytes(OwnedBytes::new(enabled)).unwrap();
        assert!(dict.load_pruning().unwrap());
        for literal in ["", "a", "bc", "abc", "bcd", "bcde", "cxy", "xyz"] {
            assert_eq!(matches(&dict, literal, true), matches(&old, literal, false));
        }
        for (ord, term) in terms.iter().enumerate() {
            assert_eq!(dict.get(term).unwrap(), Some(ord as u64));
            assert_eq!(dict.term_ord(term).unwrap(), Some(ord as u64));
        }
    }
}

#[test]
fn version_two_fixture() {
    // One void-valued term and a v2 delta-encoded index, independent of today's writer.
    let mut bytes = vec![
        5, 0, 0, 0, 0, 0x30, b'a', b'b', b'c', 0, 0, 0, 0, 9, 0, 0, 0, 0, 1, 0, 9, 0, 0x30, b'a',
        b'b', b'c', 0, 0, 0, 0,
    ];
    bytes.extend(13u64.to_le_bytes());
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(2u32.to_le_bytes());
    let mut dict =
        Dictionary::<tantivy_sstable::VoidSSTable>::from_bytes(OwnedBytes::new(bytes)).unwrap();
    assert!(!dict.load_pruning().unwrap());
    assert_eq!(dict.get(b"abc").unwrap(), Some(()));
    let mut stream = dict
        .search(Regex::new(".*bc.*").unwrap())
        .prune_with_substring("bc")
        .into_stream()
        .unwrap();
    assert!(stream.advance());
    assert_eq!(stream.key(), b"abc");
    assert_eq!(stream.term_ord(), 0);
    assert!(!stream.advance());
}

#[test]
fn configuration_and_corruption_errors() {
    let mut writer = Dict::builder(Vec::new()).unwrap();
    let options = PruningOptions {
        budget_bytes: 10_000,
        max_block_presence: 0.5,
        max_blocks_per_trigram: usize::MAX,
    };
    assert!(
        writer
            .enable_pruning(
                PruningOptions {
                    max_block_presence: f64::NAN,
                    ..options
                },
                |v| *v
            )
            .is_err()
    );
    writer.insert(b"aaa", &1).unwrap();
    assert!(writer.enable_pruning(options, |v| *v).is_err());
    let terms = vec![b"aaaa".to_vec(), b"aaab".to_vec()];
    let bytes = build(&terms, 4000, Some(10_000));
    for length in 0..28 {
        assert!(Dict::from_bytes(OwnedBytes::new(bytes[..length].to_vec())).is_err());
    }
    for offset in [bytes.len() - 28, bytes.len() - 20] {
        let mut corrupt = bytes.clone();
        corrupt[offset..offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(Dict::from_bytes(OwnedBytes::new(corrupt)).is_err());
    }
    let start = u64::from_le_bytes(
        bytes[bytes.len() - 28..bytes.len() - 20]
            .try_into()
            .unwrap(),
    ) as usize;
    for delta in 1..=7 {
        for offset in [start - delta, start + delta] {
            let mut corrupt = bytes.clone();
            let footer = corrupt.len() - 28;
            corrupt[footer..footer + 8].copy_from_slice(&(offset as u64).to_le_bytes());
            assert!(Dict::from_bytes(OwnedBytes::new(corrupt)).is_err());
        }
    }
    let mut corrupt = bytes;
    corrupt[start] = 0;
    let mut dict = Dict::from_bytes(OwnedBytes::new(corrupt)).unwrap();
    assert_eq!(matches(&dict, "aaa", true).len(), 2);
    assert!(dict.load_pruning().is_err());
    assert_eq!(matches(&dict, "aaa", true).len(), 2);
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn searches_equal_full_term_truth(
        words in prop::collection::btree_set("[a-c]{0,12}", 1..80),
        literal in "[a-c]{0,6}",
        block_len in 0usize..80,
        budget in prop_oneof![Just(0usize), Just(1500), Just(100_000)],
        cap in prop_oneof![Just(usize::MAX), 0usize..10],
    ) {
        let terms: Vec<_> = words.iter().map(|word| word.as_bytes().to_vec()).collect();
        let mut dict = Dict::from_bytes(OwnedBytes::new(build_with_cap(&terms, block_len, Some(budget), cap))).unwrap();
        let expected: Vec<_> = words.iter().enumerate().filter(|(_, word)| word.contains(&literal))
            .map(|(ord, word)| (word.as_bytes().to_vec(), ord as u64)).collect();
        prop_assert_eq!(matches(&dict, &literal, true), expected.clone());
        dict.load_pruning().unwrap();
        prop_assert_eq!(matches(&dict, &literal, true), expected);
    }
}
