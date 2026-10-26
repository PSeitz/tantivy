# Parent handoff: same 89-query presence-cap retest

## Status and paths

Implementation, experiment and independent validation completed successfully.
The independent full rebuild produced identical report JSON and Markdown.
Final status is recorded in `completion.json` and `logs/validate.log`.

Owned source files:

- `dictionary_analysis/long_queries/thresholds.py`
- `dictionary_analysis/long_queries/test_thresholds.py`
- `dictionary_analysis/long_queries/thresholds.md`

All new artifacts are inside
`KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_query_thresholds/`.
Earlier code, cohort, source-family mapping, truth, indexes and reports remain
hash-identical. The parent retains ownership of the main README and final answer.

**Canonical results:** `report.json` and `report.md`. The Markdown shows all eight
requested caps for all five budgets, including every query. This handoff groups
30–80 only where their independently verified equivalence makes that clearer.

## Main findings

1. **10% is a meaningful new policy.** It gives the lowest equal-query mean
   candidate count for both natural-present and natural-absent groups at every
   tested budget. This is a group result. Individual queries such as `successful`
   can prefer 20% at smaller budgets.
2. **10% excludes every sampled ID-query gram.** Measured across the frozen
   source-derived queries, all 232 distinct grams have block DF 1961–3566,
   above the inclusive 10% limit of 1679.4. Every one of the 40 ID queries has
   exactly the byte-only candidate set at every 10%-cap budget, including 1 MB.
   This statement concerns these sampled grams. It makes no universal assertion
   about every possible hex gram or identifier workload.
3. **20% exactly reproduces the previous longer-query experiment.** All five
   serialized files equal their earlier deployed files byte-for-byte. All 89
   query masks match the prior run on every block, including byte/full references.
4. **30%, 40%, 50%, 60%, 70%, 80% are equivalent here.** Maximum observed row
   presence is 3588/16794 = **21.364773133262%**. These six caps admit all 47,796
   rows. Full eligible-key lists, selected-key lists, copied frames, file bytes,
   hashes and query masks match at each budget. All 30 files were still explicitly
   built and verified.
5. **Relaxation trades useful grams under a fixed budget.** At 400 KB,
   natural-present means are **17.714 / 87.643 / 1185.000** for 10 / 20 / 30–80%.
   ID-query means move in the other direction: **16379.925 / 13012.350 /
   12897.225**. At 1 MB the ID mean prefers 20% over 30–80%, despite the latter's
   slightly better mean for 32-byte queries. The longer-ID result depends on
   family and length.
6. **Controls favor different caps by budget.** The eight-control mean prefers
   10% at 200/300 KB and ties across 20–80% at 400 KB, 500 KB and 1 MB. `trace`
   is the main counterexample to uniformly favoring 10%: its `ace` gram exceeds
   10%, while the looser indexes retain it from 400 KB onward.
7. **No adjacent-budget regressions occurred in this tested grid.** Selected
   row sets were nested for each fixed cap, and no larger-budget transition
   newly allowed a candidate block for any query. This is a measured result.
   The skip-nonfitting algorithm generally permits nonnested selection. Same-
   budget cap relaxation did remove rows and allow new candidate blocks.

## Actual serialized costs

These are additional metadata bytes, including the same 5323 B byte/prefix
sidecars charged once. Original metadata is 195640 B. Each table cell is
**actual B / retained rows / additional percent of original**. The six equivalent
caps each have their own actual serialized file.

| Budget B | 10% | 20% | Each of 30/40/50/60/70/80% |
|---:|---:|---:|---:|
| 200000 | 199996 / 299 / +102.226539% | 199999 / 304 / +102.228072% | 199996 / 306 / +102.226539% |
| 300000 | 299989 / 482 / +153.337252% | 300000 / 476 / +153.342875% | 299996 / 424 / +153.340830% |
| 400000 | 400000 / 686 / +204.457166% | 399990 / 646 / +204.452055% | 399991 / 621 / +204.452566% |
| 500000 | 499998 / 864 / +255.570435% | 499997 / 828 / +255.569924% | 499996 / 760 / +255.569413% |
| 1000000 | 1000000 / 2063 / +511.142916% | 999997 / 1223 / +511.141382% | 999999 / 1207 / +511.142404% |

