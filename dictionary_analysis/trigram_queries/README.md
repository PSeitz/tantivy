# Broader queries against unchanged document-ranked trigram indexes

This experiment changes the **search values only**. It reads the existing 195,640-byte (276 rows) and 999,997-byte (1,223 rows) document-ranked cap-80 indexes. It performs no selection, reranking, or index construction. Both costs include 5,275 bytes of byte postings and the 48-byte full-prefix extension sidecar. References are byte+prefix alone (5,323 bytes) and full trigrams+byte+prefix (38,073,845 bytes).

`cohort.json` freezes the original seven queries plus the 35 supplied substrings before looking at their retention or pruning. All matching uses case-sensitive raw bytes. `*X` and `*x` are suffixes. Every other query is a substring, including across newlines. The 42 values form a diagnostic cohort, not a production-workload sample.

## Results

Outputs are under `KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_queries/`.

- `report.md`: every query's exact matching term count, true blocks, candidate blocks, skip percentage, false-positive blocks and retained grams. All four references appear side by side.
- `report.json`: those measurements plus full-term distinct-document DF and remainder block DF for each query gram, source hashes, validations, and present/absent × short/long aggregates.
- `truth.tsv`: fresh per-block exact matching term counts, key ordinals, term-stream byte offsets, first/last key, actual common prefix and remainder byte mask.
- `decisions.jsonl`: every block's 42 term counts, truth mask and four candidate masks. Masks are **six-byte little-endian hex**, bit `j` means query `j` in `cohort.json`. This format supports all 42 queries independently of the old seven-query u8 mask.
- `source_hashes_before.json`, `source_hashes_after.json`: identical size/SHA-256 manifests for all consumed source data, both existing partial indexes, references, code, and lookup binary. Earlier experiment input hashes are also checked.

For the **34 present queries with at least three bytes**:

| Mode | Mean candidate blocks out of 16,794 | Mean skip | Queries with ≥99% skip |
| --- | ---: | ---: | ---: |
| Byte+prefix | 16,360.03 | 2.5841% | 0 |
| 195,640 bytes | 5,101.59 | 69.6226% | 19 / 34 |
| 999,997 bytes | 2,216.03 | 86.8046% | 26 / 34 |
| Full trigrams | 377.29 | 97.7534% | 30 / 34 |

The 195,640-byte result is excellent for many values, including `player` (14 candidates), `debug` (12), `database` (2), `status` (8) and `exception` (10). It is highly query-dependent. `error`, `user`, `server`, `service`, `team` and `trace` each retain about 16,300 candidates at this size. The 999,997-byte index improves the first five to 15, 143, 6, 9 and 128 candidates. `trace` still has 2,125.

The four present one/two-byte queries average **2.5813% skip in every mode**. Trigrams provide no constraint for them. Many blocks genuinely match: `a` appears in 16,376 blocks, `ab` in 15,490, `ms` in 14,065, and suffix `x` in 16,022.

The absent queries are reported separately:

| Query | 195,640 candidates | 999,997 candidates | Full candidates |
| --- | ---: | ---: | ---: |
| `*X` | 0 | 0 | 0 |
| `*warning*` | 1,047 | 89 | 0 |
| `*login*` | 1,396 | 1,396 | 17 |
| `*deadbeef*` | 2,029 | 2,029 | 16 |

Among the three absent ≥3-byte values, mean skip is 91.1238%, 93.0253%, and 99.9345% respectively. Including absent uppercase `X` would inflate this comparison.

## Failure cases and interpretation

