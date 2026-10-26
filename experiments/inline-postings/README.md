# Inline postings experiment

`examples/inline_postings.rs` measures the original in-memory, byte-preserving SSTable prototype. Its tables describe that prototype, not the subsequent production encoding. The example does not write a new index format to disk and requires source lists stored externally.

The production block-local layout is documented in [`FORMAT.md`](../../src/termdict/sstable_termdict/FORMAT.md). It supports whole lists of up to 32 documents, including frequencies and positions, with inlining disabled by default. New inline lists use per-list document-ID deltas. Dictionary reads merge gaps up to 512 KiB. Postings reads additionally minimize requests within a byte allowance, as measured in [`REQUEST_BUDGET.md`](REQUEST_BUDGET.md).

## Run

```sh
cargo test --example inline_postings --features quickwit
cargo clippy --example inline_postings --features quickwit -- -D warnings
cargo run --release --example inline_postings --features quickwit -- \
  /tmp/wildcard-data/splits/KH4Y_01M5MTJQ3SVHDTK2W3XCB5/4fea6c896fd045aba3cece0808f3e41e.term \
  /tmp/wildcard-data/splits/KH4Y_01M5MTJQ3SVHDTK2W3XCB5/4fea6c896fd045aba3cece0808f3e41e.idx \
  > /tmp/inline-postings-measurements.json
```

The default features enable zstd compression. This example additionally requires `quickwit` and `columnar-zstd-compression`. It reads complete local components, verifies their Tantivy footer compatibility and CRC, and selects physical field 3. It intentionally targets the handoff's one split and seven ASCII wildcard patterns, including the added `*aba*`, `*player*`, and `*ms*` patterns.

`MEASUREMENTS.md` contains the original 4 MiB-gap tables. `measurements.json` retains that run's exact byte counts, all three scan timings, matching/inline list counts, and external read plans. Timing runs are sequential, with resident inputs and no CPU isolation.

`CUTOFFS.md` compares inline cutoffs of one through four documents, including the added three-document measurement, and examines coalescing around 512 KiB. `COALESCING.md` compares gaps from 0 through 1 MiB. `coalescing.json` retains the sweep output for all document/byte cutoffs and seven query patterns: `*a*`, `*ab*`, `*aba*`, `*traaa*`, `traaa*`, `*player*`, and `*ms*`. The example emits `coalescing_gap_bytes` with the tested gaps, and each measurement's `queries[gap_index][pattern_index]` holds its plan. Zero gap still merges touching ranges. The earlier sweep reproduced the original 4 MiB results exactly. The current sweep omits 2–8 MiB and reproduces all retained original-pattern plans and dictionary sizes exactly. Across the sweep, request bytes are nondecreasing and request counts are nonincreasing as the gap grows. Zero-gap request bytes equal matching external-list bytes. Baseline plans at shared gap sizes also match the earlier independent analyzer.

## Representation

The block header retains the term count, initial external postings offset, and initial positions offset. Each term contains:

1. Document count as the existing VInt.
2. `(payload_length << 1) | inline_flag` as a VInt.
3. Positions length as the existing VInt.
4. The unchanged document-ID delta VInts and frequency VInts when inline.

An external term advances the cumulative external postings offset by its payload length. An inline term owns its payload and advances the offset by zero. Position offsets and lengths remain unchanged. Selection applies to whole lists with 1–127 documents. Longer lists, including their short final blocks, remain external.

The custom `InlineTable` reader and writer exist only inside the example. They use the ordinary SSTable key encoding, compression, block index, and streaming/lookup APIs. The reported dictionary size includes four bytes for the existing dictionary-type trailer as a size convention. The prototype has no production format identifier and cannot be opened by production `TermDictionary` readers.

The experiment computes compacted external ranges without writing a compacted `.idx` file. The surviving bytes would remain unchanged. Every rewritten key, document count, position range, inline payload, and external range is checked against the input before timing.

## Validation

