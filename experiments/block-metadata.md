# Dictionary metadata: serialized size and candidate blocks

Input: split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, segment `4fea6c896fd045aba3cece0808f3e41e`, field 3 (`message`).
These measurements preserve its original physical dictionary blocks. The [dictionary baseline](dictionary.md) describes the extracted keys and existing index.

| Input | Measurement |
| --- | ---: |
| Distinct dictionary terms | 7,671,280 |
| Physical blocks | 16,794 |
| Documents, with no deletes | 10,002,000 |
| Compressed data blocks | 51,482,326 B |
| Existing index, including FST-length field | 195,640 B |

A candidate is a physical block that the metadata permits. A true block contains
at least one matching full key. Matching-term counts count distinct dictionary
keys. Block skipping is `100 × (16794 − candidates) / 16794`. Additional-metadata
percentages divide serialized bytes by **195,640**, rather than by data size or
the requested budget. KB and MB are decimal. These are storage and block-pruning
measurements. Query execution, network reads and latency were not measured.

Matching is case-sensitive on raw bytes. `*literal*` means substring and `*X` and `*x` mean suffix.
`*` consumes any bytes, including newline. The evaluator applies no production analyzer or lowercasing. Uppercase `X` occurs in no key.

## Full-prefix handling and exact postings

Each block's **actual longest common prefix (LCP)** is removed before extracting within-term byte, pair and trigram memberships.
Grams cross neither term boundaries nor the removed-prefix boundary. Existing index fences plus a 48-byte sparse prefix-extension sidecar reconstruct the actual prefixes.

The predicate ORs prefix-contained, remainder-contained and viable prefix-crossing
matches. Each crossing checks only its remainder continuation. For example,
`player` requires `pla`, `lay`, `aye`, `yer` when wholly in the remainder. A shorter
crossing continuation requires only its own grams. Continuations of length zero,
one or two remain possible even when no trigram can test them. Intersecting all
whole-literal grams after removing a prefix would discard valid matches.

The postings are exact feature-to-block sets. Features can occur in different
terms or unrelated positions within a block, so their intersection can still
admit blocks with no matching key. A suffix query also needs an end-position test
to reach its exact-match ceiling. The byte postings here record presence anywhere.

### Serialized representation costs

Rows are independently accessible through a sparse observed vocabulary and byte
offsets. The adaptive encoding chooses the smallest row among raw IDs, delta
varints, runs and bitmaps, in positive or complement polarity, plain or individually
zstd-compressed at level 3. Selection uses serialized size and receives no queries.
Headers, vocabulary, offsets, codec/polarity tags and compression framing are charged.

| Feature | Observed rows | Block memberships | Adaptive-zstd posting file B |
| --- | ---: | ---: | ---: |
| Bytes | 41 | 595,304 | 5,275 |
| Pairs | 1443 | 18,626,806 | 1,586,053 |
| Trigrams | 47796 | 52,785,814 | 38,068,522 |

The following totals include the 48-byte prefix sidecar once. Combined layouts
concatenate independent postings. Byte fallback is charged only where named.
These are actual serialized sizes for the prefix-excluded predicates below,
separate from the whole-term layout estimates in the dictionary baseline.

| Metadata | Additional B | Additional / existing index |
| --- | ---: | ---: |
| Bytes | 5,323 | +2.72% |
| Pairs | 1,586,101 | +810.72% |
| Pairs + bytes | 1,591,376 | +813.42% |
| Full trigrams | 38,068,570 | +19458.48% |
| Full trigrams + bytes | 38,073,845 | +19461.18% |
| Full trigrams + pairs + bytes | 39,659,898 | +20271.88% |

### Original seven queries

Cells are **candidate blocks / block skip %**. Oracle cells give true blocks and
the maximum block skip %. The existing index retains all 16,794 blocks for each
query. Pair-only and trigram-only columns have no byte fallback.

