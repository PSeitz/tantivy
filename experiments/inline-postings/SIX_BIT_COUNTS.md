# Experimental six-bit counts: inline limits 0–32

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, message field 3, 7,671,280 terms. Every integer cutoff from **0 through 32** was measured with absolute document IDs.

This is an **isolated encoding experiment**. Production remains at ≤8 with four-bit count/position entries. The working repository's production writer, reader, validator, and indexing threshold were left unchanged.

The follow-up [`SIX_BIT_DELTAS.md`](SIX_BIT_DELTAS.md) compares per-list document-ID deltas at every cutoff 0–32. At ≤32, deltas save another 4.709 MB, reducing the combined total to 458.190 MB (−11.29% versus the original).

## Layout

The experimental entry uses **five bits for document count minus one**, plus **one bit indicating positions**. Six-bit entries are packed consecutively, least-significant bits first. Four entries occupy three bytes. Unused high bits in the final byte must be zero.

All experimental nonzero cutoffs use this same six-bit layout, including cutoffs 1–8. Document IDs remain absolute. Frequency tagging, position deltas, four zstd-compressed byte lanes per stream, compression level, block framing, and key-based block boundaries remain unchanged. No whole list longer than the selected cutoff is embedded.

The copied codec uses experimental block version 2 and validates lists through 32 documents. It preserves decoding of the original all-external source format. It is not a production format migration: production version-1 inline blocks are not accepted by this experimental decoder. The indexing serializer still has its original ≤8 threshold; this example builds the wider-cutoff dictionaries directly from decoded source lists.

## Summary

Decimal MB. Total includes dictionary + external postings + external positions. External sizes model byte-preserving compaction.

| Layout / cutoff | Dictionary MB | Total MB | Saved MB versus original | Reduction |
|---|---:|---:|---:|---:|
| Original / rebuilt 0 | 51.678 | 516.500 | 0.000 | 0.00% |
| Production four-bit / ≤8 | 74.501 | 491.115 | 25.385 | 4.91% |
| Experimental six-bit / ≤8 | 74.616 | 491.230 | 25.270 | 4.89% |
| Experimental six-bit / ≤16 | 93.078 | 481.071 | 35.430 | 6.86% |
| Experimental six-bit / ≤32 | 129.426 | 462.899 | 53.602 | 10.38% |

- At cutoff 8, widening the count/position entry adds **115,053 bytes**: 114,973 bytes of compressed metadata and 80 bytes of index/footer growth. Document/frequency/position lanes, keys, framing, external sizes, and embedded entry counts are identical to the four-bit control.
- Cutoff 32 saves **53,601,542 bytes (10.38%)** versus the original. This is **28,216,065 additional bytes saved** versus production's four-bit cutoff 8.
- That additional net saving comes from **83,140,847 fewer external bytes**, offset by **54,924,782 more dictionary bytes**. The dictionary grows from 74.501 MB to 129.426 MB.
- At cutoff 32, **7,537,196 lists**, **23,784,883 document entries**, and **24,078,623 position entries** are embedded.
- Combined storage decreases at every step through 32. **32 is the smallest total among the tested cutoffs.** No optimum beyond 32 is inferred.

## Every cutoff with the experimental six-bit layout

The original baseline is **516,500,105 bytes**. The limit-0 rebuild matches all original bytes except the final dictionary-type marker.

