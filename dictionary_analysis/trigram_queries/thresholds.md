# Frozen-query block-presence cap comparison

Compare maximum block presence **20%, 21%, and no cutoff**, each at total decimal-byte budgets **200000, 300000, 400000, 500000, 1000000**. Selection retains the exact existing full-term document-DF ranking and skip-nonfitting whole-row policy. Queries never enter selection.

The builder's `cutoff` is minimum block skip. The corresponding values are **80, 79, 0**. Eligibility uses the inclusive integer comparison `100 * block_df <= cap * 16794`.

## Reproduce

Run from the repository root with the existing source artifacts and release Rust binary:

```sh
ROOT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
OUT="$ROOT/trigram_query_thresholds"
mkdir -p "$OUT"
python3 dictionary_analysis/trigram_queries/thresholds.py "$ROOT" \
  > "$OUT/run.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_*.py' -v \
  > "$OUT/tests.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/trigram_documents -p 'test_*.py' -v \
  > "$OUT/document-tests.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/trigram_selection -p 'test_*.py' -v \
  > "$OUT/selection-tests.log" 2>&1
```

The existing `trigram_queries/README.md` and `budgets.md` describe source extraction, the frozen cohort, independent truth generation and previous budget checks. Those files remain unchanged because their hashes belong to previous experiments. This wrapper reuses the verified 42-query truth and never rescans the full key stream.

Only missing index directories are built, through `trigram_documents.budget.run`, at `trigram_document_budgets/{budget}_skip{80|79|0}`. Existing directories must contain complete, verifiable reports and files. The runner refuses an incomplete existing directory rather than overwriting it. It reuses the five existing 20% indexes and creates the ten missing 21%/uncapped indexes. Rerunning verifies and reuses all fifteen. Per-build stdout goes to `build_{budget}_skip{cutoff}.log`. Overall errors and tracebacks go to `run.log`. No historical source, index, query artifact or report is rewritten.

## Output

`trigram_query_thresholds/report.md` contains:

- The primary 15-cell matrix for 34 present literals of at least three bytes: mean block skip, number reaching 99%, actual serialized metadata bytes, percent of the original 195640 B, retained rows and eligible rows.
- All improving and worsening queries at the same budget, with absolute candidate counts.
- Detailed examples for `123`, `000`, `client`, `error`, `user`, `server`, `service`, `trace`, `traa`. Query gram retention distinguishes cap exclusion from budget/document-DF omission.
- Changed query grams and newly eligible popular-row examples, including full-term document DF, remainder block DF and incremental serialized cost.
- All 42 results with exact matching terms, true blocks, false positives and separated presence/length summaries.
- A cohort-only recommendation and limitations. These measurements establish pruning, without a production latency claim.

`report.json` additionally records every added/removed row for each same-budget policy comparison, per-query candidate-set additions/removals, per-gram omission reasons, source/truth hashes and Rust subset-verification results.

`{budget}/decisions.jsonl` persists six-byte masks for all 42 queries, separately for bytes, full trigrams and the three actual cap policies. The runner reloads and recomputes every policy decision from disk-loaded postings. It checks exact recounts and `truth ⊆ full ⊆ partial ⊆ bytes` for every block/query. Full-prefix-contained and crossing cases use the existing prefix callback. Omitted partial rows stay unknown.

The fixed sidecars total exactly **5323 B** and are charged once per index. All actual serialized rows receive a fresh Rust `verify-subset` check against the complete posting file. Selected keys are recomputed from the generic ranking. The five existing 20% results must match the prior budget report exactly.

`source_hashes_before.json` and `source_hashes_after.json` snapshot inherited sources, truth, query reports, previous budget reports/indexes and this experiment's source files. They must match, and every file is rehashed at completion. `created_index_hashes.json` covers index files created by this experiment and is retained and reverified on reruns. Previously created indexes also enter the inherited snapshot on reruns.

## Higher-cap equivalence

The runner independently finds the maximum block DF over all 47796 rows. It compares complete eligible-key lists for **22%, 30%, 50%, 75%, 100%** against uncapped. For each of the five budgets it separately reruns selection for each counterfactual cap and checks complete selected-key and selection-accounting equality. Key-list SHA-256 fingerprints are persisted, and uncapped keys are checked against actual serialized files. Identical selected keys reuse identical source frames and produce identical serialized indexes. Duplicate files are unnecessary.

## Tests

`test_thresholds.py` covers strict inclusion at exact cap boundaries, integer boundaries at 16794 blocks, equivalence of higher caps, omission reasons, serialization/readback, fixed/actual accounting, high query bits, corrupted-mask audits, prefix crossings, unknown semantics and separated summaries. Persisted tests verify all hashes, real selections and cap equivalence, all wide masks and candidate-set transitions, aggregates, and equality with historical 20% and reference results.

The command above also runs all historical query and budget tests. Document and selection tests validate the unchanged selection/serialization helpers. Runtime and helper-memory costs are experiment resources rather than deployable metadata.
