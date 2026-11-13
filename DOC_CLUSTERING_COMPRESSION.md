# Document clustering compression results

Negative percentages mean less storage. `+` combines fields into one fingerprint; `→` adds a grouping level.

## August 2026: Pomsky fingerprint clustering

### What was tested

| Dataset | Tests |
|---|---|
| 10 staging files: 4.05M documents, 31.58 GB | Field, message-shape and structure grouping; 5 MiB/s or 20 MB/s per shard; one final split. |
| 100 staging files: 40.69M documents, 317.17 GB | Three finalists at 5 MiB/s per shard; three final splits. |

### 10 files: dimensions, message shape and structure

| Configuration | Runs | Compression | Stored GB | Size change vs averaged no-clustering baseline |
|---|---:|---:|---:|---:|
| **source + service → status → full structure** | 1 | **8.683851×** | **3.637** | **−26.13%** |
| source + service → status | 2 | 8.642712× | 3.654 | −25.78% |
| source + service → status → message(100) | 2 | 8.625281× | 3.661 | −25.63% |
| source + service → message(25) | 1 | 8.618291× | 3.664 | −25.57% |
| source + service | 1 | 8.615475× | 3.666 | −25.54% |
| source + service → message(100) | 1 | 8.613565× | 3.666 | −25.53% |
| source + service → status → message(25) | 1 | 8.610474× | 3.668 | −25.50% |
| source → full structure → service | 1 | 8.509650× | 3.711 | −24.62% |
| Full structure | 1 | 7.750533× | 4.075 | −17.24% |
| status → message(100) | 1 | 7.602869× | 4.154 | −15.63% |
| status → message(25) | 1 | 7.578740× | 4.167 | −15.36% |
| message(100) | 1 | 7.493825× | 4.214 | −14.40% |
| message(25) | 1 | 7.462473× | 4.232 | −14.04% |
| status | 1 | 6.668240× | 4.736 | −3.80% |
| Reduced structure | 1 | 6.423311× | 4.917 | −0.13% |
| No clustering — reference | 3 | 6.414739× | 4.923 | 0.00% |

### 10 files: putting structure first

| Configuration | Compression | Stored GB | Size change vs source + service → status (8.642712×, previous batches) |
|---|---:|---:|---:|
| **Full structure → source + service** | **7.823374×** | **4.037** | **+10.47%** |
| Full structure → service | 7.804115× | 4.047 | +10.75% |
| Full structure → source + service → message(100) | 7.803797× | 4.047 | +10.75% |
| Full structure → source | 7.800417× | 4.049 | +10.80% |
| Full structure → source → status | 7.798620× | 4.049 | +10.82% |
| Full structure → source + service → status | 7.787490× | 4.055 | +10.98% |
| Full structure → status → source + service | 7.780506× | 4.059 | +11.08% |
| Full structure → source + service → status → message(100) | 7.770369× | 4.064 | +11.23% |
| Full structure → source → status → message(100) | 7.769137× | 4.065 | +11.24% |
| Full structure → source + service → status → message(25) | 7.766591× | 4.066 | +11.28% |
| Full structure | 7.758402× | 4.070 | +11.40% |
| Full structure → status | 7.750521× | 4.075 | +11.51% |

### 10 files: 20 MB/s per shard

| Configuration | Compression | Stored GB | Size change vs matching no-clustering baseline |
|---|---:|---:|---:|
| **source + service → status → full structure** | **9.395656×** | **3.361** | **−31.59%** |
| source + service → status → message(100) | 9.333555× | 3.384 | −31.14% |
| source + service → status → message(25) | 9.320109× | 3.388 | −31.04% |
| source + service → message(100) | 9.318500× | 3.389 | −31.03% |
| source + service → message(25) | 9.261157× | 3.410 | −30.60% |
| source + service | 9.256491× | 3.412 | −30.56% |
| source + service → status | 9.248354× | 3.415 | −30.50% |
| source → full structure → service | 9.220073× | 3.425 | −30.29% |
| Full structure | 8.428609× | 3.747 | −23.74% |
| status → message(100) | 7.931564× | 3.982 | −18.96% |
| No clustering — reference | ~6.427× | 4.913 | 0.00% |

### 100 files: completed validation runs

| Configuration | Compression | Stored GB | Size change vs message(25) refinement |
|---|---:|---:|---:|
| **source + service → status → full structure** | **8.502501×** | **37.303** | **−0.42%** |
| source + service → status → message(25) — reference | 8.466542× | 37.461 | 0.00% |
| source + service → status → message(100) | 8.445995× | 37.552 | +0.24% |

### Interpretation limits

