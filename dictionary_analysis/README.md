# Message dictionary analysis

**Start with [prefix-excluded block skipping versus metadata cost, separated by whether caller patterns are required](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/comparison.md).** All prices are serialized additional metadata, and all skipping percentages refer to blocks.

**Current convention: always remove each block's complete actual common prefix before building summaries.** Handle prefix-contained and crossing matches separately. Prefixes are reconstructed from index bounds plus a 48-byte sparse extension table. The current runner is `prefix_summaries.py`; the older full-term runners below are retained as historical baselines.

## Run the current comparison

```sh
python3 -m unittest discover -s dictionary_analysis -p 'test_*.py'
python3 dictionary_analysis/prefix_summaries.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis --config dictionary_analysis/selected_sequences.json --modes actual bounds
python3 dictionary_analysis/comparison.py
```

The `actual` mode strips the complete common prefix. The separate `bounds` diagnostic strips only the index-derived prefix. [Detailed results and layouts](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/prefix_summaries/prefix_summaries.md). Requires existing extraction/evaluation artifacts and the `zstd` executable.

## Document-frequency-ranked trigram selection

[Exact document-frequency results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_documents/report.md) use the real original message postings to count distinct documents per full-term trigram. Documents are unioned across matching terms. The unchanged stored block-posting rows still summarize full-prefix-excluded remainders. Synthetic key-only dictionaries have no document assignments and are excluded.

This ranking identifies repeated text that block frequency missed: `pla` appears in **6,026,979 documents** and 988 remainder blocks, while control `3ic` appears in **1,256 documents** and 974 blocks. The four player grams rank 17–21 by document frequency. `*player*` matches 5,884,377 distinct documents through 21 distinct terms. Summing those terms' document frequencies would overcount at 13,655,986.

At the 1 MB allowance, caps through 92% retain all four player grams and match full-trigram pruning: **14 candidates / 99.9166% skipping**. An independent rerun at the original **195,640-byte (+100%) allowance**, with the 80% per-row absence cutoff, retains **276 rows** and achieves the same player result. Cost includes 190,317 B selected trigrams, 5,275 B byte fallback and 48 B prefixes. This meets the historical player/metadata target with query-independent construction on the original dictionary.

The trade-off remains workload-dependent: `tra` and `raa` have low document frequency and stay omitted, so `*traa*` remains at 16,341 candidates (2.70% skipping). Short queries retain byte-only pruning. All tested predicates have zero false negatives. See [`trigram_documents/README.md`](trigram_documents/README.md) for reproduction and [the original-budget artifact](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_document_budgets/195640_skip80/report.json) for the smaller-budget verification.

### Broader queries and smaller budgets

A [42-query frozen-cohort evaluation](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_queries/report.md) checks the unchanged 195,640 B and 999,997 B indexes against fresh truth from every original key. Among the **34 present literals of at least three bytes**, the smaller index skips **69.62%** of blocks on an equally weighted average and reaches ≥99% skipping for **19/34** queries. The larger index skips **86.80%** on average and reaches ≥99% for **26/34**. This is a diagnostic cohort, not a representative workload.

At 195,640 B, `database`, `debug`, `status`, and `exception` retain 2, 12, 8, and 10 blocks. `error`, `user`, `server`, `service`, `team`, and `trace` retain about 16,300 blocks each because none of their trigrams were selected. At 999,997 B, those six improve to 15, 143, 6, 9, 128, and 2,125 candidates. `traa`, `3ic`, and `a76` remain poorly pruned at both budgets. The 80% cutoff excludes `123` regardless of these budgets. Retained `000` skips only 77.92% after prefix-safe continuations, illustrating that per-row absence is not a query-wide skipping guarantee. Short and absent queries are reported separately.

