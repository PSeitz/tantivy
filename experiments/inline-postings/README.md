# Inline-postings experiments

These experiments move short posting lists into SSTable dictionary blocks to reduce external postings reads. They compare storage size, read plans, and RAM-resident dictionary scans on one synthetic split with 7,671,280 terms.

## Main results

The latest size experiment stores document IDs as per-list deltas, together with frequencies and positions. The cutoff is the maximum number of documents in an inline list.

| Cutoff | Dictionary MB | Dictionary + external postings + positions MB | Reduction versus no inlining |
|---:|---:|---:|---:|
| 0 | 51.678 | 516.500 | — |
| 8 | 74.477 | 491.090 | 4.92% |
| 32 | 124.717 | 458.190 | 11.29% |
| 64 | 138.647 | 447.924 | 13.28% |
| 128 | 143.411 | 444.163 | 14.01% |

Larger cutoffs reduce total storage while increasing the dictionary size. Compared with absolute document IDs, deltas save 4.709 MB at cutoff 32. Full dictionary scans in RAM took about 1.8% longer at that cutoff in the measured run.

Dictionary sizes are actual serialized bytes. External-file sizes model byte-preserving compaction. The experiment verifies every term after rewriting the dictionary. Cutoffs 64 and 128 are size experiments, rather than complete query-engine implementations.

See [the full size sweep and decoding comparison](SIZE_UP_TO_128.md) for every cutoff from 0 through 16, plus 32, 64 and 128.

## Read planning

[Request coalescing](REQUEST_BUDGET.md) compares external-postings read plans. It first merges gaps up to 512 KiB, then bridges the cheapest remaining gaps within an extra-byte allowance. This minimizes request count within that allowance and never adds requests relative to the fixed-gap baseline.

The plans describe requested ranges and downloaded bytes. Query latency, network behavior, and cache effects were not measured. The [earlier absolute-ID read experiment](ABSOLUTE_ID_READS.md) also accounts for cold dictionary-block reads.

## Earlier experiments

| Study | What it measures |
|---|---|
| [Initial prototype](MEASUREMENTS.md) | Embedding existing posting bytes, with a 4 MiB read-coalescing gap |
| [Small cutoffs](CUTOFFS.md) | Cutoffs 1–4 and their read plans |
| [Fixed-gap coalescing](COALESCING.md) | Request-count and byte trade-offs across merge gaps |
| [Absolute-ID storage](ABSOLUTE_ID_SIZE.md) | Encoding short lists with frequencies and positions |
| [Absolute-ID cutoffs 0–8](ABSOLUTE_ID_CUTOFFS.md) | Storage trade-offs with the earlier fixed-width layout |
| [Six-bit counts](SIX_BIT_COUNTS.md) | Absolute-ID size sweep through every cutoff 0–32 |
| [Six-bit counts with deltas](SIX_BIT_DELTAS.md) | The corresponding document-ID delta sweep |

The [experimental format notes](../../src/termdict/sstable_termdict/FORMAT.md) describe the block layout and reader/writer behavior. Earlier reports retain their measured layouts and configurations, which differ from later experiments.

This branch publishes Markdown reports only. Source programs, raw result files, patches, and split data remain outside this snapshot. Commands in individual reports describe the original experiment environment.
