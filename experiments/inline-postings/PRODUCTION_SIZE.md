# Production size: message field on KH4Y_01M5MTJQ3SVHDTK2W3XCB5

Measured with the production dictionary writer: whole lists of up to eight documents, absolute document IDs, separate compressed byte lanes, and singleton positions embedded in tagged frequencies. This differs from the older byte-preserving prototype in `MEASUREMENTS.md` and `COALESCING.md`.

## Results

Decimal MB (1 MB = 1,000,000 bytes). Postings and positions columns count external storage only. The new dictionary includes embedded postings and positions.

| Component | Original bytes | New bytes | Change bytes | Change |
|---|---:|---:|---:|---:|
| Message dictionary | 51,677,994 | 74,501,195 | +22,823,201 | +44.16% |
| External postings | 253,765,218 | 219,453,464 | −34,311,754 | −13.52% |
| Dictionary + external postings | 305,443,212 | 293,954,659 | −11,488,553 | −3.76% |
| External positions | 211,056,893 | 197,159,969 | −13,896,924 | −6.58% |
| **Dictionary + postings + positions** | **516,500,105** | **491,114,628** | **−25,385,477** | **−4.91%** |

The dictionary grows **22.82 MB**, replacing **48.21 MB** of external postings and positions. Net saving: **25.39 MB** for this field.

- Terms: 7,671,280.
- Embedded lists: 6,597,536.
- Embedded document entries: 7,261,570. A document can contribute to multiple lists.
- Embedded position entries: 7,299,388.

The absolute-ID sweep for maximum list lengths **0, 1, 2, 3, 4, 5, 6, 7, and 8** is in [`PRODUCTION_CUTOFFS.md`](PRODUCTION_CUTOFFS.md). Total size ranges from 492.833 MB at ≤1 to 491.115 MB at ≤8.

## Dictionary growth breakdown

The following contributions sum exactly to the **22,823,201-byte dictionary increase**. Stream sizes include their VInt length prefixes and zstd frame overhead.

| Category | Change bytes | Change decimal MB |
|---|---:|---:|
| Embedded document IDs | +22,582,300 | +22.582 |
| Embedded frequencies + singleton positions | +3,090,455 | +3.090 |
| Remaining embedded positions (frequency >1) | +305,100 | +0.305 |
| Keys + term metadata | −3,610,064 | −3.610 |
| Block framing | +453,438 | +0.453 |
| Block index + footers | +1,972 | +0.002 |
| **Total dictionary growth** | **+22,823,201** | **+22.823** |

The three embedded streams occupy **25,977,855 bytes**. Changes to keys, metadata, framing, and the index offset that by **3,154,654 bytes**.

The new dictionary stores 44,281,242 bytes of encoded keys and 3,507,050 bytes of encoded term metadata, compared with 51,398,356 bytes of jointly encoded keys/metadata in the original dictionary. Because the original block compressed both together, their individual contributions to that decrease cannot be separated exactly. New metadata includes the inline markers and packed count nibbles as well as external term metadata.

The frequency stream embeds the sole position for frequency-one entries. Its 3.090 MB therefore covers frequencies **and** these positions. Compression operates on the combined tagged values, so there is no separate exact byte count for either component within this stream.

The breakdown is parsed from the actual encoded blocks. Every byte is assigned once, including block headers, lane-length prefixes, the block index, and footer/trailer bytes. Raw per-byte-lane sizes are in `production-size.json`. The measurement tool also tests accounting for both legacy and short-postings blocks.

## Per-list document-ID deltas

Compared absolute IDs with `first_doc, doc[1] - doc[0], doc[2] - doc[1], …`, resetting at every short list. Both use the same four byte lanes and zstd level 3. Frequencies, positions, keys, metadata, and block boundaries are unchanged. The block index is rebuilt for the changed block sizes.

| Component | Absolute IDs, bytes | Per-list deltas, bytes | Saving |
|---|---:|---:|---:|
| Document-ID streams, including length prefixes | 22,582,300 | 22,557,496 | 24,804 (0.110%) |
| Whole dictionary | 74,501,195 | 74,476,359 | 24,836 (0.0333%) |
| Dictionary + external postings + external positions | 491,114,628 | 491,089,792 | 24,836 (0.00506%) |