| Query | Matching terms | Oracle blocks / skip | Bytes | Pairs | Pairs + bytes | Trigrams | Trigrams + bytes | Trigrams + pairs + bytes |
| --- | ---: | --- | --- | --- | --- | --- | --- | --- |
| `*X` | 0 | 0 / 100.00% | 0 / 100.00% | 16,794 / 0.00% | 0 / 100.00% | 16,794 / 0.00% | 0 / 100.00% | 0 / 100.00% |
| `*a*` | 1921269 | 16,376 / 2.49% | 16,376 / 2.49% | 16,794 / 0.00% | 16,376 / 2.49% | 16,794 / 0.00% | 16,376 / 2.49% | 16,376 / 2.49% |
| `*ab*` | 75774 | 15,490 / 7.76% | 16,373 / 2.51% | 15,513 / 7.63% | 15,513 / 7.63% | 16,794 / 0.00% | 16,373 / 2.51% | 15,513 / 7.63% |
| `*ms*` | 50585 | 14,065 / 16.25% | 16,339 / 2.71% | 14,084 / 16.14% | 14,084 / 16.14% | 16,794 / 0.00% | 16,339 / 2.71% | 14,084 / 16.14% |
| `*player*` | 21 | 13 / 99.92% | 16,331 / 2.76% | 9,804 / 41.62% | 9,804 / 41.62% | 14 / 99.92% | 14 / 99.92% | 14 / 99.92% |
| `*traa*` | 42 | 38 / 99.77% | 16,341 / 2.70% | 11,818 / 29.63% | 11,818 / 29.63% | 141 / 99.16% | 141 / 99.16% | 141 / 99.16% |
| `*x` | 153990 | 16,022 / 4.60% | 16,354 / 2.62% | 16,794 / 0.00% | 16,354 / 2.62% | 16,794 / 0.00% | 16,354 / 2.62% | 16,354 / 2.62% |

For `*traa*`, full trigrams retain 141 blocks for 38 true blocks. For `*ab*`,
pairs retain 15,513 for 15,490 true blocks. The two-byte query gets only byte
pruning from trigrams + bytes, retaining 16,373 blocks.

## Document-frequency selection at a fixed 20% block-presence limit

**Ranking** counts the distinct-document union for each trigram across full raw-byte
terms. Repeated grams within a term and documents shared across matching terms are
deduplicated. Adding term document frequencies would overcount this union.
**Eligibility** instead counts blocks containing the gram in their LCP-excluded
remainders. A row qualifies exactly when `blockDF × 100 ≤ 16794 × 20`.
This admits 47,198 of the 47,796 observed remainder-trigram rows.

Eligible rows are ordered by descending exact document DF, then ascending u24 gram
value. Whole rows that fit are retained. A nonfitting row is skipped and selection
continues. Queries and their truth counts never enter selection. The budget includes
5,275 B of byte postings, 48 B of prefixes and a partial file costing
`18 + 7 × retained_rows + sum(encoded_frame_bytes)`.

An omitted partial-index gram means **unknown**, including an unobserved gram.
It cannot reject a block. A missing row in the complete trigram index means absent.
Both eligibility exclusions and budget omissions use this conservative partial
semantics. Prefix continuations and byte fallback remain active at every budget.

These grams illustrate the two different frequencies. Global rank is before eligibility filtering.

| Gram | Document DF | Block DF | Global rank |
| --- | ---: | ---: | ---: |
| `pla` | 6026979 | 988 | 17 |
| `lay` | 5892760 | 992 | 19 |
| `aye` | 5915284 | 1032 | 18 |
| `yer` | 5884479 | 1067 | 21 |
| `tra` | 1653 | 1036 | 6213 |
| `raa` | 1292 | 1020 | 23974 |
| `3ic` | 1256 | 974 | 38839 |

### Five budgets, same 42 queries

Mean skip below weights each of the **34 present literals of at least three bytes**
equally. The full table also retains four present one/two-byte queries and four
absent queries (`*X`, `*warning*`, `*login*`, `*deadbeef*`).

| Budget B | Actual additional B | % of 195640 | Retained rows | Mean skip % (34 queries) | Queries ≥99% skip |
| --- | ---: | ---: | ---: | ---: | ---: |
| 200000 | 199999 | 102.23 | 304 | 69.6226 | 19 |
| 300000 | 300000 | 153.34 | 476 | 83.9472 | 24 |
| 400000 | 399990 | 204.45 | 646 | 86.7808 | 26 |
| 500000 | 499997 | 255.57 | 828 | 86.8004 | 26 |
| 1000000 | 999997 | 511.14 | 1223 | 86.8046 | 26 |

