# Exact inverted trigram block postings

All observed within-term trigrams after full actual block LCP removal. Ten layouts match the prior inverted pair codec search: positive raw IDs, delta-varints, runs and bitmaps, each plain or per-row zstd level 3, plus adaptive plain and adaptive plain/zstd. Adaptive tests every codec in positive and complement polarity. Observed u24 vocabulary, u32 offsets, headers, tags and compression framing are charged. The smallest serialized row wins independently of queries. Prefix extensions are charged once. All percentages reference the original 195640-byte index, including augmented dictionaries. These are metadata/selectivity measurements, not production latency or index-size estimates.

## Serialized metadata

Cells are bytes (+% of original index reference). These totals include one prefix-extension sidecar. Byte and pair layouts reuse the validated best inverted postings from the previous study.

| Scenario | Blocks | Bytes | Pairs | Trigrams | Trigrams + bytes | Trigrams + pairs + bytes |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 16,794 | 5,323 (+2.72%) | 1,586,101 (+810.72%) | 38,068,570 (+19458.48%) | 38,073,845 (+19461.18%) | 39,659,898 (+20271.88%) |
| v4_canonical_10000000 | 100,876 | 231,360 (+118.26%) | 4,522,580 (+2311.68%) | 95,466,157 (+48796.85%) | 95,695,598 (+48914.13%) | 100,216,259 (+51224.83%) |
| v7_canonical_10000000 | 84,362 | 15,500 (+7.92%) | 1,585,438 (+810.39%) | 76,984,062 (+39349.86%) | 76,989,343 (+39352.56%) | 78,564,562 (+40157.72%) |
| alphanumeric36_10000000 | 103,582 | 8,801 (+4.50%) | 6,142,193 (+3139.54%) | 233,435,059 (+119318.68%) | 233,441,158 (+119321.79%) | 239,580,649 (+122459.95%) |

All tested trigram totals exceed the historical +100% / 195,640 B allowance. All four exceed the historical 90% player-skip target. These statements concern the tested layouts.

## Vocabulary and memberships

| Scenario | Observed trigrams | Block memberships | Density | Mean / median posting | Directory B | Prefix B |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 47796 | 52,785,814 | 6.576% | 1104.4 / 1011.0 | 334,590 | 48 |
| v4_canonical_10000000 | 48565 | 244,808,679 | 4.997% | 5040.8 / 1010.0 | 339,973 | 1919 |
| v7_canonical_10000000 | 48565 | 200,377,714 | 4.891% | 4126.0 / 1012.0 | 339,973 | 10219 |
| alphanumeric36_10000000 | 47782 | 352,466,791 | 7.121% | 7376.6 / 7432.0 | 334,492 | 2702 |

Each observed trigram has a three-byte value, one offset, and an independently decodable row. The possible u24 domain is 16,777,216 values. Only observed values are serialized. Directory size is a property of this format, rather than a lower bound for all exact representations.

## All ten tested layouts

Posting-file bytes, before adding prefix/byte/pair sidecars. Each fixed layout uses positive postings. Adaptive tests positive and absent-block raw IDs, delta-varints, runs and bitmaps. Adaptive-zstd also tests each plain or as an independent zstd-3 frame. Candidate frames are calculated once per row and shared across the ten layouts. Every file is written, reopened, every row decoded and compared, and SHA-256 recorded. Only the smallest layout is retained. The other nine files are deleted after verification.

| Scenario | adaptive | adaptive_zstd | bitmap | bitmap_zstd | delta | delta_zstd | raw | raw_zstd | runs | runs_zstd |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | 52,190,407 | 38,068,522 | 100,945,170 | 46,223,744 | 53,508,617 | 38,095,437 | 106,145,198 | 103,530,069 | 94,190,346 | 49,761,186 |
| v4_canonical_10000000 | 111,115,357 | 95,464,238 | 612,987,448 | 111,714,220 | 252,167,661 | 109,404,266 | 735,008,835 | 594,543,471 | 289,252,598 | 123,741,482 |
| v7_canonical_10000000 | 94,335,250 | 76,973,843 | 512,749,288 | 85,312,470 | 201,195,936 | 87,272,264 | 601,715,940 | 476,125,767 | 241,946,949 | 101,110,465 |
| alphanumeric36_10000000 | 353,097,473 | 233,432,357 | 619,254,738 | 288,823,376 | 353,125,181 | 233,440,431 | 1,057,973,775 | 867,452,993 | 651,883,928 | 317,284,312 |