Only **664,034 of 7,261,570 document entries (9.14%)** become gaps. Each of the 6,597,536 lists retains its first absolute ID. The two low-byte lanes keep the same compressed size. The third lane saves 24,804 bytes; the high-byte lane and length prefixes stay the same size. The rebuilt index saves another 32 bytes.

This experiment restores and verifies every document ID after decompressing the alternative lanes. Rebuilding with absolute IDs reproduces the complete production dictionary byte-for-byte. Tests cover delta resets across lists and the existing byte-accounting checks. The experiment lives in `examples/short_postings_size/encodings.rs` and runs with the command below.

**Production remains absolute-ID encoded.** The small size reduction does not establish a reason to change the format. This is an in-memory encoding experiment with the original metadata held fixed. It emits measurements rather than a usable delta-format dictionary. Decode throughput and query latency have not been measured. Cross-term deltas would be a separate experiment.

## Per-lane zstd plus an outer compression pass

Production already compresses each of the four byte lanes separately with zstd level 3. These experiments retain absolute document IDs and those per-lane frames, then try an outer zstd level-3 pass.

| Layout | Dictionary bytes | Change from production | Field total including external postings/positions |
|---|---:|---:|---:|
| Current: separately compressed lanes, metadata, and keys | 74,501,195 | — | 491,114,628 |
| Outer pass over raw metadata + compressed lane payload + raw keys | 79,311,420 | +4,810,225 (+6.46%) | 495,924,853 |
| Outer pass preserving all current inner frames, used only when smaller | 73,414,728 | −1,086,467 (−1.46%) | 490,028,161 |

The second alternative keeps the existing multipart block intact inside an outer frame. It compresses 15,424 of 16,794 blocks further and retains the original representation for the others. The extra **1.086 MB** saving is **0.221% of the current field total**. External postings and positions sizes stay unchanged in both experiments. Both alternatives rebuild the block index at their new offsets.

Checks and scope:

- The raw-metadata/raw-keys alternative uses existing block flags 0/1. The production reader opens it and all 7,671,280 terms, including their embedded document IDs, frequencies, and positions, match exactly.
- The preserved-inner-frames alternative uses an experiment-only nested-block tag in memory. Unwrapping it and rebuilding the original index reproduces the production dictionary **byte-for-byte**. It requires format dispatch and reader support before adoption. No nested-format dictionary is emitted to disk.
- Resident dictionary scans for the first alternative had medians of 681.2 ms for current production and 675.4 ms for the alternative. These are three interleaved samples per layout, with no CPU isolation. They do not establish a speed improvement. The preserved-inner-frames alternative has no production-reader throughput measurement.
- Production encoding remains unchanged. Live-query latency, indexing throughput, and deployment compatibility for a nested-block format remain untested.

## Method and checks

`examples/short_postings_size.rs` reads the original message field (field 3), decodes each short list's document IDs, frequencies, and position deltas, and writes the actual production dictionary encoding. External lists keep their original encoded bytes with compacted offsets. The dictionary is reopened and **every term** is checked: keys, document counts, external offsets, and all short document/frequency/position arrays. Two release runs produced identical sizes.

All three source component footers and CRCs were validated. The `.pos` component was fetched read-only from the same S3 split, bytes `[899245506, 1110302609)`. SHA256: `70d9bb1b03950ca893095e7c4c88d1d4b54cdb292e20bd3cd4c295037ea77e29`.

The dictionary byte count is measured from encoded output. External byte counts model byte-preserving compaction. No complete rewritten split is emitted. Counts exclude the unchanged eight-byte token count, composite-file metadata, component footers, and other fields. They do not include cache effects or live-query latency. The 512 KiB read-coalescing setting changes read plans rather than stored sizes.

Raw results: [`production-size.json`](production-size.json).

Reproduce:

```sh
cargo run --release --example short_postings_size --features quickwit -- \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.term \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.idx \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.pos
```
