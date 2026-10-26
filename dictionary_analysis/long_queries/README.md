# Longer queries against unchanged partial trigram indexes

This experiment evaluates longer substring queries. The folder name describes that scope. It imports existing scanner, lookup, accounting, prefix-safety and decision-audit helpers. It changes no prior helper, production source, selection, index or prior artifact.

Output directory:

`KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_queries/`

## Cohort frozen before evaluation

`run.py` declares the exact 41 natural words from the handoff. None were replaced after observing presence or performance. Eight controls are `error`, `user`, `service`, `client`, `server`, `failed`, `trace`, and `traa`.

The source sample reads only `terms.bin` and physical boundaries. Eligible terms are unique ASCII alphanumeric byte strings with length at least 32. Select the eight smallest `(SHA256(seed + term), raw term)` pairs. The seed bytes are `b'tantivy-long-queries-v1\x00'`. For each selected term, take the centered 32-byte window starting at `(len(term)-32)//2`. Its first 8, 12, 16, 24 and 32 bytes form five nested queries.

There are 151,143 eligible terms. All eight selected terms are 32-byte lowercase hex strings. This classification was recorded after selection and applied no additional filter. The 40 derived literals are diagnostic ID-like queries, rather than a representative traffic sample. They each match one dictionary term in one block.

`source_samples.json` records terms, SHA-256, zero-based ordinals, physical block IDs, window offsets, classification and family mappings. `cohort.json` deduplicates evaluation literals while retaining mappings. Both files and their hashes are persisted before opening index selections or truth counts. The final cohort contains 89 distinct queries: 41 natural, 40 source-derived and eight controls. Only the eight controls overlap the prior 42-query cohort.

All queries use case-sensitive raw-byte substring matching. The outer stars in serialized query names denote substring mode. The literals themselves contain only ASCII alphanumeric bytes. Terms may contain arbitrary bytes, including NUL and newline.

## Results

Each cell below is mean candidate-block skip percentage. Each query has equal weight. The block count is 16,794. Present and absent natural queries remain separate.

| Mode | Actual additional bytes | Natural present (14) | Natural absent (27) | Source present (40) | Controls (8) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Byte+prefix | 5,323 | 2.8412% | 2.8712% | 2.4656% | 2.7324% |
| 200 KB budget | 199,999 | 85.5892% | 95.1742% | 12.3627% | 25.5076% |
| 300 KB budget | 300,000 | 92.0945% | 99.4407% | 22.5179% | 73.9989% |
| 400 KB budget | 399,990 | 99.4781% | 99.7073% | 22.5179% | 86.0315% |
| 500 KB budget | 499,997 | 99.9690% | 99.7559% | 22.5179% | 86.1141% |
| 1 MB budget | 999,997 | 99.9838% | 99.7561% | 63.2235% | 86.1141% |
| Full trigrams | 38,073,845 | 99.9911% | 100.0000% | 99.8059% | 99.7402% |

All budgets are additional decimal bytes. Each total charges the 5,323-byte byte/prefix sidecars once, plus the actual serialized partial index. The original metadata reference is 195,640 bytes. The unchanged partial indexes retain 304 / 476 / 646 / 828 / 1,223 rows. They use the original exact full-term document-DF ranking and maximum 20% remainder-block presence.

Useful examples:

- Present `authentication`: 4 / 4 / 1 / 1 / 1 candidates across the five budgets, with one true block.
- Present `unavailable`: 16,317 / 16,317 / 1,037 / 12 / 1, with one true block. Its first retained query gram is `ble` at 400 KB.
- Present `heartbeat`: 16,332 / 1,048 / 3 / 3 / 3, with three true blocks. Its seven grams are all retained starting at 400 KB.
- Absent `keepalive`: 16,322 / 947 / 947 / 947 / 947. Only `ive` is retained from 300 KB onward. Full trigrams reject every block.
- Absent `healthcheck`: 1,078 / 1,078 / 108 / 108 / 108. Full trigrams reject every block.
- Present `validation`: seven candidates at every partial budget, three full-reference candidates and two true blocks. The full reference itself retains a false positive.

### Length benefit is conditional

Across eight nested source families, mean candidates at lengths 8 / 12 / 16 / 24 / 32 are:

| Mode | 8 | 12 | 16 | 24 | 32 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Byte+prefix | 16,424.625 | 16,370.125 | 16,369.125 | 16,368.125 | 16,367.625 |
| 200 KB | 14,762.5 | 14,708 | 14,707 | 14,706 | 14,705.5 |
| 300–500 KB | 13,080.375 | 13,046.375 | 13,045.375 | 13,044.375 | 12,845.25 |
| 1 MB | 7,524.375 | 7,259.5 | 7,258.5 | 5,348.875 | 3,490 |
| Full trigrams | 158.125 | 1.875 | 1 | 1 | 1 |

All eight families improve between 8 and 32 bytes in every mode. Some improvements are only 1–6 blocks and also occur in byte fallback. At 200 KB no family gains a retained gram with length, and no family's candidate reduction exceeds its byte-only reduction. At 300–500 KB one family gains retained grams. At 1 MB six families gain retained grams. Seven families have an absolute reduction greater than byte fallback, which also reflects prefix-continuation behavior.

