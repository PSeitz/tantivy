# Prefix-excluded summaries

Original **16,794 blocks / 7,671,280 terms**. Existing index and +100% additional budget: **195,640 bytes**.

Every row includes the shared remainder-byte masks once and prefix storage once. Feature costs include serialized headers, definitions, IDs and compressed-group offsets. Each row charges its selected sidecars. Diagnostic JSON and alternate tested layouts are output artifacts rather than required metadata. Sizes describe standalone sidecars. Runtime allocations and production-container overhead are excluded. Generic methods derive all observed remainder pairs. Configured methods use the external supplied lists unchanged.

Actual mode removes the full-term block LCP, verified from sorted first/last terms and against every key. Prefixes are reconstructed from the existing lower/upper index bounds plus a sparse table of extensions; only that extra table is charged. Bounds mode removes only the LCP of existing lower/upper index bounds and needs no extension table. These are separate experiments.

Query decisions OR a match inside the prefix, a match inside the remainder, and every viable prefix-crossing continuation. Crossing continuations begin at remainder phase zero. All byte/ending/sequence/pair bits describe remainders only. Ending-byte constraints apply only in the ending-byte row. For suffixes matching the end of a prefix, the reader conservatively allows an empty remainder without storing an empty-remainder flag. Untracked configured features impose no constraint. Generic vocabulary absence proves a remainder pair absent.

## Actual prefixes

Stored prefix cost: **48 bytes**. Raw prefix bytes: 33,354. Nonempty prefixes: 16,759 blocks. Empty remainders: 1 terms. Observed remainder pairs: 1,443. Prefix extensions beyond index-derived prefixes: 4 blocks, 4 bytes before framing.

### Table 1. Serialized additional metadata

| Method | Caller patterns | Prefix B | Byte masks B | Feature B | Total B | Increase | Within +100% |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Remainder bytes | none | 48 | 20,307 | 0 | 20,355 | 10.40% | yes |
| Bytes + ending bytes | none | 48 | 20,307 | 61,325 | 81,680 | 41.75% | yes |
| Bytes + pair unions / 5 blocks | none | 48 | 20,307 | 151,352 | 171,707 | 87.77% | yes |
| Bytes + all pairs | none | 48 | 20,307 | 1,448,844 | 1,469,199 | 750.97% | no |
| Bytes + all pairs even/odd | none | 48 | 20,307 | 4,590,348 | 4,610,703 | 2356.73% | no |
| Bytes + 10 supplied pairs | required | 48 | 20,307 | 33,664 | 54,019 | 27.61% | yes |
| Bytes + 10 supplied pairs even/odd | required | 48 | 20,307 | 50,458 | 70,813 | 36.20% | yes |
| Bytes + 8 supplied mixed sequences | required | 48 | 20,307 | 16,864 | 37,219 | 19.02% | yes |
| Bytes + 4 supplied literals | required | 48 | 20,307 | 16,840 | 37,195 | 19.01% | yes |