Budget columns are absolute candidate blocks. False-positive blocks equal candidates minus true blocks. Every false-negative count is zero.

| Query | Matching terms | True blocks | 200000 | 300000 | 400000 | 500000 | 1000000 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `*X` | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `*a*` | 1921269 | 16376 | 16376 | 16376 | 16376 | 16376 | 16376 |
| `*ab*` | 75774 | 15490 | 16373 | 16373 | 16373 | 16373 | 16373 |
| `*ms*` | 50585 | 14065 | 16339 | 16339 | 16339 | 16339 | 16339 |
| `*player*` | 21 | 13 | 14 | 14 | 14 | 14 | 14 |
| `*traa*` | 42 | 38 | 16341 | 16341 | 16341 | 16341 | 16341 |
| `*x` | 153990 | 16022 | 16354 | 16354 | 16354 | 16354 | 16354 |
| `*error*` | 7 | 6 | 16341 | 15 | 15 | 15 | 15 |
| `*warning*` | 0 | 0 | 1047 | 89 | 89 | 89 | 89 |
| `*info*` | 42 | 35 | 146 | 146 | 146 | 146 | 146 |
| `*debug*` | 2 | 2 | 12 | 12 | 12 | 12 | 12 |
| `*request*` | 22 | 11 | 101 | 11 | 11 | 11 | 11 |
| `*response*` | 11 | 4 | 97 | 12 | 4 | 4 | 4 |
| `*timeout*` | 6 | 5 | 139 | 14 | 14 | 14 | 14 |
| `*failed*` | 2 | 1 | 996 | 996 | 100 | 2 | 2 |
| `*success*` | 9 | 4 | 21 | 21 | 21 | 21 | 5 |
| `*user*` | 42 | 36 | 16337 | 143 | 143 | 143 | 143 |
| `*server*` | 11 | 6 | 16334 | 19 | 19 | 6 | 6 |
| `*client*` | 25 | 15 | 1070 | 1070 | 15 | 15 | 15 |
| `*session*` | 4 | 2 | 87 | 87 | 87 | 87 | 87 |
| `*connection*` | 9 | 5 | 9 | 9 | 6 | 5 | 5 |
| `*login*` | 0 | 0 | 1396 | 1396 | 1396 | 1396 | 1396 |
| `*trace*` | 1 | 1 | 16340 | 16340 | 2125 | 2125 | 2125 |
| `*cache*` | 9 | 4 | 1078 | 1078 | 1078 | 1078 | 1078 |
| `*database*` | 5 | 2 | 2 | 2 | 2 | 2 | 2 |
| `*process*` | 12 | 6 | 11 | 6 | 6 | 6 | 6 |
| `*message*` | 13 | 7 | 146 | 15 | 15 | 15 | 7 |
| `*status*` | 9 | 7 | 8 | 8 | 8 | 8 | 8 |
| `*service*` | 27 | 9 | 16323 | 9 | 9 | 9 | 9 |
| `*http*` | 66 | 38 | 130 | 130 | 130 | 130 | 130 |
| `*exception*` | 14 | 7 | 10 | 10 | 7 | 7 | 7 |
| `*game*` | 71 | 44 | 126 | 126 | 126 | 126 | 126 |
| `*team*` | 42 | 37 | 16336 | 128 | 128 | 128 | 128 |
| `*match*` | 4 | 2 | 23 | 23 | 23 | 23 | 23 |
| `*time*` | 66 | 51 | 139 | 139 | 139 | 139 | 139 |
| `*name*` | 50 | 47 | 148 | 148 | 148 | 148 | 148 |
| `*world*` | 1 | 1 | 1362 | 1362 | 1362 | 1362 | 1362 |
| `*3ic*` | 1255 | 1008 | 16349 | 16349 | 16349 | 16349 | 16349 |
| `*a76*` | 2871 | 2103 | 16376 | 16376 | 16376 | 16376 | 16376 |
| `*000*` | 8333 | 3320 | 3708 | 3708 | 3708 | 3708 | 3708 |
| `*123*` | 12372 | 3443 | 16794 | 16794 | 16794 | 16794 | 16794 |
| `*deadbeef*` | 0 | 0 | 2029 | 2029 | 2029 | 2029 | 2029 |

