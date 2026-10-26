# Dictionary and external postings sizes through cutoff 128

Measured on split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, segment
`4fea6c896fd045aba3cece0808f3e41e`, message field 3 (7,671,280 terms).

The experiment re-encodes the dictionary with **per-list document-ID deltas**,
compressed byte lanes, tagged singleton positions, and a dictionary-level cutoff
(type 4). The first ID in each list is absolute. Subsequent values are gaps.
Short-postings codec version 2 identifies this encoding. The reader also accepts
version 1 with absolute IDs.
Count/positions entries use 4, 6, 7, and 8 bits at cutoffs 8, 32, 64, and 128.
The production limit remains **32**, with inlining disabled by default.

## Sizes

Decimal MB. Dictionary sizes include inline documents, frequencies, and positions.
Postings and positions columns contain only the remaining external data.

| Maximum inline documents | Dictionary MB | External postings MB | External positions MB | Total MB | Total reduction |
|---|---:|---:|---:|---:|---:|
| 0 | 51.678 | 253.765 | 211.057 | 516.500 | 0.00% |
| 1 | 71.786 | 222.905 | 198.140 | 492.831 | 4.58% |
| 2 | 71.940 | 222.712 | 198.075 | 492.726 | 4.60% |
| 3 | 72.163 | 222.503 | 198.011 | 492.677 | 4.61% |
| 4 | 72.391 | 222.205 | 197.924 | 492.521 | 4.64% |
| 5 | 72.646 | 221.791 | 197.807 | 492.244 | 4.70% |
| 6 | 73.092 | 221.220 | 197.648 | 491.961 | 4.75% |
| 7 | 73.698 | 220.454 | 197.435 | 491.587 | 4.82% |
| 8 | 74.477 | 219.453 | 197.160 | 491.090 | 4.92% |
| 9 | 75.667 | 218.132 | 196.798 | 490.598 | 5.01% |
| 10 | 76.977 | 216.425 | 196.333 | 489.735 | 5.18% |
| 11 | 78.615 | 214.281 | 195.752 | 488.648 | 5.39% |
| 12 | 80.602 | 211.661 | 195.043 | 487.306 | 5.65% |
| 13 | 82.939 | 208.575 | 194.211 | 485.726 | 5.96% |
| 14 | 85.615 | 205.051 | 193.263 | 483.929 | 6.31% |
| 15 | 88.605 | 201.119 | 192.207 | 481.932 | 6.69% |
| 16 | 91.810 | 196.914 | 191.079 | 479.803 | 7.11% |
| 32 | 124.717 | 154.032 | 179.441 | 458.190 | 11.29% |
| 64 | 138.647 | 134.993 | 174.284 | 447.924 | 13.28% |
| 128 | 143.411 | 128.280 | 172.472 | 444.163 | 14.01% |

- **32 → 64:** dictionary grows 13,929,581 bytes. External postings shrink
  19,038,513 bytes and positions shrink 5,156,828 bytes. Net saving: **10,265,760 bytes**.
- **64 → 128:** dictionary grows 4,763,794 bytes. External postings shrink
  6,712,702 bytes and positions shrink 1,812,372 bytes. Net saving: **3,761,280 bytes**.

## Compared with absolute document IDs

Only the dictionary changes. External postings and positions match the
[absolute-ID measurements](size-up-to-128-absolute.json) at every cutoff.

| Cutoff | Absolute dictionary MB | Delta dictionary MB | Bytes saved |
|---|---:|---:|---:|
| 8 | 74.501 | 74.477 | 24,448 |
| 16 | 93.114 | 91.810 | 1,304,049 |
| 32 | 129.426 | 124.717 | 4,708,905 |
| 64 | 145.202 | 138.647 | 6,555,395 |
| 128 | 150.681 | 143.411 | 7,270,577 |

Deltas have small costs at some cutoffs: 1–6 produce dictionaries 37–8,114 bytes
larger than absolute IDs. At cutoff 1 the document lanes are identical; the
37-byte increase comes from compressed metadata/index changes for version 2.
Decoding also requires a checked prefix sum per list.

### Decoding cost

The production reader was timed over full RAM-resident dictionary streams on an
Apple M4 Max (64 GiB, Rust 1.98 release build). Each stream visits all 7,671,280
terms. Both encodings use the same binary, two warmup rounds, and 12 measured
repeats with alternating order. File I/O and dictionary opening are excluded.

