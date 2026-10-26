# Timestamp injection experiment

Merge slowly increasing timestamp keys into the actual sorted message dictionary, rebuild actual key-only SSTables, and measure prefix-aware byte pruning. All new helper files live here. All generated artifacts live in `OUT/timestamp_injection/`.

## Run

From the repository root:

```bash
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
HELPER=dictionary_analysis/timestamp_injection
mkdir -p "$OUT/timestamp_injection"
cargo test --locked --manifest-path "$HELPER/Cargo.toml" > "$OUT/timestamp_injection/test-rust.log" 2>&1
cargo build --release --locked --manifest-path "$HELPER/Cargo.toml" > "$OUT/timestamp_injection/build.log" 2>&1
"$HELPER/target/release/timestamp-injection" "$OUT" > "$OUT/timestamp_injection/run.log" 2>&1
PYTHONPATH=dictionary_analysis python3 -m unittest -v test_prefix_summaries > "$OUT/timestamp_injection/test-python.log" 2>&1
python3 "$HELPER/report.py" "$OUT" > "$OUT/timestamp_injection/report.log" 2>&1
```

Read `OUT/timestamp_injection/report.md`. Machine-readable results are in `report.json`.

Inputs are the existing `terms.bin`, `blocks.jsonl`, `evaluation.json`, `extraction.json`, original source SSTable, and `prefix_summaries/actual/` baseline artifacts. The Python helper imports the existing prefix-aware predicate and sidecar encoders without modifying them. Python 3.10+ and Rust with edition 2024 support are required. The existing Python layout tests optionally use the `zstd` executable. Rust dependencies are pinned in this helper's `Cargo.lock`.

## Construction

- Consecutive scenarios: 0, 10,000, 100,000, 1,000,000 and 10,000,000 distinct timestamps, starting at `1700000000000`, step 1 millisecond.
- Optional sensitivity case: 1,000,000 timestamps with step 1,000 milliseconds. This changes both key deltas and the extent of lexical interleaving with original keys.
- A streaming merge walks original length-prefixed records and generated fixed-width keys, deduplicating exact collisions. Only the source record buffer, current timestamp and current decoded block are needed, rather than a vector of all merged keys.
- Actual `VoidSSTable::writer` with public `insert`/`finish`, unchanged default 4,000 key-delta-byte target. The target is a flush threshold, rather than an exact physical block size.
- Every serialized key is decoded through the existing reader and compared with the merged sequence. Ordinals, strict ordering and fences are checked. SHA-256 covers both serialized SSTables and decoded length-prefixed term streams.
- Scenario zero runs first. It must exactly match all original block IDs, first ordinals, term counts, first/last keys and index fences before any augmentation is attempted.
- Summaries exclude the full actual common prefix of each rebuilt block. Masks are generic per-block sets of observed remainder bytes. No query-selected summaries or neighbor unions.
- Query definitions are loaded after SSTable construction, solely for evaluation. Independent exact truth scans full decoded keys. Python evaluates the existing conservative `permits` predicate after sidecars have been written and read back.
- PM01 mask sharing and PX02 prefix extensions include actual framing, tables and IDs. Prefix bases come from each new index's own fences. Baseline sidecars must be byte-identical to the previous actual-prefix experiment, and all baseline per-block truth and decisions must match.

## Scope

This is synthetic dictionary augmentation and actual reblocking, rather than a scan of pre-existing numeric IDs. Distinct timestamp vocabulary determines dictionary growth. Repeated timestamp occurrences across documents collapse to one key.

Pure-numeric blocks contain only nonempty ASCII digit keys. Timestamp-only blocks contain only keys belonging to the generated range/step, including any original collisions. Membership is defined directly from generation parameters.

The output uses valid unit values and contains no fabricated term infos or postings. Key boundaries and pruning decisions are measured on actual writer output. Key-only data and index sizes are diagnostic artifacts, not production size estimates. Skipped percentages refer to blocks. No compressed-byte, physical-I/O or latency savings are inferred.

Original files and production source remain unchanged. Only this helper and `OUT/timestamp_injection/` are written.
