# Exact inverted bigram and unigram block postings

Exact within-term bigrams and unigrams after removing full actual block LCP. Rows indexed by sparse sorted feature vocabulary and byte offsets. Each posting independently accessible. Zstd level 3 is per posting. No whole-file compression. Best chosen by serialized bytes, without query criteria. Query evaluation uses reconstructed masks and prefix_summaries.permits, not a direct posting-intersection planner or runtime benchmark. All percentages use the original 195640-byte index as a reference for every scenario, not rebuilt production index estimates.

## Findings

- Inversion is useful for compressing byte summaries. On UUIDv4, byte+prefix metadata falls from **2,184,368 B to 231,360 B (89.41% smaller)** with exactly the same seven query decisions. The previous PM01 cost was a property of that interning layout. The new layout still costs +118.26% of the original index reference on v4. Baseline, v7 and alphanumeric byte+prefix costs fall to 5,323 B, 15,500 B and 8,801 B respectively.
- Exact pairs have substantially more information to store. Baseline has 1,443 observed pairs but **18,626,806 pair/block memberships**. Its raw postings occupy 37,269,503 B. Adaptive per-row compression reduces that to 1,586,053 B before the 48 B prefix sidecar. Only 8,676 B is the header/vocabulary/offset directory. The vocabulary is small. The per-pair block sets dominate cost.
- Dense baseline and alphanumeric pair rows mostly select compressed absent-ID deltas. UUIDv4 pair rows mostly select compressed positive runs. For v4 unigrams, 20 rows select compressed bitmaps, while the other scenarios chiefly use compact absent runs. The measured codec choices capture their different block-membership patterns.
- Pair-only and pairs+bytes have identical candidates on the four multi-byte queries here. The byte summary adds single-byte-query pruning. For `*player*`, pair candidates are **9,804 / 41.62% skipped** on baseline, **9,273 / 90.81%** on v4, **9,719 / 88.48%** on v7, and **73,929 / 28.63%** on alphanumeric. All four have 21 exact matching terms in 13 blocks. The large UUID skip percentages also reflect the larger block denominators. Alphanumeric adds many false-positive candidate blocks.
- Among these layouts, every exact pair index exceeds the historical +100% (195,640 B) metadata allowance. V4 reaches 90% player skipping at +2311.68% pair+prefix cost. Baseline inversion preserves the previous all-pairs pruning and costs more than the earlier 1,469,199 B grouped bitplane pair+byte summary. Per-pair independent access is a different layout. These are empirical comparisons, not lower-bound or production-performance claims.

## Serialized costs

Every byte below is present in a retained artifact, including headers, observed feature vocabulary, offsets, codec/polarity tags and zstd framing. Prefix cost is charged once. The prefix sidecar uses existing scenario index fences plus sparse full-LCP extensions. The figures are additional metadata, expressed as +% of the original index reference.

| Scenario | Blocks | Prefix B | Pairs + prefix | Inverted bytes + prefix | Old PM01 + prefix | Pairs + best bytes + prefix |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 16,794 | 48 | 1,586,101 (+810.72%) | 5,323 (+2.72%) | 20,355 (+10.40%) | 1,591,376 (+813.42%) |
| v4_canonical_10000000 | 100,876 | 1,919 | 4,522,580 (+2311.68%) | 231,360 (+118.26%) | 2,184,368 (+1116.52%) | 4,752,021 (+2428.96%) |
| v7_canonical_10000000 | 84,362 | 10,219 | 1,585,438 (+810.39%) | 15,500 (+7.92%) | 97,870 (+50.03%) | 1,590,719 (+813.08%) |
| alphanumeric36_10000000 | 103,582 | 2,702 | 6,142,193 (+3139.54%) | 8,801 (+4.50%) | 107,301 (+54.85%) | 6,148,292 (+3142.66%) |

Best byte layout compares inverted unigram formats with the previous PM01 mask interning. Pair+byte totals concatenate the independent summaries and one prefix sidecar. Joint pair/byte compression was not tested.

## Structure

