# Six-bit counts: absolute document IDs versus per-list deltas

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, message field 3. Every cutoff **0–32** was rerun in the isolated six-bit-count experiment. Production remains unchanged at ≤8 with absolute IDs.

## Encoding and controls

The first document ID in each list remains absolute. Each subsequent value is the difference from the previous ID. Delta state resets at every list boundary. The resulting u32 values use the same four zstd-compressed byte lanes, compression level, six-bit count/position entries, and dictionary block boundaries as the absolute control.

Only document-ID lane bytes, their length prefixes, enclosing payload/block lengths, and the block index change. Metadata, keys, frequencies, positions, and external ranges are preserved. The alternative retains the absolute-format metadata and version for this controlled measurement. It is verified and measured in memory; it is **not emitted as a usable delta dictionary format**. Production has no new delta-format reader.

## Results

Decimal MB. Total includes dictionary + external postings + external positions. Savings in the final column are relative to the original **516.500105 MB**.

| Maximum documents | Absolute total MB | Delta total MB | Extra saved by deltas MB | Delta total reduction |
|---|---:|---:|---:|---:|
| 0 | 516.500 | 516.500 | 0.000 | 0.00% |
| 8 | 491.230 | 491.205 | 0.025 | 4.90% |
| 12 | 487.605 | 487.249 | 0.356 | 5.66% |
| 16 | 481.071 | 479.766 | 1.304 | 7.11% |
| 20 | 473.643 | 471.058 | 2.585 | 8.80% |
| 24 | 467.979 | 464.333 | 3.646 | 10.10% |
| 28 | 464.658 | 460.331 | 4.327 | 10.88% |
| 32 | 462.899 | 458.190 | 4.709 | 11.29% |

At cutoff 32:

- Dictionary: **129,425,977 → 124,717,390 bytes**, saving **4,708,587 bytes (3.64%)**.
- Combined total: **462,898,563 → 458,189,976 bytes**, an additional **1.02%** reduction versus absolute IDs at the same cutoff.
- Saving versus the original: **58,310,129 bytes (11.29%)**.
- External postings remain **154,031,512 bytes** and external positions remain **179,441,074 bytes**.
- **16,247,687 of 23,784,883 document entries (68.31%)** become gaps. At cutoff 8, only **9.14%** become gaps, so the effect is much smaller.
- Document-ID streams save **4,708,491 bytes** and the index/footer saves **96 bytes**. Almost all lane savings come from the third byte lane.

Cutoffs 0 and 1 produce identical absolute and delta sizes. At cutoffs 2–6, deltas slightly increase size, by 1,580–7,829 bytes. At 7 they save 1,209 bytes, and at 8 they save 24,804 bytes. The previously measured four-bit cutoff-8 layout saved 24,836 bytes; its document-lane savings are the same, while the index-size effect differs.

The delta totals decrease at every cutoff through 32, making **32 the smallest tested total**. This establishes a storage result, not a query-latency optimum.

## Every cutoff

All positive cutoffs below use the same six-bit layout. A negative delta saving means the delta dictionary is larger. Decimal kB = 1,000 bytes.

