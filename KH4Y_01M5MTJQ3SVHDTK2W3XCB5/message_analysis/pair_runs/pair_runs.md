# Per-block pair ranges — no caller-pattern input

No caller-pattern input. No neighboring-block union. Original blocks, full common prefixes excluded. Global observed ordered u16 pair vocabulary is serialized. Phase masks are independent.

## Format and interpretation

- Pair value is `(a << 8) | b`. Ranges index its sorted global observed vocabulary (1,443 ranks), rather than intervals of bitmap integers. Globally unobserved pairs still prove absence.
- Stored ranges are zeros (proven absence). Dropping a range fills only zero bits with ones. `internal T` drops runs of length ≤T except edge runs. `all T` also drops edge runs. `longest K` retains at most K longest zero runs, including edges, with ties resolved by lower start rank. No runs are borrowed from another block or phase.
- Ordinary masks combine all remainder offsets. Parity stores even and odd as two independent rank-domain masks. Remainder offset zero defines even. Prefix-contained and crossing matches use the existing conservative `permits` logic and decoded full prefixes.
- PR01: 4-byte magic, little-endian u32 block count, u32 vocabulary count, u8 phase count, u8 layout flag. Then sorted u16 vocabulary. Each phase stores sequential block records, or u32 unique-record count + u8 ID width + unique records + one fixed-width little-endian ID per block. Each record is unsigned-varint run count, followed by (start minus previous end, length) varint pairs. Previous end starts at zero. Every header, count, ID, and vocabulary byte is charged.
- Records are self-framed and phase-major. There are **no serialized random-access offsets**. Load all metadata and construct in-memory masks. Optional `.zst` wraps the entire PR01 stream with zstd level 3 and requires full-file decompression. Reported compressed sizes are disk bytes, not decoded memory or latency. JSON includes each uncompressed stream size. Runtime allocations and production-container overhead are excluded.
- Every total includes the 48-byte prefix extension table and 20,307-byte remainder-byte sidecar exactly once. Existing index and extra budget are each 195,640 bytes. Existing bounds reconstruct prefix bases. Copied shared sidecars and each chosen range file form a complete experiment layout.
- All 76 candidate rules are built and their four layouts serialized before queries are evaluated. Layout choice minimizes actual stored bytes. Configuration choice: For each phase count, minimize filled zero bits among layouts within budget. Break ties by serialized size then ID. Queries play no role.
- Exact per-block decisions match prior exact ordinary/parity summaries. All decoded masks from all four layouts equal build-time masks and supersets of their original phase masks. Seven query checks use previously independently scanned per-block full-term truth. No fresh term scan was performed.
- Exhaustive unit tests cover every mask through eight ranks with every threshold/K through domain size + 1, all 256 two-phase combinations on four ranks across all configured rules and both layouts, all 128 sets of binary remainders through length two, and all 120 pairs of binary remainders through length three. Query tests cover four prefixes, every ternary literal of length 1–3, both substring/suffix semantics, and both phase counts. Additional tests cover unknown global pairs, varint boundaries, parity padding, multi-byte IDs, compression roundtrips, and malformed streams. See `tests.log` for the seven-test run.

## Storage-selected candidates

| Method | Feature B | Total B | Full stream B | Layout |
| --- | --- | --- | --- | --- |
| ordinary_internal_3 | 168419 | 188774 | 278826 | ordinary_internal_3_dedup.bin.zst |
| parity_internal_10 | 158340 | 178695 | 238780 | parity_internal_10_dedup.bin.zst |

## Under-budget alternatives without whole-file compression

Same storage-only choice rule, restricted to raw PR01 streams. These still load sequential metadata into memory.

| Method | Feature B | Total B | Layout |
| --- | --- | --- | --- |
| ordinary_internal_5 | 149335 | 169690 | ordinary_internal_5_dedup.bin |
| parity_internal_14 | 170569 | 190924 | parity_internal_14_dedup.bin |

## Existing prefix-excluded baselines

| Method | Total B | *a* | *ab* | *traa* | *X | *x | *player* | *ms* |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bytes | 20355 | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| pairs | 1469199 | 418 (2.49%) | 1281 (7.63%) | 4976 (29.63%) | 16794 (100.00%) | 440 (2.62%) | 6990 (41.62%) | 2710 (16.14%) |
| parity | 4610703 | 418 (2.49%) | 1281 (7.63%) | 7319 (43.58%) | 16794 (100.00%) | 440 (2.62%) | 11176 (66.55%) | 2710 (16.14%) |

## All rules: smallest actual serialized layout and blocks skipped

