# Frozen-query metadata budget comparison

`budgets.py` evaluates the same frozen 42 case-sensitive queries at **200000,
300000, 400000, 500000, 600000, 750000, 900000 and 1000000 decimal bytes**.
Results live in `OUT/trigram_query_budgets/report.md` and `report.json`, where
`OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis`.

## Reproduce

Run from the repository root with Python 3 and the existing release posting
reader. The frozen query experiment and exact document-frequency experiment
must already exist. No additional dependencies or full-key scan are required.

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/trigram_queries/budgets.py "$OUT" \
  --binary dictionary_analysis/timestamp_injection/target/release/trigram-postings
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_budgets.py' -v
python3 -m unittest discover -s dictionary_analysis/trigram_queries -p 'test_queries.py' -v
python3 -m unittest discover -s dictionary_analysis/trigram_documents -p 'test_documents.py' -v
```

The runner reuses every existing `trigram_document_budgets/<budget>_skip80`
directory without writing to it. Missing directories are constructed through
`trigram_documents/budget.py` in their normal locations. In the initial run,
200000/300000/500000/1000000 already exist, and 400000/600000/750000/900000 are
built. Existing incomplete or inconsistent directories cause an assertion
failure instead of being replaced. The new query-budget output can be
regenerated. Construction provenance (`reused_existing_index`) will reflect
which directories exist at the beginning of each run.

## Selection and storage

Selection is generic and consumes no query information. Each trigram is ranked
by exact full-term distinct-document-union DF, descending, with ascending u24
keys breaking ties. Rows qualify only when `100 * block_DF <= 20 * 16794`
(the fixed 80% block-skip cap). Selection skips nonfitting rows and continues.
All selected posting rows are exact full-prefix-excluded remainder memberships.
Missing partial rows mean **unknown**, while missing full rows mean absent.

Actual deployable metadata is **5323 bytes + the actual partial-file size**:
5275 bytes of byte postings and 48 bytes of full-prefix extensions. Partial
files include their IU01 header, vocabulary, offsets and posting frames. The
percentage column compares actual bytes with **195640**, rather than with the
requested budget. Truth, decision masks, analysis reports and helper memory
are analysis artifacts and are excluded from metadata accounting.

## Verification and persisted masks

Before reuse, the runner verifies every artifact SHA-256 from the frozen query
report and every inherited source hash, including terms, SSTable, physical
blocks and indexes. It rechecks each truth row's physical block ID, ordinal,
term count, offsets, first/last key, prefix and remainder-byte mask. Prefixes
and bytes are freshly decoded from serialized sidecars. The independent
7,671,280-key truth scan is reused after these checks, rather than repeated.
A changed artifact or source stops the run. Recreate the independent truth
experiment separately if source data intentionally changes.

The immutable four-mode decisions are freshly recomputed. For each budget,
the runner independently recomputes the generic selection and checks its
sorted keys, accounting and construction report. `verify-subset` decodes and
compares **every** retained serialized row with the complete index. Query rows
are freshly read from disk. Existing prefix-contained, crossing and remainder
semantics use `trigram_queries.run.keep` and its unknown-safe callback.

Each budget has `<budget>/decisions.jsonl`. Every physical-block row records:

- block ID and all 42 exact matching-term counts;
- a truth mask;
- byte, full-trigram and that budget's partial candidate masks.

Masks are six little-endian bytes encoded as 12 hexadecimal characters. Bit
`j` is cohort query `j`. The upper six bits must be zero. The runner reloads
all persisted masks, recomputes each partial decision, verifies the previously
recomputed full/byte references, and independently recounts every query. Every
block/query must satisfy **truth ⊆ full ⊆ partial ⊆ byte**, with zero false
negatives. One/two-byte queries must equal the byte-only results. Modes are
passed explicitly. The existing runner's fixed `MODES` global remains intact.

`source_hashes_before.json` and `source_hashes_after.json` preserve identical
hashes for existing sources, reports and indexes. Newly constructed index
files are recorded in `created_index_hashes.json` and rechecked at the end.
The JSON report fingerprints the new decision artifacts for later verification.
The tests check disk fingerprints, six-byte masks including query 41,
dynamic modes, prefix/unknown semantics, no-FN/subset invariants, independent
recounts, aggregation, actual-byte accounting and query regressions.

## Reading results

The primary table covers the **34 present literals of at least three bytes**.
Every query has equal weight. Separate aggregates cover the four present
one/two-byte queries, three absent literals of at least three bytes and one
absent one-byte query. The report also provides every one of the 42 queries'
matching terms, true blocks, candidates, skipped blocks, skip percentage and
false-positive blocks at every budget. All false-negative counts are zero.

The client table shows `cli`, `lie`, `ien`, `ent` retention, candidates and
skip percentage. Thresholds are explicitly **first tested budget** with all
four rows retained and **first tested budget** with at least 99% skip. They do
not establish the minimum possible budget. Additional tables show error,
user, server, service, team, trace and traa.

Skip-nonfitting selection can replace previously selected small rows with
larger, higher-ranked rows. The runner checks every adjacent selection for
nesting and records removed keys. It compares every query's candidate counts
and individual block sets, reporting any count increase or newly admitted
block, including swaps with unchanged or decreasing totals.

This frozen diagnostic cohort is not a production workload sample. Present
and absent values remain separate. A per-row DF cutoff does not guarantee
query-wide skip, and complete trigrams can still admit false positives from
cross-term gram co-occurrence and prefix continuations. Term counts are
exact dictionary-key counts, rather than document-query DFs. This experiment
measures storage and candidate-block pruning. It makes no production latency
claim and changes no production code.