### Winning codec rows

- baseline: adaptive_zstd, absent_bitmap_zstd=690, present_bitmap_zstd=260, present_delta=212, present_delta_zstd=45906, present_raw=628, present_runs=100
- v4_canonical_10000000: adaptive_zstd, absent_bitmap_zstd=2796, absent_delta_zstd=128, present_bitmap_zstd=1908, present_delta=350, present_delta_zstd=42853, present_raw=391, present_runs=107, present_runs_zstd=32
- v7_canonical_10000000: adaptive_zstd, absent_bitmap_zstd=2563, absent_delta_zstd=384, present_bitmap_zstd=1885, present_delta=473, present_delta_zstd=42780, present_raw=368, present_runs=112
- alphanumeric36_10000000: adaptive_zstd, present_delta=430, present_delta_zstd=46856, present_raw=398, present_runs=98

### Encoded frame bytes per feature in the winning layout

Frame lengths include codec/polarity tags, raw-length field and any compression framing. The vocabulary/offset directory is charged separately above.

| Scenario | Min | Median | Mean | P90 | P99 | Max |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 6.00 | 763.00 | 789.48 | 798.00 | 1,431.00 | 1,469.00 |
| v4_canonical_10000000 | 6.00 | 944.00 | 1,958.70 | 1,064.00 | 12,094.00 | 12,383.00 |
| v7_canonical_10000000 | 6.00 | 767.00 | 1,577.96 | 876.00 | 9,966.00 | 10,235.00 |
| alphanumeric36_10000000 | 6.00 | 4,938.00 | 4,878.36 | 5,057.00 | 6,047.00 | 6,208.00 |

## Full-prefix-safe queries

Cells are candidate blocks / skipped %. Oracle blocks contain at least one exact full-key match. `*X` and `*x` are case-sensitive suffix queries. The other five are substrings. Every mode has zero false-negative blocks. Trigrams alone retain all blocks for one- and two-byte queries. Adding bytes supplies byte fallback. Adding pairs supplies two-byte fallback too. All summaries are applied to each viable remainder continuation, including continuations of length 0, 1 or 2. The evaluator ORs prefix-contained, remainder-contained and prefix-crossing matches. `player` uses `pla`, `lay`, `aye`, `yer` only when those grams belong to the same continuation. A whole-literal gram intersection across a removed prefix would be unsafe.

### baseline

| Query | Exact terms | Added terms | Oracle blocks / skip | bytes | pairs | pairs_bytes | trigrams | trigrams_bytes | trigrams_pairs_bytes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.00% | 0 / 100.00% | 16,794 / 0.00% | 0 / 100.00% | 16,794 / 0.00% | 0 / 100.00% | 0 / 100.00% |
| *a* | 1921269 | 0 | 16,376 / 2.49% | 16,376 / 2.49% | 16,794 / 0.00% | 16,376 / 2.49% | 16,794 / 0.00% | 16,376 / 2.49% | 16,376 / 2.49% |
| *ab* | 75774 | 0 | 15,490 / 7.76% | 16,373 / 2.51% | 15,513 / 7.63% | 15,513 / 7.63% | 16,794 / 0.00% | 16,373 / 2.51% | 15,513 / 7.63% |
| *ms* | 50585 | 0 | 14,065 / 16.25% | 16,339 / 2.71% | 14,084 / 16.14% | 14,084 / 16.14% | 16,794 / 0.00% | 16,339 / 2.71% | 14,084 / 16.14% |
| *player* | 21 | 0 | 13 / 99.92% | 16,331 / 2.76% | 9,804 / 41.62% | 9,804 / 41.62% | 14 / 99.92% | 14 / 99.92% | 14 / 99.92% |
| *traa* | 42 | 0 | 38 / 99.77% | 16,341 / 2.70% | 11,818 / 29.63% | 11,818 / 29.63% | 141 / 99.16% | 141 / 99.16% | 141 / 99.16% |
| *x | 153990 | 0 | 16,022 / 4.60% | 16,354 / 2.62% | 16,794 / 0.00% | 16,354 / 2.62% | 16,794 / 0.00% | 16,354 / 2.62% | 16,354 / 2.62% |

### v4_canonical_10000000

