# Inline postings: storage, decoding, and external read plans

Input: split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, segment `4fea6c896fd045aba3cece0808f3e41e`, field 3 (`message`), with **7,671,280 terms**. The [dictionary report](dictionary.md) describes the original blocks and keys.

An inline list stores its document IDs, frequencies, and positions inside its dictionary block. The cutoff is the maximum document count for a whole list to qualify. Longer lists remain external, including their short final blocks.

This experimental encoding uses compressed byte lanes and per-list document-ID deltas: the first ID is absolute, subsequent values are gaps. The accumulator resets for each list. Cutoffs 64 and 128 extend the dictionary-encoding experiment beyond the indexing implementation's limit of 32.

## 1. Storage

Decimal MB. The dictionary column includes inline documents, frequencies, and positions. The other columns contain only remaining external data.

Dictionary sizes are actual serialized output. External sizes model byte-preserving removal and compaction of qualifying lists. These are field-level component sizes, excluding unchanged token-count headers, composite metadata, footers around components, and other fields. No complete rewritten split was produced.

Selected cutoffs from the sweep of 0–16, 32, 64, and 128:

| Maximum inline documents | Dictionary MB | External postings MB | External positions MB | Total MB | Total reduction |
|---|---:|---:|---:|---:|---:|
| 0 | 51.678 | 253.765 | 211.057 | 516.500 | 0.00% |
| 1 | 71.786 | 222.905 | 198.140 | 492.831 | 4.58% |
| 8 | 74.477 | 219.453 | 197.160 | 491.090 | 4.92% |
| 16 | 91.810 | 196.914 | 191.079 | 479.803 | 7.11% |
| 32 | 124.717 | 154.032 | 179.441 | 458.190 | 11.29% |
| 64 | 138.647 | 134.993 | 174.284 | 447.924 | 13.28% |
| 128 | 143.411 | 128.280 | 172.472 | 444.163 | 14.01% |

From cutoff 32 to 64, the dictionary grows by 13,929,581 bytes, external postings shrink by 19,038,513 bytes, and positions shrink by 5,156,828 bytes. The net saving is 10,265,760 bytes. From 64 to 128, the net saving is 3,761,280 bytes.

## 2. Absolute document IDs versus deltas

Only the dictionary changes between these encodings. External postings and positions are identical at every cutoff.

| Cutoff | Absolute dictionary MB | Delta dictionary MB | Bytes saved |
|---|---:|---:|---:|
| 8 | 74.501 | 74.477 | 24,448 |
| 16 | 93.114 | 91.810 | 1,304,049 |
| 32 | 129.426 | 124.717 | 4,708,905 |
| 64 | 145.202 | 138.647 | 6,555,395 |
| 128 | 150.681 | 143.411 | 7,270,577 |

At cutoffs 1–6, delta dictionaries are 37–8,114 bytes larger. At cutoff 1 the document-ID lanes are identical. Its 37-byte increase comes from compressed metadata/index changes for the codec version. Reconstructing IDs adds a checked prefix sum per list.

### Full dictionary scans in RAM

Apple M4 Max, 64 GiB, Rust 1.98 release build. Each scan visits all 7,671,280 terms. Both encodings use the same binary, two warmup rounds, and 12 measured repetitions in alternating order. Exact keys and complete term values were compared before timing.

| Cutoff | Absolute median ms [range] | Delta median ms [range] | Median change |
|---|---:|---:|---:|
| 8 | 758.33 [755.14–778.73] | 767.15 [756.87–781.74] | +1.16% |
| 32 | 1,034.70 [1,023.34–1,068.91] | 1,053.27 [1,041.26–1,087.68] | +1.79% |

Timing includes decompression, allocation, validation, positions, and a checksum over every inline document ID. It excludes file I/O and dictionary opening. These are full-scan elapsed times on a shared workstation, rather than query timings or an isolated measurement of the prefix-sum operation.

## 3. External postings read plans

The planner starts with the matching external lists after compaction. Every list with document frequency at or below the cutoff is removed before offsets are remapped, including lists that do not match the query.

Two policies are compared per field, segment, and call:

