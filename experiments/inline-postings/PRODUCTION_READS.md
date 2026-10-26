# Query read plans with production short postings

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, message field 3. Absolute document IDs and the unchanged production encoding. Decimal MB throughout.

## Current cutoff ≤8 versus the original code

The original code coalesced gaps up to 4 MiB. The current code uses 512 KiB. This comparison includes both changes. The original 4 MiB setting is a historical baseline, not an active sweep candidate.

“Cold blocks” includes dictionary-block reads plus external postings, assuming the dictionary index is already open. “Postings” is the additional data when dictionary blocks are fully cached and postings are uncached. External positions are unnecessary for these automaton queries. Inline positions are part of the dictionary bytes.

| Query | Original postings MB / requests | ≤8 postings MB / requests | Original cold-block total MB | ≤8 cold-block total MB |
|---|---:|---:|---:|---:|
| `*a*` | 247.862 / 2 | 79.121 / 33 | 299.345 | 153.425 |
| `*ab*` | 247.835 / 2 | 1.948 / 30 | 299.317 | 76.251 |
| `*aba*` | 237.355 / 4 | 0.657 / 5 | 288.837 | 74.961 |
| `*traaa*` | 0.000010 / 2 | 0.000 / 0 | 51.482 | 74.304 |
| `traaa*` | 0.000 / 0 | 0.000 / 0 | 0.002940 | 0.004161 |
| `*player*` | 17.346 / 6 | 4.699 / 11 | 68.828 | 79.003 |
| `*ms*` | 247.101 / 2 | 1.557 / 16 | 298.583 | 75.860 |

All six leading-wildcard queries select the entire dictionary's data blocks in this measurement: **51,482,326 bytes originally**, rising to **74,303,555 bytes at ≤8**, each requested in one read. This is why `*traaa*` and `*player*` fetch more data with cold dictionary blocks despite smaller external postings reads. The unmatched prefix query `traaa*` still fetches one candidate dictionary block: 2,940 bytes originally and 4,161 bytes at ≤8.

## Cutoff comparison at the same 512 KiB coalescing gap

These tables use 512 KiB for both the original dictionary and all cutoffs. They isolate the encoding/compaction effect from the change in coalescing policy. A subsequent limit-0 rebuild reproduced the original dictionary bytes apart from its type marker (see `PRODUCTION_CUTOFFS.md`), so its block read plans are identical to the original baseline. The reproduction command now includes that additional cutoff.

### Dictionary-block + external postings MB

| Query | Original | ≤1 | ≤2 | ≤3 | ≤4 | ≤5 | ≤6 | ≤7 | ≤8 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `*a*` | 246.726 | 206.694 | 175.232 | 161.489 | 156.120 | 152.954 | 155.375 | 155.534 | 153.425 |
| `*ab*` | 214.028 | 76.781 | 75.250 | 74.745 | 74.793 | 75.078 | 75.492 | 76.061 | 76.251 |
| `*aba*` | 145.121 | 71.895 | 71.904 | 72.059 | 72.279 | 72.593 | 73.033 | 73.645 | 74.961 |
| `*traaa*` | 51.482 | 71.592 | 71.752 | 71.907 | 72.127 | 72.441 | 72.889 | 73.502 | 74.304 |
| `traaa*` | 0.002940 | 0.004158 | 0.004161 | 0.004161 | 0.004161 | 0.004161 | 0.004161 | 0.004161 | 0.004161 |
| `*player*` | 56.366 | 76.293 | 76.452 | 76.607 | 76.827 | 77.140 | 77.588 | 78.201 | 79.003 |
| `*ms*` | 192.122 | 73.662 | 73.323 | 73.471 | 73.686 | 73.999 | 74.446 | 75.058 | 75.860 |

### External postings MB / requests

These are incremental reads with fully cached dictionary blocks and uncached postings. Bytes include the holes between coalesced lists.

| Query | Original | ≤1 | ≤2 | ≤3 | ≤4 | ≤5 | ≤6 | ≤7 | ≤8 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `*a*` | 195.244 / 48 | 135.102 / 83 | 103.480 / 101 | 89.582 / 94 | 83.993 / 73 | 80.512 / 52 | 82.485 / 34 | 82.032 / 30 | 79.121 / 33 |
| `*ab*` | 162.545 / 76 | 5.189 / 89 | 3.498 / 68 | 2.838 / 56 | 2.665 / 44 | 2.636 / 39 | 2.603 / 36 | 2.559 / 34 | 1.948 / 30 |
| `*aba*` | 93.638 / 140 | 0.303 / 9 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 | 0.151 / 7 | 0.144 / 7 | 0.144 / 6 | 0.657 / 5 |
| `*traaa*` | 0.000010 / 2 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 |
| `traaa*` | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 | 0.000 / 0 |
| `*player*` | 4.883 / 11 | 4.701 / 11 | 4.700 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 |
| `*ms*` | 140.639 / 101 | 2.070 / 28 | 1.571 / 18 | 1.563 / 18 | 1.559 / 18 | 1.558 / 18 | 1.557 / 18 | 1.557 / 16 | 1.557 / 16 |

Among cutoffs 1–8, the smallest cold-block byte counts occur at ≤5 for `*a*`, ≤3 for `*ab*`, ≤1 for `*aba*`, and ≤2 for `*ms*`. Storage size and query read volume therefore favor different cutoffs. This does not establish latency-optimal settings.

The `*aba*` external bytes increase at ≤8 even as matching external lists decrease. Compaction brings ranges within 512 KiB of each other, so coalescing fetches a larger intervening hole. Similar threshold crossings make the `*a*` sweep nonmonotonic.

## Measurement and validation

- Dictionary ranges are recorded from the actual `into_stream_async_merging_holes` path using the seven regex automata corresponding to these wildcard patterns.
- The dictionary index/footer is opened before tracking asynchronous block requests. The subsequent synchronous enumeration reads are assumed cache hits, matching production warmup's cache contract.
- Postings requests model production's inclusive gap coalescing over matching terms' remapped ranges. Empty inline ranges generate no external request.
- All matching-list counts, inline counts, useful postings bytes, coalesced postings bytes, and request counts match the historical prototype plans for overlapping cutoffs at 512 KiB. The external layout is identical despite the new dictionary encoding.
- All previously measured storage sizes and dictionary breakdowns remain unchanged. Every rewritten term is still verified for every cutoff.
- Tests cover gap boundaries, empty ranges, and inline-only matches that fetch dictionary bytes but no external postings. Example Clippy passes with warnings denied.

Reproduce the full cutoff sweep and both historical baselines:

```sh
cargo run --release --example short_postings_size --features quickwit -- \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.term \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.idx \
  /path/to/4fea6c896fd045aba3cece0808f3e41e.pos reads
```

Raw bytes and counts: [`production-read-plans.json`](production-read-plans.json). Storage-only sweep: [`PRODUCTION_CUTOFFS.md`](PRODUCTION_CUTOFFS.md).

Scope: these are planned inverted-index reads. They exclude external positions, index-opening reads, other fields, hit/document retrieval, cache reuse between queries, retries, storage-specific read alignment/request-size limits, and live network latency. Each cold-block query is independent. No complete split is rewritten or deployed.
