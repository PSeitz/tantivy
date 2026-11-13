# Document clustering benchmarks

Current measurements supporting [the design](DOC_CLUSTERING.md). Superseded
schema, admission-policy and storage-backend experiments are omitted.

## RAM comparison: clustering, sorting and arrival order

The comparison uses the same 100,000 CloudPrem-mapped org2 records,
`RamDirectory`, Zstd level 8 and 1 MB final-store blocks in all three paths:

- **DocClustering:** stage compact documents, then index in cluster order.
- **Sort/remap:** index in arrival order while collecting a status/service grouping
  tree of document IDs, then apply its permutation during finalization. Include
  grouping, mapping construction, index remapping and the temporary-store rewrite.
- **Arrival order:** index immediately with neither clustering nor sorting/remapping.

Clustering and sorting traverse largest subtrees first and preserve arrival order within leaves.
These measurements used a benchmark-only first-arrival tie-break for equal-sized
groups. The reusable runner now uses that rule in shared grouping code.
Message tokens are off. Input adaptation and missing stored tags/runtime metadata
are as described below; no tags proxy is added.

Medians of three fresh processes, decimal MB:

| Path | Collection allocation peak | Finalization allocation peak | Max RSS | Collect / finalize seconds | Total seconds | Final output |
|---|---:|---:|---:|---:|---:|---:|
| DocClustering in RAM | 212.83 | 350.63 | 556.34 | 4.40 / 5.51 | 9.90 | 66.20 |
| Sorting/remapping in RAM | 562.85 | 620.57 | 530.55 | 8.00 / 1.95 | 9.94 | 66.20 |
| Arrival order, no sorting | 293.86 | 353.45 | 386.15 | 8.02 / 0.89 | 8.90 | 80.77 |

Against sorting/remapping, DocClustering reduces the **requested Rust allocation
peak by 43.5%**, with slightly higher RSS and essentially unchanged total time.
Against unsorted indexing, its allocation peak is similar, its RSS is higher,
and its final output is **18.0% smaller**. Unsorted indexing has the lowest RSS
and observed total time. Its runs were added in a later batch, so timing comparisons
remain exploratory.

Requested allocations include unused capacity. RSS reflects resident pages and
includes native codec and allocator effects; these tests establish no RSS
improvement from DocClustering.

The sorting path holds about **290 MB of temporary RAM-directory buffers** at
collection end, including capacity. Those buffers remain live through inverted
serialization, where its peak occurs. Its temporary store uses uncompressed
16,000-byte blocks. Finalization rewrites it in the target order, compresses it,
and garbage-collects the temporary file. Compact staging retains 177.43 MB instead;
the baseline's ID-only grouping tree uses about 0.985 MB.

Clustering and sorting produce exactly the same ordinal permutation. The unsorted
control preserves arrival order. Stored data and full logical-index fingerprints
match across all three paths. Across a 1k smoke pair and nine 100k runs, 902,000
stored-document/ordinal checks passed. Full source and index validation covered
302,000 documents; repeat timing runs use lighter checks.

Harness, raw traces, provenance, tie-break patch and reproduction:
`/tmp/doc-clustering-ram-sort/README.md` and `RESULTS.md`.

## Timestamp sorting and small-group pooling

### Feature review — removed

Optional within-group sorting and small-group pooling have been removed. The saved
100k comparisons provided little justification for keeping them solely for compression:

- Org 2 timestamp sorting reduces total index size by **484,236 B (0.731%)**.
  Adding small-group pooling increases it by **2,335 B**. The selected timestamp
  codec produces the same column size in all three layouts.
- Yahoo-sim timestamp sorting reduces its timestamp column by **58.81%**, but
  increases the full index by **2,285,408 B (11.515%)**. This input has one group,
  so it does not exercise pooling.
- Forced BlockwiseLinear saves 94,504 B even in baseline order. Sorting adds
  16,883 B of savings, and pooling adds only 24 B. These are encoding diagnostics,
  separate from the actual index-size results.

Recommendation implemented: clustering keeps arrival order inside groups. Removing
sorting and pooling simplifies the implementation. This is a complexity trade-off,
not a measured speedup. Incremental CPU/RSS and query benefits remain unmeasured,
lifecycle timings are single observations, and the current source/service/pod
grouping has no matched sorting/pooling comparison.

The results below preserve historical evidence from before removal. Reproduction
uses the saved source snapshots and artifacts under `/tmp`, which remain unchanged.

### Org 2 results