| Cutoff | Absolute median ms [range] | Delta median ms [range] | Median change |
|---|---:|---:|---:|
| 8 | 758.33 [755.14–778.73] | 767.15 [756.87–781.74] | +1.16% |
| 32 | 1,034.70 [1,023.34–1,068.91] | 1,053.27 [1,041.26–1,087.68] | +1.79% |

These elapsed times include decompression, allocations, validation, positions,
and a checksum over every inline document ID. Exact keys and full term values
were compared before timing. Shared-workstation variance limits precision.
The result shows a small CPU cost in this scan, not a query/network latency result.
Samples are in [`dictionary-scan.json`](dictionary-scan.json), with the benchmark
source in [`dictionary_scan.rs`](dictionary_scan.rs).

The smaller external files come with a larger dictionary. These storage sizes
exclude query execution and cache behavior. Cold dictionary read costs at the
larger cutoffs remain unmeasured. External query read plans are recorded in
[`REQUEST_BUDGET.md`](REQUEST_BUDGET.md).

## Method and validation

An isolated source copy permits dictionary values through 128 documents. It
constructs dictionaries directly, bypassing the index writer's inline buffering.
Every dictionary is reopened and **all 7,671,280 terms at each cutoff** are checked
against the source, including document IDs, frequencies, position deltas, and
remapped external ranges. Dictionary byte accounting reconciles the encoded size.

Exactly 128 documents use a full compressed source block rather than a VInt tail.
The experiment decodes these through the block reader. All five actual source
lists of that length were independently checked through the full postings reader,
including per-document positions. Targeted tests cover dictionary roundtrips at
64 and 128, version-1 compatibility, per-list accumulator resets, and
malformed-delta rejection.

The sweep covers every cutoff from 0 through 16, plus 32, 64, and 128. The
external sizes and entry counts match the absolute-ID sweep at all 20 cutoffs.
Cutoff 0 reproduces the absolute-ID dictionary exactly. It is one byte larger
than the original source due to its dictionary-level configuration byte.

Dictionary sizes are actual serialized sizes. External sizes model byte-preserving
compaction. No complete rewritten split is emitted. Counts exclude the unchanged
eight-byte token count, composite metadata, component footers, and other fields.
Supporting 128 in production also requires writer and postings-cursor changes at
the full-block boundary. The experiment does not implement those changes.

Exact results: [`size-up-to-128.json`](size-up-to-128.json).
The isolated modifications are preserved in
[`size-up-to-128.patch`](size-up-to-128.patch).

## Reproduce in an isolated copy

From this working tree, with the original split available locally:

```sh
experiment=$(mktemp -d /tmp/large-cutoff-size.XXXXXX)
rsync -a --exclude=.git --exclude=target ./ "$experiment/"
patch -d "$experiment" -p1 < experiments/inline-postings/size-up-to-128.patch
source=/tmp/wildcard-data/splits/KH4Y_01M5MTJQ3SVHDTK2W3XCB5/4fea6c896fd045aba3cece0808f3e41e
cargo run --manifest-path "$experiment/Cargo.toml" --release \
  --example short_postings_size --features quickwit -- \
  "$source.term" "$source.idx" "$source.pos" sweep > /tmp/large-cutoff-size.json
cargo test --manifest-path "$experiment/Cargo.toml" --release \
  --example short_postings_size --features quickwit
```

The run emits full measurements and fixture data. The saved JSON contains a compact
summary with exact sizes, entry counts, and verification counts.

To reproduce the RAM-scan comparison, set `DICTIONARY_DIR=/tmp/dictionary-scan`
on the measurement command above. This also writes absolute and delta dictionaries
at cutoffs 8 and 32, using the encoding-comparison helper to normalize each version.
Then run:

```sh
cp experiments/inline-postings/dictionary_scan.rs "$experiment/examples/"
cargo run --manifest-path "$experiment/Cargo.toml" --release \
  --example dictionary_scan --features quickwit -- /tmp/dictionary-scan 12 \
  > /tmp/dictionary-scan-timings.json
```

Use a separate target directory for each source copy, or force a rebuild when
switching copies. Verify dictionary byte sizes before timing to detect stale
same-package build artifacts.