| Maximum documents | Absolute dictionary MB | Delta dictionary MB | Absolute total MB | Delta total MB | Saved by deltas kB | Delta total reduction |
|---|---:|---:|---:|---:|---:|---:|
| 0 | 51.678 | 51.678 | 516.500 | 516.500 | 0.000 | 0.00% |
| 1 | 71.826 | 71.826 | 492.871 | 492.871 | 0.000 | 4.57% |
| 2 | 72.000 | 72.002 | 492.787 | 492.789 | -1.580 | 4.59% |
| 3 | 72.164 | 72.169 | 492.678 | 492.683 | -4.444 | 4.61% |
| 4 | 72.394 | 72.401 | 492.524 | 492.531 | -6.985 | 4.64% |
| 5 | 72.718 | 72.726 | 492.316 | 492.324 | -7.829 | 4.68% |
| 6 | 73.180 | 73.186 | 492.048 | 492.054 | -5.436 | 4.73% |
| 7 | 73.803 | 73.802 | 491.693 | 491.692 | 1.209 | 4.80% |
| 8 | 74.616 | 74.591 | 491.230 | 491.205 | 24.804 | 4.90% |
| 9 | 75.674 | 75.613 | 490.605 | 490.543 | 61.570 | 5.03% |
| 10 | 77.046 | 76.922 | 489.804 | 489.680 | 124.293 | 5.19% |
| 11 | 78.779 | 78.558 | 488.811 | 488.591 | 220.470 | 5.40% |
| 12 | 80.901 | 80.545 | 487.605 | 487.249 | 355.957 | 5.66% |
| 13 | 83.420 | 82.888 | 486.207 | 485.674 | 532.667 | 5.97% |
| 14 | 86.322 | 85.570 | 484.636 | 483.884 | 751.735 | 6.31% |
| 15 | 89.575 | 88.563 | 482.902 | 481.890 | 1012.796 | 6.70% |
| 16 | 93.078 | 91.773 | 481.071 | 479.766 | 1304.493 | 7.11% |
| 17 | 96.776 | 95.158 | 479.170 | 477.552 | 1617.620 | 7.54% |
| 18 | 100.523 | 98.580 | 477.273 | 475.331 | 1942.273 | 7.97% |
| 19 | 104.219 | 101.950 | 475.410 | 473.141 | 2269.621 | 8.39% |
| 20 | 107.759 | 105.174 | 473.643 | 471.058 | 2585.391 | 8.80% |
| 21 | 111.083 | 108.197 | 472.001 | 469.115 | 2886.062 | 9.17% |
| 22 | 114.144 | 110.978 | 470.501 | 467.334 | 3166.198 | 9.52% |
| 23 | 116.898 | 113.477 | 469.152 | 465.731 | 3420.600 | 9.83% |
| 24 | 119.314 | 115.668 | 467.979 | 464.333 | 3646.031 | 10.10% |
| 25 | 121.417 | 117.577 | 466.959 | 463.118 | 3840.735 | 10.34% |
| 26 | 123.180 | 119.148 | 466.057 | 462.025 | 4032.371 | 10.55% |
| 27 | 124.685 | 120.490 | 465.299 | 461.104 | 4195.430 | 10.73% |
| 28 | 125.979 | 121.652 | 464.658 | 460.331 | 4326.866 | 10.88% |
| 29 | 127.065 | 122.621 | 464.110 | 459.666 | 4444.004 | 11.00% |
| 30 | 127.973 | 123.429 | 463.646 | 459.102 | 4544.005 | 11.11% |
| 31 | 128.755 | 124.121 | 463.247 | 458.614 | 4633.519 | 11.21% |
| 32 | 129.426 | 124.717 | 462.899 | 458.190 | 4708.587 | 11.29% |

## Verification

- All 33 absolute controls reproduce the previous six-bit measurement's complete size breakdowns, external sizes, and entry counts exactly.
- For each cutoff, all 7,671,280 terms in the absolute dictionary are reopened and checked against the source, including embedded document IDs/frequencies/position deltas and remapped external offsets.
- The absolute rewrite control reproduces each complete dictionary byte-for-byte, including the rebuilt index and footer.
- Every transformed document ID is decoded and reconstructed with checked prefix sums, then compared with its absolute source value. Frequency/position bytes and metadata/key parts are copied unchanged.
- Parsed embedded-list and document-entry counts match the independently counted source lists at every cutoff. The 0/1 no-change controls and all total-size arithmetic pass.
- Five example tests pass. A new regression covers every list length 1–32, descending first-ID anchors across lists, and a complete mixed-length dictionary rewrite. Scoped Clippy passes with warnings denied.

Exact results: [`six-bit-deltas.json`](six-bit-deltas.json). In that file, `after` is the absolute control, `delta_after` contains the delta sizes, and `per_list_doc_id_deltas` contains the lane measurements and roundtrip-control result. The absolute-only experiment remains in [`SIX_BIT_COUNTS.md`](SIX_BIT_COUNTS.md).

## Reproduction

Historical recipe: these patches target the implementation before dictionary-level inline-size configuration. Current measurements derive the packing width from the dictionary maximum, while this saved experiment uses six bits at every positive cutoff.

Apply the existing six-bit patch and the additional delta-sweep patch to a fresh copy of this worktree. The additional patch enables the existing delta comparison for every cutoff and adds the regression test. It does not change the compression algorithm.

```sh
# Run from this worktree. Include its existing uncommitted implementation/examples.
copy=$(mktemp -d /tmp/short-postings-deltas.XXXXXX)
rsync -a --exclude=.git --exclude=target --exclude=experiments ./ "$copy/"
patches="$PWD/experiments/inline-postings"
cd "$copy"
git apply "$patches/six-bit-counts.patch"
git apply "$patches/six-bit-delta-sweep.patch"
cargo test --example short_postings_size --features quickwit
cargo clippy --example short_postings_size --features quickwit -- -D warnings
cargo run --release --example short_postings_size --features quickwit -- \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.term \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.idx \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.pos sweep
```

The raw command output contains the absolute sizes and nested delta measurements. `delta_after.total_bytes` in the saved report is calculated as delta dictionary bytes + unchanged external postings bytes + unchanged external positions bytes. Both patch SHA-256 values are recorded in the JSON.

## Limits

External sizes model byte-preserving compaction. No complete split or usable delta-format dictionary is emitted or deployed. Query fetch plans, cache behavior, live latency, and delta decoding throughput at these cutoffs remain unmeasured. The figures cover the message field, excluding the unchanged token-count header, composite metadata, component footers, other fields, and document retrieval.
