# UUID versions, alphabets and fixed lengths: measured rebuilds

## Result

**UUID text is a small alphabet, not all ASCII.** Canonical lowercase UUIDs use `0-9a-f-` at 36 bytes. Compact UUIDs use `0-9a-f` at 32 bytes. The fixed-length control uses all 36 symbols in `0-9a-z`, at 36 bytes. It includes every letter in `player`, `traa`, `ms`, and lowercase `x`. Uppercase `X` is absent in all generated cases.

For the seven existing queries, byte+length makes exactly the same decisions as byte-only in all 17 scenarios: **zero incremental length rejections**. A 36-byte term can contain a six-byte literal. With an unbounded star, only `max_full_length < literal_length` rejects a block. Minimum term length cannot reject these patterns. Length-only also rejects zero blocks in this workload. Longer literals can benefit from maximum length, and exact anchored whole-token queries could use both bounds, but those are different predicates.

Measured highlights for `*player*`:

- Baseline: **2.76% skipped / 16,331 candidates**. With 1m canonical IDs, v1/v3/v4/v5 reach only 4.28–4.78% skipped and roughly 24,150–24,249 candidates. Their generated IDs interleave with original terms, producing no generated-only blocks. v6/v7 reach 30.84%/30.72% with 16,319/16,316 candidates, and thousands of generated-only blocks.
- At 10m canonical IDs: **v4 76.29% / 23,913**, **v7 80.66% / 16,314**, **alphanumeric 0.42% / 103,148**. All retain the original 21 matching terms, and this alphanumeric sample adds zero `player` matches. Its letter coverage retains many false-positive blocks.
- v7’s large skip percentage is principally added, rejectable blocks: candidates move by only -17. v4 candidates remain 7,582 above baseline, while original terms inside candidate blocks fall by 1,670,565 (from 7,303,590 to 5,633,025). Repartitioning therefore changes existing-term candidate work as well as the denominator. These metrics measure different work units.
- Actual byte-only sidecars at 10m: **v4 2,184,368 B**, **v7 97,870 B**, **alphanumeric 107,301 B**. v4 creates 61,896 unique masks in mostly mixed blocks, making this shared-mask encoding expensive. Length sidecars add cost with zero benefit for these queries.

## Construction and validation

- Original 7,671,280 keys retained. IDs are sorted, duplicate generated keys and original collisions deduplicated. Every serialized key is read through actual Writer-generated block addresses and compared with the sorted merged input. All source hashes still match provenance.
- Zero-injection matches all 16,794 original block IDs, first ordinals, counts, first/last keys and index fences exactly. Original byte/prefix sidecars are byte-identical. Per-block truth and decisions agree.
- Actual unchanged `VoidSSTable::writer` uses its default 4,000 key-delta-byte target. Rebuilt index fences and physical block addresses are read back. Real values/postings are omitted.
- Generic remainder-byte masks exclude each block’s **full actual common prefix**. PX02 reconstructs that prefix from the rebuilt index plus sparse extensions. Prefix-contained and crossing matches use the existing conservative `permits` predicate. No neighboring unions or query-selected summary features. Lengths describe full terms, including the prefix.
- Independent exact scans over complete decoded keys recompute all seven truths. Original-key truth remains unchanged. Generated IDs introduce new matches. Zero false-negative blocks for both predicates in every scenario. Original-term membership is tracked during merge.
- Validation tests: **4 Rust generator/merge tests, 3 new Python predicate/serialization tests, and all 48 existing Python tests pass**. Full experiment assertions additionally cover every decoded key, every block boundary, sidecar roundtrips, original truth and false-negative safety.
- **Whole-token assumption:** each ID becomes one dictionary term. This is a controlled vocabulary augmentation, not end-to-end Datadog tokenization. A tokenizer splitting hyphens would produce different terms and block boundaries. No document/posting frequencies are modeled.

## Reproducible generation

