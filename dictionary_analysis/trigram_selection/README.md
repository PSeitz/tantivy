# Frequency-first trigram selection

This experiment keeps exact trigram posting rows in descending block-frequency order under a **1,000,000-byte total added-metadata budget**. A minimum individual absence fraction excludes overly common rows before ranking. Construction uses every observed trigram, its frequency and serialized size. It receives no queries.

## Result

All **44 actual partial indexes** completed: four dictionaries × eleven cutoffs (0%, 50%, 70%, 80%, 85%, 90%, 92%, 94%, 95%, 98%, 99%). Every total includes the selected trigram file, best existing inverted byte fallback and prefix-extension sidecar once. The allowance equals **+511.14% of the original 195,640-byte index**.

**No selection retains any of the six query grams. Selected trigrams add zero pruning beyond bytes for all seven queries at every cutoff.** The 80% cap changes stored rows on three dictionaries. Its query decisions equal the uncapped decisions on all four. This workload supports no universal cutoff recommendation.

At 80%:

| Dictionary | Rows retained | Total B | `*player*` candidates / skip | `*traa*` candidates / skip |
|---|---:|---:|---:|---:|
| Baseline | 733 | 999,999 | 16,331 / 2.76% | 16,341 / 2.70% |
| UUIDv4 +10M | 762 | 999,999 | 23,913 / 76.29% | 35,259 / 65.05% |
| UUIDv7 +10M | 1,208 | 999,998 | 16,314 / 80.66% | 16,334 / 80.64% |
| Alphanumeric36 +10M | 162 | 1,000,000 | 103,148 / 0.42% | 103,152 / 0.42% |

These are byte-fallback results. Trigrams-only retains every block. `player` has 13 true blocks / 21 terms in each dictionary. Full exact trigrams + bytes retain only 14 / 14 / 13 / 19 blocks, with much larger metadata costs recorded in the report.

At 80%, baseline excludes 598 of 47,796 observed rows. Each UUID dictionary excludes 4,864 of 48,565. The alphanumeric control excludes none of 47,782. Stricter cutoffs eventually exhaust eligibility on baseline, UUIDv7 and control, leaving most of the allowance unused. See the complete sweep for those costs.

## Selection and persisted semantics

- Eligibility is inclusive: `df * 100 <= block_count * (100 - cutoff_percent)`. Integer arithmetic preserves the boundary.
- Rank by descending df, then ascending u24 gram value. Keep each row that fits, skip nonfitting rows and continue.
- Charge `18 + 7 * row_count + sum(frame_bytes)` for the selected trigram file. Fixed fallback costs are 5,323 / 231,360 / 15,500 / 8,801 B.
- `IU01` persists **omitted row = unknown/all blocks**. `IP01` retains its original complete-vocabulary missing-row=absent semantics. The new reader returns `unknown:true, ids:null` for omitted `IU01` keys. It returns exact IDs with `unknown:false` for retained keys. Even truly unobserved grams are unknown in `IU01`.
- `IU01` otherwise shares the complete index's directory: 14-byte header, sorted three-byte little-endian gram values, `(row_count+1)` u32 offsets, independent exact posting frames. Each frame includes its original tag, raw-length field and compressed or plain payload. No retained block IDs are removed.
- Short predicates and short prefix-crossing continuations use bytes in the deployed `trigrams_bytes` mode. The `trigrams` ablation uses all-byte permission. Both selections reserve and charge the mandatory byte sidecar.

`prefix_summaries.permits` evaluates prefix-contained, remainder-contained and each viable crossing continuation. Only the continuation reaches the gram callback. One unknown gram imposes no condition. Other retained grams can still reject that continuation.

## Files and validation

New helper: `dictionary_analysis/trigram_selection/sweep.py`.

Minimal reusable Rust extension: `dictionary_analysis/trigram_postings/src/main.rs` recognizes `IU01` in lookup/frequency commands, distinguishes unknown from absent, and adds `verify-subset FULL PARTIAL` to independently decode and compare every retained posting. The complete-index constructor and its commands retain existing behavior. Production code and parent documentation are unchanged.

Output: `KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_selection/`.

- `report.md`: all 44 costs, eligibility, df ranges, post-evaluation retained-query-gram diagnostics, all seven query tables, exact/byte references and interpretation.
- `report.json`: structured results, hashes and validation.
- `originals_manifest.json`: 28 read-only inputs checked by SHA-256 before and after the run.
- Per dictionary: eleven `skip_XX.bin` partial indexes, eleven compact `skip_XX_decisions.bin` files, fresh all-row `frequencies.jsonl`, one `bytes.bin` and one `prefixes.bin`.
- Build, regression and run logs. `initial_import_failure.log` records the resolved startup failure caused by the initial helper filename shadowing Python's `select` module. The helper was renamed before index construction.

Every partial directory and frame is read back. Every retained frame equals the original bytes. Rust independently decodes both indexes and verifies every retained membership. Reader probes distinguish unknown, known empty, retained true and truly unobserved grams. Source prefix reconstruction and byte masks match prior validated artifacts. All per-block truth and byte decisions agree with the prior complete-index evaluation. All seven predicates have **zero false negatives**.

Each partial's query postings are independently read back. Identical retained-query-gram subsets reuse evaluation only after their postings equal the full exact rows. All eleven subsets happen to be empty per dictionary, so only one per-block evaluation is needed per dictionary. A separate artifact test independently replays ranking/budget decisions from disk and checks all 44 per-block decision files against the byte/truth references.

Regression validation: **76 tests passed** (48 root Python, 5 existing trigram Python, 4 pair Python, 9 selection Python including the artifact audit, 6 trigram Rust, 4 pair Rust).

Unit tests cover inclusive cutoff boundaries, header/directory budget accounting, skip-nonfitting continuation, frequency/tie ordering, empty selection, exact retained postings, unknown versus known absence, partial constraints with unknown rows, prefix crossings leaving 0/1/2 bytes, short-query byte fallback and exhaustive small-dictionary safety.

## Reproduce

Run from the repository root after the full trigram and byte experiments have produced their validated artifacts:

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
export CARGO_TARGET_DIR=dictionary_analysis/timestamp_injection/target
cargo test --manifest-path dictionary_analysis/trigram_postings/Cargo.toml
cargo build --release --manifest-path dictionary_analysis/trigram_postings/Cargo.toml
/usr/bin/time -l python3 dictionary_analysis/trigram_selection/sweep.py "$OUT" \
  > "$OUT/trigram_selection/run.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/trigram_selection -p 'test_*.py' -v
```

The artifact test skips when output does not exist. Other tests use temporary indexes. The helper creates no SSTables and changes no existing input artifacts.

## Limits

The cap describes individual absence in prefix-removed block remainders. It does not guarantee full-query skip. Frequency is per block, not per document. Highest-frequency-first favors weaker individual rejection among eligible rows, and random-background rows can occupy the budget before useful workload grams. Subtracting a shared minimum frequency preserves ranking.

These seven predicates provide a narrow workload. Recommending a cutoff from them would be workload tuning. Costs are actual serialized sidecars. Diagnostic files, build memory and experiment timing are excluded from deployable metadata. This experiment makes no production latency, I/O or globally optimal compression claim.
