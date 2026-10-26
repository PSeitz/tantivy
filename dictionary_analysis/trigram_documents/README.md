# Exact trigram document frequency

This experiment reads real `message` term postings from the original segment, computes exact distinct-document unions, and ranks whole trigram block-posting rows under the existing 1,000,000 B budget.

## Semantics

- Ranking: case-sensitive raw-byte within-term trigrams from **full terms**. Deduplicate repeated grams in a term and document IDs shared across matching terms.
- Universe: the existing 47,796 remainder-posting rows. Extra full-term grams outside this universe do not enter selection.
- Stored postings: unchanged original physical blocks, summarized from **full-LCP-excluded remainders**.
- Eligibility: `blockDF * 100 <= blocks * (100 - cutoff)`.
- Ordering: descending exact document DF, then ascending u24 gram. Keep a whole row when its frame plus 7 B directory entry fits. Continue after nonfitting rows.
- Cost: partial IU01 file + 5,275 B byte fallback + 48 B prefix extensions. Header/directory is `18 + 7 * rows`.
- Query safety: omitted grams are unknown. Reuse the existing `permits` continuation callback and decoded prefix sidecar. Construction receives no queries.
- Scope: original single-segment message field, 10,002,000 documents, no deletes. Synthetic dictionaries have no document assignments and are excluded.

## Source reader

`src/main.rs` uses the unchanged public Tantivy `SegmentReader`, `InvertedIndexReader`, and `BlockSegmentPostings` APIs. It creates a disposable isolated directory with byte-identical copies of the original `.term`/`.idx` and generated auxiliary files to satisfy reader opening. Those auxiliary store, fast-field, fieldnorm, and positions files are never queried or claimed as original data. Raw fixture tokenizers only construct these unused files. They never process source terms.

The original schema records positions. Basic postings requests decode actual document IDs while skipping frequency/position retrieval. Both original composites pass managed CRC and provenance SHA-256 checks. The helper validates the exact message SSTable slice, every physical address/ordinal, all 7,671,280 original keys, posting ranges, every term's document count, strict document ordering, and document bounds. Input hashes are checked again at completion.

Document postings are decoded once into a u32 cache. Gram-to-term lists are deduplicated per term. One reusable 1,250,256 B bitset unions the document IDs for each gram. `term_df_sum_upper_bound` is a diagnostic u64 sum, never the ranking value. Full-query distinct-document unions are separate diagnostics.

## Reproduce

Run from the repository root. Outputs are confined to `message_analysis/trigram_documents/` and the existing shared Cargo target directory. The source and previous indexes are read-only inputs.

```bash
DATA=KH4Y_01M5MTJQ3SVHDTK2W3XCB5
OUT="$DATA/message_analysis/trigram_documents"
mkdir -p "$OUT"
export CARGO_TARGET_DIR=dictionary_analysis/timestamp_injection/target
cargo build --release --locked --manifest-path dictionary_analysis/trigram_documents/Cargo.toml
/usr/bin/time -l "$CARGO_TARGET_DIR/release/trigram-documents" "$DATA" \
  > "$OUT/extract_run.log" 2> "$OUT/extract_resources.log"
/usr/bin/time -l python3 dictionary_analysis/trigram_documents/run.py "$DATA/message_analysis" \
  > "$OUT/selection_run.log" 2> "$OUT/selection_resources.log"
python3 dictionary_analysis/trigram_documents/run.py "$DATA/message_analysis" --report-only

cargo test --release --locked --manifest-path dictionary_analysis/trigram_documents/Cargo.toml
python3 -m unittest discover -s dictionary_analysis/trigram_documents -v
python3 -m unittest discover -s dictionary_analysis/trigram_selection -v
python3 -m unittest discover -s dictionary_analysis/trigram_postings -v
python3 -m unittest discover -s dictionary_analysis -p 'test_prefix_summaries.py' -v
```

`/usr/bin/time -l` is the macOS resource interface. `--report-only` parses its completed logs. The existing `trigram-postings` binary supplies frequency dumps, lookup, and independent retained-posting verification. Its path can be overridden with `--binary`.

## Artifacts

- `document_frequencies.jsonl`: exact DF, matching term count, max matching term DF, and sum upper bound for every candidate.
- `extraction.json`, `source_manifest.json`: source validation, document/query counts, resource counters, hashes.
- `baseline/skip_XX.bin`: eleven actual IU01 files, with byte/prefix sidecars shared per deployment.
- `baseline/skip_XX_decisions.bin`: every per-block query decision for both modes plus truth.
- `selection_rows.jsonl`: every gram's block DF, document DF, serialized row cost, global/eligible ranks, and retained caps.
- `report.json`, `report.md`: actual costs, all seven queries across all eleven caps, original policy comparison, reference costs, top retained examples, validation, and resources.
- `selection_inputs.json`: selection input hashes checked before/after.
- `failures.md` and failure logs: resolved fixture-tokenizer and JSON-report-key failures, plus workload failures.

The only shared-helper change is optional `ranking` in `trigram_selection.sweep.choose`. Its default remains descending block DF. Existing 44-index regression tests remain passing. Parent documentation and production code are untouched.

## Original +100% metadata allowance

The parent also reran the same document-ranking rule with an 80% per-row absence cutoff and the original **195,640 B (+100%) total allowance**:

```sh
python3 dictionary_analysis/trigram_documents/budget.py "$DATA/message_analysis"
```

This retains **276 trigram rows**, including `pla`, `lay`, `aye`, and `yer`. Actual cost is **190,317 B selected trigrams + 5,275 B bytes + 48 B prefixes = 195,640 B**. It retains **14 player blocks (99.9166% skipped)**, matching the full trigram index. All other tested queries remain at byte-only pruning, including `traa` at 16,341 candidates. All retained postings were decoded and compared with the full index, and every query decision was evaluated from the persisted partial index. No false negatives occurred. Construction uses document counts, block counts and encoded sizes, independently of queries.

Outputs are separate under `message_analysis/trigram_document_budgets/195640_skip80/`, preserving the 1 MB experiment and its reproducibility hashes. The helper accepts `--budget` and `--cutoff` for additional explicitly measured settings.

## Main finding

`pla/lay/aye/yer` have about 5.9–6.0 million distinct documents and become ranks 17–21. Caps through 92% retain all four and reduce `*player*` to 14 candidate blocks, matching the complete trigram reference. At 94% only `pla/lay` survive, giving 134 candidates. Caps 95–99% exclude all four.

`tra/raa` have only 1,653/1,292 distinct documents and remain omitted under every tested cap. `*traa*` stays at the byte-only 16,341 candidates. The 80% cap improves none of these seven queries over the uncapped document-DF policy. These observations support this workload comparison, rather than a universal optimum.