A separate [14-budget sweep](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_document_budgets/budget-sweep-report.md) keeps the 80% cutoff fixed. **19,998 B (+10.22%)** retains `pla`, `lay`, and `aye`, yielding **23 player candidates / 99.8630% skipping**. **25,000 B (+12.78%)** also retains `yer`, yielding **14 candidates / 99.9166% skipping**, unchanged at all larger tested budgets. The 20,000 B allowance is the first tested successful budget, rather than a proven minimum. All costs include bytes, prefixes, directories and frames. Other original queries retain byte-only pruning throughout this budget sweep.

Both experiments preserve existing indexes and source data. Every retained row is read back, new query truth is independent of selection, full-prefix crossing is preserved, and every measured decision has zero false negatives. Reproduction: [`trigram_queries/README.md`](trigram_queries/README.md), [`trigram_documents/budget-sweep.md`](trigram_documents/budget-sweep.md).

### Intermediate budgets on the same 42 queries

The [200 KB–1 MB comparison](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_query_budgets/report.md) holds exact document-DF ranking, the 80% cutoff and the frozen query cohort fixed. Eight actual indexes include all byte/prefix overhead:

| Budget, decimal KB | Actual added B | +original index % | Mean skip, 34 present ≥3-byte queries | Queries ≥99% skip | Client candidates |
| --- | ---: | ---: | ---: | ---: | ---: |
| 200 | 199,999 | 102.23% | 69.6226% | 19/34 | 1,070 |
| 300 | 300,000 | 153.34% | 83.9472% | 24/34 | 1,070 |
| 400 | 399,990 | 204.45% | 86.7808% | 26/34 | 15 |
| 500 | 499,997 | 255.57% | 86.8004% | 26/34 | 15 |
| 600 | 599,997 | 306.68% | 86.8004% | 26/34 | 15 |
| 750 | 749,997 | 383.36% | 86.8032% | 26/34 | 15 |
| 900 | 899,999 | 460.03% | 86.8046% | 26/34 | 15 |
| 1,000 | 999,997 | 511.14% | 86.8046% | 26/34 | 15 |

At 300 KB, error/user/server/service/team improve to 15/143/19/9/128 candidates. At 400 KB, all four client grams fit, reaching its 15 true blocks (99.9107% skipping). This is the first tested budget achieving that result, rather than a proven minimum. At 500 KB, server improves to 6 and failed to 2 candidates. The mean largely plateaus around 400–500 KB on this cohort. Traa and the omitted random fragments remain poorly pruned. Absent and short queries are reported separately.

All 5,642,784 query/block decisions were persisted, reloaded and recomputed. Zero false negatives occurred, and existing indexes/reports remain hash-identical. Selection sets are nonnested at one step (`os.` disappears at 900 KB), but no tested query gains candidate blocks as the budget increases. Reproduction and validation: [`trigram_queries/budgets.md`](trigram_queries/budgets.md).

### Relaxing the maximum block-presence limit

The [threshold comparison](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_query_thresholds/report.md) tests 20%, 21% and all-row eligibility across five budgets with the same frozen 42 queries. The separately requested **40%, 60%, 70% and 80% maximum-presence limits** were each serialized at every budget: all four produce byte-identical indexes at a given budget and match the independently evaluated all-row reference. [Requested-limit verification and reproduction](trigram_queries/requested_caps.md).

The maximum observed remainder-block frequency is only **21.3648%** (`217`: 3,588/16,794 blocks). Therefore every limit ≥22% admits all 47,796 observed rows. The 20% and 21% limits admit 47,198 and 47,763 respectively. This equivalence applies to the original dictionary, rather than all workloads.

For 34 present literals ≥3 bytes, relaxing from 20% to any requested higher limit changes mean block skipping as follows: 200 KB **69.6226→69.6226%**, 300 KB **83.9472→83.9044%**, 400 KB **86.7808→86.7798%**, 500 KB **86.8004→86.8004%**, and 1 MB **86.8046→89.0745%**. Actual higher-limit totals are 199,996 / 299,996 / 399,991 / 499,996 / 999,999 B, including all sidecars, corresponding to +102.23 / +153.34 / +204.45 / +255.57 / +511.14% of the original index.

