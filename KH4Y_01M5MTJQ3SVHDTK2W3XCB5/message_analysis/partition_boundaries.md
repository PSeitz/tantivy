# Pattern-aware sorted block boundaries

This experiment preserves the global sorted order and **every original physical block boundary**. Inside each original block, it inserts a boundary wherever adjacent terms differ in their presence bits for the supplied sequences. The resulting runs are homogeneous for those bits. It counts boundaries without rebuilding an SSTable.

Original dictionary: **16,794 blocks, 7,671,280 terms**.

| Split according to | Extra boundaries | Resulting blocks | Block-count increase | Single-term blocks | Median terms/block |
|---|---:|---:|---:|---:|---:|
| sequence:player | 30 | 16,824 | +0.18% | 15 | 397 |
| sequence:traa | 75 | 16,869 | +0.45% | 37 | 397 |
| sequence:ms | 77,477 | 94,271 | +461.34% | 37,781 | 20 |
| sequence:ab | 115,002 | 131,796 | +684.78% | 56,360 | 8 |
| literals | 191,588 | 208,382 | +1140.81% | 94,592 | 4 |
| mixed_sequences | 203,673 | 220,467 | +1212.77% | 100,775 | 4 |
| pairs | 769,239 | 786,033 | +4580.44% | 411,088 | 1 |

The per-sequence rows isolate that feature alone. `literals`, `mixed_sequences`, and `pairs` use the complete caller-provided lists from `dictionary_analysis/selected_sequences.json`. All configurations, including pair-plus-trigram combinations, are in `partition_boundaries.json`.

## Interpretation

- Sorted boundary selection fits the existing lexical range and term-ordinal model. It changes block boundaries rather than term order.
- Rare full literals can be isolated with relatively few cuts. Common short sequences are interleaved with other terms and create many tiny runs.
- These are **counts**, not measured metadata size or compressed byte savings. More blocks add frame/index overhead and can reduce prefix/zstd compression. A +100% metadata budget cannot be checked from block count alone.
- Splitting at every bit change is an aggressive reference. A practical builder could evaluate candidate cuts subject to minimum block size and an explicit metadata budget, weighted by an externally provided query workload. That policy has not been implemented or measured here.
- Pattern bits only enable pruning if exposed before reading data. Homogeneous blocks do not make their signature inferable from lexical fences automatically.

## More invasive partitioning

Grouping all terms with a given pattern signature into separate dictionaries could avoid tiny lexical runs. Each partition can remain internally sorted, but global terms would no longer occupy one lexically ordered block stream. Readers would need partition routing, merged range/automaton traversal, and a revised ordinal mapping. Values/postings must stay attached to their terms, and all ordinal consumers must be audited. The full 2^k signature partition scheme can also create many small partitions. This is a new dictionary layout, not just a writer flush heuristic.

## Reproduce

```sh
python3 dictionary_analysis/partition_boundaries.py --config dictionary_analysis/selected_sequences.json --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

Source data and library code remain unchanged. Every original term is accounted for exactly once in every proposed split plan.