| Query | Exact terms | Added terms | Oracle blocks / skip | bytes | pairs | pairs_bytes | trigrams | trigrams_bytes | trigrams_pairs_bytes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.00% | 0 / 100.00% | 100,876 / 0.00% | 0 / 100.00% | 100,876 / 0.00% | 0 / 100.00% | 0 / 100.00% |
| *a* | 10838333 | 8917064 | 100,465 / 0.41% | 100,465 / 0.41% | 100,876 / 0.00% | 100,465 / 0.41% | 100,876 / 0.00% | 100,465 / 0.41% | 100,465 / 0.41% |
| *ab* | 1153596 | 1077822 | 99,629 / 1.24% | 100,464 / 0.41% | 99,654 / 1.21% | 99,654 / 1.21% | 100,876 / 0.00% | 100,464 / 0.41% | 99,654 / 1.21% |
| *ms* | 50585 | 0 | 14,872 / 85.26% | 35,265 / 65.04% | 14,891 / 85.24% | 14,891 / 85.24% | 100,876 / 0.00% | 35,265 / 65.04% | 14,891 / 85.24% |
| *player* | 21 | 0 | 13 / 99.99% | 23,913 / 76.29% | 9,273 / 90.81% | 9,273 / 90.81% | 14 / 99.99% | 14 / 99.99% | 14 / 99.99% |
| *traa* | 42 | 0 | 38 / 99.96% | 35,259 / 65.05% | 11,223 / 88.87% | 11,223 / 88.87% | 142 / 99.86% | 142 / 99.86% | 141 / 99.86% |
| *x | 153990 | 0 | 21,856 / 78.33% | 51,202 / 49.24% | 100,876 / 0.00% | 51,202 / 49.24% | 100,876 / 0.00% | 51,202 / 49.24% | 51,202 / 49.24% |

### v7_canonical_10000000

| Query | Exact terms | Added terms | Oracle blocks / skip | bytes | pairs | pairs_bytes | trigrams | trigrams_bytes | trigrams_pairs_bytes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.00% | 0 / 100.00% | 84,362 / 0.00% | 0 / 100.00% | 84,362 / 0.00% | 0 / 100.00% | 0 / 100.00% |
| *a* | 10226774 | 8305505 | 83,929 / 0.51% | 83,929 / 0.51% | 84,362 / 0.00% | 83,929 / 0.51% | 84,362 / 0.00% | 83,929 / 0.51% | 83,929 / 0.51% |
| *ab* | 905113 | 829339 | 83,076 / 1.52% | 83,926 / 0.52% | 83,097 / 1.50% | 83,097 / 1.50% | 84,362 / 0.00% | 83,926 / 0.52% | 83,097 / 1.50% |
| *ms* | 50585 | 0 | 14,003 / 83.40% | 16,330 / 80.64% | 14,025 / 83.38% | 14,025 / 83.38% | 84,362 / 0.00% | 16,330 / 80.64% | 14,025 / 83.38% |
| *player* | 21 | 0 | 13 / 99.98% | 16,314 / 80.66% | 9,719 / 88.48% | 9,719 / 88.48% | 13 / 99.98% | 13 / 99.98% | 13 / 99.98% |
| *traa* | 42 | 0 | 38 / 99.95% | 16,334 / 80.64% | 11,806 / 86.01% | 11,806 / 86.01% | 139 / 99.84% | 139 / 99.84% | 137 / 99.84% |
| *x | 153990 | 0 | 16,007 / 81.03% | 16,343 / 80.63% | 84,362 / 0.00% | 16,343 / 80.63% | 84,362 / 0.00% | 16,343 / 80.63% | 16,343 / 80.63% |

### alphanumeric36_10000000

| Query | Exact terms | Added terms | Oracle blocks / skip | bytes | pairs | pairs_bytes | trigrams | trigrams_bytes | trigrams_pairs_bytes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.00% | 0 / 100.00% | 103,582 / 0.00% | 0 / 100.00% | 103,582 / 0.00% | 0 / 100.00% | 0 / 100.00% |
| *a* | 8293851 | 6372582 | 103,157 / 0.41% | 103,157 / 0.41% | 103,582 / 0.00% | 103,157 / 0.41% | 103,582 / 0.00% | 103,157 / 0.41% | 103,157 / 0.41% |
| *ab* | 342084 | 266310 | 97,405 / 5.96% | 103,156 / 0.41% | 97,507 / 5.86% | 97,507 / 5.86% | 103,582 / 0.00% | 103,156 / 0.41% | 97,507 / 5.86% |
| *ms* | 317095 | 266510 | 96,270 / 7.06% | 103,155 / 0.41% | 96,396 / 6.94% | 96,396 / 6.94% | 103,582 / 0.00% | 103,155 / 0.41% | 96,396 / 6.94% |
| *player* | 21 | 0 | 13 / 99.99% | 103,148 / 0.42% | 73,929 / 28.63% | 73,929 / 28.63% | 19 / 99.98% | 19 / 99.98% | 19 / 99.98% |
| *traa* | 240 | 198 | 228 / 99.78% | 103,152 / 0.42% | 84,542 / 18.38% | 84,542 / 18.38% | 975 / 99.06% | 975 / 99.06% | 971 / 99.06% |
| *x | 432136 | 278146 | 101,349 / 2.16% | 103,155 / 0.41% | 103,582 / 0.00% | 103,155 / 0.41% | 103,582 / 0.00% | 103,155 / 0.41% | 103,155 / 0.41% |