At 1 MB, `123` improves from 16,794 to 3,809 candidates. It passes the relaxed cap but remains budget-omitted through 500 KB. Newly eligible high-document-frequency rows also displace other rows: at 300 KB, `server` worsens 19→138 and `timeout` 14→139. At 1 MB, `success` worsens 5→21 and `message` 7→15. Client and traa remain unchanged at every same-budget comparison. The 21% limit gives the same 1 MB result. All decisions preserve prefix safety and have zero false negatives. These results support a cohort-specific trade-off rather than a universal recommendation to relax the cap.

### Longer queries against the unchanged indexes

The [longer-query experiment](long_queries/README.md) freezes **89 queries** before evaluation: 41 predefined log-related words, 40 nested substrings from eight deterministically sampled real terms, and the eight prior table queries as controls. A fresh scan supplies exact truth for all 7,671,280 keys. The original 20%-presence-limit indexes remain unchanged. [Full results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_queries/report.md) and [examples/length interpretation](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_queries/interpretation.md).

The natural-word group contains **14 present and 27 absent queries**. Mean block skipping for the present words rises from **85.5892% / 92.0945% / 99.4781% / 99.9690% / 99.9838%** at 200/300/400/500 KB/1 MB respectively. Actual added bytes remain 199,999 / 300,000 / 399,990 / 499,997 / 999,997 B, including fallback and prefixes (+102.23 / +153.34 / +204.45 / +255.57 / +511.14% of the original index). `authentication` has 4/4/1/1/1 candidates for one true block. `heartbeat` has 16,332/1,048/3/3/3 for three true blocks. `unavailable` has 16,317/16,317/1,037/12/1 for one true block. Absent queries are reported separately: `keepalive` still has 947 candidates from 300 KB through 1 MB, versus zero with full trigrams.

All eight source samples are existing 32-byte lowercase hex IDs. Their nested 8/12/16/24/32-byte literals each match one term in one block. At 1 MB, mean candidates fall from **7,524.375 at 8 bytes to 3,490 at 32 bytes**. At 200 KB, the change is only **14,762.5→14,705.5**, entirely matching the byte-only reduction. Full trigrams average **158.125→1**, reaching one candidate by 16 bytes. Longer queries help when additional usable features or byte constraints are available. The small document-ranked indexes still lack many ID grams.

All 10,462,662 persisted query/mode/block decisions were reloaded, recomputed and audited with zero false negatives. All eight controls match previous per-block decisions. All 118 input fingerprints remain unchanged. Eighteen new and 34 prior tests passed, and an independent complete validation rerun reproduced results. An initial report-total variable-shadowing error was fixed and regression-tested; its truth and decision artifacts match the corrected run byte-for-byte, and the failed attempt is preserved. This frozen cohort is diagnostic, rather than representative traffic or a production-latency measurement.

### Longer-query retest at 10% presence steps

The [presence-limit retest](long_queries/thresholds.md) evaluates the **same 89-query cohort** at maximum block presence **10/20/30/40/50/60/70/80%**, each with 200/300/400/500 KB and 1 MB budgets. [Full results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_query_thresholds/report.md). All **40 configurations** were independently selected, serialized and verified. They yield **15 distinct indexes**: caps 30–80% remain byte-identical at a given budget because the maximum observed presence is 21.3648%. Every 20% result reproduces the previous longer-query experiment block-for-block.

**10% gives the best natural-word group means at every budget**, for both the 14 present and 27 absent literals. For the present group, mean candidates at 10% are **1,342 / 1,264.643 / 17.714 / 2.714 / 1.500** across the five budgets. At 400 KB the 10% / 20% / 30–80% means are **17.714 / 87.643 / 1,185**. For example, `unavailable` has **12 / 1,037 / 16,317** candidates respectively. Budget competition explains these differences: relaxing the cap admits popular numeric grams and displaces useful word grams. Individual words can favor a looser cap: `successful` has 116 candidates at 10% through 400 KB, versus 21 at the other caps.

