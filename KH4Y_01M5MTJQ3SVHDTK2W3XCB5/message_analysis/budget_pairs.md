# Adjacent-byte pair masks under the +100% metadata budget

## Exact per-block pair masks

There are **1,444 distinct adjacent-byte pairs globally**, and **16,613 distinct pair masks across 16,794 blocks**. Unlike whole-byte masks (107 variants), pair masks barely repeat.

Using a global pair-rank table, each exact mask takes 181 bytes. Raw mask payload is 3,039,714 bytes. Sharing identical masks plus two-byte IDs requires 3,023,747 payload bytes, before the global pair table and framing. Deduplication alone saves nothing here.

I also serialized and tested zstd-compressed exact masks in both block-major and transposed pair-major order. Every compressed group was decoded and compared with its input. The pair-major transpose was independently checked against the original masks.

| Layout | Added pair metadata bytes | Increase over current 195,640 B index |
|---|---:|---:|
| block-major, 128 blocks/group, zstd 3 | 1,727,589 | +883.04% |
| pair-major, 128 blocks/group, zstd 3 | 1,447,903 | +740.09% |
| block-major, 512 blocks/group, zstd 3 | 1,739,892 | +889.33% |
| pair-major, 512 blocks/group, zstd 3 | 1,469,984 | +751.37% |
| block-major, 16794 blocks/group, zstd 3 | 1,738,316 | +888.53% |
| block-major, 16794 blocks/group, zstd 19 | 1,616,693 | +826.36% |

Sizes include the global pair table, headers, and group offsets. Whole-key byte masks add another 20,231 bytes. Compression reduces size, but every tested exact layout exceeds the +100% budget. No latency or decompression-memory benefit was measured.

These exact pair masks reproduce the earlier pair decisions. All queries are case-sensitive. `*player*` and `*ms*` are substrings. This pair study applies whole-key bytes/pairs to suffix queries too, with no ending-byte filter.

| Query | Whole-byte data-byte skipping | Whole-byte + exact-pair data-byte skipping |
|---|---:|---:|
| `*a*` | 2.35% | 2.35% |
| `*ab*` | 2.37% | 7.62% |
| `*traa*` | 2.60% | 32.15% |
| `*X` | 100.00% | 100.00% |
| `*x` | 2.51% | 2.51% |
| `*player*` | 2.67% | 44.14% |
| `*ms*` | 2.61% | 18.30% |

## Share a union across neighboring blocks

A cheaper safe approximation stores the union of all pairs in a group of consecutive blocks. Every constituent block's pairs remain present, so there are no false negatives. Extra pairs can cause additional blocks to be retained. Identical group masks share one table entry.

The following are **actually serialized** sizes including the 20,231-byte exact whole-key byte-mask table:

| Blocks/group | Unique union masks | Added total bytes | Metadata increase | `*ab*` data-byte skipping | `*traa*` data-byte skipping | `*player*` data-byte skipping | `*ms*` data-byte skipping |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 8,249 | 1,533,003 | +783.58% | 2.81% | 13.21% | 18.73% | 8.55% |
| 3 | 2,312 | 452,808 | +231.45% | 2.57% | 8.78% | 12.33% | 6.04% |
| 4 | 1,149 | 239,507 | +122.42% | 2.43% | 6.79% | 9.10% | 4.63% |
| 5 | 785 | 171,943 | +87.89% | 2.41% | 5.21% | 7.17% | 3.96% |
| 6 | 557 | 129,555 | +66.22% | 2.42% | 4.26% | 5.99% | 3.59% |
| 8 | 411 | 101,731 | +52.00% | 2.37% | 3.75% | 4.48% | 3.01% |
| 16 | 231 | 66,001 | +33.74% | 2.37% | 2.60% | 2.91% | 2.61% |
| 32 | 125 | 46,290 | +23.66% | 2.37% | 2.60% | 2.67% | 2.61% |
| 64 | 97 | 40,960 | +20.94% | 2.37% | 2.60% | 2.67% | 2.61% |
| 128 | 80 | 37,752 | +19.30% | 2.37% | 2.60% | 2.67% | 2.61% |

All generated union masks roundtrip exactly. Every group's mask contains each member block's exact pairs. Every query was checked against the previous exact term-scan results, with zero false negatives. The group IDs use fixed-size records, so a block ID directly identifies its group. These union files are uncompressed.

The whole-byte-only baseline skips **2.37%** for `*ab*` and **2.60%** for `*traa*`. Group sharing meets the budget at a substantial selectivity cost.

## Another option: select the tracked pairs

Instead of representing all pairs, reserve one bit for each of a selected set of pairs. Tracking **80 pairs costs 10 bytes/block**, or 167,940 bytes. Adding the measured shared whole-byte masks and a 160-byte pair table comes to approximately **188 KB (+96%)**, with a small header still inside the budget.

This encoding is an estimate, not an implemented or evaluated selection strategy. A tracked zero bit proves absence. Untracked pairs must conservatively retain the block. Choosing pairs from production query frequencies could make the limited bits useful. Choosing just the pairs in the current test queries would overfit this small query sample. If `ab`, `tr`, `ra`, and `aa` are tracked, the existing exact pair decisions for `*ab*` and `*traa*` are preserved.

## Reproduce

Run `python3 dictionary_analysis/budget_pairs.py` from the repository root. Requires the `zstd` executable and the earlier extraction/filter/byte-mask outputs.

See `budget_pairs.json` for all counts, serialized sizes, filenames, and per-query decisions. The standalone sidecars are experiments, not production SSTable format changes.

Exact compressed files: `EP01`, u32 block count, u32 pair count, u32 group size, u8 order (0 block-major / 1 pair-major), u8 zstd level, sorted u16 pair table, u32 group offsets including final offset, then independently compressed groups.

Union files: `EU01`, u32 block count, u32 pair count, u32 group size, u32 unique mask count, u8 ID width, sorted u16 pair table, unique fixed-width masks, then one ID per group. Integers and masks are little-endian. Pair values are `(first_byte << 8) | second_byte`. No grams cross term boundaries.