- SplitMix64 seed `0x4d595df4d0f33173`. Canonical and compact layouts use the same UUID values. The 1m sequence is the prefix of each corresponding 10m sequence before lexical sorting.
- v1/v6: one UUID per millisecond from Unix `1700000000000`, Gregorian 100ns offset `0x01b21dd213814000`, node `020000000001`, clock sequence `0x1234`. v1 stores timestamp-low first and wraps about every 429.5 seconds. v6 reorders time chronologically. The 1m sequence crosses v1 wraps. Fixed fields affect `a`/`ab` truth, so these are results for the stated fields.
- v7: first 48 bits are increasing Unix milliseconds, followed by deterministic random bits with version/variant fields applied. v4 uses random bits with version/variant fields applied.
- v3 is actual MD5 and v5 actual SHA-1 over DNS namespace `6ba7b810-9dad-11d1-80b4-00c04fd430c8` and UTF-8 name `uuid-pruning/{i}`, decimal `i` from zero. Generator tests verify standard namespace/name vectors, timestamp fields, versions and variant bits.
- Alphanumeric bytes are sampled uniformly by rejecting random bytes >=252, then indexing `0123456789abcdefghijklmnopqrstuvwxyz` by byte modulo 36. No query-driven generation choices.
- v2 legacy DCE security UUIDs and application-defined v8 are omitted. No single generator would characterize all v8 layouts. RNG is reproducible synthetic data, not a cryptographic security claim.

## Rebuild matrix

| Scenario | Terms | Blocks | Generated-only | Mixed | Original-only | Collisions / generated duplicates |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 7,671,280 | 16,794 | 0 | 0 | 16,794 | 0 / 0 |
| v1_canonical_1000000 | 8,671,280 | 25,363 | 0 | 12,324 | 13,039 | 0 / 0 |
| v1_compact_1000000 | 8,671,280 | 24,370 | 0 | 11,337 | 13,033 | 0 / 0 |
| v3_canonical_1000000 | 8,671,280 | 25,335 | 0 | 12,286 | 13,049 | 0 / 0 |
| v3_compact_1000000 | 8,671,280 | 24,342 | 0 | 11,306 | 13,036 | 0 / 0 |
| v4_canonical_1000000 | 8,671,280 | 25,335 | 0 | 12,299 | 13,036 | 0 / 0 |
| v4_compact_1000000 | 8,671,280 | 24,342 | 0 | 11,307 | 13,035 | 0 / 0 |
| v5_canonical_1000000 | 8,671,280 | 25,334 | 0 | 12,301 | 13,033 | 0 / 0 |
| v5_compact_1000000 | 8,671,280 | 24,342 | 0 | 11,312 | 13,030 | 0 / 0 |
| v6_canonical_1000000 | 8,671,280 | 23,597 | 6,802 | 2 | 16,793 | 0 / 0 |
| v6_compact_1000000 | 8,671,280 | 22,818 | 6,023 | 2 | 16,793 | 0 / 0 |
| v7_canonical_1000000 | 8,671,280 | 23,551 | 6,756 | 2 | 16,793 | 0 / 0 |
| v7_compact_1000000 | 8,671,280 | 22,818 | 6,023 | 2 | 16,793 | 0 / 0 |
| alphanumeric36_1000000 | 8,671,280 | 25,494 | 0 | 25,039 | 455 | 0 / 0 |
| v4_canonical_10000000 | 17,671,280 | 100,876 | 117 | 87,732 | 13,027 | 0 / 0 |
| v7_canonical_10000000 | 17,671,280 | 84,362 | 67,567 | 2 | 16,793 | 0 / 0 |
| alphanumeric36_10000000 | 17,671,280 | 103,582 | 0 | 103,156 | 426 | 0 / 0 |

Generated-only means zero original keys. Mixed means both original and generated membership. Collisions have both memberships. Actual measured collisions and generated duplicates are shown above.

## Byte pruning and exact block oracle

Each cell: **byte skip % / absolute candidate blocks / exact-oracle skip %**. Byte+length has the same skip and candidate values. Oracle skips precisely the blocks with zero complete-key matches. All percentages count blocks, not bytes, documents or latency.

### Canonical 1m + control

