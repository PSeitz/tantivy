# Retest the frozen longer-query cohort at maximum presence caps

This experiment uses the same 89 queries and source-family mapping as the completed
`long_queries` run. It changes only the maximum remainder-block-presence cap:
**10%, 20%, 30%, 40%, 50%, 60%, 70%, 80%**, each at **200000, 300000,
400000, 500000, 1000000 decimal bytes** of additional metadata.

## Policy and accounting

- A row is eligible exactly when `blockDF * 100 <= cap * 16794`. The imported
  selection helper takes minimum absence, so the argument is `100 - cap`.
- Rank the complete eligible vocabulary by descending exact distinct-document DF,
  then ascending u24 key. Retain whole posting frames that fit. Skip nonfitting
  rows and continue. Selection receives no queries or truth.
- Every configuration is serialized to its own `IU01` file. Its omitted rows
  mean unknown. Full-LCP prefix-contained and prefix-crossing evaluation uses
  the unchanged safe callback and byte fallback.
- Actual additional metadata is `5323 + partial_file_bytes`.
  The partial file costs `18 + 7 * retained_rows + sum(frame_bytes)`.
  The byte/prefix sidecars are shared on disk and charged once per deployable
  index. Percentages divide actual additional metadata by the original
  **195640 B** index. The complete trigram reference costs **38073845 B**.
  Audit files, reports and evaluation memory are separate from deployable cost.

All 40 selections are explicit. All 40 files are reopened, their retained frames
compared to the full index, and their complete retained postings decoded by Rust
`verify-subset`. Query rows are decoded from every file. Duplicate computation is
allowed only after full byte equality and hash equality, selected-key equality,
and decoded-query-row equality. Every logical configuration still appears in the
persisted 89-bit (12-byte little-endian) masks and every report matrix.

## Frozen truth and immutability

The runner authenticates the completed earlier run's source manifests, artifact
hashes, completion manifest, cohort/sample freeze hashes, and selection inputs
before reusing its exact `truth.tsv`. It rechecks every physical block boundary,
first/last key, prefix and byte mask. It decodes both sidecars again and seeks the
eight sampled source blocks to verify term identity, ordinal, sampling digest
and literal containment. It preserves the natural/source/control grouping and
all five nested lengths per family.

The verified truth covers 7,671,280 full dictionary keys. Another complete key scan
would reproduce unchanged truth, so this experiment reuses it. Source hashes are
verified again after evaluation. All new binaries and diagnostics are confined
to `message_analysis/long_query_thresholds/`. Earlier code and artifacts remain
unchanged.

The 20% files must be byte-identical to the earlier deployed budget files. Their
89-query decisions must match the earlier run block-for-block and in every
reported count. Caps 30–80 must have equal complete eligible and selected lists,
serialized bytes, and masks at each budget. These are assertions verified from
the data, rather than assumed policy equivalences.

The runner reloads persisted masks and reopens distinct serialized indexes for a
second query-row lookup and independent mask computation. Every query/block
must satisfy `truth ⊆ full ⊆ partial ⊆ bytes`, with zero false negatives.
Aliases are checked against those freshly recomputed masks. Nested source-family
containment is checked per block. Reports distinguish logical configurations,
distinct index computations, and the two reference computations.

## Reproduction

Run from the repository root using Python 3.10+ without `-O`:

```bash
export PYTHONDONTWRITEBYTECODE=1
ROOT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/long_queries/thresholds.py "$ROOT"
python3 dictionary_analysis/long_queries/thresholds.py "$ROOT" --validate
python3 -m unittest dictionary_analysis.long_queries.test_thresholds -v
python3 -m unittest dictionary_analysis.long_queries.test_long_queries -v
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_*.py' -v
```

The ordinary run refuses an existing destination, including an incomplete one.
Use a fresh input/output copy to reproduce initial construction after completion.
`--validate` works on the completed destination: it checks all persisted artifact
hashes, then runs a separate full 40-file construction/evaluation in a temporary
child directory and requires identical report JSON and Markdown. It reuses the
same authenticated truth and removes its temporary files on success.

The full experiment takes several minutes. Run it with a sufficiently long
command timeout, or launch it in the background. The runner writes progress to
`long_query_thresholds/run.log`.

## Outputs

Inside `message_analysis/long_query_thresholds/`:

- `indexes/budget_{budget}_presence{cap}.bin`: 40 independently serialized files.
- `indexes/{bytes,prefixes}.bin`: shared immutable sidecar copies.
- `indexes.json`: exact costs, eligibility, hashes, decoded-row verification and
  computation aliases for all 40 logical configurations.
- `cohort.json`, `source_samples.json`: byte-identical frozen manifests.
- `decisions.jsonl`: truth, exact matching-term counts, and masks for all 40
  configurations plus byte/full references on every block.
- `report.json`: all 89 × 40 query results, separate natural-present,
  natural-absent, source-derived and control groups, natural/source length groups,
  family counts, adjacent-cap and adjacent-budget candidate membership changes,
  selection displacement and per-query-gram omission reasons.
- `report.md`: all eight caps in cost, group, length and every-query matrices.
- `source_hashes_before.json`, `source_hashes_after.json`: unchanged source audit.
- `parent_report.md`: concise findings, examples with actual gram DFs/costs,
  run/test history, failures and parent-ready reproduction details.
- `completion.json` and `logs/`: final validation/test outcomes and code hashes.

Every group mean weights queries equally. True and candidate counts describe
physical dictionary blocks and matching dictionary terms. The source-derived
queries are ID-like diagnostics. These results support workload-specific
tradeoffs rather than a universal best cap, query length or production-latency
claim. No new queries, quadgrams or UUID dictionaries are built.