Eligible rows: **43700 at 10%, 47198 at 20%, 47796 at each cap 30–80%**.
Partial-file bytes equal `18 + 7*rows + sum(original frame bytes)`. Reports,
truth, decisions and helper memory are excluded. The complete trigram reference
costs **38073845 B**, including the same sidecars.

## Separate group means

Candidate physical blocks, lower is better. Every query receives equal weight.
The order in each cell is **10% / 20% / each cap 30–80%**. All false-negative
counts are zero.

| Budget B | Natural present (14) | Natural absent (27) | Source-derived IDs (40) | Controls (8) |
|---:|---:|---:|---:|---:|
| 200000 | 1342.000 / 2420.143 / 2422.143 | 809.222 / 810.444 / 819.889 | 16379.925 / 14717.800 / 14717.800 | 6530.750 / 12510.250 / 12510.250 |
| 300000 | 1264.643 / 1327.643 / 1327.786 | 93.889 / 93.926 / 133.148 | 16379.925 / 13012.350 / 12897.225 | 4134.500 / 4366.625 / 4381.500 |
| 400000 | 17.714 / 87.643 / 1185.000 | 45.296 / 49.148 / 85.741 | 16379.925 / 13012.350 / 12897.225 | 4108.875 / 2345.875 / 2345.875 |
| 500000 | 2.714 / 5.214 / 10.500 | 40.963 / 41.000 / 45.296 | 16379.925 / 13012.350 / 12897.225 | 4108.875 / 2332.000 / 2332.000 |
| 1000000 | 1.500 / 2.714 / 4.071 | 4.259 / 40.963 / 41.000 | 16379.925 / 6176.250 / 6941.700 | 4108.875 / 2332.000 / 2332.000 |

True-block sums are 20 / 0 / 40 / 112 for these four separate groups. Full-
reference means are 1.500 / 0 / 32.600 / 43.625. The natural-present mean at
10%, 1 MB equals the full reference, including `validation`'s residual full-
reference false positive. The ID means remain far above the complete reference.

Best tested ID-group caps: 20–80% tie at 200 KB, 30–80% tie at 300/400/500 KB,
and 20% wins at 1 MB. These recommendations apply to these frozen groups and
weights, rather than a universal best cap.

## Requested natural-word examples

Each cell is **10% / 20% / each cap 30–80%** candidates. Queries retain the same
exact truth at every setting. All budgets are decimal bytes.

| Literal | Terms / true blocks | Full candidates | 200000 | 300000 | 400000 | 500000 | 1000000 |
|---|---:|---:|---:|---:|---:|---:|---:|
| successful | 3 / 1 | 1 | 116 / 21 / 21 | 116 / 21 / 21 | 116 / 21 / 21 | 5 / 21 / 21 | 1 / 5 / 21 |
| validation | 2 / 2 | 3 | 7 / 7 / 16 | 7 / 7 / 7 | 7 / 7 / 7 | 7 / 7 / 7 | 3 / 7 / 7 |
| unavailable | 1 / 1 | 1 | 16317 / 16317 / 16317 | 16317 / 16317 / 16317 | 12 / 1037 / 16317 | 1 / 12 / 12 | 1 / 1 / 1 |
| heartbeat | 4 / 3 | 3 | 1048 / 16332 / 16332 | 84 / 1048 / 1048 | 3 / 3 / 84 | 3 / 3 / 3 | 3 / 3 / 3 |
| authentication | 1 / 1 | 1 | 4 / 4 / 4 | 1 / 4 / 4 | 1 / 1 / 1 | 1 / 1 / 1 | 1 / 1 / 1 |
| keepalive | 0 / 0 | 0 | 16322 / 16322 / 16322 | 947 / 947 / 947 | 947 / 947 / 947 | 947 / 947 / 947 | 1 / 947 / 947 |
| healthcheck | 0 / 0 | 0 | 1078 / 1078 / 1078 | 1078 / 1078 / 1078 | 108 / 108 / 1078 | 108 / 108 / 108 | 108 / 108 / 108 |

`report.md` gives all eight original controls at every cap/budget, plus every
other frozen query. Examples: at 200 KB, 10% improves `service` from 16323 to 9,
`user` from 16337 to 1011 and `server` from 16334 to 138. At 400 KB, `trace`
instead improves from 16340 at 10% to 2125 at 20–80%. `traa` remains at 16341
candidates throughout this grid, against 38 true blocks and 141 full-reference
candidates.

## Actual omission and displacement evidence

