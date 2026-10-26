# SSTable measurements

Measurements on the `message` field of one synthetic split: 7,671,280 terms in 16,794 dictionary blocks.

1. **[Dictionary contents and pruning limits](dictionary.md)** — term distribution, existing block metadata, exact query matches, and byte/pair/trigram filter measurements.
2. **[Block metadata: bytes versus candidate blocks](block-metadata.md)** — serialized postings costs, per-query results at five metadata budgets, and log-word versus ID queries.
3. **[Inline postings](inline-postings.md)** — dictionary/postings/positions sizes, absolute-ID versus delta decoding, and external read plans.

Reports only. Experiment programs and input files are outside this snapshot.