**10% excludes all 232 distinct grams in the sampled ID queries**, whose block DFs are 1,961–3,566. All 40 ID queries consequently equal byte fallback at every 10%-cap budget. For these ID queries, best mean candidates use 20–80% at 200 KB, 30–80% at 300–500 KB, and 20% at 1 MB. At 1 MB, mean candidates across all ID lengths are **16,379.925 / 6,176.250 / 6,941.700** for 10% / 20% / 30–80%. Restricting to 32-byte literals instead slightly favors 30–80%: **3,470.875** versus **3,490** at 20%. The frozen groups support workload-specific trade-offs, rather than one universal threshold.

Actual additional 10%-cap sizes are **199,996 / 299,989 / 400,000 / 499,998 / 1,000,000 B**, including fallback and prefixes (+102.23 / +153.34 / +204.46 / +255.57 / +511.14% of the original index). All other actual sizes are in the report. All 27,779 retained row instances across 40 files were decoded and verified. Each audit pass recomputed **25,409,322 distinct query/block decisions** including references, with **59,786,640 logical partial decisions** covered through independently checked byte-identical aliases. Zero false negatives, all 362 source fingerprints preserved, 63 tests passed, and an independent full rebuild reproduced both reports. An initial execution timeout was preserved before a successful retry. A parent test rerun exposed an unclosed-file warning in the new test; the handle was closed explicitly and all 29 longer-query tests then passed cleanly.

## Frequency-first trigram selection under 1 MB

[The partial-index experiment](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_selection/report.md) fills a **1,000,000-byte total metadata budget (+511.14% of the original index)** with highest-block-frequency trigram rows, after excluding rows below an individual absence-fraction cutoff. The cutoff sweep is 0/50/70/80/85/90/92/94/95/98/99%, across all four dictionaries. Byte fallback and full-prefix extensions are included. The new `IU01` format persists omitted rows as unknown, while retained postings remain exact.

**All 44 selections omit all six trigrams used by the tested long queries. Every query consequently retains exactly the byte-only candidates.** At 80%, baseline stores 733 trigram rows at 999,999 total bytes but still retains 16,331 player blocks (2.76% skipping). Of those 733 retained grams, 732 consist entirely of hexadecimal characters. UUIDv4, UUIDv7 and alphanumeric cases retain 762/1,208/162 rows at approximately 1 MB, also with zero incremental pruning for this workload. These results concern the tested predicates rather than every possible query.

The per-row cutoff does not address which query grams receive the limited storage. Block frequency measures individual selectivity. Document frequency, which this experiment does not measure, could supply a different ranking signal for repeatedly used text. See [`trigram_selection/README.md`](trigram_selection/README.md) for reproduction, validation and persisted unknown semantics.

## Inverted trigram block postings

[Exact trigram posting measurements](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_postings/report.md) extend the independently accessible inverted index to every observed three-byte gram. All ten prior posting layouts were serialized and round-tripped on the original dictionary and the three ten-million-ID augmentations. Full common prefixes remain excluded. Prefix-contained and crossing matches are handled through continuation-aware predicates.

On the original dictionary, 47,796 observed trigrams produce 52,785,814 block memberships. The best measured index plus prefix sidecar costs **38,068,570 bytes (+19,458.48% of the original index)** and retains only **14 blocks for `*player*`**, versus 9,804 with bigrams. There are 13 true matching blocks. UUIDv4, UUIDv7 and alphanumeric augmentations retain 14, 13 and 19 blocks at 95,466,157, 76,984,062 and 233,435,059 bytes, respectively. All four meet the 90% skipping target and exceed the original 195,640-byte allowance in every tested layout.

Byte and bigram fallbacks preserve pruning for queries shorter than three bytes. On the original dictionary, the complete byte+bigram+trigram combination costs 39,659,898 bytes (+20,271.88%). The full report includes all seven queries, metadata percentages, exact matches and absolute candidates. The inverted trigram+byte layout is 18.58% smaller than the previous exact trigram bitset after normalizing byte-sidecar costs, with identical baseline decisions. These are measured layouts, rather than a compression lower bound or production performance result.

