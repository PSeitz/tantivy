# Document-DF selection across budgets

Original baseline: 7,671,280 terms, 16,794 blocks, 10,002,000 documents. Rank exact full-term distinct-document union DF descending, u24 ascending ties. Eligible rows have block DF × 100 ≤ blocks × 20 (at least 80% individual-row skipping). Keep whole rows that fit and continue past nonfits. Selection takes no query information. Larger budgets can admit large high-ranked rows, so retained sets need not be nested and row counts can decrease.

Costs are actual added serialized metadata: IU01 trigrams + 5,275 B byte fallback + 48 B prefix extensions. Percentages divide by the original 195,640 B index. Diagnostics and decisions are excluded. Both query modes use the same charged total. T is the trigrams-only ablation, T+B includes bytes.

**First tested player budget with ≥90% skipping: 20,000 B (T+B), 20,000 B (T).** These are first successes among the listed budgets, not minimum possible budgets. Gram entry below also means first tested retention.

| Player gram | First tested budget B |
| --- | --- |
| pla | 20000 |
| lay | 20000 |
| aye | 20000 |
| yer | 25000 |

| Budget B | Actual B | % original | Rows | Player T / T+B | Player T+B skip % | traa T / T+B | Player grams |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 5341 | 5341 | 2.7300 | 0 | 16794 / 16331 | 2.7569 | 16794 / 16341 | none |
| 7500 | 7495 | 3.8310 | 39 | 16794 / 16331 | 2.7569 | 16794 / 16341 | none |
| 10000 | 10000 | 5.1114 | 15 | 16794 / 16331 | 2.7569 | 16794 / 16341 | none |
| 15000 | 14995 | 7.6646 | 38 | 16794 / 16331 | 2.7569 | 16794 / 16341 | none |
| 20000 | 19998 | 10.2218 | 22 | 23 / 23 | 99.8630 | 16794 / 16341 | pla, lay, aye |
| 25000 | 25000 | 12.7786 | 62 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 50000 | 49996 | 25.5551 | 63 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 100000 | 99996 | 51.1122 | 150 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 150000 | 150000 | 76.6714 | 236 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 195640 | 195640 | 100.0000 | 276 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 200000 | 199999 | 102.2281 | 304 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 300000 | 300000 | 153.3429 | 476 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 500000 | 499997 | 255.5699 | 828 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |
| 1000000 | 999997 | 511.1414 | 1223 | 14 / 14 | 99.9166 | 16794 / 16341 | pla, lay, aye, yer |

## All seven queries and references

Raw case-sensitive bytes. `*X` and `*x` are suffix queries, the others are substrings. `*X` is absent. The other six queries are present. This cohort is not a representative production workload.

| Query | Exact terms | True blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 16794 | 0 |
| *a* | 1921269 | 16376 | 16376 | 16794 | 16376 |
| *ab* | 75774 | 15490 | 16373 | 16794 | 16373 |
| *ms* | 50585 | 14065 | 16339 | 16794 | 16339 |
| *player* | 21 | 13 | 16331 | 14 | 14 |
| *traa* | 42 | 38 | 16341 | 141 | 141 |
| *x | 153990 | 16022 | 16354 | 16794 | 16354 |

| Reference | Actual B | % original |
| --- | --- | --- |
| bytes | 5323 | 2.7208 |
| trigrams | 38068570 | 19458.4799 |
| trigrams_bytes | 38073845 | 19461.1761 |

### trigrams: candidate blocks

| Budget B | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 5341 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 |
| 7500 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 |
| 10000 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 |
| 15000 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 |
| 20000 | 16794 | 16794 | 16794 | 16794 | 23 | 16794 | 16794 |
| 25000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 50000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 100000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 150000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 195640 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 200000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 300000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 500000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |
| 1000000 | 16794 | 16794 | 16794 | 16794 | 14 | 16794 | 16794 |

### trigrams_bytes: candidate blocks

| Budget B | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 5341 | 0 | 16376 | 16373 | 16339 | 16331 | 16341 | 16354 |
| 7500 | 0 | 16376 | 16373 | 16339 | 16331 | 16341 | 16354 |
| 10000 | 0 | 16376 | 16373 | 16339 | 16331 | 16341 | 16354 |
| 15000 | 0 | 16376 | 16373 | 16339 | 16331 | 16341 | 16354 |
| 20000 | 0 | 16376 | 16373 | 16339 | 23 | 16341 | 16354 |
| 25000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 50000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 100000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 150000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 195640 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 200000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 300000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 500000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |
| 1000000 | 0 | 16376 | 16373 | 16339 | 14 | 16341 | 16354 |

False positives = candidates − true blocks. JSON includes all candidate, skipped, skip %, false-positive and false-negative counts. All 14 × 7 × 2 evaluations have zero false negatives.

## Validation and interpretation

Every file was reopened. Every retained frame was compared with the full index and independently decoded by Rust `verify-subset`. Counts and membership totals match query-independent selection. Omitted observed and unobserved rows read as unknown. Prefix extensions round-trip, and all decisions use the existing prefix-contained, remainder-contained and crossing-continuation checks. Fresh evaluations equal persisted decision bytes. Truth and byte decisions match the existing full-index reference per block, and partial candidates contain full-index candidates.

The 5,341 B case contains an empty 18 B IU01 file and the 5,323 B fallback. The existing 195,640 B directory was audited in place and its files preserved byte-for-byte. Input hashes were checked before and after. See JSON for the preservation manifest and artifact hashes.

Player benefits at small tested budgets. Neither tra nor raa is retained at any tested budget, so traa remains at byte-only pruning, far behind its full-trigram reference. One/two-byte queries retain every block in T and use the unchanged byte fallback in T+B. No production latency or universal budget recommendation follows from these measurements.

Reproduction and tests: `dictionary_analysis/trigram_documents/budget-sweep.md`.
