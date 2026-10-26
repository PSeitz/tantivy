# Extract message dictionary and existing metadata

Read-only extraction of schema field 3 (`message`) from segment `4fea6c896fd045aba3cece0808f3e41e` in split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`. This standalone Cargo workspace uses the repository's public SSTable APIs. It does not change library code.

Run from the repository root:

```sh
cargo test --release --locked --manifest-path dictionary_analysis/extract/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/extract/Cargo.toml -- KH4Y_01M5MTJQ3SVHDTK2W3XCB5
python3 dictionary_analysis/extract/verify.py KH4Y_01M5MTJQ3SVHDTK2W3XCB5
```

Outputs go to `<input directory>/message_analysis/`, or an optional second argument selects a staging output directory. Existing extraction outputs in that directory are replaced. To preserve concurrent readers, extract into staging, compare immutable `terms.bin` and `message.sstable`, and atomically publish changed metadata. Input files are opened read-only. The source `.term` SHA-256 is compared before and after extraction, and its managed footer CRC32 is verified.

## Extraction

1. Verify `meta.json` schema field 3 is named `message`.
2. Parse the managed footer (JSON, little-endian u32 JSON byte length, little-endian u32 magic 1337). Verify CRC32 of its body.
3. Parse the composite footer with `common::VInt` and little-endian u32 field IDs. Select field 3, sub-index 0.
4. Remove the field dictionary's trailing little-endian u32 dictionary-type tag (2 = SSTable). Copy all remaining field bytes unchanged to `message.sstable`. Its own data, index, and footer remain intact.
5. Open `Dictionary` and enumerate `sstable_index.get_block_for_automaton(&AlwaysMatch)` for physical block IDs, addresses, and first ordinals. Read the v3 FST separately to export its private separators, checking FST IDs against the public enumeration.
6. Use the public `BlockReader` for decompression. Parse the actual TermInfo value header and delta keys. Export full raw keys, without a field prefix, normalization, UTF-8 conversion, or numeric conversion.
7. Run the exact current `get_block_for_automaton` for all seven requested byte-glob automata. This calls the repository's current `can_block_match_automaton`. Candidate flags are its actual decisions, rather than a recreated approximation.

This helper targets the observed multi-block v3 format. Unsupported versions fail explicitly. TermInfo values use stop-bit varints (`common::VInt`). Key delta lengths use continuation-bit varints. Block addresses are relative to the extracted dictionary, not the composite source file.

## Outputs

- `message.sstable`: unchanged dictionary bytes after removing only surrounding wrappers.
- `terms.bin`: repeated `u32 LE length` and exactly that many full key bytes. Terms follow physical block order and strict global byte sorting. No text dump of all keys is created.
- `blocks.jsonl`: required block schema with `existing_candidates` boolean fields `*a*`, `*ab*`, `*traa*`, `*X`, `*x`, `*player*`, `*ms*`. `terms_offset` points to the first key's length prefix in `terms.bin`. All byte ranges are half-open. `lower_hex` is the preceding stored separator, exclusive, or null on the first block. `upper_hex` is this block's stored separator, inclusive. Additional fields describe value headers, compression, byte lengths, strict numeric count, and the end of the block's terms stream.
- `existing_metadata.tsv`: decoded existing on-disk block metadata alongside explicitly prefixed `derived_` columns. UTF-8 display columns are JSON-escaped and use replacement characters for invalid UTF-8. Hex columns are authoritative and lossless.
- `address_store_metadata.tsv`: actual fixed-width v3 address-store group headers, without computed shifts. Each group header is 36 bytes. Addresses and ordinals in `existing_metadata.tsv` are decoded from these headers and the packed residuals.
- `extraction.json`: source provenance, hashes, wrapper removal, byte accounting, term and numeric counts, baseline candidate counts, format definitions, and successful validation checks.
- `validation.json`: independent Python verification of every exported term and block, source dictionary byte identity, and the SHA-256 of `terms.bin`.

## Stored versus derived

Stored: FST separator-to-block-ID mapping, address-store group headers and packed address/ordinal residuals, per-block 4-byte length and 1-byte compression flag, TermInfo values, delta-encoded keys, and SSTable footer index offset / total key count / format version. The address-store starts with an 8-byte group metadata length. Its 36-byte headers contain residual-byte offset, reference byte start, reference first ordinal, byte-start slope, ordinal slope, ordinal bit width, byte-start bit width, and group `block_len`. The v3 index ends with an 8-byte FST length.

Derived: preceding separator, exact first and last key, numeric count, min/max key length, expanded key stream offsets, uncompressed lengths, and candidate booleans. Physical term counts are independently counted from decoded keys and checked against both stored TermInfo value counts and adjacent first ordinals. Existing metadata stores no per-block numeric count, key-length summary, substring summary, or suffix summary.

Strict numeric means the entire nonempty raw key matches ASCII `[0-9]+`. Signed integers `[+-][0-9]+` and decimals `[+-]?[0-9]+[.][0-9]+` are counted separately, with no overlap with strict numeric. Percentages count distinct dictionary keys, not token occurrences or documents.

`*` in the custom byte-glob DFA accepts every byte, including newline, NUL, and invalid UTF-8. Matching is case-sensitive. Query order is `[*a*, *ab*, *traa*, *X, *x, *player*, *ms*]`. Indices 0–2 and 5–6 are substring matches. Only indices 3–4 match an uppercase/lowercase final byte. Exhaustive tests over short strings, including non-ASCII bytes and newline, compare DFA results against direct literal substring/suffix tests.

## Validation

The helper asserts contiguous physical byte ranges, one frame per indexed block, consecutive block IDs, first ordinal and per-block counts, global strict key sorting, all keys inside stored separators, stored value count matching decoded keys, a four-byte data terminator, output byte counts, extracted SSTable SHA-256 identity, managed CRC32, and unchanged source SHA-256.
