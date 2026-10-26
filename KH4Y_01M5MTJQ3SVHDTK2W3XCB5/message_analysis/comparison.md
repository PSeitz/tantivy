# Prefix-excluded block skipping and metadata cost

**Current test convention: remove each block's actual longest common prefix before building every byte, ending-byte, pair, parity, or configured-sequence summary.** Parity starts at remainder offset zero. Evaluate prefix-contained and prefix-crossing matches separately, so removing a prefix introduces no false negatives.

**16,794 original blocks. Existing index metadata: 195,640 bytes. Additional budget: 195,640 bytes (+100%).**

All costs are actually serialized **additional metadata**, including shared remainder-byte masks and prefix metadata exactly once. The index bounds already supply the entire actual prefix for all but **4 blocks**. Those four blocks each need one extra prefix byte. Their sparse extension table costs **48 bytes including framing**, and is included in every row. No dictionary block needs reading to reconstruct a prefix.

All percentages below describe **blocks skipped**. Earlier reports often used compressed-byte percentages. The original physical boundaries are unchanged. Every evaluated method has zero false negatives. Sizes exclude production-container integration and runtime allocation. No latency or physical I/O claim is made.

## A. No caller-supplied patterns

These summaries derive from all dictionary remainders. Generic tuning parameters such as compression group size are not pattern input.

| Method | Additional bytes | Increase | Within +100% budget |
| --- | --- | --- | --- |
| Remainder bytes | 20,355 | +10.40% | yes |
| Bytes + ending bytes | 81,680 | +41.75% | yes |
| Bytes + pair unions / 5 blocks | 171,707 | +87.77% | yes |
| Bytes + pairs, fill gaps ≤5, run records | 169,690 | +86.74% | yes |
| Bytes + pairs, fill gaps ≤3, run records + zstd | 188,774 | +96.49% | yes |
| Bytes + all pairs | 1,469,199 | +750.97% | no |
| Bytes + all pairs even/odd | 4,610,703 | +2356.73% | no |

### Blocks skipped, out of 16,794

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Remainder bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + ending bytes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 772 (4.60%) | 463 (2.76%) | 455 (2.71%) |
| Bytes + pair unions / 5 blocks | 418 (2.49%) | 426 (2.54%) | 770 (4.58%) | 16,794 (100.00%) | 440 (2.62%) | 1,001 (5.96%) | 621 (3.70%) |
| Bytes + pairs, fill gaps ≤5, run records | 418 (2.49%) | 457 (2.72%) | 1,094 (6.51%) | 16,794 (100.00%) | 440 (2.62%) | 1,299 (7.73%) | 865 (5.15%) |
| Bytes + pairs, fill gaps ≤3, run records + zstd | 418 (2.49%) | 483 (2.88%) | 1,545 (9.20%) | 16,794 (100.00%) | 440 (2.62%) | 1,924 (11.46%) | 1,062 (6.32%) |
| Bytes + all pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + all pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,176 (66.55%) | 2,710 (16.14%) |

## B. Caller-supplied patterns required

These methods require externally provided sequence lists before indexing. The reader uses the mapping persisted with the dictionary. Untracked sequences impose no constraint. Missing summaries retain blocks.

| Method | Additional bytes | Increase | Within +100% budget |
| --- | --- | --- | --- |
| Bytes + 10 supplied pairs | 54,019 | +27.61% | yes |
| Bytes + 10 supplied pairs even/odd | 70,813 | +36.20% | yes |
| Bytes + 8 supplied mixed sequences | 37,219 | +19.02% | yes |
| Bytes + 4 supplied literals | 37,195 | +19.01% | yes |

### Blocks skipped, out of 16,794

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Bytes + 10 supplied pairs | 418 (2.49%) | 1,281 (7.63%) | 4,976 (29.63%) | 16,794 (100.00%) | 440 (2.62%) | 6,990 (41.62%) | 2,710 (16.14%) |
| Bytes + 10 supplied pairs even/odd | 418 (2.49%) | 1,281 (7.63%) | 7,319 (43.58%) | 16,794 (100.00%) | 440 (2.62%) | 11,176 (66.55%) | 2,710 (16.14%) |
| Bytes + 8 supplied mixed sequences | 418 (2.49%) | 1,281 (7.63%) | 16,653 (99.16%) | 16,794 (100.00%) | 440 (2.62%) | 16,780 (99.92%) | 2,710 (16.14%) |
| Bytes + 4 supplied literals | 418 (2.49%) | 1,281 (7.63%) | 16,360 (97.42%) | 16,794 (100.00%) | 440 (2.62%) | 16,386 (97.57%) | 2,710 (16.14%) |

The supplied lists are unchanged from the prior experiment:

```json
{
  "10_pairs": ["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms"],
  "8_mixed_sequences": ["ab", "ms", "tra", "raa", "pla", "lay", "aye", "yer"],
  "4_literals": ["ab", "traa", "player", "ms"]
}
```

