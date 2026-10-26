# Short postings in the term dictionary

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, 10,002,000 documents, physical `message` field 3. The field contains 7,671,280 terms. Existing external postings occupy 253,765,218 bytes, excluding the eight-byte token count.

MB means 1,000,000 bytes. Document cutoffs count documents per full postings list. Byte cutoffs measure the full encoded list, including document IDs and frequencies. Only lists with fewer than 128 documents are eligible.

## Compressed dictionary and displaced postings

Dictionary sizes include jointly compressed keys and values, block framing, the regenerated block index, and trailers. The writer uses its existing 4,000-byte key-block target and zstd level 3. Value bytes do not participate in that block target, so inline payload enlarges blocks while preserving their term boundaries.

Scan time is the median of three in-memory full dictionary scans on one machine. It includes decompression, key reconstruction, metadata decoding, and allocation/copying of owned inline payloads. It excludes decoding document IDs/frequencies, scoring, positions, storage reads, and network latency.

| Cutoff | Inline lists | Displaced postings (MB) | Dictionary (MB) | Dictionary growth (MB) | Full scan (ms) |
|---|---:|---:|---:|---:|---:|
| Existing format | 0 | 0.000 | 51.678 | 0.000 | 168.9 |
| Tagged format, no inline lists | 0 | 0.000 | 51.640 | -0.038 | 183.1 |
| ≤1 document | 6,445,408 | 30.860 | 81.118 | 29.440 | 323.2 |
| ≤2 documents | 6,466,337 | 31.054 | 81.296 | 29.618 | 324.8 |
| ≤4 documents | 6,498,883 | 31.560 | 81.769 | 30.091 | 327.1 |
| ≤7 documents | 6,567,145 | 33.311 | 83.343 | 31.665 | 335.2 |
| ≤8 documents | 6,597,536 | 34.312 | 84.222 | 32.544 | 337.4 |
| ≤5 encoded bytes | 6,445,408 | 30.860 | 81.118 | 29.440 | 330.3 |
| ≤8 encoded bytes | 6,447,746 | 30.879 | 81.135 | 29.457 | 328.7 |
| ≤16 encoded bytes | 6,483,296 | 31.285 | 81.518 | 29.840 | 332.1 |
| ≤24 encoded bytes | 6,521,422 | 32.050 | 82.215 | 30.537 | 329.1 |
| ≤32 encoded bytes | 6,575,412 | 33.574 | 83.579 | 31.901 | 333.9 |

## External postings read plans

Each plan removes inline lists, compacts all remaining postings offsets, and merges consecutive matching external ranges when their gap is ≤4 MiB (4,194,304 bytes). Request bytes include unrelated lists inside those gaps. Individual matching lists and merged requests are separate quantities.

These are offline plans. They exclude dictionary requests, positions, cache behavior, retries, and downstream coalescing. `*a*` matches 1,921,269 lists and `*ab*` matches 75,774 lists at every cutoff.

| Cutoff | `*a*` request bytes (MB) | `*a*` requests | `*ab*` request bytes (MB) | `*ab*` requests |
|---|---:|---:|---:|---:|
| Existing format | 247.862 | 2 | 247.835 | 2 |
| Tagged format, no inline lists | 247.862 | 2 | 247.835 | 2 |
| ≤1 document | 217.619 | 2 | 130.427 | 16 |
| ≤2 documents | 212.973 | 3 | 80.805 | 19 |
| ≤4 documents | 189.725 | 7 | 42.434 | 14 |
| ≤7 documents | 117.382 | 5 | 40.329 | 10 |
| ≤8 documents | 117.070 | 4 | 38.243 | 9 |
| ≤5 encoded bytes | 217.619 | 2 | 130.427 | 16 |
| ≤8 encoded bytes | 217.602 | 2 | 125.432 | 16 |
| ≤16 encoded bytes | 204.969 | 4 | 56.407 | 18 |
| ≤24 encoded bytes | 133.724 | 11 | 40.637 | 12 |
| ≤32 encoded bytes | 117.302 | 4 | 40.261 | 9 |

At ≤7 documents, 1,916,479 of the `*a*` matching lists and 75,507 of the `*ab*` matching lists are inline. Their remaining external lists number 4,790 and 267 respectively.

`*traaa*` matches two single-document lists. The existing plan reads 10 bytes in two requests. Every nonzero tested cutoff makes both lists inline, giving zero external requests. `traaa*` matches no terms and has zero external requests throughout.