| Query | baseline | v1_canonical_1000000 | v3_canonical_1000000 | v4_canonical_1000000 | v5_canonical_1000000 | v6_canonical_1000000 | v7_canonical_1000000 | alphanumeric36_1000000 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `*a*` | 2.49% / 16,376 / 2.49% | 1.67% / 24,940 / 1.67% | 1.67% / 24,913 / 1.67% | 1.65% / 24,918 / 1.65% | 1.61% / 24,927 / 1.61% | 1.79% / 23,174 / 1.79% | 1.80% / 23,127 / 1.80% | 1.69% / 25,064 / 1.69% |
| `*ab*` | 2.51% / 16,373 / 7.76% | 1.68% / 24,938 / 7.80% | 1.67% / 24,912 / 4.94% | 1.65% / 24,918 / 4.89% | 1.61% / 24,925 / 4.94% | 1.81% / 23,171 / 11.40% | 1.81% / 23,124 / 5.46% | 1.70% / 25,061 / 7.06% |
| `*traa*` | 2.70% / 16,341 / 99.77% | 3.57% / 24,458 / 99.85% | 3.17% / 24,532 / 99.85% | 3.25% / 24,512 / 99.85% | 3.07% / 24,555 / 99.85% | 30.76% / 16,338 / 99.84% | 30.62% / 16,339 / 99.84% | 1.75% / 25,049 / 99.78% |
| `*X` | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% |
| `*x` | 2.62% / 16,354 / 4.60% | 2.66% / 24,689 / 22.45% | 2.53% / 24,693 / 22.11% | 2.53% / 24,694 / 22.21% | 2.55% / 24,689 / 22.25% | 30.70% / 16,352 / 32.17% | 30.58% / 16,348 / 32.04% | 1.74% / 25,050 / 3.13% |
| `*player*` | 2.76% / 16,331 / 99.92% | 4.78% / 24,150 / 99.95% | 4.29% / 24,247 / 99.95% | 4.37% / 24,227 / 99.95% | 4.28% / 24,249 / 99.95% | 30.84% / 16,319 / 99.94% | 30.72% / 16,316 / 99.94% | 1.78% / 25,039 / 99.95% |
| `*ms*` | 2.71% / 16,339 / 16.25% | 3.62% / 24,445 / 41.97% | 3.13% / 24,542 / 41.95% | 3.35% / 24,487 / 42.08% | 3.12% / 24,544 / 41.74% | 30.77% / 16,336 / 40.59% | 30.63% / 16,338 / 40.45% | 1.74% / 25,050 / 11.99% |

### Compact 1m

| Query | v1_compact_1000000 | v3_compact_1000000 | v4_compact_1000000 | v5_compact_1000000 | v6_compact_1000000 | v7_compact_1000000 |
| --- | --- | --- | --- | --- | --- | --- |
| `*a*` | 1.74% / 23,945 / 1.74% | 1.72% / 23,924 / 1.72% | 1.69% / 23,931 / 1.69% | 1.68% / 23,932 / 1.68% | 1.83% / 22,400 / 1.83% | 1.84% / 22,399 / 1.84% |
| `*ab*` | 1.75% / 23,944 / 7.46% | 1.73% / 23,922 / 5.11% | 1.69% / 23,930 / 5.02% | 1.68% / 23,932 / 4.96% | 1.85% / 22,397 / 9.93% | 1.85% / 22,397 / 5.71% |
| `*traa*` | 3.17% / 23,598 / 99.84% | 2.82% / 23,655 / 99.84% | 2.84% / 23,650 / 99.84% | 2.74% / 23,675 / 99.84% | 28.39% / 16,339 / 99.83% | 28.39% / 16,340 / 99.83% |
| `*X` | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% |
| `*x` | 2.58% / 23,741 / 19.99% | 2.37% / 23,765 / 19.78% | 2.37% / 23,765 / 19.73% | 2.41% / 23,755 / 19.84% | 28.33% / 16,353 / 29.77% | 28.32% / 16,355 / 29.75% |
| `*player*` | 4.08% / 23,375 / 99.95% | 3.69% / 23,444 / 99.95% | 3.69% / 23,443 / 99.95% | 3.57% / 23,472 / 99.95% | 28.46% / 16,325 / 99.94% | 28.47% / 16,321 / 99.94% |
| `*ms*` | 3.21% / 23,587 / 39.93% | 2.87% / 23,644 / 39.86% | 2.94% / 23,626 / 39.63% | 2.92% / 23,630 / 39.77% | 28.39% / 16,341 / 38.34% | 28.36% / 16,346 / 38.37% |

