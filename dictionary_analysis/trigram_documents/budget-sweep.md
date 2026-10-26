# Budget sweep: document-ranked trigrams

This experiment varies total metadata budget at a fixed **80% individual-row block-skip cap**. It reuses `budget.py` and the original document-frequency extraction. It changes no production code, SSTables, source dictionaries, or existing experiment files.

## Reproduce

Run from the repository root with the existing extraction and posting artifacts available:

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/trigram_documents/budget_sweep.py "$OUT" \
  > "$OUT/trigram_document_budgets/budget-sweep.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/trigram_documents \
  -p 'test_budget_sweep.py' -v \
  > "$OUT/trigram_document_budgets/budget-sweep-tests.log" 2>&1
```

The existing Rust reader is `dictionary_analysis/timestamp_injection/target/release/trigram-postings`. Override with `--binary PATH` if needed. The helper uses the existing `zstd` executable. Python assertions are validation checks, so run without `-O`.

Budgets in bytes: **5,341, 7,500, 10,000, 15,000, 20,000, 25,000, 50,000, 100,000, 150,000, 195,640, 200,000, 300,000, 500,000, 1,000,000**.

The wrapper calls `budget.run` only for new directories. It reopens and audits every existing directory without rewriting its files. The 195,640 B directory must already exist. A before/after SHA-256 manifest preserves all its files, including `report.json`. An incomplete output directory causes a failure rather than an overwrite. Repeat runs reuse and freshly validate the generated artifacts.

Outputs under `$OUT/trigram_document_budgets/`:

- `budget-sweep-report.md` and `budget-sweep-report.json`: costs, all seven queries in both modes, references, first tested thresholds, preservation manifest, hashes.
- `<budget>_skip80/`: actual `trigrams.bin`, `bytes.bin`, `prefixes.bin`, `decisions.bin`, and the original helper's `report.json`.
- `<budget>_skip80.log`: helper output for new budgets.
- `budget-sweep.log` and `budget-sweep-tests.log`: reproduction logs.

## Accounting and selection

Each budget includes the selected trigram file, **5,275 B** byte fallback and **48 B** full-prefix extensions, charged once. IU01 uses **18 + 7 × retained rows + retained frame bytes**. An empty selection therefore costs **5,341 B** including fallback. Percentages are added metadata divided by the original **195,640 B** index. Diagnostic reports, decisions, extraction data and runtime memory are separate.

Construction ranks exact distinct-document union DF over full-term trigrams descending, with ascending u24 ties. A row is eligible when `blockDF × 100 <= 16,794 × 20`. Whole rows that fit are kept, nonfits are skipped, and scanning continues. This procedure accepts no query information. Different budgets can retain different sets and row counts can decrease as a larger high-ranked row starts fitting. The selected sets need not be nested.

IU01 omission means unknown, including a trigram absent from the complete dictionary. Retained postings are exact block memberships of full-prefix-excluded term remainders. Queries use the shared prefix-aware `permits` callback for prefix-contained, remainder-contained and crossing matches. Both measured modes are charged the mandatory-fallback total. `trigrams` disables byte filtering as an ablation, while `trigrams_bytes` uses the fallback.

## Checks

For each budget, the wrapper independently reconstructs generic selection, reads every retained frame, and invokes Rust `verify-subset` to decode and compare every retained posting. It checks exact sizes, membership totals, sidecar identity, prefix reconstruction, omitted observed/unobserved lookup semantics, and input hashes. Evaluation is fresh for each actual file. All 7 original queries × 2 modes are compared with the full-index per-block truth and decisions. The recomputed decisions must equal `decisions.bin` byte-for-byte.

The tests independently walk document-DF order for accounting and retained membership, verify every frame and hash, parse persisted decisions, check no false negatives and full-index candidate inclusion, and reconstruct reported counts. They also check empty-file overhead, exact fits, nonfit skipping, tie order, the cap, threshold summaries, preservation, and byte identity with the prior 1 MB selection.

## Measured outcome

- Through the tested **15,000 B** budget, player has **16,331** combined candidates, the byte-only result.
- At **20,000 B** budget, actual **19,998 B** (**10.2218%** of the original index), **22 rows** include `pla`, `lay`, `aye`. Player has **23** candidates and **99.8630%** skipping in both modes.
- At **25,000 B**, actual **25,000 B** (**12.7786%**), **62 rows** also include `yer`. Player has **14** candidates and **99.9166%** skipping, matching the full-trigram reference. It has 13 true blocks and one false-positive block.
- All larger tested budgets give the same player result. The preserved **195,640 B** run retains **276 rows**. The **1,000,000 B** budget serializes to **999,997 B**, retains **1,223 rows**, and matches the earlier 1 MB cap-80 selection byte-for-byte.
- `tra` and `raa` remain omitted at all tested budgets. Traa has **16,341** combined candidates (**2.6974%** skipping) versus **141** with full trigrams. Its oracle has **38** true blocks / **42** exact terms.
- All seven queries in both modes have **zero false negatives**. `*X` is absent and reported separately from the six present queries.

**20,000 B is the first tested budget with ≥90% player skipping.** Gram entries are first tested retention: `pla`, `lay`, `aye` at 20,000 B and `yer` at 25,000 B. These statements concern the tested grid. They do not establish the minimum possible budget. Results describe this cohort and serialized metadata, without production-latency or representative-workload claims.