See [`trigram_postings/README.md`](trigram_postings/README.md) for reproduction. Only winning files are retained. Nonwinning layouts were verified before deletion and their sizes/hashes remain recorded.

## Inverted bigram and byte block postings

[Exact posting-list measurements](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/pair_postings/report.md) compare raw IDs, deltas, runs, bitmaps, and adaptive positive/complement encodings with per-posting zstd. Each feature has an independently accessible posting. All headers, sparse vocabularies, offsets and full-prefix extensions are included. Full block prefixes remain excluded and query continuations are checked conservatively. These experiments validate pruning through reconstructed masks, rather than implementing or benchmarking a direct posting-intersection planner.

The original dictionary has only 1,443 observed bigrams, but 18,626,806 pair/block memberships. Its best measured inverted pair index costs 1,586,101 bytes (+810.72% of the original index), preserving 41.62% `*player*` skipping. With ten million UUIDv4 keys, pair pruning reaches 90.81% at 4,522,580 bytes (+2311.68%). Exact inverted pairs exceed the original +100% allowance in all four tested dictionaries.

**The earlier byte costs depend strongly on encoding.** Inverted unigram postings preserve every byte-filter decision while reducing baseline byte metadata from 20,355 to 5,323 bytes (+2.72%), UUIDv4 from 2,184,368 to 231,360 bytes (+118.26%), UUIDv7 from 97,870 to 15,500 bytes (+7.92%), and alphanumeric IDs from 107,301 to 8,801 bytes (+4.50%). Augmented cases contain ten million added keys. All percentages use the original 195,640-byte index as a reference, rather than a rebuilt production index estimate. Earlier reports retain their measured PM01 costs for reproducibility.

See [`pair_postings/README.md`](pair_postings/README.md) for reproduction and the full report for all queries, absolute candidates, and pair-plus-byte totals.

## UUID versions, byte alphabets and fixed lengths

[Controlled UUID rebuilds](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/uuid_injection/report.md) test v1/v3/v4/v5/v6/v7 as canonical 36-byte and compact 32-byte whole tokens, plus fixed 36-byte random `0-9a-z` IDs. All original keys are preserved and complete common prefixes excluded. The canonical alphabet is `0-9a-f-`, rather than all ASCII. Byte+length bounds add **zero** rejections for the seven short unbounded-star queries in all 17 scenarios.

With one million canonical IDs, `*player*` skipping is 4.28–4.78% for v1/v3/v4/v5 and 30.72–30.84% for v6/v7. In the tested one-ID-per-millisecond sequence, chronological v6/v7 prefixes form thousands of generated-only blocks; v1 and hash/random versions interleave with the original keys. Using the original PM01 byte-mask interning layout, at ten million IDs v4 skips 76.29% at 2,184,368 sidecar bytes (+1116.52% of the original index reference), v7 skips 80.66% at 97,870 bytes (+50.03%), and alphanumeric IDs skip 0.42% at 107,301 bytes (+54.85%). Absolute candidate counts are 23,913, 16,314 and 103,148, respectively, versus baseline 16,331. Higher skip percentages do not necessarily mean fewer candidate blocks.

Sidecar costs are actually serialized; percentages use the original index as a reference, rather than estimating a production rebuild index. These whole-token experiments do not run the message tokenizer. Fixed node/clock fields, RNG seed and generation times are documented. See [`uuid_injection/README.md`](uuid_injection/README.md) for reproduction.

## Slowly increasing timestamps: insertion and block rebuild

[Controlled timestamp injection](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/timestamp_injection/report.md) adds consecutive numeric millisecond keys to the original dictionary and rebuilds actual key-only SSTables using the unchanged 4,000-byte key-delta target. The zero-injection case exactly reproduces all original block boundaries and byte sidecars. This directly tests timestamp clustering, separately from classifying existing numeric IDs.