65,536 is the possible bigram value count, not the index size. Each observed bigram needs a posting over block IDs. Raw IDs need two bytes on baseline and three on the 10m scenarios. An inverted index preserves the existing exact all-pairs information and its pruning. The same pairs can occur at unrelated positions or in different terms, so false positives remain.

| Scenario | Feature | Vocabulary | Memberships | Density | Mean list | Median list | Best format | Posting file B |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | pairs | 1443 | 18,626,806 | 76.86% | 12908.4 | 14041.0 | adaptive_zstd | 1,586,053 |
| baseline | bytes | 41 | 595,304 | 86.46% | 14519.6 | 16354.0 | adaptive_zstd | 5,275 |
| v4_canonical_10000000 | pairs | 1475 | 43,911,780 | 29.51% | 29770.7 | 14914.0 | adaptive_zstd | 4,520,661 |
| v4_canonical_10000000 | bytes | 42 | 2,724,154 | 64.30% | 64860.8 | 51218.0 | adaptive_zstd | 229,441 |
| v7_canonical_10000000 | pairs | 1476 | 38,086,038 | 30.59% | 25803.5 | 14043.5 | adaptive_zstd | 1,575,219 |
| v7_canonical_10000000 | bytes | 42 | 1,743,694 | 49.21% | 41516.5 | 16344.5 | adaptive_zstd | 5,281 |
| alphanumeric36_10000000 | pairs | 1442 | 125,046,699 | 83.72% | 86717.5 | 96232.5 | adaptive_zstd | 6,139,491 |
| alphanumeric36_10000000 | bytes | 41 | 3,720,116 | 87.60% | 90734.5 | 103155.0 | adaptive_zstd | 6,099 |

## All posting formats

These are posting-file bytes. Add the prefix sidecar above for a complete summary. Each zstd row is a separate frame. Adaptive tests positive and absent-block raw/delta/run/bitmap payloads. Adaptive-zstd also tests each payload compressed and uncompressed. The smallest row wins independently, ties use deterministic codec order. No query-selected vocabulary or neighboring unions are used.

| Scenario | Feature | adaptive | adaptive_zstd | bitmap | bitmap_zstd | delta | delta_zstd | raw | raw_zstd | runs | runs_zstd |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | pairs | 2,498,974 | 1,586,053 | 3,046,191 | 1,655,850 | 18,644,753 | 2,538,195 | 37,269,503 | 35,432,152 | 3,739,651 | 1,855,769 |
| baseline | bytes | 7,041 | 5,275 | 86,528 | 6,826 | 595,829 | 7,737 | 1,191,036 | 1,133,769 | 7,239 | 5,543 |
| v4_canonical_10000000 | pairs | 8,131,147 | 4,520,661 | 18,615,993 | 5,213,667 | 43,952,684 | 4,825,616 | 131,751,583 | 103,116,404 | 8,274,544 | 4,572,413 |
| v4_canonical_10000000 | bytes | 257,476 | 229,441 | 530,058 | 231,045 | 2,724,858 | 296,207 | 8,172,900 | 6,409,664 | 888,051 | 301,529 |
| v7_canonical_10000000 | pairs | 3,603,377 | 1,575,219 | 15,582,150 | 1,674,232 | 38,106,603 | 2,553,474 | 114,274,368 | 88,665,680 | 3,748,878 | 1,872,965 |
| v7_canonical_10000000 | bytes | 7,740 | 5,281 | 443,370 | 6,877 | 1,744,276 | 8,068 | 5,231,520 | 4,106,068 | 7,901 | 5,558 |
| alphanumeric36_10000000 | pairs | 9,239,820 | 6,139,491 | 18,686,896 | 7,645,147 | 125,064,883 | 12,438,859 | 375,155,977 | 291,114,732 | 16,193,447 | 8,127,759 |
| alphanumeric36_10000000 | bytes | 7,915 | 6,099 | 531,296 | 8,876 | 3,720,710 | 10,090 | 11,160,776 | 8,568,135 | 8,513 | 6,460 |