- Five unit tests passed: mixed inline/external entries across blocks and point lookup, payload ownership after stream/dictionary destruction, cutoff boundaries including 127–131 documents, truncated payloads, offset overflow, coalescing at the 4 MiB boundary, a sweep checking zero-gap and exact-boundary merging, and positive/negative substring matching for `*player*` and `*ms*`.
- All twelve prototype scenarios completed on 7,671,280 real terms, with complete rewritten-value checks.
- Rewriting through the existing dictionary builder reproduces the input dictionary byte-for-byte: 51,677,994 bytes.
- Document-cutoff counts and displaced bytes match the handoff's independent 1–128-document histogram exactly.
- All four original wildcard matching counts and baseline request plans match the earlier pinned-revision analyzer exactly.
- Example Clippy passed with warnings denied. The release build passed.

## Inputs and implementation base

Source branch HEAD before the experiment: `b9c4ed1c6`. Runtime measurement baseline: `2e87c2eb0c8577c96c82a777f345bbf236a985fe`. The byte-identical legacy rewrite checks this branch's output against the actual runtime dictionary, rather than assuming the revisions are interchangeable.

The `.idx` component was fetched with the existing `fetch_split_dictionaries.py` range-fetch helper from the synthetic split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5.split` (source bucket omitted), bytes `[323771542, 899245506)`. Component size: 575,473,964 bytes. SHA-256: `f936d2260d4cac945f258e073b9c2a9acd23fef003dcc213ebe73c697c62c141`. Retrieval metadata is beside the local component in `postings-provenance.json`. The source objects were read only. No positions payload was fetched.

The old `single_doc_posting_list` worktree was inspected and left untouched. Its uncommitted change suppresses tails of 1–3 documents without storing them elsewhere, including tails of longer lists. It supplies no reusable inline reader or format.

## Production integration

The inline limit is now configurable through Quickwit's `IndexSettings.max_inline_docs` (0–32, default 0). Each type-4 dictionary stores its maximum once and derives its packed posting-list width. See [`FORMAT.md`](../../src/termdict/sstable_termdict/FORMAT.md). The saved measurements below predate this change and retain their fixed-width layouts. Current example sweeps use the selected maximum's derived width. Historical patch recipes target the earlier implementation.

The production change covers `TermInfo`, field serialization, SSTable block/value encoding, regular postings open/reset, positions, every warmup path, and format version dispatch. Default-feature FST dictionaries remain external. Tests cover term scoring, seeking/reset, record options, phrase queries, segment merging/deletion remapping, old/new dictionary formats, and inline-only/mixed async warmup.

Historical absolute-ID measurements in [`PRODUCTION_SIZE.md`](PRODUCTION_SIZE.md) cover the message field, including positions: 516.50 MB → 491.11 MB combined (−4.91%). [`PRODUCTION_CUTOFFS.md`](PRODUCTION_CUTOFFS.md) compares maximum list lengths 0–8 with absolute IDs. Rebuilding limit 0 reproduces the original 51,677,994-byte dictionary exactly apart from its dictionary-type marker. Embedding only single-document lists gives 93.23% of the ≤8 size saving. [`PRODUCTION_READS.md`](PRODUCTION_READS.md) compares query reads, including dictionary blocks, across all cutoffs and against the original layout. Preliminary resident dictionary-scan timings and outer-zstd comparisons are also included. Indexing throughput, actual term-query/wildcard latency, and cache behavior still need measurement. The current delta-encoding sweep is in [`SIZE_UP_TO_128.md`](SIZE_UP_TO_128.md), covering every cutoff from 1 through 16 and larger cutoffs. It preserves the absolute-ID baseline and includes a RAM-scan comparison. Cutoffs 64 and 128 remain isolated experiments.

[`SIX_BIT_COUNTS.md`](SIX_BIT_COUNTS.md) extends the size sweep through every cutoff 0–32 in an isolated experimental build, using five count bits plus one positions flag. At ≤32, combined size is 462.899 MB (−10.38%), with a 129.426 MB dictionary. Production remains at ≤8. The experiment patch and exact results are preserved beside the report. [`SIX_BIT_DELTAS.md`](SIX_BIT_DELTAS.md) repeats all 0–32 cutoffs with per-list document-ID deltas: at ≤32, the dictionary falls to 124.717 MB and the total to 458.190 MB (−11.29% versus the original). Production remains unchanged.
