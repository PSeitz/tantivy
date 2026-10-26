# SSTable and regex experiment reports

This branch collects Markdown reports from the inline-postings and wildcard-workload studies. It includes historical measurements, failed-attempt reports, and subsequent corrections.

- [Inline postings: study index](inline-postings/README.md)
- [Inline postings: document-ID delta sizes and decoding cost](inline-postings/SIZE_UP_TO_128.md)
- [Inline postings: request coalescing](inline-postings/REQUEST_BUDGET.md)
- [Wildcard workload: study index](../dictionary_analysis/README.md)
- [Wildcard workload: measured results](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/README.md)
- [Initial dictionary analysis](../analysis.md)
- [Inline-postings format](../src/termdict/sstable_termdict/FORMAT.md)

## Publication scope

Only Markdown documents are included in this report snapshot. Implementation changes, experiment programs, JSON/JSONL results, dictionaries, exported terms, postings, generated indexes, executable files, and build products are omitted. No Git LFS objects are used.

Split identifiers and measurements refer to the synthetic workload. The original worktrees retain their source files and data. References and commands within the reports describe those original worktrees. Links to omitted artifacts will not resolve in this report-only branch, and reproducing the experiments requires the corresponding implementation and inputs.