- **Fixed 512 KiB:** merge every gap of at most 512 KiB.
- **Request budget:** start with that fixed-gap plan, then merge the cheapest remaining gaps within an extra-byte allowance of `max(2 MiB, floor(baseline_fetched_bytes / 4))`.

Each additional merge removes one request. Taking gaps in ascending size minimizes requests within the allowance, then minimizes fetched bytes for that request count. The allowance applies independently to each call. Its 2 MiB minimum permits increases greater than 25% for small plans.

These plans count one contiguous range as one request. Planned bytes include holes between useful posting ranges. They exclude dictionary reads, positions, index opening, hit/document retrieval, retries, cache reuse, and storage-specific request limits. No postings requests were executed.

### Per-query results at cutoff 8

Matching includes inline lists. Useful bytes count only matching external postings. Each policy cell is **requests / planned external bytes**.

| Pattern | Matching lists | External lists | Useful bytes | Fixed 512 KiB | Request budget |
|---|---:|---:|---:|---:|---:|
| `*a*` | 1,921,269 | 3,870 | 34,697,432 | 33 / 79,121,102 | 11 / 97,597,707 |
| `*ab*` | 75,774 | 203 | 288,249 | 30 / 1,947,701 | 28 / 3,401,299 |
| `*aba*` | 2,817 | 16 | 143,930 | 5 / 657,307 | 4 / 2,015,570 |
| `*traaa*` | 2 | 0 | 0 | 0 / 0 | 0 / 0 |
| `traaa*` | 0 | 0 | 0 | 0 / 0 | 0 / 0 |
| `*player*` | 21 | 21 | 4,221,761 | 11 / 4,699,006 | 10 / 5,943,196 |
| `*ms*` | 50,585 | 27 | 825,018 | 16 / 1,556,712 | 14 / 2,922,609 |

Patterns use case-sensitive bytes. `*traaa*` has three consecutive `a` bytes, unlike the `*traa*` query in the dictionary-pruning report. The prefix query `traaa*` matches no terms.

For `*a*`, the request budget removes 22 requests while adding 18,476,605 planned bytes. For `*ms*`, it removes two requests while adding 1,365,897 bytes.

### Across cutoffs

Sums over the same seven independent queries, each evaluated once, with no cross-query caching:

| Cutoff | Useful bytes | Fixed requests / planned bytes | Request-budget requests / planned bytes |
|---:|---:|---:|---:|
| 0 | 50,089,825 | 378 / 596,950,546 | 185 / 743,579,954 |
| 8 | 40,176,390 | 95 / 87,981,828 | 67 / 111,880,381 |
| 32 | 39,993,334 | 73 / 59,797,495 | 53 / 77,019,020 |
| 64 | 39,959,502 | 54 / 54,016,768 | 37 / 69,984,821 |
| 128 | 39,950,059 | 47 / 53,577,891 | 32 / 67,019,221 |

The external ranges are identical for the absolute-ID and delta encodings. Cutoffs 64 and 128 model external-list removal without implementing the corresponding full indexing and postings-cursor changes.

### Dictionary reads

Inlining shifts data into the dictionary. In the separate absolute-ID cutoff-8 measurement, a cold leading-wildcard scan planned **74,303,555 bytes** of dictionary-block reads. Adding that cost to `*a*` gives 153,424,657 bytes with fixed merging and 171,901,262 bytes with the request budget.

That dictionary measurement uses the absolute-ID encoding. Cold dictionary read plans for the delta dictionaries above were not measured. The external-postings totals therefore describe only one component of query I/O.

## 4. Validation

- Every generated dictionary was reopened. All 7,671,280 terms were checked at each of the 20 cutoffs, including document IDs, frequencies, position deltas, and remapped external ranges.
- Exactly 128 documents occupy a full compressed source block. All five source lists of that length were independently checked through the full postings reader, including positions.
- Dictionary byte accounting reconciled the serialized size. External sizes and entry counts matched the absolute-ID sweep at all cutoffs. Cutoff 0 differs from the original dictionary by one configuration byte.
- All 35 cutoff/query read plans passed coverage, ordering, request-count, and byte-budget checks. Results matched an independent cheapest-gap calculation. Repeated runs produced identical results.
- End-to-end query latency, cache effects, indexing throughput, and storage request overhead remain unmeasured.
