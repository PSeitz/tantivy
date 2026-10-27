use common::VInt;
use proptest::prelude::*;

use super::PostingsSerializer;
use crate::directory::{FileSlice, OwnedBytes};
use crate::postings::{BlockInfo, BlockSegmentPostings, SegmentPostings};
use crate::schema::IndexRecordOption;
use crate::{DocId, DocSet, TERMINATED};

fn serialize(serializer: &mut PostingsSerializer, docs: &[DocId]) -> Vec<u8> {
    for &doc in docs {
        serializer.write_doc(doc, 1);
    }
    let mut bytes = Vec::new();
    serializer
        .close_term(docs.len() as u32, &mut bytes)
        .unwrap();
    bytes
}

fn open(docs: &[DocId]) -> BlockSegmentPostings {
    let bytes = serialize(&mut PostingsSerializer::for_doc_set(), docs);
    BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap()
}

fn collect(mut postings: BlockSegmentPostings) -> Vec<DocId> {
    let mut docs = Vec::new();
    while !postings.docs().is_empty() {
        docs.extend_from_slice(postings.docs());
        postings.advance();
    }
    postings.advance();
    assert!(postings.docs().is_empty());
    docs
}

proptest! {
    #[test]
    fn test_doc_set_random_ids(mut docs in proptest::collection::vec(0..TERMINATED, 0..1025)) {
        docs.sort_unstable();
        docs.dedup();
        let postings = open(&docs);
        prop_assert_eq!(collect(postings.clone()), docs.clone());
        let mut ranks = postings;
        for (rank, &doc) in docs.iter().enumerate() {
            prop_assert_eq!(ranks.rank(doc), rank as u32);
        }
        prop_assert_eq!(ranks.rank(TERMINATED), docs.len() as u32);
    }
}

#[test]
fn test_doc_set_roundtrip_seek_rank_clone() {
    for count in [0, 1, 2, 127, 128, 129, 255, 256, 257, 1025] {
        for start in [0, 1, 100_000, TERMINATED - 10_000] {
            let docs: Vec<DocId> = (0..count).map(|index| start + index * 3).collect();
            let postings = open(&docs);
            assert_eq!(postings.doc_freq(), count);
            assert_eq!(collect(postings.clone()), docs);
            let mut ranks = postings.clone();
            let mut seeks = postings.clone();
            let mut targets = vec![0, TERMINATED];
            for &doc in &docs {
                targets.extend([doc.saturating_sub(1), doc, doc + 1]);
            }
            targets.sort_unstable();
            for target in targets {
                let rank = docs.partition_point(|&doc| doc < target);
                assert_eq!(ranks.rank(target), rank as u32);
                let within = seeks.seek(target);
                assert_eq!(
                    seeks.doc(within),
                    docs.get(rank).copied().unwrap_or(TERMINATED)
                );
            }
            let mut cursor = SegmentPostings::from_block_postings(postings, None);
            for &doc in &docs {
                assert_eq!(cursor.doc(), doc);
                cursor.advance();
            }
            assert_eq!(cursor.doc(), TERMINATED);
        }
    }

    let docs: Vec<DocId> = (0..400).map(|index| index * 2).collect();
    let mut postings = open(&docs);
    postings.seek(300);
    let cloned = postings.clone();
    postings.advance();
    assert_eq!(collect(cloned), docs[128..]);
    assert_eq!(collect(postings), docs[256..]);
}

#[test]
fn test_doc_set_zero_bit_blocks_and_independent_tail() {
    for start in [0, 1, 100_000, TERMINATED - 128] {
        let docs: Vec<DocId> = (start..start + 128).collect();
        let bytes = serialize(&mut PostingsSerializer::for_doc_set(), &docs);
        assert_eq!(&bytes[..4], &128u32.to_le_bytes());
        assert_eq!(&bytes[4..8], &start.to_le_bytes());
        // Header, first document, one-byte skip length, one Basic skip entry, no payload.
        assert_eq!(bytes.len(), 4 + 4 + 1 + 5);
        let postings = BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap();
        assert!(matches!(
            postings.skip_reader().block_info(),
            BlockInfo::BitPacked {
                doc_num_bits: 0,
                strict_delta_encoded: true,
                ..
            }
        ));
        assert_eq!(collect(postings), docs);
    }

    for second_start in [128, 100_000, TERMINATED - 130] {
        let docs: Vec<DocId> = (0..128).chain(second_start..second_start + 130).collect();
        let bytes = serialize(&mut PostingsSerializer::for_doc_set(), &docs);
        // Three first-doc entries, two zero-bit full blocks, two one-byte tail deltas.
        assert_eq!(bytes.len(), 4 + 3 * 4 + 1 + 2 * 5 + 2);
        let mut envelope = &bytes[16..];
        let skip_len = VInt::deserialize_u64(&mut envelope).unwrap() as usize;
        assert_eq!(skip_len, 10);
        assert_eq!(&envelope[skip_len..], &[0x80, 0x81]);
        assert_eq!(collect(open(&docs)), docs);
    }
}