### Table 2. Blocks skipped (percentage of all 16,794 blocks)

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Remainder bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + ending bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 772 (4.60%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + pair unions / 5 blocks | 418 (2.49%) | 426 (2.54%) | 770 (4.58%) | 16,794 (100.00%) | 440 (2.62%) | 1,001 (5.96%) | 621 (3.70%) |
| Bytes + all pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + all pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,176 (66.55%) | 2,710 (16.14%) |
| Bytes + 10 supplied pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + 10 supplied pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,176 (66.55%) | 2,710 (16.14%) |
| Bytes + 8 supplied mixed sequences | 418 (2.49%) | 1,281 (7.63%) | 16,653 (99.16%) | 16,794 (100.00%) | 440 (2.62%) | 16,780 (99.92%) | 2,710 (16.14%) |
| Bytes + 4 supplied literals | 418 (2.49%) | 1,281 (7.63%) | 16,360 (97.42%) | 16,794 (100.00%) | 440 (2.62%) | 16,386 (97.57%) | 2,710 (16.14%) |

### Table 3. Tested exact compressed layouts

| Phases | Order | Header B | Pair table B | Offsets B | Compressed payload B | Total B |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | block | 19 | 2886 | 532 | 1716667 | 1720104 |
| 1 | pair | 19 | 2886 | 532 | 1445407 | 1448844 |
| 2 | block | 19 | 2886 | 532 | 4920595 | 4924032 |
| 2 | pair | 19 | 2886 | 532 | 4586911 | 4590348 |

## Bounds prefixes

Stored prefix cost: **0 bytes**. Raw prefix bytes: 33,350. Nonempty prefixes: 16,758 blocks. Empty remainders: 0 terms. Observed remainder pairs: 1,443. Prefix extensions beyond index-derived prefixes: 0 blocks, 0 bytes before framing.

### Table 4. Serialized additional metadata

| Method | Caller patterns | Prefix B | Byte masks B | Feature B | Total B | Increase | Within +100% |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Remainder bytes | none | 0 | 20,307 | 0 | 20,307 | 10.38% | yes |
| Bytes + ending bytes | none | 0 | 20,307 | 61,325 | 81,632 | 41.73% | yes |
| Bytes + pair unions / 5 blocks | none | 0 | 20,307 | 151,352 | 171,659 | 87.74% | yes |
| Bytes + all pairs | none | 0 | 20,307 | 1,448,843 | 1,469,150 | 750.95% | no |
| Bytes + all pairs even/odd | none | 0 | 20,307 | 4,590,355 | 4,610,662 | 2356.71% | no |
| Bytes + 10 supplied pairs | required | 0 | 20,307 | 33,664 | 53,971 | 27.59% | yes |
| Bytes + 10 supplied pairs even/odd | required | 0 | 20,307 | 50,458 | 70,765 | 36.17% | yes |
| Bytes + 8 supplied mixed sequences | required | 0 | 20,307 | 16,864 | 37,171 | 19.00% | yes |
| Bytes + 4 supplied literals | required | 0 | 20,307 | 16,840 | 37,147 | 18.99% | yes |

### Table 5. Blocks skipped (percentage of all 16,794 blocks)

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Remainder bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + ending bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 772 (4.60%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + pair unions / 5 blocks | 418 (2.49%) | 426 (2.54%) | 770 (4.58%) | 16,794 (100.00%) | 440 (2.62%) | 1,001 (5.96%) | 621 (3.70%) |
| Bytes + all pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + all pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,175 (66.54%) | 2,710 (16.14%) |
| Bytes + 10 supplied pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + 10 supplied pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,175 (66.54%) | 2,710 (16.14%) |
| Bytes + 8 supplied mixed sequences | 418 (2.49%) | 1,281 (7.63%) | 16,653 (99.16%) | 16,794 (100.00%) | 440 (2.62%) | 16,780 (99.92%) | 2,710 (16.14%) |
| Bytes + 4 supplied literals | 418 (2.49%) | 1,281 (7.63%) | 16,360 (97.42%) | 16,794 (100.00%) | 440 (2.62%) | 16,386 (97.57%) | 2,710 (16.14%) |

### Table 6. Tested exact compressed layouts

| Phases | Order | Header B | Pair table B | Offsets B | Compressed payload B | Total B |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | block | 19 | 2886 | 532 | 1716668 | 1720105 |
| 1 | pair | 19 | 2886 | 532 | 1445406 | 1448843 |
| 2 | block | 19 | 2886 | 532 | 4920557 | 4923994 |
| 2 | pair | 19 | 2886 | 532 | 4586918 | 4590355 |

## Table 7. Independent full-term matches

| Query | Matching terms | Blocks with matches |
| --- | --- | --- |
| *a* | 1921269 | 16376 |
| *ab* | 75774 | 15490 |
| *traa* | 42 | 38 |
| *X | 0 | 0 |
| *x | 153990 | 16022 |
| *player* | 21 | 13 |
| *ms* | 50585 | 14065 |

## Interpretation

A supplied full-literal bit records presence entirely within the remainder. A prefix-crossing match can leave a shorter, untracked continuation. That case falls back to remainder-byte tests, so the supplied-literal row is no longer an exact block-presence oracle and can prune less than the mixed-sequence row. Mixed sequences can constrain fragments that the supplied full literals do not track. The same conservative continuation handling applies to generic and configured pairs.

Only four actual prefixes extend the prefix already derivable from existing index bounds, each by one byte. Actual mode therefore needs a 48-byte sparse extension table (12-byte header plus four 9-byte entries), rather than storing every prefix again. Bounds mode has zero prefix cost but leaves those four bytes in the corresponding remainders.

## Serialized layouts and validation

- `PX02`: magic, u32 block count, u32 extension-entry count, then entries containing u32 block ID, u32 extension length, raw extension bytes. Entries are sorted by block ID. Reconstruct each actual prefix by appending its extension (if any) to the LCP of existing index bounds. Decode into memory; no extra serialized offset table is needed. Header and entries are charged.
- `PM01`: magic, u32 block count, u32 bit count, u32 group size, u32 unique-mask count, u32 definition-byte count, u8 ID width. Definitions, sorted unique masks (ceil(bits/8) bytes each), then one little-endian mask ID per group. Byte and ending masks have 256 bits and group size 1. Pair unions use group size 5 and sorted u16 pair definitions.
- `PC01`: magic, u32 block count, u32 pair count, u32 group size, u8 order (0 block / 1 pair), u8 phases (1 / 2), u8 zstd level (3). Sorted u16 pair definitions, u32 group offsets including final offset, compressed payloads. Both orders use groups of 128 original blocks. Pair-major means one bitmap of blocks per pair/phase. Last groups use their actual block count. Smallest of the two tested orders is charged independently per exact row.
- Configured sequences use existing `SQ01` and configured parity uses `SP01`: magic, u32 block count, u32 sequence count, u32 row width, repeated u32 length + raw definitions, then fixed-width per-block masks. Parity bit 2*i is even and 2*i+1 is odd. All integers and masks are little-endian. Pair value is (first_byte << 8) | second_byte.
- Every sidecar is read back from disk and decoded. All decoded masks and definitions equal build-time data. Every compressed group is decompressed and transposed back. Evaluation uses decoded summaries and actual-mode decoded prefixes. Generic dictionaries are freshly derived from remainders, with no old full-term summaries reused.
- Fresh full-term scans validate each block and query against the independent evaluator. Every method has zero false-negative blocks. Per-block decisions and prefixes are in each mode’s `blocks.jsonl`. JSON includes candidate counts, block percentages, compressed-byte statistics, all tested layouts and supplied configurations.

## Reproduce

```sh
python3 -m unittest discover -s dictionary_analysis -p test_prefix_summaries.py
python3 dictionary_analysis/prefix_summaries.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis --config dictionary_analysis/selected_sequences.json --modes actual bounds
```

Requires the `zstd` executable. These are metadata/selectivity experiments, with no latency or physical I/O measurement.