The same 100,000 mapped records and RAM/store configuration above, using the
then-current reusable `examples/index_file/runner.rs` with the actual mapper and
tokenizers. One fresh process per layout. Sort ascending within emitted groups; threshold 6
pools smaller service groups separately under each retained status parent.

| Clustered layout | Emitted groups | Timestamp column bytes | Selected codec | Total index bytes |
|---|---:|---:|---|---:|
| Arrival order within groups | 1,486 | 312,526 | Bitpacked | 66,202,734 |
| Timestamp sort | 1,486 | 312,526 | Bitpacked | 65,718,498 |
| Timestamp sort + threshold 6 | 748 | 312,526 | Bitpacked | 65,720,833 |

The timestamp column remains **312,526 bytes** because all three layouts select
Bitpacked. Its count, min/max, GCD and packed width are invariant under permutation.
Pooling combines 741 small groups containing 1,573 documents into three remainders.
The full index is 0.728% smaller than baseline; pooling adds 2,335 bytes compared
with timestamp sorting alone on this input.

### Blockwise Linear is smaller, but its estimate rejects it

Re-encoding the exact persisted timestamp values with forced BlockwiseLinear gives
the sizes below. Each encoding round-trips every value. Full-column sizes include
the unchanged index/footer and codec header. These are diagnostic encodings;
the saved indexes still use the automatically selected Bitpacked codec.

| Layout | Current BlockwiseLinear size estimate | Actual forced size | Estimate after normalization |
|---|---:|---:|---:|
| Arrival order within groups | 449,625 | 218,022 | 218,022 |
| Timestamp sort | 369,786 | 201,139 | 201,139 |
| Timestamp sort + threshold 6 | 369,250 | 201,115 | 201,115 |

For sorting plus pooling, BlockwiseLinear would save **111,411 bytes (35.65%)**
versus the selected timestamp encoding. Sorting alone saves 16,883 bytes versus
forced BlockwiseLinear in arrival-within-group order. Pooling saves a further
24 bytes.

The estimator trains on raw mapped nanoseconds and subtracts an approximate GCD
saving. Serialization first subtracts the minimum and divides by GCD, then trains
its lines. This column has millisecond precision and GCD 1,000,000 ns. Estimating
in that same normalized domain, with the original stats-header length, matches
the actual forced sizes exactly here. The raw-domain estimate instead exceeds
Bitpacked's 312,526-byte estimate, so selection rejects BlockwiseLinear.

One contributing limitation is the line fitter's zero-slope fallback for endpoint
differences of at least 2^31, roughly 2.147 seconds in raw nanoseconds. With sorting
and pooling, 124 of 196 raw-domain blocks hit that limit; zero normalized-domain
blocks do. See `columnar/src/column_values/u64_based/{mod,blockwise_linear,line}.rs`.
Production codec code is unchanged.

A fourth run using manual remapping with the same sort/threshold produces the
same order, every fast-field column size, and total index size as compact
clustering. Validation covers 400,000 ordinal/fast-timestamp checks, 300,000 exact
stored-document comparisons, and independent expected orders/group sizes derived
from baseline group boundaries. The baseline reproduces historical order,
stored checksum and output size. Source/config SHA-256 remain unchanged.
These runs add no allocation/RSS measurements or new exhaustive postings checks.

Saved indexes, reports for all fast-field columns, codec diagnostics, source
snapshot and reproduction: `/tmp/doc-clustering-timestamp-sort/README.md`,
`RESULTS.md` and `summary.json`.

### Dividing timestamps before encoding

An additional encoding-only test reads the same saved timestamp columns, divides
signed nanoseconds before the u64 mapping, and runs ordinary codec selection.
Dividing by 1,000 gives microseconds; dividing by 1,000,000 gives milliseconds.
All values divide exactly, preserving the existing fast-field precision.

| Layout | Original nanoseconds | /1,000: microseconds | /1,000,000: milliseconds |
|---|---:|---:|---:|
| Arrival order within groups | 312,526 B | 218,021 B | 218,020 B |
| Timestamp sort | 312,526 B | 201,138 B | 201,137 B |
| Timestamp sort + threshold 6 | 312,526 B | 201,114 B | 201,113 B |

Original values select Bitpacked. **Both divisions select BlockwiseLinear in all
three layouts.** For sorted/pooled data, the BlockwiseLinear estimate falls from
369,250 B to 221,570 B with /1,000, or 202,290 B with /1,000,000. Blocks hitting the
slope limit fall from 124/196 to 2/196 or 0/196, respectively.