#[test]
fn test_doc_set_empty_and_truncated_header() {
    let bytes = serialize(&mut PostingsSerializer::for_doc_set(), &[]);
    assert_eq!(bytes, vec![0; 4]);
    let mut postings = BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap();
    assert_eq!(postings.rank(TERMINATED), 0);
    assert!(collect(postings).is_empty());
    for bytes in [vec![], vec![0; 3], vec![1, 0, 0, 0]] {
        assert!(BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).is_err());
    }
}

#[test]
fn test_doc_set_serializer_reuse_and_clear() {
    let mut serializer = PostingsSerializer::for_doc_set();
    for count in [257, 0, 128, 1, 129] {
        let docs: Vec<DocId> = (0..count).collect();
        let bytes = serialize(&mut serializer, &docs);
        assert_eq!(
            collect(BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap()),
            docs
        );
    }
    for doc in 100_000..100_257 {
        serializer.write_doc(doc, 1);
    }
    serializer.clear();
    let bytes = serialize(&mut serializer, &[0, 1]);
    assert_eq!(
        collect(BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap()),
        [0, 1]
    );
}

#[test]
fn test_doc_set_count_validation() {
    let mut serializer = PostingsSerializer::for_doc_set();
    serializer.write_doc(1, 1);
    let mut bytes = Vec::new();
    assert_eq!(
        serializer.close_term(0, &mut bytes).unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert!(bytes.is_empty());
    serializer.close_term(1, &mut bytes).unwrap();
    assert_eq!(
        collect(BlockSegmentPostings::open_doc_set(OwnedBytes::new(bytes)).unwrap()),
        [1]
    );
}

#[test]
fn test_doc_set_rejects_invalid_ids() {
    for docs in [
        vec![1, 1],
        vec![2, 1],
        vec![TERMINATED],
        (0..128).chain([127]).collect(),
    ] {
        assert!(std::panic::catch_unwind(|| open(&docs)).is_err());
    }
}

#[test]
fn test_doc_set_reset_to_disk() {
    let mut postings = open(&(100_000..100_257).collect::<Vec<_>>());
    postings.seek(100_200);
    for count in [128, 1, 129, 0, 257] {
        let docs: Vec<DocId> = (1..=count).collect();
        let mut serializer = PostingsSerializer::new(0.0, IndexRecordOption::Basic, None);
        let bytes = serialize(&mut serializer, &docs);
        postings.reset(count, OwnedBytes::new(bytes)).unwrap();
        assert_eq!(collect(postings.clone()), docs);
    }
}

#[test]
fn test_doc_set_disk_format_unchanged() {
    let mut serializer = PostingsSerializer::new(0.0, IndexRecordOption::Basic, None);
    assert_eq!(serialize(&mut serializer, &[1, 4, 130]), [0x81, 0x83, 0xfe]);
    serializer.clear();
    let docs: Vec<DocId> = (0..256).collect();
    let bytes = serialize(&mut serializer, &docs);
    assert_eq!(bytes, [0x8a, 127, 0, 0, 0, 0x40, 255, 0, 0, 0, 0x40]);
    serializer.clear();
    let docs: Vec<DocId> = (1..129).collect();
    let bytes = serialize(&mut serializer, &docs);
    // Disk offset=0 still means a virtual -1, so a block starting at 1 needs one bit.
    assert_eq!(bytes.len(), 1 + 5 + 16);
    assert_eq!(bytes[5], 0x41);
    let postings = BlockSegmentPostings::open(
        128,
        FileSlice::from(bytes),
        IndexRecordOption::Basic,
        IndexRecordOption::Basic,
    )
    .unwrap();
    assert_eq!(collect(postings), docs);
}