| Maximum documents | Dictionary MB | External postings MB | External positions MB | Total MB | Saved MB | Reduction |
|---|---:|---:|---:|---:|---:|---:|
| 0 | 51.678 | 253.765 | 211.057 | 516.500 | 0.000 | 0.00% |
| 1 | 71.826 | 222.905 | 198.140 | 492.871 | 23.629 | 4.57% |
| 2 | 72.000 | 222.712 | 198.075 | 492.787 | 23.713 | 4.59% |
| 3 | 72.164 | 222.503 | 198.011 | 492.678 | 23.822 | 4.61% |
| 4 | 72.394 | 222.205 | 197.924 | 492.524 | 23.976 | 4.64% |
| 5 | 72.718 | 221.791 | 197.807 | 492.316 | 24.184 | 4.68% |
| 6 | 73.180 | 221.220 | 197.648 | 492.048 | 24.452 | 4.73% |
| 7 | 73.803 | 220.454 | 197.435 | 491.693 | 24.807 | 4.80% |
| 8 | 74.616 | 219.453 | 197.160 | 491.230 | 25.270 | 4.89% |
| 9 | 75.674 | 218.132 | 196.798 | 490.605 | 25.896 | 5.01% |
| 10 | 77.046 | 216.425 | 196.333 | 489.804 | 26.696 | 5.17% |
| 11 | 78.779 | 214.281 | 195.752 | 488.811 | 27.689 | 5.36% |
| 12 | 80.901 | 211.661 | 195.043 | 487.605 | 28.895 | 5.59% |
| 13 | 83.420 | 208.575 | 194.211 | 486.207 | 30.293 | 5.87% |
| 14 | 86.322 | 205.051 | 193.263 | 484.636 | 31.864 | 6.17% |
| 15 | 89.575 | 201.119 | 192.207 | 482.902 | 33.598 | 6.50% |
| 16 | 93.078 | 196.914 | 191.079 | 481.071 | 35.430 | 6.86% |
| 17 | 96.776 | 192.499 | 189.895 | 479.170 | 37.330 | 7.23% |
| 18 | 100.523 | 188.048 | 188.702 | 477.273 | 39.227 | 7.59% |
| 19 | 104.219 | 183.664 | 187.527 | 475.410 | 41.090 | 7.96% |
| 20 | 107.759 | 179.480 | 186.404 | 473.643 | 42.857 | 8.30% |
| 21 | 111.083 | 175.566 | 185.353 | 472.001 | 44.499 | 8.62% |
| 22 | 114.144 | 171.972 | 184.385 | 470.501 | 45.999 | 8.91% |
| 23 | 116.898 | 168.742 | 183.512 | 469.152 | 47.348 | 9.17% |
| 24 | 119.314 | 165.918 | 182.746 | 467.979 | 48.521 | 9.39% |
| 25 | 121.417 | 163.464 | 182.077 | 466.959 | 49.541 | 9.59% |
| 26 | 123.180 | 161.373 | 181.504 | 466.057 | 50.443 | 9.77% |
| 27 | 124.685 | 159.600 | 181.014 | 465.299 | 51.201 | 9.91% |
| 28 | 125.979 | 158.087 | 180.592 | 464.658 | 51.843 | 10.04% |
| 29 | 127.065 | 156.812 | 180.233 | 464.110 | 52.390 | 10.14% |
| 30 | 127.973 | 155.742 | 179.931 | 463.646 | 52.854 | 10.23% |
| 31 | 128.755 | 154.824 | 179.669 | 463.247 | 53.253 | 10.31% |
| 32 | 129.426 | 154.032 | 179.441 | 462.899 | 53.602 | 10.38% |

## Validation

- Source component footers and CRCs are checked. Source postings and positions ranges must be contiguous and account for all input bytes.
- Every resulting dictionary is reopened. All **7,671,280 terms at each of 33 cutoffs** are verified: keys, document counts, complete embedded document/frequency/position arrays, and remapped external offsets. No dictionary-only size estimate substitutes for an encoded output.
- All dictionary bytes reconcile to the recorded stream, metadata, key, framing, and index/footer categories.
- The limit-0 body is byte-identical to the original. Across cutoffs 1–8, all payload-lane sizes, key/framing sizes, external sizes, and entry counts match the four-bit production measurements. Only compressed metadata and index/footer sizes change.
- Codec tests cover document counts 1–32 with and without positions, byte-lane roundtrips and truncation, tagged positions, all six-bit padding alignments, and rejection of length 33. The focused library run passed 14 tests. All four example tests and scoped example Clippy passed.
- Arithmetic and monotonicity checks passed for all 33 measurements.

Exact bytes and stream breakdowns: [`six-bit-cutoffs.json`](six-bit-cutoffs.json). Four-bit controls remain in [`production-cutoffs.json`](production-cutoffs.json).

## Reproduction

Historical recipe: the patch below targets the implementation before dictionary-level inline-size configuration. The current example supports 0–32 directly and derives each dictionary's packing width. Its output therefore differs from this fixed-six-bit experiment.

[`six-bit-counts.patch`](six-bit-counts.patch) contains the experiment-only changes to the codec, validation limit, size-example cutoff sweep, and its metadata parser. Apply it only to a **separate copy** of this worktree, including the existing uncommitted production implementation and examples:

```sh
# Run from this worktree. Use a fresh destination.
copy=$(mktemp -d /tmp/short-postings-32.XXXXXX)
rsync -a --exclude=.git --exclude=target --exclude=experiments ./ "$copy/"
patch_file="$PWD/experiments/inline-postings/six-bit-counts.patch"
cd "$copy"
git apply "$patch_file"
cargo test --lib --features quickwit short_postings
cargo test --example short_postings_size --features quickwit
cargo clippy --example short_postings_size --features quickwit -- -D warnings
cargo run --release --example short_postings_size --features quickwit -- \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.term \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.idx \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.pos sweep
```

The recorded patch SHA-256 is included in the JSON. The measurement command emits dictionaries only in memory and writes measurements to stdout.

## Limits

No full split was rewritten or deployed. New cutoffs' query fetch plans, live latency, indexing throughput, and dictionary decoding cost remain unmeasured. The much larger dictionary is a cache/read-volume trade-off, even though total storage improves. A five-bit layout specialized for limits 9–16 was not tested. These figures cover the message field's three components, excluding the unchanged token-count header, composite metadata, component footers, other fields, and document retrieval.
