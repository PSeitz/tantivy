# Inverted remainder-feature block postings

This standalone experiment maps each observed within-term remainder bigram or byte to the sorted IDs of blocks containing it. It reads the four retained UUID-study SSTables without changing their keys, boundaries, source files or production code.

The **entire actual common prefix** of every block is excluded before extracting features. Pair windows are taken independently inside each term. The vocabulary contains every observed feature, independent of queries.

## Run

From the repository root:

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
export CARGO_TARGET_DIR=dictionary_analysis/timestamp_injection/target
cargo test --locked --manifest-path dictionary_analysis/pair_postings/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/pair_postings/Cargo.toml -- "$OUT"
python3 -m unittest discover -s dictionary_analysis/pair_postings -v
python3 dictionary_analysis/pair_postings/report.py "$OUT"
```

The Rust command accepts an optional scenario name after `OUT`. The default covers `baseline`, `v4_canonical_10000000`, `v7_canonical_10000000`, and `alphanumeric36_10000000`. Sources live under `OUT/uuid_injection`; all new artifacts live under `OUT/pair_postings`. Python uses the existing `prefix_summaries` predicates and the CLI `zstd` to cross-check the previous baseline PC01 artifact. Posting compression uses Rust's zstd library, level 3.

See `OUT/pair_postings/report.md` and `report.json` for measured results. All 80 posting artifacts are retained, including intentionally large raw controls. They occupy approximately 1.6 GiB in total, including validation artifacts. Actual sizes and SHA-256 hashes are in the report.

## Posting format IP01

All integers are little-endian, except feature semantics: a pair value is `(first_byte << 8) | second_byte`.

- Four bytes `IP01`.
- `u32` block universe size, `u32` observed vocabulary size.
- `u8` feature width (1 unigram, 2 bigram), `u8` raw block-ID width.
- Sorted vocabulary, using the feature width.
- `vocabulary_size + 1` `u32` offsets relative to the payload start. First offset is zero, final offset is the exact payload size. Each posting row is independently addressable by binary-searching the vocabulary and reading its two offsets.
- Each row: `u8` tag, `u32` uncompressed payload length, payload. Tag bits 0–1 select raw/delta/runs/bitmap, bit 2 complements the decoded posting within the block universe, bit 3 selects a separate zstd frame. All remaining bits are zero.

Payloads:

- **Raw:** tightly packed fixed-width block IDs, using the smallest whole-byte width for `blocks - 1`. Baseline uses two bytes. The 10m cases use three. A universe of 65,536 blocks still fits two bytes because the largest ID is 65,535.
- **Delta:** unsigned LEB128 first ID followed by unsigned LEB128 differences from the previous ID.
- **Runs:** unsigned LEB128 gap from previous run's exclusive end (initially zero), then unsigned LEB128 run length, repeated to payload end.
- **Bitmap:** `ceil(blocks / 8)` bytes, least-significant bit first. Unused final bits are zero.

Every format charges every vocabulary, directory, row framing, polarity, codec and compression byte. The four positive-only codecs are each measured raw and per-row zstd. Adaptive tests all four codecs with positive and absent-block lists. Adaptive-zstd additionally tests each choice compressed and uncompressed. Selection uses serialized bytes only. There is no whole-file/load-all compressed comparison.

## Other artifacts

- `measurement.json`: Rust extraction provenance, feature statistics, file sizes, hashes and row codec distributions.
- `{pairs,bytes}_decoded_masks.bin`: validation masks reconstructed from the smallest posting file **after reading and decoding it**. Format `IM01`, `u32 blocks`, `u32 vocabulary_size`, sorted `u16` vocabulary, then one little-endian bitmask of `ceil(vocabulary_size / 8)` bytes per block. These are validation artifacts, excluded from metadata totals.
- `prefixes.bin`: the existing PX02 representation, regenerated into this experiment's own directory from scenario fences plus full-LCP extensions. Round-tripped and checked against that scenario's sidecar. Charged once per combined summary.
- `decisions.bin`: `ID01`, `u32 blocks`, `u32 JSON-definition-length`, definitions, then four bytes per block: unigram decisions, pair decisions, combined decisions, exact-truth bits. Query bit order is recorded in the definitions. This is a validation artifact, excluded from metadata totals.
- `rust-tests.log`, `python-tests.log`, `existing-tests.log`, `run.log`, `evaluation.log`: retained validation logs.

## Safety and scope

Every source SSTable's hash and full decoded-key record digest must match the earlier rebuild. Every block's ordinals, count, first/last terms, fences, actual LCP, byte mask and seven independently rescanned full-term truths must agree. Every posting is decoded and compared for every format. Baseline pairs additionally match the earlier exact 1,443-feature PC01 artifact block for block.

Query evaluation reconstructs masks from decoded postings and calls `prefix_summaries.permits`. Pair-only uses all 256 byte bits set to disable byte pruning. Single-byte queries retain all blocks. Prefix-contained and prefix-crossing matches are evaluated conservatively, with requirements only on the remaining continuation. This validates pruning semantics, rather than implementing a direct inverted-posting query planner or measuring latency.

All percentages use the **original 195,640-byte index reference**, including generated scenarios. They describe additional metadata, not an estimate of each rebuilt production index. Best unigram cost compares these inverted files against previous PM01 mask interning. This distinction matters for reassessing UUIDv4's previous large PM01 cost.