### Canonical 10m + control

| Query | v4_canonical_10000000 | v7_canonical_10000000 | alphanumeric36_10000000 |
| --- | --- | --- | --- |
| `*a*` | 0.41% / 100,465 / 0.41% | 0.51% / 83,929 / 0.51% | 0.41% / 103,157 / 0.41% |
| `*ab*` | 0.41% / 100,464 / 1.24% | 0.52% / 83,926 / 1.52% | 0.41% / 103,156 / 5.96% |
| `*traa*` | 65.05% / 35,259 / 99.96% | 80.64% / 16,334 / 99.95% | 0.42% / 103,152 / 99.78% |
| `*X` | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% | 100.00% / 0 / 100.00% |
| `*x` | 49.24% / 51,202 / 78.33% | 80.63% / 16,343 / 81.03% | 0.41% / 103,155 / 2.16% |
| `*player*` | 76.29% / 23,913 / 99.99% | 80.66% / 16,314 / 99.98% | 0.42% / 103,148 / 99.99% |
| `*ms*` | 65.04% / 35,265 / 85.26% | 80.64% / 16,330 / 83.40% | 0.41% / 103,155 / 7.06% |

## Exact matching terms

Every cell is a fresh complete-key count after rebuild, including originals.

| Scenario | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | 1,921,269 | 75,774 | 42 | 0 | 153,990 | 21 | 50,585 |
| v1_canonical_1000000 | 2,278,503 | 98,489 | 42 | 0 | 153,990 | 21 | 50,585 |
| v1_compact_1000000 | 2,278,503 | 98,489 | 42 | 0 | 153,990 | 21 | 50,585 |
| v3_canonical_1000000 | 2,813,247 | 184,059 | 42 | 0 | 153,990 | 21 | 50,585 |
| v3_compact_1000000 | 2,813,247 | 205,233 | 42 | 0 | 153,990 | 21 | 50,585 |
| v4_canonical_1000000 | 2,812,770 | 183,513 | 42 | 0 | 153,990 | 21 | 50,585 |
| v4_compact_1000000 | 2,812,770 | 204,592 | 42 | 0 | 153,990 | 21 | 50,585 |
| v5_canonical_1000000 | 2,812,656 | 183,375 | 42 | 0 | 153,990 | 21 | 50,585 |
| v5_compact_1000000 | 2,812,656 | 204,464 | 42 | 0 | 153,990 | 21 | 50,585 |
| v6_canonical_1000000 | 2,278,503 | 91,312 | 42 | 0 | 153,990 | 21 | 50,585 |
| v6_compact_1000000 | 2,278,503 | 94,629 | 42 | 0 | 153,990 | 21 | 50,585 |
| v7_canonical_1000000 | 2,751,653 | 158,711 | 42 | 0 | 153,990 | 21 | 50,585 |
| v7_compact_1000000 | 2,751,653 | 180,584 | 42 | 0 | 153,990 | 21 | 50,585 |
| alphanumeric36_1000000 | 2,558,397 | 102,427 | 64 | 0 | 182,100 | 21 | 77,528 |
| v4_canonical_10000000 | 10,838,333 | 1,153,596 | 42 | 0 | 153,990 | 21 | 50,585 |
| v7_canonical_10000000 | 10,226,774 | 905,113 | 42 | 0 | 153,990 | 21 | 50,585 |
| alphanumeric36_10000000 | 8,293,851 | 342,084 | 240 | 0 | 432,136 | 21 | 317,095 |

## Denominator effects and existing-text candidate work