Within either codec, actual encoding sizes decrease by only one/two bytes because
serialization already removes the GCD. The GCD header becomes shorter. The large
saving comes from changing which codec the estimator selects.

All nine cases round-trip both codecs and reconstruct the original nanoseconds
exactly (1,800,000 value comparisons). Saved index hashes remain unchanged. This
test changes neither the DateTime format nor its readers. Details and raw results:
`/tmp/doc-clustering-timestamp-sort/units/RESULTS.md` and `results.json`.

### Remaining residual cost and alternative encodings

Selecting BlockwiseLinear improves on Bitpacked, but its 201,115-byte output still
contains **198,696 bytes of residuals**, versus only 2,419 bytes of metadata.
For the sorted/pooled layout, 116 of 196 fixed-size codec blocks cross group
boundaries. Those blocks account for 139,392 residual bytes (70.15% of residual
payload). Within-group blocks still average 11.68 residual bits/value. Sorting
event times does not make bursts, repeats and long gaps follow a straight line.

There are 14,780 distinct millisecond timestamps among 100,000 documents, and
50,936 adjacent pairs are equal in the sorted/pooled order. Encoding-only size
prototypes preserve that order and precision:

| Encoding, 512 values per block | Column bytes |
|---|---:|
| Existing forced BlockwiseLinear | 201,115 |
| Restartable zigzag delta-varints | 113,210 |
| Zigzag delta-varints + independent Zstd8 frames | **70,748** |

Prototype payloads are actually encoded and round-tripped. Sizes include specified
headers and block-offset directories; these are not implemented Tantivy column
formats. Delta+Zstd saves 64.82% versus BlockwiseLinear, with a different access
cost: fetching a value would decompress a frame. Read/indexing speeds are unmeasured.

Globally sorting the same values gives 57,983 bytes with the existing
BlockwiseLinear codec, but changes status/service cluster order. Separately encoding
each existing group takes 209,976 bytes including its directory, so simply splitting
into standalone group streams does not solve the problem. These controls separate
codec selection from the remaining poor compression of the grouped time sequence.

Actual block attribution, smaller-block size models, prototype details and
validation: `/tmp/doc-clustering-timestamp-sort/fit/RESULTS.md`. Saved indexes and
production codec sources remain unchanged.

## Yahoo-sim split prefix

A second dataset uses the first 100,000 stored documents, in persisted document-ID
order, from a downloaded Published Yahoo-sim `datadog` split. Replay uses its
captured mapper/tokenizers and the same RAM/Zstd8/1 MB store configuration above.
All reindexed stored documents match the original split export. Nonstored inputs,
including typed `tag`, remain absent; this is a matched stored-document replay
comparison rather than reconstruction of the original full index.

This prefix has one status value and no service, producing **one group**. Sorting
within that group is global timestamp sorting. Threshold 6 leaves it unchanged.

| Layout | Timestamp bytes | Actual codec | Total index bytes |
|---|---:|---|---:|
| Original split order | 237,525 | Bitpacked | 19,847,144 |
| Timestamp sort | 97,846 | BlockwiseLinear | 22,132,552 |
| Timestamp sort + threshold 6 | 97,846 | BlockwiseLinear | 22,132,552 |

The timestamp column shrinks **58.81%**, and automatic selection chooses
BlockwiseLinear after sorting. However, the full index **grows 11.52%**: postings
add 1,153,109 B, store 953,952 B, positions 166,617 B, combined fast fields 8,725 B
and terms 3,005 B. Other fast fields offset the timestamp saving.

The sorted timestamp column still contains 96,028 residual bytes, averaging
7.68 bits/value, even though no block crosses a group boundary. It has 60,779
distinct millisecond timestamps and 39,221 equal adjacent pairs after sorting.
For the same order and precision, the delta-varint + independent Zstd8-frame size
prototype takes **48,369 B**, including specified headers and block-offset
directory, versus 97,846 B for BlockwiseLinear. Payloads round-trip exactly.
This prototype changes lookup costs and has no read-speed benchmark. Plain
restartable delta-varints take 101,919 B and do not beat BlockwiseLinear here.

Dividing the sorted inputs by 1,000 or 1,000,000 leaves BlockwiseLinear selected
and saves only one or two header bytes. The remaining compression cost is separate
from codec selection. A smaller-block model using the existing fitter gives
83,139 B at 64 values/block; this is a size model, not an implemented codec.

Three indexing runs pass 300,000 exact logical stored-document comparisons,
ordinal/order checks and fast-timestamp checks. Threshold 1/6 produce identical
orders, component totals and all fast-field sizes. No allocation/RSS measurements
or new exhaustive postings validation were added. Source/export hashes and
production code remain unchanged.

