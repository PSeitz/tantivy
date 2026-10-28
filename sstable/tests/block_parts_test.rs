use std::io;

use common::OwnedBytes;
use tantivy_sstable::{BlockReader, encode_block_parts};

fn prepend_block_length(body: &[u8]) -> Vec<u8> {
    let mut bytes = (body.len() as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(body);
    bytes
}

fn decode_block_body(body: &[u8]) -> io::Result<Vec<u8>> {
    let mut reader = BlockReader::new(OwnedBytes::new(prepend_block_length(body)));
    assert!(reader.read_block()?);
    Ok(reader.buffer().to_vec())
}

#[test]
fn roundtrip_parts() {
    for (metadata, payload, keys) in [
        (vec![], vec![], vec![]),
        (b"meta".to_vec(), b"payload".to_vec(), b"keys".to_vec()),
        (vec![7; 4000], vec![8; 4000], vec![9; 4000]),
    ] {
        let mut body = Vec::new();
        encode_block_parts(&metadata, &payload, &keys, &mut body).unwrap();
        assert_eq!(body[0], 2);
        let mut pos = 1;
        for (part_index, original) in [&metadata, &payload, &keys].into_iter().enumerate() {
            let stored_len = u32::from_le_bytes(body[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let codec = body[pos + 8];
            if part_index == 1 || !cfg!(feature = "zstd-compression") || original.len() < 100 {
                assert_eq!(codec, 0);
            } else {
                assert_eq!(codec, 1);
                assert!(stored_len < original.len());
            }
            pos += 9 + stored_len;
        }
        assert_eq!(
            decode_block_body(&body).unwrap(),
            [metadata, payload, keys].concat()
        );
    }
}

#[test]
fn rejects_corrupt_parts() {
    let mut body = Vec::new();
    encode_block_parts(b"meta", b"payload", b"keys", &mut body).unwrap();
    // Truncated length, missing flag, and incomplete body. Part truncations are checked below.
    let bytes = prepend_block_length(&body);
    for len in [1, 2, 3, 4, bytes.len() - 1] {
        let mut reader = BlockReader::new(OwnedBytes::new(bytes[..len].to_vec()));
        assert!(reader.read_block().is_err(), "outer truncation {len}");
    }
    for len in 1..body.len() {
        assert!(decode_block_body(&body[..len]).is_err(), "part truncation {len}");
    }
    for (offset, replacement) in [(1, u32::MAX), (5, u32::MAX), (1, 3)] {
        let mut corrupt = body.clone();
        corrupt[offset..offset + 4].copy_from_slice(&replacement.to_le_bytes());
        assert!(decode_block_body(&corrupt).is_err());
    }
    for offset in [9, 22, 38] {
        let mut corrupt = body.clone();
        corrupt[offset] = 99;
        assert!(decode_block_body(&corrupt).is_err());
    }
    let mut trailing = body.clone();
    trailing.push(0);
    assert!(decode_block_body(&trailing).is_err());
    assert!(decode_block_body(&[99, 0]).is_err());

    body[9] = 1;
    #[cfg(feature = "zstd-compression")]
    assert!(decode_block_body(&body).is_err());
    #[cfg(not(feature = "zstd-compression"))]
    {
        assert_eq!(
            decode_block_body(&body).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(
            decode_block_body(&[1, 0]).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
    body[9] = 0;
    body[22] = 1; // The payload must remain raw.
    assert_eq!(
        decode_block_body(&body).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[cfg(feature = "zstd-compression")]
#[test]
fn rejects_decoded_length_mismatch() {
    let mut body = Vec::new();
    encode_block_parts(&vec![0; 4000], b"", b"", &mut body).unwrap();
    for length in [0u32, 3999, 4001, u32::MAX] {
        body[1..5].copy_from_slice(&length.to_le_bytes());
        assert!(decode_block_body(&body).is_err());
    }
}
