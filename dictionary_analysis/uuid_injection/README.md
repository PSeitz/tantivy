# Controlled UUID injection/rebuild experiment

Adds whole dictionary terms while retaining every original key. Uses the actual unchanged `VoidSSTable` writer and reads its serialized block addresses and index fences. This helper owns only `dictionary_analysis/uuid_injection/` and `OUT/uuid_injection/`. Original source files and the timestamp helper remain unchanged.

## Reproduce

From the repository root:

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
mkdir -p "$OUT/uuid_injection"
cargo test --manifest-path dictionary_analysis/uuid_injection/Cargo.toml \
  > "$OUT/uuid_injection/test-rust.log" 2>&1
cargo build --release --manifest-path dictionary_analysis/uuid_injection/Cargo.toml \
  > "$OUT/uuid_injection/build.log" 2>&1
dictionary_analysis/uuid_injection/target/release/uuid-injection "$OUT" \
  > "$OUT/uuid_injection/run.log" 2>&1
python3 -m unittest discover -s dictionary_analysis/uuid_injection -p 'test_*.py' \
  > "$OUT/uuid_injection/test-python.log" 2>&1
python3 dictionary_analysis/uuid_injection/report.py "$OUT" \
  > "$OUT/uuid_injection/report.log" 2>&1
```

The measured run reused the timestamp helper's Cargo target cache via `CARGO_TARGET_DIR=dictionary_analysis/timestamp_injection/target`. Its source was unchanged. The commands above use a separate target directory instead.

## Matrix

- Zero-injection baseline, with exact comparison to all original Writer boundaries.
- 1 million each of UUIDv1/v3/v4/v5/v6/v7, both canonical lowercase 36-byte and compact lowercase 32-byte layouts.
- 1 million fixed 36-byte IDs from `0-9a-z`.
- 10 million each of canonical v4/v7 and the alphanumeric control.

All seeds, node, clock sequence, namespace and names are persisted in `rebuilds.json`. v3/v5 use actual MD5/SHA-1. Generation tests include known hash vectors and timestamp extraction. Whole UUIDs are terms here. Tokenizer hyphen splitting is outside this experiment.

## Validation and output

`src/generate.rs` generates fixed-array IDs, sorts and deduplicates them. `src/main.rs` merges the original length-prefixed stream into actual key-only SSTables. Every decoded key equals the merged input, and origin bits distinguish original, generated and colliding membership. Exact truth is scanned over full decoded keys independently of the remainder-byte summary. Original truth is checked in every scenario.

`report.py` serializes PM01 remainder-byte masks, PX02 sparse prefixes relative to each new index, and LN01 full-length bounds. LN01 is an 8-byte magic/count header followed by min/max u8 pairs. All sidecars are reopened and decoded before evaluation. Full actual prefixes are excluded from byte masks. Full term lengths include those prefixes by definition.

All seven original queries remain in the results. Byte+length applies `max_full_length >= literal_length`, a necessary condition for substring/suffix queries with unbounded stars. Minimum length supplies no additional rejection for these patterns. Unit tests cover exactly-36 and longer literals as predicate checks, not replacement workloads.

The report contains actual sidecar bytes and percentages relative to the **original 195,640-byte production index, reference only**. Key-only index bytes do not estimate production rebuild storage. Candidate percentages count blocks. Original terms inside candidate blocks are also counted to distinguish denominator changes from changes in existing-key candidate work. No production latency or I/O claims.

Artifacts remain under `OUT/uuid_injection/`: readable `report.md`, machine `report.json`, generation/rebuild provenance, actual SSTables and hashes, per-block summaries/truth, binary sidecars, decisions and logs. Large SSTables are retained for audit.
