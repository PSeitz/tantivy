# Exact inverted trigram → block postings

This standalone experiment derives **every observed trigram inside each term remainder**, after removing the full actual physical-block LCP. It reads the four retained key-only SSTables. It preserves source keys, physical boundaries, existing experiment files and production code.

Sources under `OUT/uuid_injection`:

- `baseline`: original physical block boundaries.
- `v4_canonical_10000000`: baseline plus 10m canonical UUIDv4 keys.
- `v7_canonical_10000000`: baseline plus 10m canonical UUIDv7 keys.
- `alphanumeric36_10000000`: baseline plus 10m 36-character `0-9a-z` keys.

The new outputs live exclusively under `OUT/trigram_postings`. The byte and pair fallbacks reuse the verified best layouts and decoded validation masks under `OUT/pair_postings`.

## Reproduce

From the repository root:

```sh
OUT=KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
export CARGO_TARGET_DIR=dictionary_analysis/timestamp_injection/target
cargo test --release --manifest-path dictionary_analysis/trigram_postings/Cargo.toml
python3 -m unittest discover -s dictionary_analysis/trigram_postings -p 'test_*.py'
cargo build --release --manifest-path dictionary_analysis/trigram_postings/Cargo.toml
mkdir -p "$OUT/trigram_postings"
for s in baseline v4_canonical_10000000 v7_canonical_10000000 alphanumeric36_10000000; do
  /usr/bin/time -l "$CARGO_TARGET_DIR/release/trigram-postings" "$OUT" "$s" \
    > "$OUT/trigram_postings/$s.log" 2>&1 || exit
done
python3 dictionary_analysis/trigram_postings/report.py "$OUT" \
  > "$OUT/trigram_postings/evaluation.log" 2>&1
```

`/usr/bin/time -l` is the macOS resource recorder. On Linux use an appropriate `/usr/bin/time` option. The report's optional resource-log parsing expects the macOS log labels. Evaluation needs the `zstd` executable for streaming the previous baseline exact-bitset file.

The binary also supports generic row lookup, independently of the built-in seven-query evaluation:

```sh
"$CARGO_TARGET_DIR/release/trigram-postings" lookup FILE u24_integer [u24_integer ...]
```

The output is one JSON line per requested feature, with its sorted block IDs. Missing vocabulary entries return empty postings. The lookup uses the per-feature directory and decodes only the requested rows.

To inspect the block-frequency background without exporting every block ID:

```sh
"$CARGO_TARGET_DIR/release/trigram-postings" frequencies FILE
python3 dictionary_analysis/trigram_postings/frequency.py "$OUT"
```

The first command decodes all observed posting rows and emits their counts. The second checks source hashes and aggregate counts, saves `frequencies.jsonl` for baseline and alphanumeric augmentation, and writes `trigram_postings/frequency.json`. It reports minima, quantiles and minimum-subtracted query counts for all observed grams, `0-9a-z` grams, and letter-only grams. These are block frequencies, rather than document frequencies. Both dictionaries contain all 46,656 alphanumeric trigrams. Their alphanumeric minima/medians are 889/1,012 and 7,003/7,436 respectively. The global observed minimum is 1 in both cases. Subtracting a common minimum preserves count differences and ordering. It is a diagnostic transformation and changes no posting memberships.

## Layout and tested compression scope

`IP01` extends the prior pair-postings format with feature width 3:

- Four-byte magic, u32 block count, u32 observed-feature count, u8 feature width, u8 block-ID width: 14 B.
- Sorted observed u24 values, three little-endian bytes each. A gram's numeric value is `(a << 16) | (b << 8) | c`.
- One little-endian u32 payload offset per feature, plus a final offset. Overflow is checked before writing each frame.
- Independently decodable posting frames. Each has a one-byte codec/polarity/compression tag and u32 uncompressed payload length, followed by plain payload or a complete zstd level-3 frame.

