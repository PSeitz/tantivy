# Block skipping and metadata cost: with and without caller input

**16,794 original blocks. Existing index metadata: 195,640 bytes. Additional budget: 195,640 bytes (+100%).**

Every cost below is **additional metadata**, including the shared whole-byte masks exactly once. Final metadata size is the existing 195,640 bytes plus that cost. All listed sizes are actually serialized standalone files, including their headers and lookup tables. Production container integration, runtime allocation, and decompression memory are not included.

**Requires input** means the caller must supply a pattern/sequence list. **No input** means no such list; the summaries are derived from all dictionary keys. Both cases still require ordinary indexing data and algorithm parameters.

All skipping percentages in this report refer to **blocks**, not compressed bytes. Counts preserve original physical block boundaries. No false negatives were observed. These are not I/O or latency benchmarks.

## A. No caller-supplied patterns

| Method | Additional bytes | Increase over existing index | Within +100% budget |
| --- | --- | --- | --- |
| Shared byte masks | 20,231 | +10.34% | yes |
| Byte + ending-byte masks | 81,544 | +41.68% | yes |
| Pairs shared across 5 blocks + bytes | 171,943 | +87.89% | yes |
| All pairs + bytes | 1,468,134 | +750.43% | no |
| All pairs, even/odd + bytes | 4,607,849 | +2355.27% | no |

### Blocks skipped, out of 16,794

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Shared byte masks | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| Byte + ending-byte masks | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16,794 (100.00%) | 772 (4.60%) | 463 (2.76%) | 455 (2.71%) |
| Pairs shared across 5 blocks + bytes | 418 (2.49%) | 426 (2.54%) | 770 (4.58%) | 16,794 (100.00%) | 440 (2.62%) | 1,001 (5.96%) | 621 (3.70%) |
| All pairs + bytes | 418 (2.49%) | 1,304 (7.76%) | 4,992 (29.72%) | 16,794 (100.00%) | 440 (2.62%) | 6,994 (41.65%) | 2,729 (16.25%) |
| All pairs, even/odd + bytes | 418 (2.49%) | 1,304 (7.76%) | 7,339 (43.70%) | 16,794 (100.00%) | 440 (2.62%) | 11,192 (66.64%) | 2,729 (16.25%) |

## B. Caller-supplied patterns required

| Method | Additional bytes | Increase over existing index | Within +100% budget |
| --- | --- | --- | --- |
| 10 supplied pairs + bytes | 53,895 | +27.55% | yes |
| 10 supplied pairs, even/odd + bytes | 70,689 | +36.13% | yes |
| 8 supplied mixed sequences + bytes | 37,095 | +18.96% | yes |
| 4 supplied literals + bytes | 37,071 | +18.95% | yes |

### Blocks skipped, out of 16,794

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10 supplied pairs + bytes | 418 (2.49%) | 1,304 (7.76%) | 4,992 (29.72%) | 16,794 (100.00%) | 440 (2.62%) | 6,994 (41.65%) | 2,729 (16.25%) |
| 10 supplied pairs, even/odd + bytes | 418 (2.49%) | 1,304 (7.76%) | 7,339 (43.70%) | 16,794 (100.00%) | 440 (2.62%) | 11,192 (66.64%) | 2,729 (16.25%) |
| 8 supplied mixed sequences + bytes | 418 (2.49%) | 1,304 (7.76%) | 16,678 (99.31%) | 16,794 (100.00%) | 440 (2.62%) | 16,780 (99.92%) | 2,729 (16.25%) |
| 4 supplied literals + bytes | 418 (2.49%) | 1,304 (7.76%) | 16,756 (99.77%) | 16,794 (100.00%) | 440 (2.62%) | 16,781 (99.92%) | 2,729 (16.25%) |

The configured results match the full-pair versions for these test queries because the supplied pairs cover their required pairs. An unrelated query may have no configured sequences and receive only byte-mask pruning. The configured lists are examples for the tested workload, not an automatically selected general-purpose vocabulary.

## Exact supplied lists

```json
{
  "10_pairs": ["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms"],
  "8_mixed_sequences": ["ab", "ms", "tra", "raa", "pla", "lay", "aye", "yer"],
  "4_literals": ["ab", "traa", "player", "ms"]
}
```

The caller supplies these lists externally. The writer stores their definitions and stable bit mapping. A reader uses the written mapping. Untracked sequences and absent metadata cannot justify pruning.

## Interpretation

- The compact **70,689-byte even/odd result requires the 10 supplied pairs**. It is not the size of a generic all-pairs parity index.
- The generic all-pairs even/odd implementation was now actually serialized and compressed. Its smallest tested total is **4,607,849 bytes (+2355.27%)**. It exceeds the budget.
- For queries in the supplied workload, the eight mixed sequences provide stronger pruning than the supplied pair-parity scheme and use less space. Their advantage depends on having useful sequences supplied in advance.
- `*ms*` cannot skip more than 2,729 original blocks (16.25%), because its matches occur in the other 14,065 blocks. Earlier 18.30% figures referred to **compressed bytes**.
- `*player*` with the mixed sequences retains 14 blocks; the full-literal feature retains the 13-block oracle. Both round to 99.92% blocks skipped.
- `*X` is case-sensitive uppercase X. If production normalizes it to x, use `*x` instead. A single dictionary-wide byte mask can reject uppercase X on this dataset too.
- Bounds plus min/max lengths are another no-pattern-input option: an estimated 33,588 additional bytes. They skip only four blocks for `*player*` and none for the other queries. They are excluded from the serialized-size tables.
- Prefix-excluding byte masks produced identical candidate decisions for these queries. The compact shared-mask and parity sizes above use full terms. No storage benefit from prefix-excluding parity masks is claimed.

## Layout notes

- **Shared byte masks:** Actual shared masks and block IDs, whole terms including common prefixes.
- **Byte + ending-byte masks:** Exact one-byte suffix filter, with shared mask tables.
- **Pairs shared across 5 blocks + bytes:** Union of all pairs in each group of five original blocks. Identical group masks share a table entry.
- **All pairs + bytes:** Best measured exact layout: transposed groups of 128 blocks, zstd 3. Includes table/header/offsets and byte masks.
- **All pairs, even/odd + bytes:** Best measured exact parity layout: transposed pair/phase columns in groups of 128 blocks, zstd 3. Includes all bookkeeping and byte masks.
- **10 supplied pairs + bytes:** Caller supplies ab,tr,ra,aa,pl,la,ay,ye,er,ms.
- **10 supplied pairs, even/odd + bytes:** Same supplied pairs, two bits each, three bytes per block after rounding.
- **8 supplied mixed sequences + bytes:** Caller supplies ab,ms,tra,raa,pla,lay,aye,yer.
- **4 supplied literals + bytes:** Caller supplies ab,traa,player,ms. Exact block-presence tests for those substrings.

## Reproduce and inspect

- `python3 dictionary_analysis/comparison.py` regenerates these tables from saved results.
- [Pair-offset experiment](pair_parity.md) has the full-pair and caller-configured parity encodings.
- [Configured sequence experiment](selected_sequences.md) has the supplied sequence encodings and tests.
- [Whole-byte mask budget](budget.md) explains mask sharing.
- [Generic pair budget](budget_pairs.md) describes compression and grouped unions.
- [Complete original comparison](README.md) includes Bloom filters, exact trigram models, prefix-aware solvers, and compressed-byte skipping.
