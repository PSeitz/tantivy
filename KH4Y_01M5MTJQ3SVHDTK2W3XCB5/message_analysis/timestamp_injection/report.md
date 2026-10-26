# Slowly increasing timestamps: actual dictionary injection and block rebuild

## Result

Adding 10 million distinct consecutive 13-digit millisecond keys increases blocks from 16,794 to 22,085. There are 5,290 pure-numeric blocks (23.95%). All seven letter queries skip every pure-numeric block with the prefix-aware byte predicate. At 10 million, absolute candidate counts equal baseline for every query. The higher skipped percentage represents avoiding the newly added numeric blocks, rather than better pruning of mixed-text blocks.

This answers the controlled timestamp-density question by inserting new values and rebuilding blocks. Classification of pre-existing numeric IDs is a different experiment.

## Construction and validation

- Start at `1700000000000`, step 1 millisecond, fixed-width 13-digit canonical decimal keys. Merge with the original 7,671,280 sorted keys and deduplicate exact collisions.
- Each value contributes one dictionary key. Repeated timestamp occurrences across logs collapse to one key. The original segment has 10,002,000 documents. Ten million distinct milliseconds span 9,999.999 seconds (about 2 h 46 min 40 s). Many logs sharing a clock tick would add fewer distinct keys. The experiment models vocabulary density, rather than rewriting documents.
- Use the existing `VoidSSTable::writer`, its public `insert`/`finish` API, and its unchanged default 4,000-byte key-delta target. Actual SSTables are serialized, reopened, and every decoded key is compared with the merged stream. Their files and SHA-256 digests are retained.
- **Zero-injection exact verification passed for all 16,794 original blocks**: block IDs, first ordinals, term counts, first/last keys, and lower/upper index separators. Full decoded key-stream equality passed. Physical compressed offsets differ because values are omitted.
- The baseline also reproduces the original prefix and byte-mask sidecars byte-for-byte, and every original per-block exact match count and byte-pruning decision.
- Summaries exclude each block’s full actual longest common prefix. Prefixes reconstruct from the new index fences plus sparse serialized extensions. The existing conservative `permits` predicate checks prefix-contained, remainder-contained and prefix-crossing possibilities.
- Every query is case-sensitive. Exact matches are scanned independently over full decoded keys. All scenarios reproduce original exact totals. Zero false-negative blocks. All pure-numeric blocks are rejected for all seven queries.
- Summaries and mask interning use observed bytes only. No query-selected features or neighbor unions.
- Original source and extracted SSTable hashes still match extraction provenance. The original term stream hash is unchanged. Production/source files remain untouched.

## Consecutive millisecond scenarios

“Timestamp-only” means every key belongs to the generated range/step, including any original colliding key. “Pure numeric” means every full key matches nonempty ASCII `[0-9]+`.

| Distinct timestamps | Collisions | Total terms | Total blocks | Pure numeric blocks | Timestamp-only blocks | Mixed timestamp blocks |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | 7,671,280 | 16,794 | 0 | 0 | 0 |
| 10,000 | 0 | 7,681,280 | 16,800 | 4 | 4 | 2 |
| 100,000 | 0 | 7,771,280 | 16,847 | 52 | 52 | 2 |
| 1,000,000 | 0 | 8,671,280 | 17,323 | 528 | 528 | 2 |
| 10,000,000 | 0 | 17,671,280 | 22,085 | 5,290 | 5,289 | 3 |

## Byte-pruning results

**Each cell is skipped-block percentage / absolute candidate blocks.** These are block percentages, not compressed-byte or latency reductions.

| Query | 0 added timestamps | 10,000 added timestamps | 100,000 added timestamps | 1,000,000 added timestamps | 10,000,000 added timestamps |
| --- | --- | --- | --- | --- | --- |
| `*a*` | 2.4890% / 16,376 | 2.5298% / 16,375 | 2.8254% / 16,371 | 5.4667% / 16,376 | 25.8501% / 16,376 |
| `*ab*` | 2.5068% / 16,373 | 2.5417% / 16,373 | 2.8373% / 16,369 | 5.4840% / 16,373 | 25.8637% / 16,373 |
| `*traa*` | 2.6974% / 16,341 | 2.7083% / 16,345 | 3.0035% / 16,341 | 5.6861% / 16,338 | 26.0086% / 16,341 |
| `*X` | 100.0000% / 0 | 100.0000% / 0 | 100.0000% / 0 | 100.0000% / 0 | 100.0000% / 0 |
| `*x` | 2.6200% / 16,354 | 2.6607% / 16,353 | 2.9382% / 16,352 | 5.6053% / 16,352 | 25.9497% / 16,354 |
| `*player*` | 2.7569% / 16,331 | 2.8214% / 16,326 | 3.0925% / 16,326 | 5.7727% / 16,323 | 26.0539% / 16,331 |
| `*ms*` | 2.7093% / 16,339 | 2.7262% / 16,342 | 3.0272% / 16,337 | 5.6745% / 16,340 | 26.0177% / 16,339 |