These are workload examples, not an automatically selected vocabulary. Queries lacking tracked sequences receive only remainder-byte/prefix pruning. Their results equal the generic pair results for these queries because the supplied pair list covers their relevant pairs.

## Generic trigram default: target remains unmet within budget

The [generic trigram follow-up](generic_trigrams/generic_trigrams.md) builds summaries from all remainder trigrams, with no caller vocabulary, query-driven features, or neighbor unions. Optional caller-configured methods above are separate from this default requirement.

No tested generic trigram layout combines ≥90% `*player*` block skipping with the 195,640-byte added-metadata allowance. The highest observed budget-fitting trigram result skips 7.43% at 150,800 bytes. A 512-byte/block Bloom with two hashes costs 6.44–6.45 MB and skips 83.73–91.12% across three seeds. A 1,024-byte/block Bloom with one hash costs 15.33–15.34 MB and skips 97.55–98.61% across those seeds. Exact trigrams skip 99.92% at 46,778,329 bytes. These are actually serialized whole-sidecar-compressed totals, including shared bytes and prefixes, and describe disk size rather than resident memory. The seed ranges are finite measurements. The conclusion applies to tested encodings, rather than every possible generic design.

The earlier budget-fitting gap-3 pair layout still skips more `*player*` blocks (11.46%) than these budget-fitting trigram layouts, while also falling short of the 90% target.

## Pair ranges with safe gap filling

The [range-encoding follow-up](pair_runs/pair_runs.md) keeps per-block zero runs that prove a pair absent and fills small internal zero gaps with ones. It uses no caller patterns and no neighbor unions. Shared identical range records are deduplicated. Gap thresholds and layouts are chosen from storage and filled-bit counts before query evaluation.

Rows labeled `run records` need no external compression. Rows labeled `run records + zstd` compress the complete range sidecar and require whole-sidecar decompression. The current 188,774-byte compressed total corresponds to 278,826 bytes of uncompressed range records plus the 20,355-byte shared sidecars. Serialized cost fits the budget; that is not a claim about decoded memory. Both layouts decode self-framed records into memory and have no serialized per-block offset table. The ordinary gap-5 raw layout improves pruning over five-block unions at slightly lower metadata cost.

## Separate deduplication of the two masks

A follow-up tests [two independently deduplicated masks, with no neighbor unions](separate_masks/README.md). It distinguishes byte-position masks (256 bits each) from exact adjacent-pair masks (one bit per observed ordered pair). Separate byte-position tables fit the budget but lose pair correlation. Exact pair tables remain expensive because almost every block's masks differ. The follow-up includes measured joint-versus-independent costs, whole-sidecar compression, and block skipping.

## What changed after removing prefixes

- Byte-mask pruning is unchanged for the seven measured queries. The actual shared remainder-byte sidecar is 20,307 bytes; with the 48-byte prefix-extension table the total is **20,355 bytes**.
- Pair and parity pruning is slightly weaker on these queries. A required pair crossing the prefix boundary is absent from remainder-only pair masks and must be handled using the prefix and remaining continuation.
- **Full-literal bits no longer equal the oracle.** For example, if prefix `pl` begins a match of `player`, the remainder only needs to start with `ayer`. A configured `player` bit cannot reject that crossing match. With `ayer` untracked, the test conservatively falls back to available remainder summaries.
- Mixed sequences retain constraints on some crossing fragments. They skip **16,653 blocks (99.16%)** for `*traa*` and **16,780 (99.92%)** for `*player*`.
- The maximum possible `*ms*` skipping remains 2,729 blocks (16.25%). The remainder-pair methods skip 2,710 because their crossing checks retain extra false positives.
- A separate diagnostic run removes only the prefix derivable from existing bounds, charging zero prefix storage. That leaves one extra byte in four blocks. Its decisions match actual-prefix mode except the parity rows retain one extra `*player*` block. The main tables above always remove the **complete actual prefix**.

## Reproduce and inspect

```sh
python3 -m unittest discover -s dictionary_analysis -p 'test_*.py'
python3 dictionary_analysis/prefix_summaries.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis --config dictionary_analysis/selected_sequences.json --modes actual bounds
python3 dictionary_analysis/comparison.py
```

- [Detailed prefix-excluded results](prefix_summaries/prefix_summaries.md): both prefix modes, serialized layouts, all query counts, validation, and limitations.
- [Machine-readable prefix-excluded results](prefix_summaries/prefix_summaries.json).
- [Historical full-term comparison](full_term_comparison.md): the prior experiment, which included prefixes in masks. Its numbers are superseded for the current test convention.

Every sidecar was decoded and compared with build-time data. Full-term matches were independently counted and checked per block against the existing evaluator. The suffix matcher conservatively allows an empty remainder when the query matches the end of the prefix; it stores no separate empty-remainder flag.