Ten million distinct timestamps create 5,290 pure-numeric blocks among 22,085 total blocks. Byte masks reject every pure-numeric block. `*player*` skipping rises from 2.76% to 26.05%, while the absolute candidate count remains 16,331. Thus the filter avoids reading new numeric blocks rather than improving pruning of the existing mixed-text content. Repeated timestamp occurrences contribute only one distinct key each. Actual mask-plus-prefix metadata grows from 20,355 to 25,646 bytes (+10.40% to +13.11% relative to the original index as a reference). Key-only payload sizes are not production storage estimates.

See [`timestamp_injection/README.md`](timestamp_injection/README.md) for reproduction and validation.

## Numeric timestamps already in the dictionary

[Timestamp-candidate analysis](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/timestamp_pruning.md) rescans all original blocks for canonical numeric Unix timestamp candidates in 2000–2035 (seconds through nanoseconds). Numeric IDs can qualify, so the dictionary alone does not establish timestamp semantics. It finds 22,635 candidate terms in 271 mixed blocks. Byte masks skip 5.90% of those blocks for `*player*`, versus 2.71% of blocks without candidates. All 271 are actually negative for that query. No block is entirely numeric or at least 50% timestamp candidates. The 201 millisecond candidates occur in 18 blocks, and byte masks retain all 18 for `*player*`.

Numeric-only blocks would reject the letter-based queries, but adding numbers to existing mixed blocks preserves their existing letters. This analysis preserves terms and boundaries. It does not simulate inserting timestamps or rebuilding the SSTable. Shared byte metadata remains 20,355 bytes (+10.40%).

```sh
python3 dictionary_analysis/timestamp_pruning.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

## Generic trigrams: pruning without supplied patterns

The [generic trigram experiment](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/generic_trigrams/generic_trigrams.md) tests all remainder trigrams, exact bitsets, gap-filled runs, and Bloom filters. It preserves original blocks, excludes complete common prefixes, and builds every summary before query evaluation. Caller-configured sequences remain an optional separate experiment.

None of 67 tested constructions meets both ≥90% block skipping for `*player*` and the 195,640-byte added-metadata allowance. The highest observed budget-fitting trigram result skips 7.43% at 150,800 bytes. A 512-byte/block Bloom with two hashes costs 6.44–6.45 MB and skips 83.73–91.12% across three seeds. A 1,024-byte/block Bloom with one hash costs 15.33–15.34 MB and skips 97.55–98.61% across those seeds. These are actual whole-sidecar-compressed sizes, not resident memory. Exact trigrams skip 99.92% at 46,778,329 bytes. Results characterize the tested encodings, rather than a lower bound on every generic design.

```sh
python3 dictionary_analysis/generic_trigrams.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

## Pair ranges with safe gap filling

[Range encoding and small-gap filling](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/pair_runs/pair_runs.md) keeps per-block ranges of proven-absent pairs. It fills only zeros with ones and uses no caller patterns or neighbor unions. Actual common prefixes remain excluded. The raw gap-5 layout uses 169,690 added bytes and skips 6.51% of blocks for `*traa*`, 7.74% for `*player*`, and 5.15% for `*ms*`. Gap-3 plus whole-sidecar zstd uses 188,774 added bytes and skips 9.20%, 11.46%, and 6.32%, respectively. Compressed disk size differs from decoded memory.

```sh
python3 dictionary_analysis/pair_runs.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/comparison.py
```

## Separate mask deduplication

[Joint versus independent deduplication](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/separate_masks/README.md) tests two IDs per block with no neighbor unions. It distinguishes 256-bit byte-position masks from exact adjacent-pair masks. Both exclude the complete common prefix and need no caller-supplied patterns.