Payload codecs are the exact prior implementations: fixed-width sorted IDs, delta-varint IDs, maximal runs encoded as previous-end gaps and lengths, and block bitmaps. Fixed layouts use positive postings. Adaptive layouts try all four codecs with positive and complement polarity. Adaptive-zstd tries both plain and compressed versions. The row with the smallest serialized frame wins. Ties preserve the prior deterministic iteration order.

All ten prior layout modes are measured: raw/delta/runs/bitmap, their four zstd variants, adaptive plain, and adaptive plain-or-zstd. This is the full prior search matrix, rather than a resource-limited shortlist. Other codecs, levels, joint summary compression and whole-file compression are outside the tested scope.

`src/codec.rs` reuses the prior codec source verbatim, with module visibility changes only. A Rust test verifies that relationship against the original source. `src/scan.rs` adapts its physical-readback audit. Keeping these pieces local preserves the completed original helper and its formats. The new streaming layout writer and u24 directory are in `src/main.rs`.

## Validation and resource behavior

The scanner checks the compressed SSTable SHA-256, complete length-prefixed decoded-key digest, physical addresses, ordinals, term counts, fence ordering, first/last keys, full LCP, byte masks and all seven independent full-key truth counts against retained source records. Trigrams never cross terms or the excluded prefix boundary.

The scanner holds one compressed SSTable and posting vectors. It decodes one source block at a time, reuses a touched-feature presence table, and uses a feature→row rank array to avoid tree lookups for every membership. It releases the source buffer and build tables before layout encoding. It frees each scenario before the next.

Serialization writes the ten files incrementally, calculating each candidate frame once per feature. It then reopens each file, validates its complete directory, decodes **every row**, compares exact sorted IDs with extraction, verifies end-of-file and size, and hashes the file incrementally. It deletes each nonwinner after validation and retains its measured size, hash and codec distribution in `measurement.json`. At peak there are ten on-disk layout files for one scenario. There is no whole-layout buffer or second full posting map. The compact best file remains independently reproducible from preserved sources.

`report.py` reads the winning file through generic row lookup. It retains only the six gram columns needed by the seven queries. This selection happens **after all-feature construction, compression selection and validation**. The old baseline block-major exact file is decompressed as a stream for column and predicate comparison. No full block×trigram mask matrix is allocated.

Query evaluation uses `prefix_summaries.permits` with `gram_permits` applied only to the current remainder continuation. Prefix-contained and viable crossing matches remain safe, including continuations of length 0, 1 or 2. Trigrams alone leave short queries unconstrained. Trigrams+bytes supplies byte fallback. Trigrams+pairs+bytes also supplies bigram fallback. Each combination charges one prefix sidecar.

Rust tests cover exact codec reuse, all codecs and polarities on empty/full/sparse sets, block-ID width boundaries, high-byte u24 collisions, directory/offset overflow, on-disk all-layout roundtrips, full-prefix exclusion and term boundaries. Python tests cover `player` crossing every prefix position, short-query fallbacks, full-prefix matches, high-byte grams and exhaustive small-dictionary no-false-negative safety.

## Outputs and interpretation

- `report.md` / `report.json`: all costs, vocabulary/memberships, all seven query tables, codec choices, validation and scope.
- Per scenario: `measurement.json`, winning trigram posting file, `prefixes.bin`, and compact `decisions.bin` with its query/mode schema.
- Scenario logs: construction timing and process memory. Test and evaluation logs are separate.

Disk costs include vocabulary, headers, offsets, tags, polarity, compression frames and prefix extensions. Validation decisions, masks, logs, construction memory and test resident memory are separate experiment overhead. All percentages use the **original 195,640 B index as a reference**, also for augmented dictionaries. The measurements support comparisons within the tested layouts and describe candidate pruning. They make no claim about production query latency, rebuilt index size, a global compression optimum or information-theoretic impossibility.