### Best-file row codec choices

- baseline pairs: absent_bitmap_zstd=3, absent_delta_zstd=1258, present_bitmap_zstd=35, present_delta=59, present_delta_zstd=20, present_raw=68
- baseline bytes: absent_delta_zstd=5, absent_raw=10, absent_runs_zstd=14, present_delta=1, present_delta_zstd=1, present_raw=2, present_runs_zstd=8
- v4_canonical_10000000 pairs: absent_delta_zstd=256, absent_runs_zstd=46, present_delta=93, present_delta_zstd=43, present_raw=24, present_runs_zstd=1013
- v4_canonical_10000000 bytes: absent_bitmap_zstd=20, absent_raw=10, absent_runs_zstd=6, present_delta=2, present_delta_zstd=2, present_raw=1, present_runs_zstd=1
- v7_canonical_10000000 pairs: absent_delta_zstd=1296, present_bitmap_zstd=2, present_delta=102, present_delta_zstd=18, present_raw=26, present_runs=32
- v7_canonical_10000000 bytes: absent_delta_zstd=1, absent_raw=10, absent_runs_zstd=19, present_delta=2, present_delta_zstd=1, present_raw=1, present_runs=1, present_runs_zstd=7
- alphanumeric36_10000000 pairs: absent_delta_zstd=1296, present_delta=100, present_delta_zstd=20, present_raw=26
- alphanumeric36_10000000 bytes: absent_delta=1, absent_raw=10, absent_runs_zstd=25, present_delta=2, present_delta_zstd=2, present_raw=1

## Prefix-safe query results

Cells show candidate blocks / skipped %. Exact terms include generated IDs. Oracle is the count of blocks containing at least one exact matching full term. Every mode has zero false-negative blocks. Pair-only uses an all-ones byte mask, which disables byte pruning. Single-byte queries retain all blocks. The predicate ORs prefix-contained, remainder-contained and viable prefix-crossing continuations. For example `player` needs `pl, la, ay, ye, er` in a remainder-contained match. A crossing continuation only needs its own remaining pairs. A naive intersection of every literal pair would be unsafe.

### baseline

| Query | Exact terms | Added matches | Oracle blocks | Bytes | Pairs | Pairs + bytes |
| --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 0 / 100.00% | 16,794 / 0.00% | 0 / 100.00% |
| *a* | 1921269 | 0 | 16376 | 16,376 / 2.49% | 16,794 / 0.00% | 16,376 / 2.49% |
| *ab* | 75774 | 0 | 15490 | 16,373 / 2.51% | 15,513 / 7.63% | 15,513 / 7.63% |
| *ms* | 50585 | 0 | 14065 | 16,339 / 2.71% | 14,084 / 16.14% | 14,084 / 16.14% |
| *player* | 21 | 0 | 13 | 16,331 / 2.76% | 9,804 / 41.62% | 9,804 / 41.62% |
| *traa* | 42 | 0 | 38 | 16,341 / 2.70% | 11,818 / 29.63% | 11,818 / 29.63% |
| *x | 153990 | 0 | 16022 | 16,354 / 2.62% | 16,794 / 0.00% | 16,354 / 2.62% |

### v4_canonical_10000000

| Query | Exact terms | Added matches | Oracle blocks | Bytes | Pairs | Pairs + bytes |
| --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 0 / 100.00% | 100,876 / 0.00% | 0 / 100.00% |
| *a* | 10838333 | 8917064 | 100465 | 100,465 / 0.41% | 100,876 / 0.00% | 100,465 / 0.41% |
| *ab* | 1153596 | 1077822 | 99629 | 100,464 / 0.41% | 99,654 / 1.21% | 99,654 / 1.21% |
| *ms* | 50585 | 0 | 14872 | 35,265 / 65.04% | 14,891 / 85.24% | 14,891 / 85.24% |
| *player* | 21 | 0 | 13 | 23,913 / 76.29% | 9,273 / 90.81% | 9,273 / 90.81% |
| *traa* | 42 | 0 | 38 | 35,259 / 65.05% | 11,223 / 88.87% | 11,223 / 88.87% |
| *x | 153990 | 0 | 21856 | 51,202 / 49.24% | 100,876 / 0.00% | 51,202 / 49.24% |