Every gram's measured block DF, document DF, global ranking position, exact
incremental cost and per-configuration omission reason is in `report.json`.
A row above the cap is excluded. An eligible missing row is omitted by budget/
DF ordering, including skip-nonfitting effects. Both read as unknown.

| Gram | Remainder block DF | Exact document DF | Incremental B (7+frame) | Explanation |
|---|---:|---:|---:|---|
| cce | 2146 | 1362450 | 1279 | Above 10%, retained at 20–80%. Explains why `successful` is worse at 10% through 400 KB. |
| ssf | 1014 | 38143 | 771 | Eligible at 10%. Retained only by 10%, 1 MB among these settings. Together with other suffix grams lets `successful` reach one candidate. |
| art | 1048 | 719900 | 782 | Eligible at every cap. At 200 KB retained at 10%, budget-omitted at 20–80%. `heartbeat`: 1048 versus 16332. |
| bea | 2057 | 227919 | 1241 | Excluded at 10%. Other retained `heartbeat` grams still reach its three true blocks by 400 KB. |
| ail | 1060 | 162854 | 791 | At 400 KB retained at 10%, budget-omitted at 20–80%. |
| abl | 999 | 173706 | 772 | At 400 KB retained at 10%, budget-omitted at 20–80%. |
| ble | 1037 | 214107 | 781 | At 400 KB retained at 10/20%, budget-omitted at 30–80%. `unavailable` consequently gives 12 / 1037 / 16317. |
| hea | 1012 | 226640 | 768 | At 400 KB retained at 10/20%, budget-omitted at 30–80%. Shared by `heartbeat` and `healthcheck`. |
| epa | 991 | 23864 | 761 | At 1 MB retained at 10%, budget-omitted at 20–80%. With `pal/ali/ive`, reduces absent `keepalive` to one candidate. |
| ace | 2125 | 339102 | 1269 | Above 10%. Retained at 20–80% from 400 KB onward, improving `trace`. |
| 026 | 3342 | 8739631 | 1442 | Above 10%, below 20%. Admitted ahead of many word grams when relaxing 10→20%. |
| 202 | 3550 | 8742968 | 1453 | Above 20%. Admitted ahead of many word grams when relaxing 20→30%. |

Only three of 266 distinct natural-query grams exceed 10%, versus all 232 sampled
ID-query grams. One of 23 distinct control grams exceeds 10%. Relaxation therefore
changes budget competition and directly admits some useful query grams. It can
improve an ID or word query while displacing rows used by another query.

At 1 MB, relaxing 20→30% adds 231 rows and removes 247. Actual ID examples:

- Family 1's 32-byte query loses `267` (block DF 3338, document DF 90951,
  1426 B), an eligible row displaced by budget/rank. Candidates worsen
  **3065→16367**.
- Family 3's 8-byte query loses `668` (3328, 85057, 1449 B). Candidates worsen
  **3080→16373**.
- Family 6's 24/32-byte queries gain newly eligible `127` (3452, 132534,
  1429 B). Candidates improve **16367→3034**.
- Family 0's 24-byte query exchanges `702` (3352, 85758, 1437 B) for newly
  eligible `209` (3450, 99582, 1418 B). Counts improve **3081→3036**, while
  **1507 blocks become newly allowed and 1552 become newly pruned**. Count
  improvement alone therefore does not establish candidate-set containment.

These facts come from serialized-row lookups and persisted candidate masks.
Prefix-contained matches require no remainder grams. Prefix-crossing matches use
only the continuation's grams. Whole-query retention alone does not establish
query-level pruning or explain every prefix case.

## Length tradeoff

Eight source families at each length. All 40 queries match one term in one block.
Full-reference means for lengths 8/12/16/24/32 are **158.125 / 1.875 / 1 / 1 / 1**.
All source-family candidate sets are nested as the literal length grows.

| Budget / cap | 8 bytes | 12 bytes | 16 bytes | 24 bytes | 32 bytes |
|---|---:|---:|---:|---:|---:|
| Any budget / 10% (byte-only) | 16424.625 | 16370.125 | 16369.125 | 16368.125 | 16367.625 |
| 200000 / 20–80% | 14762.500 | 14708.000 | 14707.000 | 14706.000 | 14705.500 |
| 300000–500000 / 20% | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.250 |
| 300000–500000 / 30–80% | 13080.375 | 13046.375 | 12853.500 | 12852.500 | 12653.375 |
| 1000000 / 20% | 7524.375 | 7259.500 | 7258.500 | 5348.875 | 3490.000 |
| 1000000 / 30–80% | 9349.250 | 9129.750 | 9080.125 | 3678.500 | 3470.875 |

