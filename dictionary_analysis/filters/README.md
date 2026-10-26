# Byte, pair, trigram Bloom, and suffix filters

Standalone Rust helper. Reads the existing exports without changing other helpers or original block boundaries.

```sh
cargo test --release --locked --manifest-path dictionary_analysis/filters/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/filters/Cargo.toml -- \
  KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis \
  KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

Inputs: `terms.bin`, `blocks.jsonl`, `evaluate_blocks.jsonl`, `evaluation.json`. Outputs: `filters.json` (aggregate configurations, metadata, baseline comparison) and `filters_blocks.jsonl` (actual decisions for every configuration and block). Every per-block candidate/count array follows `[*a*, *ab*, *traa*, *X, *x, *player*, *ms*]`. Only indices 3 and 4 are suffixes. `last_byte_candidates` remains the two-element array `[*X, *x]`. The appended `player` and `ms` queries match anywhere inside a term.

## Actual construction and semantics

All matching is case-sensitive on raw bytes. NUL, invalid UTF-8, and newline are ordinary bytes. Grams come from individual full terms and never span term boundaries. Exact pair/triple sets are computed once per block and reused by every configuration. The earlier evaluator exports cardinalities rather than the sets themselves. Reconstructed cardinalities and direct matching-term counts are asserted equal to those exports.

A whole-byte mask requires every literal byte. Exact pair tests require every literal pair. Bloom tests require every literal gram of their configured length. A short literal receives no test from a longer-gram summary. `*traa*` requires both `tra` and `raa` in the trigram filter, and `tr`, `ra`, `aa` for pair tests. `*player*` requires pairs `pl`, `la`, `ay`, `ye`, `er` and triples `pla`, `lay`, `aye`, `yer`. `*ms*` requires pair `ms` and receives no trigram test. These are necessary conditions. Grams in different terms or different positions can satisfy them without a matching term.

The helper constructs every Bloom bit array and queries the actual bits. It uses no independence approximation for pruning. No bounds, lengths, prefix solver, or existing-candidate intersection is applied. Existing evaluator aggregates are copied into `baseline_results` for comparison.

Configurations:

- `bytes`: 32-byte whole-key byte mask.
- `exact_pair`: exact pair tests, with a byte mask stored to rank the packed pair representation. One-byte queries receive no test.
- `bytes_exact_pair`: same storage as `exact_pair`, with byte tests enabled.
- `tri_SIZEB_sSEED`: trigram Bloom alone.
- `bytes_tri_SIZEB_sSEED`: byte mask plus trigram Bloom.
- `exact_pair_tri_SIZEB_sSEED`: exact pair tests plus trigram Bloom.
- `bytes_exact_pair_tri_SIZEB_sSEED`: byte and exact pair tests plus trigram Bloom. Same storage as the preceding combination because exact pairs already include the ranking mask.
- `bytes_pair_bloom_SIZEB_sSEED`: byte mask plus 64/128-byte pair Bloom, separately replacing exact pair storage.
- `last_byte`: 32-byte exact bitmap of last bytes of nonempty terms. Exactly matches the oracle for these one-byte suffix queries. Substring queries retain every block.
- `bytes_last_byte`: whole-byte and last-byte bitmaps.

Adding the last-byte bitmap to any other configuration costs 32 bytes/block and makes both suffix results equal to `last_byte`. Its exact suffix candidates are subsets of the other tested necessary conditions. This optional composition does not change substring results.

## Stable hashing

Three alternative fixed seeds, not three jointly stored filters:

| ID | Seed |
|---|---|
| s0 | `0x243f6a8885a308d3` |
| s1 | `0x13198a2e03707344` |
| s2 | `0xa4093822299f31d0` |

The gram is its unsigned big-endian byte integer. Domain separation includes gram length, so leading-zero pair/triple values remain distinct. Define `key = ((gram_length << 32) | gram_integer) XOR seed`. All arithmetic below wraps modulo 2^64:

```text
mix(x):
    x = (x XOR (x >> 30)) * 0xbf58476d1ce4e5b9
    x = (x XOR (x >> 27)) * 0x94d049bb133111eb
    return x XOR (x >> 31)