1. **Unretained grams are unknown.** At 195,640 bytes, none of the grams of `error`, `user`, `server`, `service`, `team`, or `trace` survived the original query-independent selection. These queries reduce to byte+prefix pruning. The 1MB index retains useful grams for all six.
2. **Document popularity differs from term/block coverage.** Both partial indexes omit `tra`, `raa`, `3ic`, and `a76`. Their full-term distinct-document DFs are 1,653, 1,292, 1,256 and 3,044. Consequently `traa` retains 16,341 candidates (38 true blocks), `3ic` 16,349 (1,008 true), and `a76` 16,376 (2,103 true). Full trigrams reduce these to 141, 1,534 and 2,469. `world` retains only `wor` in both partial indexes and has 1,362 candidates for one true block. Full trigrams give 12.
3. **The cap rejects some common grams independently of budget.** `123` has remainder block DF 3,425, exceeding the cap-80 limit of 3,358.8. Both partial indexes omit it and retain all 16,794 blocks. Full trigrams give 3,809 candidates, with 3,443 true blocks. More bytes alone at this cap cannot retain this row.
4. **An individual row's block skip is not a query-wide guarantee.** `000` is retained with remainder block DF 3,320, but full-prefix-safe evaluation yields 3,708 candidates and 77.9207% skip, with 3,320 true blocks. Prefix-contained and crossing continuations must also be considered. Similar prefix cases make full `3ic` yield 1,534 candidates despite its row's 974 remainder-block postings.
5. **Full postings are necessary conditions.** Even complete rows can match grams in different terms or different locations within a block. Prefix crossings can leave one/two-byte continuations checked by byte fallback. Thus full references can retain false positives, including 17 for absent `login` and 16 for absent `deadbeef`. Truth uses complete keys, so every such excess is counted explicitly.

No false negatives occurred in any of the 42 × 16,794 × 4 decisions. Aggregates weight queries equally and count query/block pairs, not unique blocks across queries. No new document-level query DF was computed. No latency, I/O-byte, deployment-memory, or representative-workload claim follows from these candidate counts.

## Implementation and audit

`truth.cpp` scans every actual key in `terms.bin`, partitioned by the original `blocks.jsonl` physical term counts. It checks strict global key ordering, reads through EOF, and independently recomputes truth, prefixes and remainder bytes. Python verifies every block's term-stream offsets, ordinal, count, first/last key and persisted metadata. The original seven truth columns match the earlier extraction exactly.

The existing posting lookup binary decodes each requested full/partial trigram row. `verify-subset` checks **every retained row** in each existing partial index against the complete posting file. IU01 omitted rows remain unknown. IP01 missing rows mean absent. The existing `permits` callback evaluates prefix-contained, remainder-contained, and all viable crossing continuations. It requests only grams in the continuation, so one/two-byte continuations correctly use byte fallback.

The stored byte posting file uses one-byte vocabulary keys. `byte_directory_u24.bin` widens only those directory keys to reuse the same posting-frame decoder. All payload frames and offsets are preserved byte-for-byte. All 256 decoded byte memberships are checked against the fresh key scan. This helper file is an audit artifact, not additional index metadata. Prefix extensions are decoded from the persisted sidecar and original physical bounds, then compared with fresh key LCPs.

The evaluator cross-checks the first seven byte/full decisions against the previous persisted decisions. It persists all 42 decisions, reloads every row, recomputes every candidate bit from the decoded indexes, and audits truth, zero false negatives and `full ⊆ partial ⊆ byte` for every block. It recounts aggregates from the reloaded masks. Source hashes are rechecked after evaluation.

## Reproduce

From the repository root, with Python 3, clang++ and the existing lookup binary available:

```sh
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_*.py' -v
python3 dictionary_analysis/trigram_queries/run.py \
  KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

Only this experiment's output directory is written. `truth-scan`, widened byte directory, query/block-count inputs and truth/decision files are reproducibility helpers. They are excluded from deployable metadata costs. The script does not build or modify either selected index.

The ten tests cover frozen cohort, case/suffix/newline matching, high query-mask bits, absent-versus-unknown rows, prefix-contained/crossing cases with short and long continuations, exhaustive small-block zero-FN checks, corrupted decision rejection, and the C++ truth scanner against independent Python matching on a binary fixture.
