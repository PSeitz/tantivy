# Failures and limits

## Resolved startup failure

The first extraction attempt stopped before decoding source postings because constructing the disposable auxiliary-file fixture required the schema's `raw_lowercase` tokenizer for field `all`. Only `datadog` had initially been registered. The error and resource measurements are preserved in `failed_tokenizer_resources.log`.

The fixture now registers a raw tokenizer for every tokenizer name referenced by the schema. These registrations generate only unused auxiliary files. Actual source terms and postings are copied byte-for-byte, opened with the original schema, and never retokenized. The successful run validates every original key and posting count/document ID. Original files were unchanged.

## Resolved report-only regeneration failure

After successful extraction, selection, and validation, the optional report-only command reloaded JSON whose cutoff object keys were strings. The Markdown renderer indexed those keys as integers and raised `KeyError: 0`. `failed_report_reload.log` preserves the failure. Top-retained diagnostic keys now consistently use strings, and a regression test renders the persisted JSON. All serialized indexes and query decisions were already complete and remained unchanged.

## Workload failures

- `tra` (document DF 1,653) and `raa` (1,292) fit the broad-cap eligibility rule but rank below the retained budget head. Neither is retained at any tested cap. `*traa*` stays at 16,341 candidates (2.6974% skipped) versus 141 with the full trigram index. The full query matches 42 distinct documents.
- At 94%, `aye` and `yer` fail the block cap. `*player*` widens from 14 to 134 candidates.
- At 95–99%, all four `player` grams fail eligibility. `*player*` returns to 16,331 byte-only candidates. These strict caps exhaust eligibility and leave most of the budget unused.
- One- and two-byte literals receive no trigram pruning. Their combined results come from the byte fallback.
- The 80% cap changes selection but improves none of the seven query results relative to uncapped document-DF ranking. This workload supplies no universal optimal cutoff.

## Scope

Only the original single-segment message field has real document assignments. Synthetic key-only dictionaries are excluded. The source has no deletes. Fixture store, positions, fast fields, and field norms are auxiliary placeholders and are never queried or validated as original source components.

All eleven serialized subsets passed retained-posting equality and persisted-unknown checks. Every query/mode/cap evaluation has zero false-negative blocks.