Family 0 (`a5b557dfa94b1f70209f6a70fbb39a5c`) gains retained `702` at length 24 in the 1 MB index. Candidates fall from 16,368 at length 16 to 3,081 at length 24. Full trigrams reach one candidate at length 16. Family 6 (`0b3da49b0ecf04c0127fae58532caf18`) retains no query grams even at 1 MB and 32 bytes, leaving 16,367 candidates for one true block.

These source-derived examples support a conditional length benefit. They do not establish representative query traffic, universal defaults or production latency.

## Artifacts and audit

- `report.md`, `report.json`: all 89 queries, seven modes, exact matching-term and true-block counts, candidate/FP/FN/skip counts, retained grams, natural/source presence and exact-length buckets, family outcomes, costs and validation.
- `interpretation.md`: parent-friendly shortlist, one nested family, detailed retained/missing-feature explanations and limitations.
- `interpretation.json`: per-family length reductions relative to byte fallback, added retained grams and aggregate remainder-length histograms.
- `truth.tsv`: fresh independent exact scan of all 7,671,280 keys using the existing dynamic-query C++ scanner. It checks strict key ordering and EOF. Every physical term count, ordinal, stream offset, first/last key, full LCP and remainder byte mask is checked against original metadata.
- `remainders.jsonl`: actual full-LCP-excluded remainder-length histograms per physical block. There are 75,510,607 raw remainder bytes, mean 9.8433 bytes, minimum zero, maximum 91 and one empty remainder.
- `decisions.jsonl`: 89-bit truth and seven candidate masks, serialized as 12-byte little-endian hex. Matching-term counts are also stored per block. Every persisted mask is reloaded, recomputed from serialized-index lookups, checked and recounted.
- `source_hashes_before.json`, `source_hashes_after.json`: identical manifests for 118 inputs including original data, prior helpers/artifacts, serialized indexes and the current evaluator. Prior selection-input hashes are authenticated as well.
- `freeze_hashes.json`: frozen cohort and sample-manifest fingerprints.
- `logs/`: final tests, earlier tests, corrected run, independent rerun and preserved failure logs.
- `failed_attempt_1/`: superseded first-run artifacts and explicit failure explanation.

Rust `verify-subset` decoded all 3,477 retained rows across the five partial indexes. Python also compares every retained serialized frame byte-for-byte to the full index. Sidecar files have identical hashes, their decoded byte/prefix values equal the fresh scan, and exact frame/directory/sidecar accounting is checked.

Every query/block satisfies `truth ⊆ full ⊆ partial ⊆ byte`. There are zero false negatives across 10,462,662 query/mode/block decisions per evaluation pass. Nested candidate sets are checked block-for-block. All eight controls reproduce their prior exact truth, totals and per-block same-index decisions at all five budgets and both references.

The existing prefix-aware `permits` helper handles prefix-contained, prefix-crossing and remainder-contained matches. Missing partial rows mean unknown. Missing full-index rows mean absent. Prefix crossings check continuation grams, so all whole-query grams are not required. Full-reference false positives measure remaining necessary-condition ambiguity. Partial/full gaps show missing-feature coverage.

Reports, truth, helper binaries, widened byte directories and decision files are audit artifacts and are excluded from deployable metadata costs.

## Reproduction

Requires existing source data/index artifacts, Python 3, clang++, zstd and the existing Rust lookup binary. From the repository root, a fresh experiment is:

```sh
export PYTHONDONTWRITEBYTECODE=1
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/long_queries/run.py "$OUT"
python3 dictionary_analysis/long_queries/interpret.py "$OUT/long_queries"
python3 -m unittest discover -s dictionary_analysis/long_queries -p 'test_*.py' -v
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_*.py' -v
```

A normal run refuses an existing `long_queries` destination. To reproduce this completed experiment without overwriting anything:

```sh
python3 dictionary_analysis/long_queries/run.py "$OUT" --validate
python3 dictionary_analysis/long_queries/interpret.py "$OUT/long_queries"
```

`--validate` authenticates existing sources/artifacts, then runs the complete sample selection, fresh truth scan, all index checks and full persist/reload/recompute evaluation in a temporary subdirectory. It requires the entire report and deterministic artifact fingerprints to match. The compiled scanner binary is excluded from artifact equality because linker output may differ. Successful validation removes the temporary directory. Interpretation output may be written only if absent or byte-identical.

Executed validation: **18 new tests + 34 existing trigram-query tests passed**, with no final skips. A separate complete `--validate` rerun reproduced the frozen sample, truth, decisions, report and source hashes exactly.

### Preserved failure

The first run's sample-provenance loop shadowed the total-term counter. It reported 3,875,114 (the last sampled ordinal) instead of 7,671,280. The complete source scan and physical-boundary checks had succeeded. A new report-total regression assertion reproduced the failure. The counter was fixed, its equality to physical term totals was asserted, and the experiment was rerun.

The first run remains under `failed_attempt_1/`. Its cohort, exact truth, decisions and remainder summaries are byte-identical to the corrected run. The sample manifest's total and derived report total were corrected. The final independent rerun and all 52 tests passed. `logs/tests-counter-regression.log` records the intentional failure before replacement.
