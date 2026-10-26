# Wildcard block-skipping experiments

These studies ask how much additional metadata is needed to skip SSTable dictionary blocks for wildcard queries. The baseline is one synthetic message dictionary: **7,671,280 terms in 16,794 blocks**, with a **195,640-byte index**.

Start with the [measured-results overview](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/README.md) for the input and baseline, or the [metadata-cost comparison](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/comparison.md) for the early designs.

## Main findings

- **Selecting trigrams by document frequency helps the tested log-word queries.** A 25 KB metadata budget retains enough trigrams to reduce `*player*` to 14 candidate blocks, matching the full-trigram result. This benefit does not extend to every query. [Budget sweep](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_document_budgets/budget-sweep-report.md).
- **The metadata budget matters across a broader cohort.** On 34 present literals of at least three bytes, mean block skipping rises from 69.62% at 200 KB to 86.78% at 400 KB with the tested selection rule. [Budget comparison](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_query_budgets/report.md).
- **Log words and random IDs favor different selections.** Limiting retained trigrams to those present in at most 10% of blocks gives the best natural-word group means in the 89-query retest. It excludes every trigram used by the sampled ID queries, leaving those queries at byte-only pruning. [Presence-limit retest](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_query_thresholds/report.md).
- **Longer queries help when their extra features are represented.** For the sampled hex IDs, a 1 MB document-ranked index averages 3,490 candidates for 32-byte literals. Full trigrams reduce those same queries to one candidate. [Query-length interpretation](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_queries/interpretation.md).

These are diagnostic query cohorts. Their results support workload-specific trade-offs rather than a universal budget or selection threshold.

## Selected-trigram studies

A partial index stores exact block postings for selected trigrams. Omitted trigrams remain unknown, so queries fall back to other available features. Document frequency ranks grams by how many distinct documents contain them. Block presence controls which rows are eligible.

| Study | Question |
|---|---|
| [Block-frequency selection](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_selection/report.md) | Does ranking by block frequency help the original query set? |
| [Document-frequency selection](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_documents/report.md) | Does counting distinct documents identify more useful grams? |
| [Small budgets](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_document_budgets/budget-sweep-report.md) | How early do useful grams fit in the tested budget grid? |
| [42-query evaluation](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_queries/report.md) | How do the selected indexes perform beyond the original queries? |
| [200 KB–1 MB budgets](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_query_budgets/report.md) | What improves as the metadata allowance grows? |
| [Presence limits on 42 queries](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_query_thresholds/report.md) | Which rows become eligible when the block-presence limit changes? |
| [89 longer queries](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_queries/report.md) | How do log words and nested ID substrings behave at different lengths? |
| [Presence limits on 89 queries](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/long_query_thresholds/report.md) | How do limits from 10% through 80% affect those two groups? |

## Metadata layouts and input composition

| Study | Scope |
|---|---|
| [Prefix handling](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/prefix_summaries/prefix_summaries.md) | Excluding shared block prefixes while preserving prefix-contained and crossing matches |
| [Byte and bigram postings](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/pair_postings/report.md) | Raw IDs, deltas, runs, bitmaps, and compressed postings |
| [Full trigram postings](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/trigram_postings/report.md) | Storage and pruning when every observed trigram is represented |
| [Trigram bitsets and Bloom filters](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/generic_trigrams/generic_trigrams.md) | Earlier query-independent layouts |
| [UUID and random-ID injection](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/uuid_injection/report.md) | Effects of alphabet, fixed length, ordering, and block composition |
| [Timestamp injection](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/timestamp_injection/report.md) | Effects of adding numeric keys and rebuilding blocks |

The [baseline overview](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/README.md) also covers earlier byte masks, caller-supplied sequences, and partitioning experiments.

## How to read the measurements

- **Cost** means serialized additional metadata. Budgeted indexes include their byte fallback and prefix metadata. Percentages use the original 195,640-byte index as the reference, including for augmented dictionaries.
- **Skipping** counts dictionary blocks. Always compare absolute candidate counts as well: adding generated-only blocks can raise the skip percentage without reducing the original candidates.
- **Prefix safety** matters. Later summaries exclude each block's complete common prefix and handle prefix-contained and crossing matches separately. Earlier whole-term experiments are labeled in their reports.
- **Validation** compares candidate decisions with exact key scans. The reported query evaluations have zero false negatives. This validates the tested predicates and inputs.
- **Scope** is storage and candidate-block pruning. These studies do not measure network I/O or end-to-end query latency.

This branch contains Markdown reports only. The linked study notes document the original methods and validation. Programs, dictionaries, binary indexes, and raw result files are omitted.