### v7_canonical_10000000

| Query | Exact terms | Added matches | Oracle blocks | Bytes | Pairs | Pairs + bytes |
| --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 0 / 100.00% | 84,362 / 0.00% | 0 / 100.00% |
| *a* | 10226774 | 8305505 | 83929 | 83,929 / 0.51% | 84,362 / 0.00% | 83,929 / 0.51% |
| *ab* | 905113 | 829339 | 83076 | 83,926 / 0.52% | 83,097 / 1.50% | 83,097 / 1.50% |
| *ms* | 50585 | 0 | 14003 | 16,330 / 80.64% | 14,025 / 83.38% | 14,025 / 83.38% |
| *player* | 21 | 0 | 13 | 16,314 / 80.66% | 9,719 / 88.48% | 9,719 / 88.48% |
| *traa* | 42 | 0 | 38 | 16,334 / 80.64% | 11,806 / 86.01% | 11,806 / 86.01% |
| *x | 153990 | 0 | 16007 | 16,343 / 80.63% | 84,362 / 0.00% | 16,343 / 80.63% |

### alphanumeric36_10000000

| Query | Exact terms | Added matches | Oracle blocks | Bytes | Pairs | Pairs + bytes |
| --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 0 / 100.00% | 103,582 / 0.00% | 0 / 100.00% |
| *a* | 8293851 | 6372582 | 103157 | 103,157 / 0.41% | 103,582 / 0.00% | 103,157 / 0.41% |
| *ab* | 342084 | 266310 | 97405 | 103,156 / 0.41% | 97,507 / 5.86% | 97,507 / 5.86% |
| *ms* | 317095 | 266510 | 96270 | 103,155 / 0.41% | 96,396 / 6.94% | 96,396 / 6.94% |
| *player* | 21 | 0 | 13 | 103,148 / 0.42% | 73,929 / 28.63% | 73,929 / 28.63% |
| *traa* | 240 | 198 | 228 | 103,152 / 0.42% | 84,542 / 18.38% | 84,542 / 18.38% |
| *x | 432136 | 278146 | 101349 | 103,155 / 0.41% | 103,582 / 0.00% | 103,155 / 0.41% |

## Validation and limits

- Actual retained SSTables are read through their block addresses. Their SHA-256 and the complete length-prefixed decoded-key digest match the earlier rebuild. Every ordinal, term count, first/last key, fence, full LCP, remainder-byte mask and all seven full-term exact counts agree per block.
- Pair extraction takes adjacent windows separately inside each term after the entire actual LCP. No pairs cross term boundaries or the excluded prefix/remainder boundary. Baseline observed vocabulary and every block pair mask exactly match the previous 1,443-pair PC01 artifact.
- Every posting in all ten formats, for both feature types and all four scenarios, is read back from disk and compared with the extraction. Best-file decoded postings generate evaluation masks. New prefix sidecars round-trip and equal the corresponding scenario prefixes. Every byte-only decision agrees with the previous study. All query modes check every truth-bearing block.
- Rust tests cover empty/full sets, complementary encodings, all codecs, directory offsets, 65,535/65,536/65,537 block universes, prefix exclusion and no cross-term grams. Python tests cover prefix crossing, prefix-contained matches, singleton remainders and exhaustive small-dictionary safety.
- Metadata comparisons describe tested layouts, not information-theoretic lower bounds. No production I/O, latency, or rebuilt production index size is inferred. A direct posting-query planner would still need prefix-continuation grouping and validation.

Artifacts: `report.json` includes every cost/reference %, row codec distribution, posting lengths, query counts, validations and artifact hashes. Each scenario contains all 20 posting files, its prefix sidecar, decoded validation masks and compact per-block decisions. Validation masks and decision logs are experiment artifacts, excluded from deployable metadata cost.