| Comparison | What the measurements establish |
|---|---|
| Full structure after source/service/status | Best measured candidate in the normalized 10-file, 20 MB/s and completed 100-file comparisons. |
| Incremental structure benefit at default shard throughput | About −0.47% stored size versus the averaged source/service/status control, or −0.30% versus its stronger individual run. This is comparable to observed run-to-run variation. |
| Incremental structure benefit at 20 MB/s | About −1.57% stored size versus source/service/status in that batch. |
| Structure-first | Best tested structure-first policy used 10.47% more storage than the source/service/status reference from earlier batches. |
| 100-file coverage | No no-clustering or plain source/service/status control completed in this batch. The full-structure candidate used 0.42% less storage than message(25) and 0.66% less than message(100). |
| Merge normalization | One final split removes split-count differences. It does not globally recluster documents. Fresh-split boundaries and ingestion concurrency still affect results. |
| Earlier screening | Results with unequal final split counts are omitted because they confounded close rankings. |

## Later Tantivy experiments: field ordering

### What was tested

| Dataset | Tests |
|---|---|
| CloudPrem / org 2: 100,000 documents | Field sorting and hierarchical grouping; Zstd8, 1 MB blocks. |
| Yahoo-sim: 100,000 replayed stored documents | 18 field-ordering configurations; Zstd8, 1 MB blocks. Selected results below. |

### CloudPrem / org 2

Within equal keys or final groups, documents retain arrival order. Full structure fingerprints the document's leaf paths.

| Fields / grouping | Ordering | Index MB | Size change vs status/service grouping |
|---|---|---:|---:|
| **source → service → tag.pod_name** | Lexicographic | **62.755** | **−5.21%** |
| source → service → tag.pod_name → message | Lexicographic | 63.057 | −4.75% |
| source → service → status → full structure | Largest group first | 63.223 | −4.50% |
| service → source → tag.pod_name | Lexicographic | 63.339 | −4.33% |
| source → service → tag.pod_name | Largest group first | 63.350 | −4.31% |
| source → service → message | Lexicographic | 63.484 | −4.11% |
| tag.pod_name | Lexicographic | 63.896 | −3.48% |
| service → source → status → full structure | Largest group first | 64.398 | −2.73% |
| service → full structure → source | Largest group first | 64.479 | −2.60% |
| service → full structure | Largest group first | 64.480 | −2.60% |
| service → source → tag.pod_name | Largest group first | 64.560 | −2.48% |
| service | Lexicographic | 64.979 | −1.85% |
| service → source → status | Largest group first | 65.232 | −1.47% |
| service → source | Largest group first | 65.340 | −1.30% |
| service → status | Largest group first | 65.575 | −0.95% |
| service | Largest group first | 65.747 | −0.69% |
| status → service — reference | Largest group first | 66.203 | 0.00% |
| message | Lexicographic | 68.493 | +3.46% |
| timestamp | Lexicographic | 79.976 | +20.81% |
| No clustering | Arrival | 80.769 | +22.00% |

### Yahoo-sim: selected field-ordering results

| Ordering | Index MB | Size change vs original split order |
|---|---:|---:|
| **custom.index → custom.source** | **18.787** | **−5.34%** |
| custom.source | 18.795 | −5.30% |
| custom.index | 18.854 | −5.00% |
| custom.index → custom.source → timestamp | 19.626 | −1.11% |
| Original split order — reference | 19.847 | 0.00% |
| message | 21.080 | +6.21% |
| timestamp | 22.133 | +11.52% |

## Sources and comparison boundaries

| Item | Source / qualification |
|---|---|
| August Pomsky tables | Recovered from benchmark session `2026-08-14T01-48-35-775Z_019ffdf4-f03f-787a-938e-5ff3a983926c.jsonl`, including recorded normalized results, completed August 18 structure-first matrix and three completed 100-file results. These experiments were not rerun for this document. |
| Later Tantivy tables | [DOC_CLUSTERING_BENCHMARKS.md — Sorting by other fields](DOC_CLUSTERING_BENCHMARKS.md#sorting-by-other-fields), its source/service/pod follow-up, and the CPU-profile section's descending-size traversal result. |
| Service-first runs | Tantivy `07de87394`, original Pomsky mapper/tokenizers. All 13 layouts passed stored-document and index-content validation; winner repeats and previous control sizes matched. Harness and exact results: `/tmp/service-grouping.j83nWu/`. |
| Percentage derivation | Historical stored-size reductions retained where recorded. New comparisons use exact bytes when available, otherwise compression-ratio quotients for identical input. Rounded displayed sizes may not reproduce the last decimal. |
| Different experiments | Pomsky hashes fingerprint groups and traverses largest groups first. The later field-sort sweep uses lexicographic values. Dataset sizes, inputs and indexing paths also differ; compare rows within each table. |
| Scope | Compression tables only. CPU, allocation/RSS, staging representations and timestamp-codec prototypes remain in the full Tantivy benchmark report. |
