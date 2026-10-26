# SSTable and regex experiments

Two studies explore storage and read costs on a synthetic message workload.

| Study | Question | Reports |
|---|---|---|
| Inline postings | Can storing short posting lists inside dictionary blocks reduce total storage and external reads? | [Overview and results](inline-postings/README.md) |
| Wildcard block skipping | How much block-summary metadata is needed to prune dictionary scans for wildcard queries? | [Study guide](../dictionary_analysis/README.md) |

The [message-analysis overview](../KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis/README.md) describes the input dictionary and baseline measurements.

## Scope

These reports describe experimental implementations, measured storage, modeled read plans, and diagnostic query cohorts. Each report states its assumptions and validation. They do not establish end-to-end query latency improvements.

This branch publishes Markdown only. Source programs, dictionaries, raw result files, and generated indexes are omitted. Commands and artifact filenames in the detailed reports refer to the original experiment environment.