```sh
python3 dictionary_analysis/separate_masks.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

## Historical experiments and source metadata

**Read the [source metadata and original measured results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/README.md).** The historical whole-term costs below include prefixes in masks and are superseded by the current comparison.

**Budget follow-up: [exact shared masks within a +100% metadata budget](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/budget.md).** Whole-key byte masks serialize to 20,231 bytes (+10.34%). Whole-key and ending-byte masks together use 81,544 bytes (+41.68%).

Input: `KH4Y_01M5MTJQ3SVHDTK2W3XCB5/`, field `message` (schema field 3). The study preserves original physical blocks and leaves source data and library code unchanged.

**Caller-configured sequence results:** [selected_sequences.md](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/selected_sequences.md) adds `*player*` and `*ms*` and tests explicit external lists of pairs, trigrams, and longer sequences. Eight supplied sequences plus shared byte masks use 37,095 bytes (+18.96% of the original index metadata).

**Pattern-aware block boundaries:** [partition_boundaries.md](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/partition_boundaries.md) measures the number of sorted runs created by cutting at caller-supplied sequence-bit changes. It also describes the larger architectural change needed for separate pattern partitions.

**Pair-offset masks:** [pair_parity.md](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/pair_parity.md) compares one overlapping-pair mask with separate even/odd-offset masks. Ten externally configured pairs plus whole-byte masks serialize to 70,689 bytes (+36.13%), skipping 45.75% of data bytes for `*traa*` and 68.12% for `*player*`.

## Readable outputs

- [Current block metadata](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/existing_metadata.tsv): all blocks, stored fields separated from derived observations.
- [Address-store group metadata](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/address_store_metadata.tsv).
- [Blocks ranked by numeric percentage](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/numeric_blocks.tsv).
- [Complete terms from the most numeric-heavy block](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/block_2738_terms.txt).

## Source extraction and historical reproduction tools

1. [extract](extract/README.md): copy the original message SSTable, decode terms and block metadata, measure numeric keys, and run current index pruning.
2. [evaluate](evaluate/README.md): compare exact query results against bounds/length/byte/pair/trigram summary models, including prefix-aware product-automaton tests.
3. [filters](filters/README.md): construct actual Bloom filters with three fixed hash seeds, compare storage/selectivity, and test ending-byte summaries.
4. `python3 dictionary_analysis/report.py`: regenerate the human-readable report and numeric-block samples from the results.
5. `python3 dictionary_analysis/budget_masks.py`: measure shared masks, serialize and roundtrip-check their tables, and regenerate the budget follow-up.
6. `python3 dictionary_analysis/budget_pairs.py`: measure pair-mask repetition, grouped zstd compression, and shared pair unions. See [pair-mask budget results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/budget_pairs.md). Requires `zstd`.

7. `python3 dictionary_analysis/selected_sequences.py --config dictionary_analysis/selected_sequences.json --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis`: build and evaluate exactly the caller-supplied sequence lists. The helper requires external configuration. It selects no sequences automatically. Tests: `python3 -m unittest discover -s dictionary_analysis -p 'test_selected_sequences.py'`.

8. `python3 dictionary_analysis/partition_boundaries.py --config dictionary_analysis/selected_sequences.json --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis`: count pattern-aware splits while preserving sorted term order and all original block boundaries. It does not measure re-encoded data or index sizes.

9. `python3 dictionary_analysis/pair_parity.py --config dictionary_analysis/selected_sequences.json --configuration pairs --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis`: build and test pair masks separated by full-term byte-offset parity. Tests: `python3 -m unittest discover -s dictionary_analysis -p 'test_pair_parity.py'`.

10. `python3 dictionary_analysis/comparison.py`: regenerate the consolidated block-skipping/storage tables, with separate sections for no pattern input and caller-supplied pattern lists.

The Rust helpers are separate Cargo workspaces with locked dependencies. Build artifacts are ignored within each helper directory. Full reproduction commands and input/output definitions are in the linked reports.

The current comparison reports original blocks skipped and actually serialized additional metadata. Historical reports also contain compressed-byte skipping and explicitly labeled layout estimates. These are not measured latency or physical I/O. Matching uses case-sensitive byte wildcards; production query normalization is outside this study.