Adding pairs also improves the longer `*traa*` query through two-byte prefix-crossing continuations: 142→141 candidates on UUIDv4, 139→137 on UUIDv7, and 975→971 on alphanumeric. Those blocks have prefixes ending in `tr`; the remaining `aa` has no trigram and can be tested by its bigram. The `player` candidate counts are unchanged by either fallback on these four sources.

## Baseline comparison with the previous exact trigram study

The previous block-major exact bitset used 46,778,329 B including its old byte sidecar. Using today's best byte sidecar gives 46,763,297 B. Inverted trigrams + bytes use 38,073,845 B. All seven prefix-safe trigram+byte decisions agree for every baseline block. The six query trigram columns are compared against a streaming decode of the old exact bitset. Total all-trigram memberships also agree. The observed vocabulary is derived afresh, without assuming the old alphabet cube is fully observed.

## Validation, resources and limits

- Source SSTable SHA-256 and the complete length-prefixed decoded-key digest are verified. Every physical address, ordinal, term count, first/last key, bound, full LCP, remainder-byte mask and all seven exact full-term match counts are checked against the retained source records. Sources are preserved.
- All rows in all ten layouts round-trip exactly, including complementary and compressed rows. The winner is chosen by serialized size without query criteria. All query posting lists are then read from the retained winner through its directory. There is no query-selected vocabulary.
- Previous byte/pair/truth decisions agree per block. Prefix sidecars round-trip and equal the source sidecars. Baseline trigram+byte predicates agree per block with the old exact study. False positives remain because grams in a block can come from different terms or unrelated positions.
- Each helper process holds one scenario. The three augmented runs were sequential. The first baseline validation briefly overlapped UUIDv4 extraction in a separate process. RSS logs describe each process separately. The compressed SSTable and posting vectors are held in memory during extraction. Serialization, hashing and readback are streamed. Readback holds one decoded row. No block-by-trigram mask matrix or whole serialized-layout buffer is allocated. Evaluation holds six small per-block gram columns. The large old exact file is decompressed as a stream.
- All ten requested layouts completed on all four sources. No resource/offset overflow or validation failures occurred. The first Python report run stopped during artifact hashing because the installed Python lacks `hashlib.file_digest`. A portable 1 MiB chunked hashing loop fixed it. The failure log is retained and the full report was rerun. Untested layouts: joint trigram/bigram/byte compression, whole-file compression, other compression levels/codecs, production posting-intersection query planner. No global compression optimum, impossibility, production I/O or latency claim is made.

| Scenario | Source scan s | Encode s | Encode + verify + hash s | Posting Vec capacity B | Peak resident B | Peak temporary layout B |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 2.10 | 10.25 | 15.09 | 284,440,144 | 636,305,408 | 682,658,696 |
| v4_canonical_10000000 | 10.00 | 47.32 | 69.73 | 1,610,331,856 | 2,269,020,160 | 3,035,399,576 |
| v7_canonical_10000000 | 9.38 | 41.26 | 59.75 | 1,019,721,264 | 1,763,147,776 | 2,478,738,172 |
| alphanumeric36_10000000 | 11.16 | 62.24 | 92.22 | 1,663,255,552 | 3,070,427,136 | 4,975,768,564 |

The timing columns describe experiment construction and validation, not query latency. `/usr/bin/time -l` logs record peak resident memory separately from serialized metadata cost. Retained artifacts are the winning trigram file, prefix sidecar, compact per-block decisions, measurements, reports and logs. The byte/pair files remain in `pair_postings`. Nonwinning trigram files are reproducible from the preserved sources and have measured sizes/hashes in JSON. Diagnostic decision files, build-time memory and logs are excluded from deployable metadata cost.