Input provenance: `/tmp/doc-clustering-yahoo-sim/README.md`. Full results, saved
indexes, per-column reports and diagnostics:
`/tmp/doc-clustering-yahoo-sim/benchmark/RESULTS.md` and `summary.json`.

## Sorting by other fields

A 41-run size experiment tests global ascending single-field and composite sorts
on both 100,000-document datasets: 23 CloudPrem and 18 Yahoo configurations,
including controls. Keys were selected from field coverage/cardinality profiles.
Original ordinals break ties; missing/non-scalar keys sort last and arrays use
their first value. These are external ordering experiments, with no production
sorting API or codec changes.

All runs use the same mapped documents, tokenizer managers and RAM/Zstd8/1 MB
store settings as their previous comparisons. Feeding the saved baseline
permutations directly to plain indexing exactly reproduces every baseline
component size and every physical fast-field size. CloudPrem's baseline is the
existing descending-size status/service grouping. Yahoo's baseline is split
order and retains the stored-document replay limitations above.

Selected results (decimal MB; changes relative to each existing baseline):

| Dataset | Ascending sort keys, leftmost first | Index MB | Change |
|---|---|---:|---:|
| CloudPrem | Existing status/service grouping | 66.203 | baseline |
| CloudPrem | source, service, tag.pod_name, message | **63.057** | **−4.75%** |
| CloudPrem | source, service, message | 63.484 | −4.11% |
| CloudPrem | tag.pod_name | 63.896 | −3.48% |
| CloudPrem | service | 64.979 | −1.85% |
| CloudPrem | message | 68.493 | +3.46% |
| CloudPrem | timestamp | 79.976 | +20.81% |
| Yahoo | Original split order | 19.847 | baseline |
| Yahoo | custom.index, custom.source | **18.787** | **−5.34%** |
| Yahoo | custom.source | 18.795 | −5.30% |
| Yahoo | custom.index | 18.854 | −5.00% |
| Yahoo | custom.index, custom.source, timestamp | 19.626 | −1.11% |
| Yahoo | message | 21.080 | +6.21% |
| Yahoo | timestamp | 22.133 | +11.52% |

CloudPrem's best sort in this initial sweep saves 3,145,643 B against existing clustering,
mainly postings (1,581,085 B) and fast fields (1,181,230 B). It is 21.93% smaller
than the reproduced arrival-order control (80,768,519 B). Yahoo's best saves
1,060,031 B, mainly postings (583,068 B), store (253,659 B) and fast fields
(143,675 B). Sorting Yahoo only by `custom.source` gives nearly the same result;
adding `custom.index` saves another 8,230 B.

The winning sorts leave timestamp sizes unchanged at 312,526 B and 237,525 B.
Global timestamp sorting minimizes those columns at 57,983 B and 97,846 B but
substantially worsens total index size. Message sorting helps as a secondary key
on CloudPrem, while the tested message-based Yahoo orders perform worse than
its source-only sort.

All 41 runs pass **4.1 million exact logical stored-document comparisons**, mapped
sort-key/order checks and fast-timestamp checks. Complete ordinal-addressed
semantic fingerprints match for each baseline and its two best sorts, covering
all terms/postings/positions, all fast values and fieldnorms. The original source
hashes remain unchanged. Sorting/mapping memory, query/read performance and
held-out generalization are unmeasured; every run includes the existing
validation ordinal, so close size rankings deserve caution.

Full rankings, component and per-column results, saved indexes, plans and
reproduction: `/tmp/doc-clustering-field-sort/RESULTS.md`, `summary.json` and
`README.md`.

### Follow-up: source, service, pod without message

Sorting CloudPrem by **source → service → tag.pod_name**, preserving original
arrival order within each three-field group, produces **62,755,454 B**. This is
**5.21% smaller** than the existing status/service baseline and **301,637 B
smaller** than adding `message` as the fourth key. It is the best CloudPrem order
among the 24 configurations tested so far.

Most of the additional saving comes from postings (287,130 B). Store saves
17,167 B and terms 9,158 B, offset by small increases in positions and fast
fields. The timestamp column remains 312,526 B.

The follow-up passes another 100,000 exact stored-document comparisons, mapped
sort-key/order checks and fast-timestamp checks. Its complete semantic fingerprint
matches the baseline. Source and initial-sweep artifact hashes remain unchanged.
Full result: `/tmp/doc-clustering-field-sort/source-service-pod.md`.

