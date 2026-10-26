# Frequency-first trigram selection under 1,000,000 B

The budget includes the selected trigram file, best existing inverted byte fallback, and full-prefix extension sidecar. It is **+511.14% of the original 195,640 B index**. It is a total added-metadata budget, not an additional allowance beyond the fallback. Percentages use that original index for every dictionary.

**Result: all 44 partial indexes omit all six query trigrams (`pla`, `lay`, `aye`, `yer`, `tra`, `raa`). Selected trigrams add zero pruning over bytes for all seven queries at every cutoff. The 80% cap changes stored rows on three dictionaries and improves none of these query decisions.** Trigrams-only retains every block for every tested query. The pruning reported for the deployed combination comes entirely from bytes.

Eligible rows satisfy `df × 100 <= N × (100 − minimum_skip_percent)`. Rank by descending block frequency, tie by ascending u24 gram. Keep each row that fits, skip nonfitting rows and continue. Construction examines every observed trigram and uses no queries. Frequencies concern prefix-removed remainders and blocks, not documents.

## Persisted representation and safety

`IU01` explicitly means omitted rows are unknown/all blocks, including truly unobserved grams. The 14 B header contains magic, u32 block count, u32 row count, u8 gram width and u8 block-ID width. Sorted little-endian u24 gram keys cost 3 B each. There are row_count+1 u32 offsets. Each copied exact row frame retains its tag, raw-length field and original independently compressed payload. Total directory/header size is `18 + 7 × rows`. Empty selection is 18 B. Retained rows preserve every block ID. Unknown rows impose no constraint while other retained grams still constrain a continuation.

All query decisions use `prefix_summaries.permits`: prefix-contained, remainder-contained and each viable prefix-crossing continuation. A gram callback sees only its continuation. The `trigrams` column is an ablation with all bytes allowed, so one/two-byte queries retain all blocks. The deployable `trigrams_bytes` column uses byte fallback, including short predicates and short prefix-crossing continuations. Both columns are charged the same mandatory-byte total budget. The bytes-only and full-exact references use their own actual costs.

Every one of 44 partial files was reopened, its directory checked, every retained frame byte-compared, and every retained posting independently decoded and compared with the full exact index by Rust. Omitted observed and unobserved keys were checked through the persisted reader. All decisions have zero false negatives. Byte and truth decisions equal the previous validated results per block. Partial candidates contain the full-exact candidate sets.

## Sweep: cost and long-query absolute pruning

Candidate / skip cells give absolute candidate blocks and percentage of all blocks skipped. Query-gram retention is a post-evaluation diagnostic. Player has 13 matching blocks / 21 exact terms in every scenario.

### baseline (16,794 blocks)

Byte fallback 5,275 B + prefix 48 B = fixed 5,323 B. Observed vocabulary 47,796.