At 1 MB, 20% is better for 8/12/16-byte means, and 30–80% is better for 24/32-byte
means. The aggregate across all 40 ID queries still favors 20%. The 10% length
benefit comes entirely from the unchanged byte/prefix fallback. No universal
length recommendation follows from these ID-like diagnostic families.

## Validation and audit accounting

- Same 89 queries: 41 predeclared natural words (14 present, 27 absent), 40
  source-derived queries (eight frozen families × five lengths), eight controls.
- Same 16794 physical blocks and 7671280 distinct terms. Same exact document-DF
  ranking, ascending u24 ties, whole frames and skip-nonfitting behavior.
- All **40 logical configurations** independently selected, serialized,
  reopened and fully frame-compared. Rust decoded and compared every retained
  posting in each file: **27779 row instances across 40 files**.
- **15 distinct partial indexes/computations**, plus two reference computations.
  Higher-cap aliases were permitted only after full-byte/hash, full selected-key
  and decoded-query-row equality.
- **59786640 logical partial query/block decisions** persisted. Per computation
  pass: **22419990 distinct partial query/block decisions**, or **25409322**
  including byte/full references. Two passes: initial computation and persisted
  reload/recomputation. Logical alias entries are separately checked against
  recomputed masks. These numbers distinguish actual work from logical coverage.
- Twelve-byte little-endian masks preserve all 89 bits. The runner reloads all
  masks, reopens distinct indexes for fresh row lookups, recomputes every distinct
  decision, recounts results, and verifies every alias mask.
- On every query/block: `truth ⊆ full ⊆ partial ⊆ bytes`. **Zero false negatives**.
  Nested source-family candidate containment verified per block.
- Frozen truth was reused after SHA-256/provenance validation. All physical
  boundaries, sampled-source identities/ordinals/digests, remainder summaries,
  full prefixes and decoded byte sidecars were rechecked. No full-key rescan.
- **362 source files** checked before/after and unchanged. The manifests include
  source dictionaries, inherited provenance, old helpers, prior reports,
  completed truth/cohort/family files and earlier deployed indexes.

## Tests, failures and reproduction

Successful test suites: **11 new threshold tests + 18 existing longer-query tests
+ 34 prior trigram-query tests = 63 tests**, zero failures and zero skips.
Tests cover requested caps/budgets, inclusive integer boundaries, document-rank
and tie ordering, skip-nonfitting behavior, unknown/prefix-crossing semantics,
89-bit high-bit corruption, frozen cohort/truth/family identity, all serialized
costs/hashes/frame equality and Rust verification, cap equivalence, cap20
block-for-block regression, persisted totals/groups/lengths, nested containment,
source immutability and report reproduction.

An independent `--validate` rebuild repeated all 40 constructions and evaluations
successfully. Its report JSON and Markdown were identical. See `completion.json`
and `logs/validate.log`.

**Preserved interruption:** the first foreground launch exceeded the tool's
120-second timeout after writing all indexes and masks, during reload/audit.
This was an execution timeout, with no assertion failure. All first-attempt files
are preserved in `failed_attempt_1/`, including `failure.json`. The clean retry
ran in the background. Artifact enumeration excludes historical attempt
subdirectories and launcher logs so the independent fresh rerun is reproducible.
There were no test failures requiring a semantic correction.

Reproduction from repository root:

```bash
export PYTHONDONTWRITEBYTECODE=1
ROOT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
# Initial construction requires an absent destination and several minutes.
python3 dictionary_analysis/long_queries/thresholds.py "$ROOT"
# On this completed output:
python3 dictionary_analysis/long_queries/thresholds.py "$ROOT" --validate
python3 -m unittest dictionary_analysis.long_queries.test_thresholds -v
python3 -m unittest dictionary_analysis.long_queries.test_long_queries -v
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_*.py' -v
```

The ordinary CLI refuses to overwrite an existing destination. The validation
command builds in a temporary child and leaves prior artifacts intact. Full
commands, semantics and file descriptions are in the owned `thresholds.md`.
No production latency, Chinese-data, new UUID assignment, new query-cohort or
quadgram claims are made.