| Query | Exact matching terms (all scenarios) | Candidate delta, 10m vs baseline |
| --- | --- | --- |
| `*a*` | 1,921,269 | +0 |
| `*ab*` | 75,774 | +0 |
| `*traa*` | 42 | +0 |
| `*X` | 0 | +0 |
| `*x` | 153,990 | +0 |
| `*player*` | 21 | +0 |
| `*ms*` | 50,585 | +0 |

Ten million inserted terms add 5,291 blocks. Nearby decimal values share long prefixes, so term share is a poor predictor of block share. Insertion also shifts nearby block boundaries, explaining small candidate-count differences.

## Sparse spacing sensitivity

The optional 1-million-key comparison uses the same 13-digit start and step 1,000 milliseconds (one second). Longer suffix changes consume more key-delta bytes, and the wider lexical range interleaves with more original terms. Both effects can change block counts.

| Step (ms) | Total blocks | Pure numeric | Timestamp-only | Mixed timestamp | Collisions |
| --- | --- | --- | --- | --- | --- |
| 1 | 17,323 | 528 | 528 | 2 | 0 |
| 1000 | 18,074 | 1,275 | 1,153 | 128 | 0 |

| Query | Step 1: skip % / candidates | Step 1000: skip % / candidates |
| --- | --- | --- |
| `*a*` | 5.4667% / 16,376 | 9.4113% / 16,373 |
| `*ab*` | 5.4840% / 16,373 | 9.4390% / 16,368 |
| `*traa*` | 5.6861% / 16,338 | 9.6105% / 16,337 |
| `*X` | 100.0000% / 0 | 100.0000% / 0 |
| `*x` | 5.6053% / 16,352 | 9.5386% / 16,350 |
| `*player*` | 5.7727% / 16,323 | 9.7267% / 16,316 |
| `*ms*` | 5.6745% / 16,340 | 9.6160% / 16,336 |

## Actual serialized metadata

PM01 stores interned 256-bit remainder-byte masks plus per-block IDs and framing. PX02 stores sparse prefix extensions beyond prefixes derived from each rebuilt index. Every sidecar is written, reopened, decoded and compared. Table totals include both complete files.

| Timestamps / step ms | Masks (bytes) | Prefixes (bytes) | Total (bytes) | % of original 195,640-byte index (reference only) |
| --- | --- | --- | --- | --- |
| 0 / 1 | 20,307 | 48 | 20,355 | 10.40% |
| 10,000 / 1 | 20,345 | 75 | 20,420 | 10.44% |
| 100,000 / 1 | 20,104 | 57 | 20,161 | 10.31% |
| 1,000,000 / 1 | 20,964 | 57 | 21,021 | 10.74% |
| 10,000,000 / 1 | 25,598 | 48 | 25,646 | 13.11% |
| 1,000,000 / 1000 | 21,651 | 102 | 21,753 | 11.12% |

**Scope:** Key-only SSTables contain valid dictionary keys and unit values. They contain no fabricated term infos or postings. Their data bytes and index bytes are not estimates of a production rebuild with real postings. No compressed-byte savings, physical-I/O savings or latency improvements are claimed. Sidecar sizes are actual standalone encodings, excluding runtime allocations and any production-container overhead.

## Artifacts

- `report.json`: complete scenario metrics, checks, hashes and query counts.
- `rebuilds.json`, `*/rebuild.json`: actual writer/readback provenance.
- `*/keys.sstable`: actual serialized key-only dictionary.
- `*/blocks.jsonl`: new bounds, ordinals, first/last keys, full prefix, remainder mask, numeric/generated membership and independent exact truth.
- `*/bytes.bin`, `*/prefixes.bin`: evaluated serialized sidecars.
- `*/decisions.jsonl`: per-block pruning decisions from decoded sidecars.
- `test-rust.log`, `test-python.log`, `build.log`, `run.log`, `report.log`: validation logs.

Reproduction instructions: `dictionary_analysis/timestamp_injection/README.md`.