## CPU profile: org 2 compact clustering

Samply 0.13.1 profiles actual DocClustering on the same 100,000 org 2 documents,
with source/service/tag.pod_name grouping and the existing mapper/tokenizers,
RAM/Zstd8/1 MB store settings. The isolated snapshot matches all 528 Rust files
in the working branch. Release optimization, debug information and frame
pointers are enabled. No production implementation changes.

This uses **descending-size group traversal**, rather than the previous
lexicographic ordering experiment. It emits 6,172 groups and a 63,349,913-byte
index. The two ordering policies should not be conflated.

Three unprofiled runs of the profiling build have median collection **4.462 s**,
finalization **5.440 s**, and combined wall time **9.902 s**. Two all-thread
1 kHz captures give the following disjoint CPU attribution (mean 10.566 CPU s):

| Work | CPU seconds | Share |
|---|---:|---:|
| Deferred indexing of staged documents | 4.336 | 41.0% |
| JSON parsing | 1.566 | 14.8% |
| Document mapping | 1.509 | 14.3% |
| Compact encoding and group admission | 1.358 | 12.9% |
| Final index serialization | 0.833 | 7.9% |
| Store-compression worker | 0.420 | 4.0% |
| Loading borrowed staged views | 0.267 | 2.5% |
| Input/adaptation and other work | 0.277 | 2.6% |

Within deferred indexing, JSON inverted indexing accounts for 2.603 CPU s and
fast-field ingestion 1.154 s. Within staging, value encoding takes 1.026 s and
the conservative reservation's owned-size estimate takes 0.182 s. Group
traversal/drain remainder is only 0.034 s; most time under `DocGroups::drain`
is deferred indexing in its callbacks.

A concrete clustering-specific hot spot is repeated UTF-8 validation in
`StagedDocument::text`, called through `StagedValue::as_value` and
`ObjectIter::next`: **0.626 CPU s, or 5.9% overall**. This cost is already included
in the indexing buckets above. Avoiding repeated validation while preserving
UTF-8/lifetime invariants is an optimization candidate, not an implemented gain.

CPU attribution uses `threadCPUDelta` in microseconds, without multiplying by
sample weights. Coalesced idle samples would otherwise inflate the worker's
contribution. First CPU measurements per thread are excluded. Physical symbols
are used rather than potentially deduplicated inline-frame attribution. CPU
seconds on concurrent threads are not additive wall time.

A status/service control reproduces its historical 66,202,734-byte index and
shows the same main hot spots. Separate unprofiled validation performs 200,000
exact stored-document comparisons, checks both grouping orders, and matches
complete postings/positions/fast-field/fieldnorm semantic fingerprints. No
validation runs inside the profiles. Source/config and historical artifact
hashes remain unchanged. Memory and query/read performance are unmeasured.

Profiles, detailed attribution and reproduction:
`/tmp/doc-clustering-org2-profile/{README.md,RESULTS.md,breakdown.json}`.
View locally with `cd /tmp/doc-clustering-org2-profile && samply load cluster-2.json.gz`.

## Disk reference: immediate indexing versus DocClustering

### Workload and configuration

- Original-order prefixes of the org2 staging export: 100,000 records for primary
  measurements and 10,000 for a smaller-segment check. No filtering or deduplication.
- Actual Pomsky mapper and tokenizers from
  `~/Development/pomsky/config/cloudprem/datadog-logs.yaml`, including fast fields,
  generated concatenates and field presence.
- `MmapDirectory`, Zstd level 8, 1,000,000-byte store blocks and a dedicated
  compression thread, matching Pomsky's defaults for this configuration.
- Group by `status` then `service`, `max_fields_per_path=256`. Optional message tokens
  apply to `message`. A separate stored-only ordinal validates input identity.
- Fixed document counts in a single segment. These tests cover the indexing
  lifecycle, rather than production caller budgets, timers or overlapping splits.

#### Input differences from production

Rename `date` → `timestamp`, `_id` → `id`, and `ddsource` → `source`. Copy eligible
string-valued custom error/title fields using Pomsky's `ExtraFts` rules, affecting
959 records. Preserve original custom data, typed tags, missing messages and
repeated exported IDs. Every record maps successfully.

The export lacks the original stored `tags` list and runtime metadata such as
`tiebreaker`, `ingest_size_in_bytes` and `discovery_timestamp`. These remain absent
in the factual-input runs. Because `tag` itself is not stored, this underrepresents
production stored-tag payload.