| Min skip % | Eligible | Kept | Kept df range | Total B (+index %) | player candidates / skip | traa candidates / skip | Retained query grams |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 47,796 | 691 | 1027–3588 | 1,000,000 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 50 | 47,796 | 691 | 1027–3588 | 1,000,000 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 70 | 47,796 | 691 | 1027–3588 | 1,000,000 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 80 | 47,198 | 733 | 1086–3358 | 999,999 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 85 | 46,796 | 792 | 238–2217 | 999,992 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 90 | 43,700 | 1230 | 4–1353 | 1,000,000 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 92 | 43,699 | 1229 | 340–1332 | 999,997 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 94 | 21,836 | 1298 | 186–1007 | 999,997 (+511.14%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 95 | 1,140 | 1140 | 1–368 | 58,258 (+29.78%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 98 | 1,122 | 1122 | 1–335 | 54,873 (+28.05%) | 16,331 / 2.76% | 16,341 / 2.70% | none |
| 99 | 850 | 850 | 1–167 | 26,903 (+13.75%) | 16,331 / 2.76% | 16,341 / 2.70% | none |

#### Reference methods

| Method | Total B (+index %) | player candidates / skip | traa candidates / skip |
| --- | --- | --- | --- |
| Bytes only | 5,323 (+2.72%) | 16,331 / 2.76% | 16,341 / 2.70% |
| Full exact trigrams | 38,068,570 (+19458.48%) | 14 / 99.92% | 141 / 99.16% |
| Full exact trigrams + bytes | 38,073,845 (+19461.18%) | 14 / 99.92% | 141 / 99.16% |

#### All seven queries

Cells: candidates / skipped blocks / skip %. `T` is trigrams-only ablation. `T+B` includes byte fallback.

| Query | Matching terms | Matching blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 16794 | 0 |
| *a* | 1921269 | 16376 | 16376 | 16794 | 16376 |
| *ab* | 75774 | 15490 | 16373 | 16794 | 16373 |
| *ms* | 50585 | 14065 | 16339 | 16794 | 16339 |
| *player* | 21 | 13 | 16331 | 14 | 14 |
| *traa* | 42 | 38 | 16341 | 141 | 141 |
| *x | 153990 | 16022 | 16354 | 16794 | 16354 |

**trigrams**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 50 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 70 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 80 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 85 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 90 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 92 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 94 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 95 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 98 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |
| 99 | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% | 16,794 / 0 / 0.00% |

**trigrams_bytes**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 50 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 70 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 80 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 85 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 90 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 92 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 94 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 95 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 98 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |
| 99 | 0 / 16,794 / 100.00% | 16,376 / 418 / 2.49% | 16,373 / 421 / 2.51% | 16,339 / 455 / 2.71% | 16,331 / 463 / 2.76% | 16,341 / 453 / 2.70% | 16,354 / 440 / 2.62% |

### v4_canonical_10000000 (100,876 blocks)

Byte fallback 229,441 B + prefix 1,919 B = fixed 231,360 B. Observed vocabulary 48,565.

| Min skip % | Eligible | Kept | Kept df range | Total B (+index %) | player candidates / skip | traa candidates / skip | Retained query grams |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 48,565 | 185 | 165–87762 | 999,998 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 50 | 47,541 | 71 | 5–41598 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 70 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 80 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 85 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 90 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 92 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 94 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 95 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 98 | 43,701 | 762 | 224–1329 | 999,999 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |
| 99 | 23,340 | 811 | 177–1008 | 999,994 (+511.14%) | 23,913 / 76.29% | 35,259 / 65.05% | none |

#### Reference methods

| Method | Total B (+index %) | player candidates / skip | traa candidates / skip |
| --- | --- | --- | --- |
| Bytes only | 231,360 (+118.26%) | 23,913 / 76.29% | 35,259 / 65.05% |
| Full exact trigrams | 95,466,157 (+48796.85%) | 14 / 99.99% | 142 / 99.86% |
| Full exact trigrams + bytes | 95,695,598 (+48914.13%) | 14 / 99.99% | 142 / 99.86% |

#### All seven queries

Cells: candidates / skipped blocks / skip %. `T` is trigrams-only ablation. `T+B` includes byte fallback.

| Query | Matching terms | Matching blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 100876 | 0 |
| *a* | 10838333 | 100465 | 100465 | 100876 | 100465 |
| *ab* | 1153596 | 99629 | 100464 | 100876 | 100464 |
| *ms* | 50585 | 14872 | 35265 | 100876 | 35265 |
| *player* | 21 | 13 | 23913 | 14 | 14 |
| *traa* | 42 | 38 | 35259 | 142 | 142 |
| *x | 153990 | 21856 | 51202 | 100876 | 51202 |

**trigrams**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 50 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 70 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 80 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 85 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 90 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 92 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 94 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 95 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 98 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |
| 99 | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% | 100,876 / 0 / 0.00% |

**trigrams_bytes**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 50 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 70 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 80 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 85 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 90 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 92 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 94 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 95 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 98 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |
| 99 | 0 / 100,876 / 100.00% | 100,465 / 411 / 0.41% | 100,464 / 412 / 0.41% | 35,265 / 65,611 / 65.04% | 23,913 / 76,963 / 76.29% | 35,259 / 65,617 / 65.05% | 51,202 / 49,674 / 49.24% |

### v7_canonical_10000000 (84,362 blocks)

Byte fallback 5,281 B + prefix 10,219 B = fixed 15,500 B. Observed vocabulary 48,565.

| Min skip % | Eligible | Kept | Kept df range | Total B (+index %) | player candidates / skip | traa candidates / skip | Retained query grams |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 48,565 | 313 | 180–67569 | 999,999 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 50 | 47,893 | 101 | 185–34698 | 999,997 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 70 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 80 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 85 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 90 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 92 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 94 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 95 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 98 | 43,701 | 1208 | 1071–1358 | 999,998 (+511.14%) | 16,314 / 80.66% | 16,334 / 80.64% | none |
| 99 | 1,141 | 1141 | 1–364 | 69,740 (+35.65%) | 16,314 / 80.66% | 16,334 / 80.64% | none |

#### Reference methods

| Method | Total B (+index %) | player candidates / skip | traa candidates / skip |
| --- | --- | --- | --- |
| Bytes only | 15,500 (+7.92%) | 16,314 / 80.66% | 16,334 / 80.64% |
| Full exact trigrams | 76,984,062 (+39349.86%) | 13 / 99.98% | 139 / 99.84% |
| Full exact trigrams + bytes | 76,989,343 (+39352.56%) | 13 / 99.98% | 139 / 99.84% |

#### All seven queries

Cells: candidates / skipped blocks / skip %. `T` is trigrams-only ablation. `T+B` includes byte fallback.

| Query | Matching terms | Matching blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 84362 | 0 |
| *a* | 10226774 | 83929 | 83929 | 84362 | 83929 |
| *ab* | 905113 | 83076 | 83926 | 84362 | 83926 |
| *ms* | 50585 | 14003 | 16330 | 84362 | 16330 |
| *player* | 21 | 13 | 16314 | 13 | 13 |
| *traa* | 42 | 38 | 16334 | 139 | 139 |
| *x | 153990 | 16007 | 16343 | 84362 | 16343 |

**trigrams**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 50 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 70 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 80 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 85 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 90 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 92 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 94 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 95 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 98 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |
| 99 | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% | 84,362 / 0 / 0.00% |

**trigrams_bytes**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 50 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 70 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 80 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 85 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 90 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 92 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 94 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 95 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 98 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |
| 99 | 0 / 84,362 / 100.00% | 83,929 / 433 / 0.51% | 83,926 / 436 / 0.52% | 16,330 / 68,032 / 80.64% | 16,314 / 68,048 / 80.66% | 16,334 / 68,028 / 80.64% | 16,343 / 68,019 / 80.63% |

### alphanumeric36_10000000 (103,582 blocks)

Byte fallback 6,099 B + prefix 2,702 B = fixed 8,801 B. Observed vocabulary 47,782.

| Min skip % | Eligible | Kept | Kept df range | Total B (+index %) | player candidates / skip | traa candidates / skip | Retained query grams |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 47,782 | 162 | 8879–10482 | 1,000,000 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 50 | 47,782 | 162 | 8879–10482 | 1,000,000 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 70 | 47,782 | 162 | 8879–10482 | 1,000,000 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 80 | 47,782 | 162 | 8879–10482 | 1,000,000 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 85 | 47,782 | 162 | 8879–10482 | 1,000,000 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 90 | 47,764 | 167 | 358–10358 | 999,994 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 92 | 43,784 | 202 | 186–8286 | 999,990 (+511.14%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 94 | 1,126 | 1126 | 1–369 | 65,666 (+33.56%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 95 | 1,126 | 1126 | 1–369 | 65,666 (+33.56%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 98 | 1,126 | 1126 | 1–369 | 65,666 (+33.56%) | 103,148 / 0.42% | 103,152 / 0.42% | none |
| 99 | 1,126 | 1126 | 1–369 | 65,666 (+33.56%) | 103,148 / 0.42% | 103,152 / 0.42% | none |

#### Reference methods

| Method | Total B (+index %) | player candidates / skip | traa candidates / skip |
| --- | --- | --- | --- |
| Bytes only | 8,801 (+4.50%) | 103,148 / 0.42% | 103,152 / 0.42% |
| Full exact trigrams | 233,435,059 (+119318.68%) | 19 / 99.98% | 975 / 99.06% |
| Full exact trigrams + bytes | 233,441,158 (+119321.79%) | 19 / 99.98% | 975 / 99.06% |

#### All seven queries

Cells: candidates / skipped blocks / skip %. `T` is trigrams-only ablation. `T+B` includes byte fallback.

| Query | Matching terms | Matching blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 103582 | 0 |
| *a* | 8293851 | 103157 | 103157 | 103582 | 103157 |
| *ab* | 342084 | 97405 | 103156 | 103582 | 103156 |
| *ms* | 317095 | 96270 | 103155 | 103582 | 103155 |
| *player* | 21 | 13 | 103148 | 19 | 19 |
| *traa* | 240 | 228 | 103152 | 975 | 975 |
| *x | 432136 | 101349 | 103155 | 103582 | 103155 |

**trigrams**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 50 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 70 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 80 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 85 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 90 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 92 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 94 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 95 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 98 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |
| 99 | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% | 103,582 / 0 / 0.00% |

**trigrams_bytes**

| Min skip % | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 50 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 70 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 80 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 85 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 90 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 92 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 94 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 95 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 98 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |
| 99 | 0 / 103,582 / 100.00% | 103,157 / 425 / 0.41% | 103,156 / 426 / 0.41% | 103,155 / 427 / 0.41% | 103,148 / 434 / 0.42% | 103,152 / 430 / 0.42% | 103,155 / 427 / 0.41% |

## Interpretation of the 80% cap

- **baseline**: 80% excludes 598 of 47,796 observed grams. *player*: 16,331 → 16,331, *traa*: 16,341 → 16,341 candidates.
- **v4_canonical_10000000**: 80% excludes 4,864 of 48,565 observed grams. *player*: 23,913 → 23,913, *traa*: 35,259 → 35,259 candidates.
- **v7_canonical_10000000**: 80% excludes 4,864 of 48,565 observed grams. *player*: 16,314 → 16,314, *traa*: 16,334 → 16,334 candidates.
- **alphanumeric36_10000000**: 80% excludes 0 of 47,782 observed grams. *player*: 103,148 → 103,148, *traa*: 103,152 → 103,152 candidates.

At 80%, retained rows are 733 / 762 / 1,208 / 162 for baseline / UUIDv4 / UUIDv7 / alphanumeric respectively. Baseline excludes 598 high-frequency rows. Each UUID dictionary excludes 4,864. The alphanumeric control excludes none. The same workload outcome holds across all eleven tested cutoffs, despite different selections and costs.

Strict cutoffs can exhaust eligibility before the budget: baseline totals fall to 58,258 B at 95%, 54,873 B at 98%, and 26,903 B at 99%. UUIDv7 falls to 69,740 B at 99%. Alphanumeric falls to 65,666 B from 94% onward. UUIDv4 stays near the budget throughout. Underfill reflects the eligibility rule rather than omitted accounting.

Highest frequency first prioritizes rows with the weakest individual rejection among the eligible grams. A cap removes that head, but a one-row absence rate does not guarantee whole-query pruning: omitted rows are unknown, gram correlations and prefix-crossing continuations matter. The broad random-background vocabulary can consume the budget before the six query grams are selected. Subtracting a shared minimum frequency preserves the ordering.

The tables describe these seven queries on four dictionaries. They support workload-specific comparison of the cutoffs, rather than an optimal universal cutoff or any guarantee of 80% full-query skipping. Construction is query-independent. Selecting a preferred cutoff from these results would be post-evaluation workload tuning. Block frequency does not distinguish meaningful text from random strings and differs from document frequency.

## Reproducibility and limits

Each scenario directory contains eleven actual `skip_XX.bin` partial indexes, their per-block `skip_XX_decisions.bin` artifacts, one `bytes.bin`, one `prefixes.bin`, and a fresh all-row `frequencies.jsonl`. Sidecars are shared once per deployable index. `report.json` includes hashes, exact costs, eligibility, retained df ranges, unknown/readback validation and every query result. `originals_manifest.json` records input hashes checked before and after the run. Diagnostic frequencies, decisions, logs and reports are excluded from metadata cost.

Each partial query posting is independently decoded from its own file. Repeated retained-query-gram subsets reuse evaluated per-block decisions only after their decoded memberships equal the full exact postings. In this run every cutoff has the same empty retained-query-gram subset, so each scenario needs one per-block evaluation. This is an experiment optimization, not a query-latency measurement. No SSTables are rebuilt. Prior complete source scans established exact truth and posting correctness. This experiment reuses those validated artifacts and compares source block truth, bytes and prefix reconstruction again. All original inputs are unchanged.

Construction, readback validation and evaluation: 20.87 seconds. Runtime memory and diagnostic storage are separate from serialized metadata. No production I/O, latency, globally optimal compression or universal cutoff claim is made.

The initial helper filename `select.py` shadowed the standard Python module. Renaming it to `sweep.py` resolved startup before any indexes were constructed. The failure log is retained. The successful run is in `run.log` and regression checks are in `*_tests.log`.