A higher block-skip percentage can arise from adding easily rejected ID blocks. Compare absolute candidates and the number of original terms inside candidate blocks. Original-bearing candidate blocks may change because the writer repartitions existing terms. Counting original terms in candidates follows the same original keys across those boundary changes. This is candidate vocabulary work, not measured physical I/O.

| Scenario | `*player*` skip % | Candidates (delta) | Original-bearing candidate blocks | Original terms in candidates (delta) | Generated-only candidates |
| --- | --- | --- | --- | --- | --- |
| baseline | 2.76% | 16,331 (+0) | 16,331 | 7,303,590 (+0) | 0 |
| v1_canonical_1000000 | 4.78% | 24,150 (+7,819) | 24,150 | 7,043,901 (-259,689) | 0 |
| v1_compact_1000000 | 4.08% | 23,375 (+7,044) | 23,375 | 7,076,712 (-226,878) | 0 |
| v3_canonical_1000000 | 4.29% | 24,247 (+7,916) | 24,247 | 7,085,184 (-218,406) | 0 |
| v3_compact_1000000 | 3.69% | 23,444 (+7,113) | 23,444 | 7,120,032 (-183,558) | 0 |
| v4_canonical_1000000 | 4.37% | 24,227 (+7,896) | 24,227 | 7,071,620 (-231,970) | 0 |
| v4_compact_1000000 | 3.69% | 23,443 (+7,112) | 23,443 | 7,124,442 (-179,148) | 0 |
| v5_canonical_1000000 | 4.28% | 24,249 (+7,918) | 24,249 | 7,076,215 (-227,375) | 0 |
| v5_compact_1000000 | 3.57% | 23,472 (+7,141) | 23,472 | 7,125,786 (-177,804) | 0 |
| v6_canonical_1000000 | 30.84% | 16,319 (-12) | 16,319 | 7,290,347 (-13,243) | 0 |
| v6_compact_1000000 | 28.46% | 16,325 (-6) | 16,325 | 7,295,692 (-7,898) | 0 |
| v7_canonical_1000000 | 30.72% | 16,316 (-15) | 16,316 | 7,292,087 (-11,503) | 0 |
| v7_compact_1000000 | 28.47% | 16,321 (-10) | 16,321 | 7,292,081 (-11,509) | 0 |
| alphanumeric36_1000000 | 1.78% | 25,039 (+8,708) | 25,039 | 7,320,248 (+16,658) | 0 |
| v4_canonical_10000000 | 76.29% | 23,913 (+7,582) | 23,913 | 5,633,025 (-1,670,565) | 0 |
| v7_canonical_10000000 | 80.66% | 16,314 (-17) | 16,314 | 7,289,970 (-13,620) | 0 |
| alphanumeric36_10000000 | 0.42% | 103,148 (+86,817) | 103,148 | 7,325,371 (+21,781) | 0 |

The machine report provides these original-key candidate metrics for every query, plus incremental length rejects, false positives, and exact generated-match deltas.

## Actual serialized sidecar costs

PM01 interns 256-bit remainder masks, including framing and per-block IDs. PX02 stores sparse prefix extensions relative to the new fences. LN01 stores an 8-byte magic/count header plus two bytes per block (minimum and maximum full-term length, u8 each). All three files are written, reopened, decoded and compared. Totals include headers. LN01 is a simple actual encoding, not a claim of optimal length compression.

**Percent denominator: original 195,640-byte production index, REFERENCE ONLY.** The key-only rebuilt index is not a production rebuild storage estimate.