A separate sensitivity run adds sorted `key:value` strings for exported tag
values, rendering booleans as `true`/`false`. It preserves indexed/fast typed `tag`.
This approximates the missing stored payload rather than reconstructing historical
tags exactly. All logical index fingerprints remain unchanged by this addition.

### Lifecycle results

Decimal MB. Medians of three fresh processes at 100,000 documents:

| Mode | Compact staging | Collection peak | Finalization peak | Max RSS | Collect / finalize seconds | Index files |
|---|---:|---:|---:|---:|---:|---:|
| Immediate indexing | — | 273.03 | 285.50 | 343.41 | 7.869 / 0.951 | 80.77 |
| Clustered, message tokens off | 177.40 | 212.79 | 337.72 | 527.45 | 4.378 / 5.557 | 66.22 |
| Clustered, message tokens on | 170.56 | 205.75 | 319.80 | 528.37 | 4.630 / 5.639 | 66.19 |

Allocation peaks include the initial writer and requested Rust allocation sizes,
including capacities. Staging excludes the initial writer. RSS includes executable
pages, native Zstd workspace and allocator residency, and is recorded before
validation. Timings include profiling overhead and are exploratory local-machine
measurements.

Clustering reduces collection memory and output size compared with immediate
indexing, and adds finalization memory and work. Message token sharing lowers the
Rust allocation peak here; process max RSS remains similar.

Message tokens reduce staging by **6.84 MB (3.86%)** and finalization allocation
peak by **17.92 MB (5.31%)** in these historical runs. Collection takes 0.252 s
longer and finalization 0.082 s longer in the reported medians. Retain this opt-in
encoding for its demonstrated allocation savings. This is historical evidence,
not a new benchmark of the terminology-only rename. Older artifacts call message
tokens "fragments".

At 10,000 documents, single-run finalization peaks were 54.87 MB for immediate
indexing and 70.62 / 70.07 MB for clustered message tokens off/on.

#### Stored-tags sensitivity

With approximate stored tags, message-tokens-off staging is **220.32 MB** and the
finalization allocation peak is **344.78 MB**. Max RSS is **575.28 MB**,
collection/finalization take **6.658 / 6.313 seconds**, and index files occupy
**72.62 MB** (three runs). The immediate-indexing control peaks at 285.36 MB and
produces 107.64 MB of files (one run).

### Memory attribution

Factual 100k input, message tokens off:

| Component | Allocated MB |
|---|---:|
| Term hash table + term arena/recorders | 71.30 |
| Postings/positions arena | 47.19 |
| Fast-field accumulation | 121.31 |
| Reported store-writer buffers, excluding compressor/queue extras | 2.11 |
| Additional term-sorting references during serialization | 30.03 |

There are 938,510 distinct terms and 15,456,934 fast-field values. Of 17,508
allocated column entries, 17,505 are populated. Posting-list payload is 28.33 MB
before block capacity, links and arena slack.

The current expanded-owned-input estimate reports **2,325.50 MB**, versus the
337.72 MB allocation peak. With approximate stored tags it reports 3,543.52 MB
versus 344.78 MB. These results motivate schema-aware accounting; the estimator
has not been changed by this experiment.

### Serialization and writing

| Phase, message tokens off | Wall seconds | Total live Rust peak MB |
|---|---:|---:|
| Drain/index, including streamed store work | 4.625 | 337.72 |
| Inverted-index serialization | 0.428 | 282.20 |
| Fast-field serialization | 0.476 | 137.77 |
| Store close and metadata completion | 0.021 | — |

The post-drain serialization/close/metadata tail totals **0.933 seconds**. Store
serialization, compression and writes already occur during drain. Peaks include
other structures still alive in each phase and are not additive.

The run writes 66.21 MB through file writers, spending about 0.023 seconds inside
delegated write calls and 0.038 seconds in file termination, including `sync_data`.
Atomic metadata writes and directory sync are tracked separately. These local
API-call durations include OS buffering; they do not establish physical-device
throughput or upload latency.

Approximate stored tags increase compressed store output from 10.89 to 17.28 MB
and drain time to 5.404 seconds. The post-drain tail remains about 0.909 seconds.

### Uncompressed controls

An uncompressed disk docstore containing **all mapped fields**, including
indexed/fast-only inputs and generated fields, occupies **836.33 MB**. Compact
staging is 78.8% / 79.6% smaller with message tokens off/on. This compares representation
size; closing a store is not a matched full-indexing lifecycle.

A store containing only the production stored subset occupies 154.11 MB, or
782.41 MB with approximate stored tags. That subset cannot replace replayable
staging because it omits indexing inputs such as typed `tag`.