h1 = mix(key)
h2 = mix(key + 0x9e3779b97f4a7c15) OR 1
position(i) = (h1 + i*h2) AND (number_of_bits - 1)
```

`mix` is the SplitMix64 bijective finalizer. Fixed-width gram encoding plus length domain has no accidental pre-hash collisions. Reducing to the finite bit array deliberately allows Bloom collisions. The odd second step avoids repeated positions within the power-of-two bit array for the chosen k. Rust's standard hash only backs temporary exact sets. Set iteration order has no effect on the final bits or output, and it does not define Bloom hashes.

For each size and gram length, `k = clamp(round(bits / mean_distinct_grams * ln(2)), 1, 16)`. Mean distinct trigrams = 3229.8991, giving k = 1, 1, 2, 4 for 256, 512, 1024, 2048 bytes. Mean distinct pairs = 1109.4887, giving k = 1 for both 64 and 128 bytes. k is global for that configuration. The formula only selects k. Reported decisions come from constructed filters. The seeds expose variation for this small query set and are not confidence intervals or an independence model.

## Metadata and byte accounting

Sizes are incremental, uncompressed layout estimates rather than Rust allocation sizes. Fixed-width arrays use block order and need no per-block offset or header. Global format parameters, allocator overhead, alignment, compression, and existing index storage are excluded.

Exact pair storage matches the evaluator's adaptive model, excluding its unused min/max lengths: 32-byte alphabet mask, `min(2*distinct_pairs, ceil(alphabet_size^2/8))` payload, one-byte format tag, unsigned LEB128 pair count, and `(blocks+1)*8` offset bytes. `exact_pair` includes the ranking mask even when it does not apply byte tests. The total is 3,392,806 bytes. Bloom metadata is its stated fixed bytes/block, added to the selected base layout.

The implemented last-byte bitmap is always 32 bytes/block. An alphabet-ranked last-byte mask could instead use `ceil(alphabet_size/8)` payload bytes/block: 82,671 bytes total. This is an **estimate only**. It assumes ranks from the whole-byte mask, not from the suffix set. A standalone variable-length layout with rank masks and its own offsets costs 754,439 bytes, more than the fixed 537,408-byte bitmap. When an exact-pair record already supplies both the alphabet and shared variable-record offsets, the modeled incremental payload could be 82,671 bytes. This excludes any additional format framing.

Retained data bytes sum the original compressed block ranges. There is no hole coalescing. Logical block/byte skipping describes neither measured physical I/O nor latency. The three seeds are alternatives, so their storage is never summed together.

## Validation

- Direct raw-term matching agrees with all exported query matching-term counts.
- Block identities, term offsets, byte totals, and exact-summary cardinalities are checked against the exports.
- Every inserted pair/trigram is queried back from every constructed filter.
- Every configuration is checked for false negatives against every block's exact matching terms.
- Last-byte decisions equal the direct suffix oracle.
- Unit tests cover all sizes/seeds, k=1/2/4/8/16, membership, empty keys, NUL/255, newline, case-sensitive suffixes, appended case-sensitive substrings, query-order suffix positions, domain separation, and a stable mixer test vector.
- Two complete seven-query runs produced byte-identical `filters.json` and `filters_blocks.jsonl`, checked with byte comparison.

## Reports and budget follow-ups

After extraction, evaluation, and filters complete, run from the repository root:

```sh
python3 dictionary_analysis/budget_masks.py
python3 dictionary_analysis/budget_pairs.py
python3 dictionary_analysis/report.py
```

Each script accepts an optional output/input directory, defaulting to the split's `message_analysis` directory. `budget_pairs.py` also requires `zstd`. The scripts take query order from the generated evaluation/filter metadata. Their tables include appended `*player*` and `*ms*` results. Pair-group comparisons show both new substrings alongside `*ab*` and `*traa*`. Only `*X` and `*x` use ending-byte decisions.

`budget_masks.py` serializes and roundtrip-checks exact whole/ending masks, leaving an existing byte-identical sidecar untouched. `budget_pairs.py` serializes exact compressed pair layouts and grouped unions, checks all roundtrips, and verifies every query against the seven-query term oracle. Reports describe retained original compressed block bytes and incremental metadata, with no measured-I/O or latency claim.