Each skip cell is count (percent). Counts include the shared byte summary. Exact rows are lossless range baselines.

| Method | Total B | Budget | *a* | *ab* | *traa* | *X | *x | *player* | *ms* |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ordinary_exact_0 | 2046710 | no | 418 (2.49%) | 1281 (7.63%) | 4976 (29.63%) | 16794 (100.00%) | 440 (2.62%) | 6990 (41.62%) | 2710 (16.14%) |
| ordinary_internal_1 | 713849 | no | 418 (2.49%) | 590 (3.51%) | 4104 (24.44%) | 16794 (100.00%) | 440 (2.62%) | 4253 (25.32%) | 1652 (9.84%) |
| ordinary_internal_2 | 324999 | no | 418 (2.49%) | 542 (3.23%) | 2169 (12.92%) | 16794 (100.00%) | 440 (2.62%) | 2688 (16.01%) | 1258 (7.49%) |
| ordinary_internal_3 | 188774 | yes | 418 (2.49%) | 483 (2.88%) | 1545 (9.20%) | 16794 (100.00%) | 440 (2.62%) | 1924 (11.46%) | 1062 (6.32%) |
| ordinary_internal_4 | 137879 | yes | 418 (2.49%) | 462 (2.75%) | 1242 (7.40%) | 16794 (100.00%) | 440 (2.62%) | 1529 (9.10%) | 948 (5.64%) |
| ordinary_internal_5 | 109023 | yes | 418 (2.49%) | 457 (2.72%) | 1094 (6.51%) | 16794 (100.00%) | 440 (2.62%) | 1299 (7.73%) | 865 (5.15%) |
| ordinary_internal_6 | 92081 | yes | 418 (2.49%) | 452 (2.69%) | 1008 (6.00%) | 16794 (100.00%) | 440 (2.62%) | 1141 (6.79%) | 809 (4.82%) |
| ordinary_internal_7 | 80413 | yes | 418 (2.49%) | 447 (2.66%) | 941 (5.60%) | 16794 (100.00%) | 440 (2.62%) | 1047 (6.23%) | 767 (4.57%) |
| ordinary_internal_8 | 72350 | yes | 418 (2.49%) | 445 (2.65%) | 887 (5.28%) | 16794 (100.00%) | 440 (2.62%) | 978 (5.82%) | 730 (4.35%) |
| ordinary_internal_10 | 62103 | yes | 418 (2.49%) | 441 (2.63%) | 821 (4.89%) | 16794 (100.00%) | 440 (2.62%) | 910 (5.42%) | 691 (4.11%) |
| ordinary_internal_12 | 56222 | yes | 418 (2.49%) | 437 (2.60%) | 769 (4.58%) | 16794 (100.00%) | 440 (2.62%) | 849 (5.06%) | 657 (3.91%) |
| ordinary_internal_14 | 52205 | yes | 418 (2.49%) | 437 (2.60%) | 731 (4.35%) | 16794 (100.00%) | 440 (2.62%) | 801 (4.77%) | 624 (3.72%) |
| ordinary_internal_16 | 49675 | yes | 418 (2.49%) | 434 (2.58%) | 687 (4.09%) | 16794 (100.00%) | 440 (2.62%) | 761 (4.53%) | 601 (3.58%) |
| ordinary_internal_32 | 38695 | yes | 418 (2.49%) | 430 (2.56%) | 521 (3.10%) | 16794 (100.00%) | 440 (2.62%) | 562 (3.35%) | 506 (3.01%) |
| ordinary_internal_64 | 31062 | yes | 418 (2.49%) | 423 (2.52%) | 468 (2.79%) | 16794 (100.00%) | 440 (2.62%) | 476 (2.83%) | 458 (2.73%) |
| ordinary_internal_128 | 30599 | yes | 418 (2.49%) | 421 (2.51%) | 459 (2.73%) | 16794 (100.00%) | 440 (2.62%) | 466 (2.77%) | 455 (2.71%) |
| ordinary_all_1 | 711961 | no | 418 (2.49%) | 590 (3.51%) | 4104 (24.44%) | 16794 (100.00%) | 440 (2.62%) | 4253 (25.32%) | 1652 (9.84%) |
| ordinary_all_2 | 322470 | no | 418 (2.49%) | 542 (3.23%) | 2169 (12.92%) | 16794 (100.00%) | 440 (2.62%) | 2688 (16.01%) | 1258 (7.49%) |
| ordinary_all_3 | 185260 | yes | 418 (2.49%) | 483 (2.88%) | 1545 (9.20%) | 16794 (100.00%) | 440 (2.62%) | 1924 (11.46%) | 1062 (6.32%) |
| ordinary_all_4 | 134299 | yes | 418 (2.49%) | 462 (2.75%) | 1242 (7.40%) | 16794 (100.00%) | 440 (2.62%) | 1529 (9.10%) | 948 (5.64%) |
| ordinary_all_5 | 105503 | yes | 418 (2.49%) | 457 (2.72%) | 1094 (6.51%) | 16794 (100.00%) | 440 (2.62%) | 1299 (7.73%) | 865 (5.15%) |
| ordinary_all_6 | 88537 | yes | 418 (2.49%) | 452 (2.69%) | 1008 (6.00%) | 16794 (100.00%) | 440 (2.62%) | 1141 (6.79%) | 809 (4.82%) |
| ordinary_all_7 | 77056 | yes | 418 (2.49%) | 447 (2.66%) | 941 (5.60%) | 16794 (100.00%) | 440 (2.62%) | 1047 (6.23%) | 767 (4.57%) |
| ordinary_all_8 | 69021 | yes | 418 (2.49%) | 445 (2.65%) | 887 (5.28%) | 16794 (100.00%) | 440 (2.62%) | 978 (5.82%) | 730 (4.35%) |
| ordinary_all_10 | 59105 | yes | 418 (2.49%) | 441 (2.63%) | 821 (4.89%) | 16794 (100.00%) | 440 (2.62%) | 910 (5.42%) | 691 (4.11%) |
| ordinary_all_12 | 52812 | yes | 418 (2.49%) | 437 (2.60%) | 769 (4.58%) | 16794 (100.00%) | 440 (2.62%) | 849 (5.06%) | 657 (3.91%) |
| ordinary_all_14 | 48261 | yes | 418 (2.49%) | 437 (2.60%) | 731 (4.35%) | 16794 (100.00%) | 440 (2.62%) | 801 (4.77%) | 624 (3.72%) |
| ordinary_all_16 | 45345 | yes | 418 (2.49%) | 434 (2.58%) | 687 (4.09%) | 16794 (100.00%) | 440 (2.62%) | 761 (4.53%) | 601 (3.58%) |
| ordinary_all_32 | 34129 | yes | 418 (2.49%) | 430 (2.56%) | 521 (3.10%) | 16794 (100.00%) | 440 (2.62%) | 562 (3.35%) | 506 (3.01%) |
| ordinary_all_64 | 23692 | yes | 418 (2.49%) | 423 (2.52%) | 468 (2.79%) | 16794 (100.00%) | 440 (2.62%) | 475 (2.83%) | 458 (2.73%) |
| ordinary_all_128 | 23133 | yes | 418 (2.49%) | 421 (2.51%) | 459 (2.73%) | 16794 (100.00%) | 440 (2.62%) | 464 (2.76%) | 455 (2.71%) |
| ordinary_longest_0 | 22378 | yes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| ordinary_longest_1 | 27553 | yes | 418 (2.49%) | 424 (2.52%) | 479 (2.85%) | 16794 (100.00%) | 440 (2.62%) | 498 (2.97%) | 465 (2.77%) |
| ordinary_longest_2 | 33429 | yes | 418 (2.49%) | 426 (2.54%) | 491 (2.92%) | 16794 (100.00%) | 440 (2.62%) | 524 (3.12%) | 479 (2.85%) |
| ordinary_longest_4 | 98606 | yes | 418 (2.49%) | 472 (2.81%) | 755 (4.50%) | 16794 (100.00%) | 440 (2.62%) | 858 (5.11%) | 541 (3.22%) |
| ordinary_longest_8 | 166803 | yes | 418 (2.49%) | 494 (2.94%) | 1052 (6.26%) | 16794 (100.00%) | 440 (2.62%) | 1275 (7.59%) | 651 (3.88%) |
| ordinary_longest_16 | 274723 | no | 418 (2.49%) | 532 (3.17%) | 1921 (11.44%) | 16794 (100.00%) | 440 (2.62%) | 1978 (11.78%) | 798 (4.75%) |
| ordinary_longest_32 | 598432 | no | 418 (2.49%) | 552 (3.29%) | 3125 (18.61%) | 16794 (100.00%) | 440 (2.62%) | 3329 (19.82%) | 1066 (6.35%) |
| parity_exact_0 | 6567036 | no | 418 (2.49%) | 1281 (7.63%) | 7319 (43.58%) | 16794 (100.00%) | 440 (2.62%) | 11176 (66.55%) | 2710 (16.14%) |
| parity_internal_1 | 3403264 | no | 418 (2.49%) | 644 (3.83%) | 6127 (36.48%) | 16794 (100.00%) | 440 (2.62%) | 7387 (43.99%) | 1830 (10.90%) |
| parity_internal_2 | 1758250 | no | 418 (2.49%) | 569 (3.39%) | 3277 (19.51%) | 16794 (100.00%) | 440 (2.62%) | 4500 (26.80%) | 1407 (8.38%) |
| parity_internal_3 | 1014909 | no | 418 (2.49%) | 497 (2.96%) | 2189 (13.03%) | 16794 (100.00%) | 440 (2.62%) | 3107 (18.50%) | 1175 (7.00%) |
| parity_internal_4 | 640630 | no | 418 (2.49%) | 468 (2.79%) | 1712 (10.19%) | 16794 (100.00%) | 440 (2.62%) | 2441 (14.53%) | 1050 (6.25%) |
| parity_internal_5 | 446470 | no | 418 (2.49%) | 463 (2.76%) | 1434 (8.54%) | 16794 (100.00%) | 440 (2.62%) | 2011 (11.97%) | 941 (5.60%) |
| parity_internal_6 | 336908 | no | 418 (2.49%) | 457 (2.72%) | 1257 (7.48%) | 16794 (100.00%) | 440 (2.62%) | 1709 (10.18%) | 871 (5.19%) |
| parity_internal_7 | 269430 | no | 418 (2.49%) | 451 (2.69%) | 1141 (6.79%) | 16794 (100.00%) | 440 (2.62%) | 1487 (8.85%) | 818 (4.87%) |
| parity_internal_8 | 228616 | no | 418 (2.49%) | 450 (2.68%) | 1046 (6.23%) | 16794 (100.00%) | 440 (2.62%) | 1329 (7.91%) | 778 (4.63%) |
| parity_internal_10 | 178695 | yes | 418 (2.49%) | 444 (2.64%) | 956 (5.69%) | 16794 (100.00%) | 440 (2.62%) | 1139 (6.78%) | 741 (4.41%) |
| parity_internal_12 | 147800 | yes | 418 (2.49%) | 442 (2.63%) | 891 (5.31%) | 16794 (100.00%) | 440 (2.62%) | 1014 (6.04%) | 692 (4.12%) |
| parity_internal_14 | 127747 | yes | 418 (2.49%) | 442 (2.63%) | 844 (5.03%) | 16794 (100.00%) | 440 (2.62%) | 954 (5.68%) | 657 (3.91%) |
| parity_internal_16 | 115357 | yes | 418 (2.49%) | 439 (2.61%) | 800 (4.76%) | 16794 (100.00%) | 440 (2.62%) | 898 (5.35%) | 641 (3.82%) |
| parity_internal_32 | 79411 | yes | 418 (2.49%) | 434 (2.58%) | 609 (3.63%) | 16794 (100.00%) | 440 (2.62%) | 665 (3.96%) | 532 (3.17%) |
| parity_internal_64 | 49795 | yes | 418 (2.49%) | 425 (2.53%) | 486 (2.89%) | 16794 (100.00%) | 440 (2.62%) | 515 (3.07%) | 464 (2.76%) |
| parity_internal_128 | 45921 | yes | 418 (2.49%) | 421 (2.51%) | 463 (2.76%) | 16794 (100.00%) | 440 (2.62%) | 474 (2.82%) | 455 (2.71%) |
| parity_all_1 | 3393740 | no | 418 (2.49%) | 644 (3.83%) | 6127 (36.48%) | 16794 (100.00%) | 440 (2.62%) | 7387 (43.99%) | 1830 (10.90%) |
| parity_all_2 | 1744911 | no | 418 (2.49%) | 569 (3.39%) | 3277 (19.51%) | 16794 (100.00%) | 440 (2.62%) | 4500 (26.80%) | 1407 (8.38%) |
| parity_all_3 | 997458 | no | 418 (2.49%) | 497 (2.96%) | 2189 (13.03%) | 16794 (100.00%) | 440 (2.62%) | 3107 (18.50%) | 1175 (7.00%) |
| parity_all_4 | 620936 | no | 418 (2.49%) | 468 (2.79%) | 1712 (10.19%) | 16794 (100.00%) | 440 (2.62%) | 2441 (14.53%) | 1050 (6.25%) |
| parity_all_5 | 429819 | no | 418 (2.49%) | 463 (2.76%) | 1434 (8.54%) | 16794 (100.00%) | 440 (2.62%) | 2011 (11.97%) | 941 (5.60%) |
| parity_all_6 | 323396 | no | 418 (2.49%) | 457 (2.72%) | 1257 (7.48%) | 16794 (100.00%) | 440 (2.62%) | 1709 (10.18%) | 871 (5.19%) |
| parity_all_7 | 256250 | no | 418 (2.49%) | 451 (2.69%) | 1141 (6.79%) | 16794 (100.00%) | 440 (2.62%) | 1487 (8.85%) | 818 (4.87%) |
| parity_all_8 | 214667 | no | 418 (2.49%) | 450 (2.68%) | 1046 (6.23%) | 16794 (100.00%) | 440 (2.62%) | 1329 (7.91%) | 778 (4.63%) |
| parity_all_10 | 164402 | yes | 418 (2.49%) | 444 (2.64%) | 956 (5.69%) | 16794 (100.00%) | 440 (2.62%) | 1139 (6.78%) | 741 (4.41%) |
| parity_all_12 | 134102 | yes | 418 (2.49%) | 442 (2.63%) | 891 (5.31%) | 16794 (100.00%) | 440 (2.62%) | 1014 (6.04%) | 692 (4.12%) |
| parity_all_14 | 113377 | yes | 418 (2.49%) | 442 (2.63%) | 844 (5.03%) | 16794 (100.00%) | 440 (2.62%) | 954 (5.68%) | 657 (3.91%) |
| parity_all_16 | 101508 | yes | 418 (2.49%) | 439 (2.61%) | 800 (4.76%) | 16794 (100.00%) | 440 (2.62%) | 898 (5.35%) | 641 (3.82%) |
| parity_all_32 | 64028 | yes | 418 (2.49%) | 434 (2.58%) | 609 (3.63%) | 16794 (100.00%) | 440 (2.62%) | 665 (3.96%) | 532 (3.17%) |
| parity_all_64 | 27464 | yes | 418 (2.49%) | 425 (2.53%) | 486 (2.89%) | 16794 (100.00%) | 440 (2.62%) | 515 (3.07%) | 464 (2.76%) |
| parity_all_128 | 24246 | yes | 418 (2.49%) | 421 (2.51%) | 463 (2.76%) | 16794 (100.00%) | 440 (2.62%) | 466 (2.77%) | 455 (2.71%) |
| parity_longest_0 | 22378 | yes | 418 (2.49%) | 421 (2.51%) | 453 (2.70%) | 16794 (100.00%) | 440 (2.62%) | 463 (2.76%) | 455 (2.71%) |
| parity_longest_1 | 39143 | yes | 418 (2.49%) | 423 (2.52%) | 464 (2.76%) | 16794 (100.00%) | 440 (2.62%) | 481 (2.86%) | 458 (2.73%) |
| parity_longest_2 | 57134 | yes | 418 (2.49%) | 424 (2.52%) | 485 (2.89%) | 16794 (100.00%) | 440 (2.62%) | 525 (3.13%) | 462 (2.75%) |
| parity_longest_4 | 234523 | no | 418 (2.49%) | 429 (2.55%) | 536 (3.19%) | 16794 (100.00%) | 440 (2.62%) | 626 (3.73%) | 488 (2.91%) |
| parity_longest_8 | 473724 | no | 418 (2.49%) | 445 (2.65%) | 729 (4.34%) | 16794 (100.00%) | 440 (2.62%) | 954 (5.68%) | 547 (3.26%) |
| parity_longest_16 | 837587 | no | 418 (2.49%) | 484 (2.88%) | 1163 (6.93%) | 16794 (100.00%) | 440 (2.62%) | 1751 (10.43%) | 648 (3.86%) |
| parity_longest_32 | 1430970 | no | 418 (2.49%) | 533 (3.17%) | 2070 (12.33%) | 16794 (100.00%) | 440 (2.62%) | 3153 (18.77%) | 823 (4.90%) |

## Validation

```json
{
  "decoded_block_phase_superset_checks": 7658064,
  "serialized_layouts": 304,
  "all_layouts_roundtripped": true,
  "exact_decisions_match_previous_per_block": true,
  "false_negative_blocks": 0,
  "truth_source": "Previously independently scanned full-term per-block counts in prefix_summaries/actual/blocks.jsonl. No fresh full-term scan."
}
```

## Reproduce

```sh
python3 -m unittest discover -s dictionary_analysis -p test_pair_runs.py
python3 dictionary_analysis/pair_runs.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

Requires Python 3 and the zstd executable. JSON records all four actual layout sizes per rule, selected artifact paths, full seven-query counts, false positives, and compressed-data-byte skip percentages. Alternate layouts and diagnostics are experiment outputs rather than cumulative required metadata.