### Validation and reproduction

All **19 runs / 1,630,000 stored-document comparisons** passed. Full logical index
fingerprints match across immediate/clustered modes and repetitions: terms,
postings/frequencies/positions, fast-field values and order, fieldnorms and token
totals. Fingerprints are non-cryptographic checksums. Source-prefix SHA-256 and
source checksums are unchanged.

Mapper/tokenizer implementations are unchanged. Two isolated query compatibility
arms reject an unused custom TieBreaker type; the CloudPrem field is ordinary i64.

Harness, configuration snapshot, provenance, raw phase traces and reproduction
instructions: `/tmp/doc-clustering-cloudprem/README.md`. Detailed aggregates:
`/tmp/doc-clustering-cloudprem/RESULTS.md` and `summary.json`.

## Borrowed group-key lookups — 2026-11-06

Version `clustering-borrowed-keys.O9Masb`, worktree HEAD `8a4cfc40f` plus
preserved working edits. Matched org 2 prefix of 100,000 documents, actual
CloudPrem mapper/tokenizers/input adaptation, status/service grouping, RAM output,
Zstd8 with 1,000,000-byte blocks, message tokens off. Three serial fresh processes
per arm and mode, alternating version order and reversing it for repetition 2.

The source snapshots differ only in `src/indexer/doc_clustering.rs`. Baseline is
`/tmp/clustering-before-borrowed-group-keys.rs` (SHA-256 prefix `2d5879d4d254`),
candidate is `131f9c18913f`. Both already use FxHashMap, direct dictionary-ID reuse
and plain additions. **This isolates the borrowed-key implementation change.
FxHashMap was not independently measured.** Manual remapping uses production
`DocIdGroups`, so both measured paths exercise the changed lookup code.

Wall medians in seconds. Total is medianed from per-process lifecycle totals:

| Path | Version | Collection | Finalization | Total |
|---|---|---:|---:|---:|
| Compact clustering | Baseline | 4.2863 | 5.4994 | 9.7857 |
| Compact clustering | Borrowed keys | 4.2722 | 5.5180 | 9.7887 |
| Manual remapping | Baseline | 8.1187 | 1.9456 | 10.0662 |
| Manual remapping | Borrowed keys | 7.9221 | 1.9259 | 9.8446 |

Compact collection changed **−0.33%**, finalization **+0.34%**, total **+0.03%**:
essentially unchanged in this run. Remapping collection changed **−2.42%**,
finalization **−1.02%**, total **−2.20%**. These are exploratory three-run local
medians without confidence intervals. Scheduling, caches and thermal effects can
contribute. Current production sources run without the historical allocator
instrumentation, so earlier RAM timings are a different timing baseline.

All twelve runs report **66,202,734 bytes** of retained RAM-directory output.
Every complete ordinal-order file and stored checksum matches. Four full 100k
validations (one per version/path) passed exact stored-source comparisons,
independent order recomputation, and matching term/posting/position/fast-field/
fieldnorm/token-total fingerprints. Four full 1k smoke cases also passed.
Fingerprints and exact order also match the historical RAM experiment. Validation
runs after timing. Input/config/source/binary hashes remained unchanged.

Mapper/schema/writer setup is outside timing. Missing original stored tags and
runtime metadata remain absent. This measures RAM indexing, with no durable-disk,
upload, query-performance or production flush-budget claim. Mapper APIs required
no compatibility changes. Repository production source and prior evidence are
preserved.

Reproduction, settings, immutable binaries, raw timings/sizes, source hashes and
validation evidence: `/tmp/clustering-borrowed-keys.O9Masb/README.md`, `RESULTS.md`,
`results.json`, `results-matrix/`, `manifest.json`, and `validation.json`. The
artifact README documents a discarded cache-freshness build attempt and a
runner-only smoke assertion correction. Both are excluded from these results.

### Repeat with unchanged binaries

A second matrix uses the same preserved baseline/candidate binaries and settings,
with three new serial fresh processes per version/path. "New-group copies" means
borrowed lookup with a string copy only when creating a group. Both versions use
FxHashMap. Wall-time medians in seconds:

| Path | Key copying | Collection | Finalization | Total |
|---|---|---:|---:|---:|
| Compact clustering | Every admission | 4.2051 | 5.4033 | 9.6084 |
| Compact clustering | New-group copies | 4.2086 | 5.4280 | 9.6367 |
| Manual remapping | Every admission | 7.7424 | 1.9033 | 9.6437 |
| Manual remapping | New-group copies | 7.8349 | 1.9214 | 9.7451 |