| Scenario | Unique masks | PM01 B | PX02 B | LN01 B | Byte-only B (% reference) | Byte+length B (% reference) |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 109 | 20,307 | 48 | 33,596 | 20,355 (10.40%) | 53,951 (27.58%) |
| v1_canonical_1000000 | 826 | 77,183 | 84 | 50,734 | 77,267 (39.49%) | 128,001 (65.43%) |
| v1_compact_1000000 | 640 | 69,245 | 120 | 48,748 | 69,365 (35.46%) | 118,113 (60.37%) |
| v3_canonical_1000000 | 641 | 71,207 | 175 | 50,678 | 71,382 (36.49%) | 122,060 (62.39%) |
| v3_compact_1000000 | 498 | 64,645 | 166 | 48,692 | 64,811 (33.13%) | 113,503 (58.02%) |
| v4_canonical_1000000 | 650 | 71,495 | 175 | 50,678 | 71,670 (36.63%) | 122,348 (62.54%) |
| v4_compact_1000000 | 511 | 65,061 | 165 | 48,692 | 65,226 (33.34%) | 113,918 (58.23%) |
| v5_canonical_1000000 | 641 | 71,205 | 147 | 50,676 | 71,352 (36.47%) | 122,028 (62.37%) |
| v5_compact_1000000 | 501 | 64,741 | 148 | 48,692 | 64,889 (33.17%) | 113,581 (58.06%) |
| v6_canonical_1000000 | 103 | 26,918 | 132 | 47,202 | 27,050 (13.83%) | 74,252 (37.95%) |
| v6_compact_1000000 | 113 | 26,459 | 66 | 45,644 | 26,525 (13.56%) | 72,169 (36.89%) |
| v7_canonical_1000000 | 106 | 26,968 | 1108 | 47,110 | 28,076 (14.35%) | 75,186 (38.43%) |
| v7_compact_1000000 | 112 | 26,427 | 66 | 45,644 | 26,493 (13.54%) | 72,137 (36.87%) |
| alphanumeric36_1000000 | 60 | 27,439 | 75 | 50,996 | 27,514 (14.06%) | 78,510 (40.13%) |
| v4_canonical_10000000 | 61,896 | 2,182,449 | 1919 | 201,760 | 2,184,368 (1116.52%) | 2,386,128 (1219.65%) |
| v7_canonical_10000000 | 102 | 87,651 | 10219 | 168,732 | 97,870 (50.03%) | 266,602 (136.27%) |
| alphanumeric36_10000000 | 31 | 104,599 | 2702 | 207,172 | 107,301 (54.85%) | 314,473 (160.74%) |

## Interpretation and limits

- Hex UUID blocks can reject `player`, `traa`, `ms`, and suffix `x`/`X` by their alphabet. They can contain `a` and `ab`; byte occurrence cannot establish adjacency or position. The oracle and exact totals quantify this distinction for each version.
- Alphanumeric IDs include the query letters. A block union often contains them even when no single ID contains `player`, so byte-only pruning can retain almost every ID block. Its fixed length still supplies no additional rejection for these short unbounded-star patterns.
- Chronological prefixes and fixed fields change prefix compression, remainder alphabets, locality and block counts. v1/v6/v7 are therefore not equivalent to v4 despite similar text alphabets. v3/v5 are hash-distributed. Compact form removes hyphens and four bytes. It also changes lexical interleaving and Writer boundaries, rather than only sidecar size. Removing hyphens also creates new adjacent character pairs: v4 at 1m has 183,513 total `ab` matches canonical versus 204,592 compact. Compact is therefore not a length-only intervention.
- Whole-token IDs, deterministic seed, fixed node/clock and one time-based ID per millisecond are controlled assumptions. Ten million IDs span 9,999.999 seconds. The original sample has 10,002,000 documents, but uniqueness and term formation determine the actual vocabulary growth.
- No production data, posting sizes, compressed-byte savings, physical-I/O savings or latency improvements are claimed. Sidecar byte costs exclude runtime allocations/container overhead.

## Artifacts

- `report.json`: complete metrics, generation parameters, source checks and hashes.
- `rebuilds.json`, `*/rebuild.json`: actual writer/readback provenance.
- `*/keys.sstable`: retained actual key-only SSTables, each SHA-256 recorded and rechecked.
- `*/blocks.jsonl`: actual bounds, ordinals, full prefix, remainder mask, full lengths, original/generated memberships and independent per-query truth.
- `*/bytes.bin`, `*/prefixes.bin`, `*/lengths.bin`, `*/decisions.jsonl`: evaluated sidecars and decisions.
- Build/run/test logs in this directory. Reproduction: `dictionary_analysis/uuid_injection/README.md`.
