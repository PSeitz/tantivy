# Production encoding: maximum list length 0–8

Recorded before dictionary-level inline-size configuration. These results retain the type-3, fixed-four-bit layout. The current example writes type 4, stores one maximum byte per dictionary, and derives the packing width from that maximum. Historical measurements are preserved below.

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, message field 3, 7,671,280 terms. This sweep uses **absolute document IDs** and the current production layout: packed four-bit counts, separate zstd-compressed byte lanes, and singleton positions embedded in tagged frequencies. Only the maximum number of documents per embedded list changes. The encoding, compression settings, and key-block boundaries stay fixed. Whole longer lists remain external.

An isolated experiment extends the cutoff sweep through **32** with six-bit count/position entries: [`SIX_BIT_COUNTS.md`](SIX_BIT_COUNTS.md). The table below retains the production four-bit layout for 0–8.

## Size results

Decimal MB (1 MB = 1,000,000 bytes). Postings and positions columns count external storage only. Embedded documents, frequencies, and positions are included in the dictionary column. Total is the sum of all three components.

| Maximum list length | Embedded lists | Dictionary MB | External postings MB | External positions MB | Total MB | Total reduction |
|---|---:|---:|---:|---:|---:|---:|
| Original, all external | 0 | 51.678 | 253.765 | 211.057 | 516.500 | — |
| 0, rebuilt with current writer | 0 | 51.678 | 253.765 | 211.057 | 516.500 | 0.00% |
| ≤1 | 6,445,408 | 71.788 | 222.905 | 198.140 | 492.833 | 4.58% |
| ≤2 | 6,466,337 | 71.948 | 222.712 | 198.075 | 492.735 | 4.60% |
| ≤3 | 6,481,871 | 72.104 | 222.503 | 198.011 | 492.617 | 4.62% |
| ≤4 | 6,498,883 | 72.324 | 222.205 | 197.924 | 492.454 | 4.66% |
| ≤5 | 6,518,262 | 72.638 | 221.791 | 197.807 | 492.236 | 4.70% |
| ≤6 | 6,540,812 | 73.087 | 221.220 | 197.648 | 491.955 | 4.75% |
| ≤7 | 6,567,145 | 73.699 | 220.454 | 197.435 | 491.588 | 4.82% |
| ≤8 | 6,597,536 | 74.501 | 219.453 | 197.160 | 491.115 | 4.91% |

- Embedding only one-document lists saves **23,666,835 bytes**, or **93.23%** of the savings at ≤8.
- Raising the maximum from 1 to 8 embeds another **152,128 lists**. The dictionary grows **2,712,842 bytes**, while external postings and positions shrink **4,431,484 bytes**. The additional net saving is **1,718,642 bytes**.
- Raising 7 to 8 saves another **473,500 bytes** overall.
- **≤8 has the smallest combined footprint among the tested cutoffs.** This size result does not select a latency-optimal cutoff. Production remains at ≤8.

Query read volume, including dictionary blocks and external postings at each cutoff, is measured separately in [`PRODUCTION_READS.md`](PRODUCTION_READS.md). Cold-block query bytes favor different cutoffs from total storage size.

## Method and validation

The production dictionary writer encodes each cutoff separately. Every resulting dictionary is reopened, and all **7,671,280 terms per cutoff** are checked against the original source: keys, document counts, all embedded document IDs/frequencies/position deltas, and remapped external ranges. Byte accounting reconciles every dictionary byte.

The **limit-0 dictionary was rebuilt with the current writer**: **51,677,994 bytes**, exactly the original size. All bytes preceding the final four-byte dictionary-type marker are identical to the original (`dictionary_matches_source_except_type = true`). The marker changes from type 2 to type 3. External postings/positions and the full encoded-byte breakdown are unchanged. Repeating cutoffs 1–8 reproduced all earlier sizes and counts.

Source footers and CRCs are checked for `.term`, `.idx`, and `.pos`. The ≤8 sizes, stream breakdown, and counts match the previous production measurement exactly. Embedded-list counts and displaced postings bytes also match the independent historical prototype measurements for its overlapping document-count cutoffs (1, 2, 3, 4, 7, 8). Those historical dictionaries use a different encoding and their sizes are separate results.

Raw bytes, per-lane breakdowns, and entry counts are in [`production-cutoffs.json`](production-cutoffs.json). [`PRODUCTION_SIZE.md`](PRODUCTION_SIZE.md) contains source provenance and the earlier encoding comparisons.

Reproduce:

```sh
cargo run --release --example short_postings_size --features quickwit -- \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.term \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.idx \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.pos sweep
```

The optional `sweep` mode measures cutoffs 0–8 with absolute IDs and skips the separate delta/outer-compression experiments. Omitting it retains the existing ≤8 measurement and encoding comparisons.

Dictionary sizes are measured from encoded output. External sizes model byte-preserving compaction. No complete rewritten split is emitted. Counts exclude the unchanged eight-byte token count, composite-file metadata, component footers, and other fields. Live-query latency, cache behavior, and indexing/decoding throughput across these cutoffs remain unmeasured.