Candidate total changed **+0.29%** for clustering and **+1.05%** for remapping.
The previous remapping improvement did not repeat. These measurements establish
no repeatable end-to-end speedup from removing the per-admission key copies.
Phase medians and lifecycle-total medians are computed independently.

All twelve 100k runs and four full 1k smoke cases passed the same order, stored
checksum and first-repetition full semantic/source validation checks. All outputs
remain **66,202,734 bytes**. Binary hashes match the previous matrix, and the
current grouping source matches the benchmark candidate snapshot. No rebuild was
performed. New artifacts: `/tmp/clustering-keys-rerun.jGnKZe/README.md`,
`results.json`, `runs.jsonl`, `results-matrix/` and `run.log`. Earlier artifacts
remain unchanged.

## Current implementation and repeated UTF-8 checks — 2026-11-10

Tantivy `07de873945af80c49c1b3d8b3f57079b6b509ee1`, original Pomsky mapper and
both tokenizer managers from `8ea72225e9e236670c41e504d8343d52a838919c`. The same
100,000-record org 2 prefix, factual input adaptations, status/service grouping,
RAM directory, Zstd8, 1,000,000-byte store blocks and dedicated compression thread.
Writer construction budget is 64,000,000 bytes. Message-token encoding and the
stored-tags proxy are off. Original message indexing remains enabled.

Three serial fresh processes per mode, alternating comparison order. Wall-time
medians in seconds, with setup and validation excluded:

| Mode | Collection | Finalization | Total |
|---|---:|---:|---:|
| Current clustering | 4.416624 | 4.923031 | **9.339655** |
| Manual remapping, production `DocIdGroups` | 8.067740 | 1.986679 | **10.044102** |
| Clustering with repeated UTF-8 checks added | 4.247967 | 5.348116 | **9.596083** |

Current clustering's median total is **7.01% lower** than manual remapping in
this run. Both paths include actual parsing/mapping and grouping. Manual indexing
occurs during collection; clustered indexing occurs during finalization. Mapping
construction and disposal are included in manual finalization. Phase and total
medians are computed independently from the per-process measurements.

The UTF-8 control changes exactly one source line: `text()` uses checked
`std::str::from_utf8(...).expect(...)` instead of unchecked conversion. It retains
all current early validation. This measures the cost of adding repeated checks
to the current implementation, rather than reproducing historical source.
Current finalization is **0.425 s (7.95%) lower**, and median total **2.67% lower**,
than this control. Its different collection timings reflect run-to-run variation
with an unchanged collection algorithm.

All nine outputs have identical component sizes totaling **66,202,734 bytes**.
Three full 1k smoke runs passed. The first 100k run per mode passed exact stored
comparisons against freshly mapped source, an independent grouping-order oracle,
and matching ordinal-addressed semantic fingerprints covering terms, postings,
frequencies, positions, fast values, fieldnorms and token totals. Repeats checked
every stored ordinal, completeness/uniqueness, stored/order checksums and component
sizes. Fingerprints are non-cryptographic; serialized indexes are not claimed
byte-identical.

The old `/tmp` benchmark binaries and some harness files were unavailable. The
harness was recovered from session records and rebuilt against isolated source
snapshots. Missing fingerprint-validator logic was reconstructed and documented.
Input-prefix SHA-256 was freshly verified as
`b218130c2db5fd927cc07378bf86c2b67bfcfa90e7655590759f09cca3c5873d`
(774,713,632 bytes including newlines). Original configuration and tokenizer hashes
also match historical provenance. The new build has its own dependency lockfile;
these timings are a matched current-code comparison, not an unchanged-binary
rerun of the historical experiments.

Host: Apple M4 Max, 64 GiB, macOS 26.7.2, Rust 1.98.0. Release builds with
`RUSTFLAGS='--cfg tokio_unstable'`, no allocator instrumentation or profiler.
OS caches were warm. CPU placement, background load and temperature were
uncontrolled. Three repetitions establish local observations, not statistical
significance. This run adds no allocation/RSS measurements.

Artifacts: `/tmp/clustering-rerun.mHkBB6/REPORT.md`, `runs.json`, `results/*.json`,
`medians.json`, `provenance.json`, `added-check.patch`, and `pomsky-changes.patch`.
The report records raw timings, validation scope, source/binary hashes, dependency
compatibility changes, reconstructed validator logic and build/run commands.
Production code and surviving historical artifacts were preserved.
