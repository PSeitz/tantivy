# Cutoffs from one to four documents

Same split and physical message field as `COALESCING.md`. The two-document cutoff was already present in the raw measurements. This run adds three documents and surfaces both in the tables. All previously measured dictionary sizes and read plans reproduce exactly.

## Storage

Dictionary sizes use actual jointly compressed SSTables. MB = 1,000,000 bytes. The original dictionary is 51.678 MB.

| Inline cutoff | Inline lists | Postings displaced (MB) | Dictionary (MB) | Dictionary increase over ≤1 (MB) |
|---|---:|---:|---:|---:|
| ≤1 document | 6,445,408 | 30.860 | 81.118 | 0 |
| ≤2 documents | 6,466,337 | 31.054 | 81.296 | 0.178 |
| ≤3 documents | 6,481,871 | 31.262 | 81.495 | 0.378 |
| ≤4 documents | 6,498,883 | 31.560 | 81.769 | 0.651 |

## External reads at a 512 KiB coalescing gap

Each cell is **request MB / request count**. Inline lists are removed and the remaining external postings compacted before coalescing. Request bytes include unrelated postings inside merged gaps. KiB = 1,024 bytes and MiB = 1,048,576 bytes.

| Pattern | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents |
|---|---:|---:|---:|---:|
| `*a*` | 135.102 / 83 | 103.480 / 101 | 89.582 / 94 | 83.993 / 73 |
| `*ab*` | 5.189 / 89 | 3.498 / 68 | 2.838 / 56 | 2.665 / 44 |
| `*aba*` | 0.303 / 9 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 |
| `*player*` | 4.701 / 11 | 4.700 / 11 | 4.699 / 11 | 4.699 / 11 |
| `*ms*` | 2.070 / 28 | 1.571 / 18 | 1.563 / 18 | 1.559 / 18 |

Removing more lists can split previously merged requests. Request counts can therefore rise as the inline cutoff increases, as for `*a*` from one to two documents.

## Coalescing around 512 KiB, at an inline cutoff of four documents

Each cell is **request MB / request count**.

| Pattern | 256 KiB | 512 KiB | 1 MiB |
|---|---:|---:|---:|
| `*a*` | 73.814 / 101 | 83.993 / 73 | 103.126 / 47 |
| `*ab*` | 1.265 / 47 | 2.665 / 44 | 18.422 / 27 |
| `*aba*` | 0.152 / 8 | 0.152 / 8 | 0.716 / 7 |
| `*player*` | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 |
| `*ms*` | 1.133 / 19 | 1.559 / 18 | 3.772 / 15 |

Increasing the gap from 256 to 512 KiB exchanges about 10.18 MB for 28 fewer requests on `*a*`. On `*ab*`, it exchanges about 1.40 MB for three fewer requests. The latency effect depends on request overhead, throughput, concurrency, and caching, which these plans do not model.

The complete sweep for every cutoff is in `coalescing.json`. The two `traaa` patterns remain unchanged: `*traaa*` has both matches inline at every displayed cutoff, and `traaa*` has no matches.

## Validation and scope

All document-cutoff list counts and displaced bytes, including the new three-document cutoff, match the independent histogram. All rewritten entries are checked before timing. Five example tests and Clippy passed. Zero-gap byte accounting and monotonicity of bytes/request counts across gap sizes were checked for every scenario and pattern.

These are offline external-read plans. Dictionary and positions reads, caches, retries, request-size caps, concurrency, and live latency are excluded. The generic byte-preserving inline format remains a prototype. The full repository test suite was not run.