At 400000 B, `client` reaches its 15 true blocks. `traa` stays at 16341 candidates
through 1000000 B because `tra` and `raa` remain omitted. Their block frequencies
meet the 20% eligibility rule. Their document ranks keep them outside these budgets.
The skip-nonfitting rule permits row replacement, so larger budgets need not form
nested selections. No query gained a candidate block in the measured budget sweep.

## Log words and ID substrings at controlled presence limits

The separate 89-query cohort was frozen before evaluation: 41 predefined log-word literals, 40 source-derived ID literals and eight controls.
Exact scans split the log words into 14 present and 27 absent queries. Controls are `error`, `user`, `service`, `client`, `server`, `failed`, `trace`, `traa`. All 89 use substring matching.

The ID sample took the eight smallest seeded SHA-256 ranks among 151,143 unique ASCII-alphanumeric terms of at least 32 bytes, independently of index decisions.
All eight sampled terms were 32-byte lowercase hex IDs. Each contributes nested 8/12/16/24/32-byte literals, each matching one term in one block.
These groups are diagnostics with equal query weighting, rather than a traffic-weighted workload.

The ranking and accounting stay fixed. Only eligibility changes to `blockDF × 100 ≤ cap × 16794`.
Caps 10%, 20% and 30% admit 43700, 47198 and 47796 rows respectively. Maximum observed remainder-block presence is 3588/16,794 (21.364773133262%), so tested caps 30% through 80% produced identical indexes.

| Budget B | Cap % | Actual additional B | Retained rows |
| --- | ---: | ---: | ---: |
| 400000 | 10 | 400000 | 686 |
| 400000 | 20 | 399990 | 646 |
| 400000 | 30 | 399991 | 621 |
| 1000000 | 10 | 1000000 | 2063 |
| 1000000 | 20 | 999997 | 1223 |
| 1000000 | 30 | 999999 | 1207 |

Cells below are mean candidate blocks. True-block sums count query/block pairs: 20 for present log words, 0 for absent log words, 40 for IDs and 112 for controls.
The references cost 5,323 B (bytes + prefixes) and 38,073,845 B (full trigrams + bytes + prefixes).

| Budget / reference | Cap % | Present log words (14) | Absent log words (27) | IDs (40) | Controls (8) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Bytes | — | 16316.857 | 16311.815 | 16379.925 | 16335.125 |
| 400000 | 10 | 17.714 | 45.296 | 16379.925 | 4108.875 |
| 400000 | 20 | 87.643 | 49.148 | 13012.350 | 2345.875 |
| 400000 | 30 | 1185.000 | 85.741 | 12897.225 | 2345.875 |
| 1000000 | 10 | 1.500 | 4.259 | 16379.925 | 4108.875 |
| 1000000 | 20 | 2.714 | 40.963 | 6176.250 | 2332.000 |
| 1000000 | 30 | 4.071 | 41.000 | 6941.700 | 2332.000 |
| Full trigrams + bytes | — | 1.500 | 0.000 | 32.600 | 43.625 |

At 1000000 B, changing the cap from 10% to 20% lowers the ID mean from 16379.925
to 6176.250 and raises the present-word mean from 1.500 to 2.714. The 10% cap
excludes every trigram used by these ID queries, leaving their byte-only decisions.
Relaxing eligibility admits rows that compete with word grams for the same budget.

## Exact-match validation

The reported runs checked source hashes, physical boundaries, ordinals, full LCPs and decoded byte summaries.
Every posting row in each tested full encoding was reopened and compared with extraction. Every retained partial row was decoded and compared with the full index.
Query decisions used serialized postings.

Independent full-key scans established truth over all 7,671,280 keys for the 42- and 89-query cohorts.
Budget and presence-limit evaluations reused that truth after hash/provenance checks. Persisted candidate masks were reloaded, recomputed and recounted.
For every query/block, `truth ⊆ full trigrams+bytes ⊆ partial+bytes ⊆ bytes`.
All seven-query representation modes also retained every truth-bearing block.
There were **zero false-negative blocks** in these reported evaluations. The
89-query cap sweep decoded 27,779 retained rows across 40 files and checked
25,409,322 distinct query/block/index decisions per pass in two passes.
