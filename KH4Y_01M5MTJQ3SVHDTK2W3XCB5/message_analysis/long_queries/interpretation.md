# Interpretation: longer-query benchmark

The 41 natural literals contain 14 present and 27 absent values. The 40 source-derived queries are all present. All eight sampled source terms are 32-byte lowercase hex strings. These groups answer different questions and remain separate.

## Natural-word examples

| Query | Present? | Terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| authentication | True | 1 | 1 | 16309 | 4 | 4 | 1 | 1 | 1 | 1 |
| configuration | True | 4 | 3 | 16309 | 10 | 3 | 3 | 3 | 3 | 3 |
| unavailable | True | 1 | 1 | 16317 | 16317 | 16317 | 1037 | 12 | 1 | 1 |
| heartbeat | True | 4 | 3 | 16332 | 16332 | 1048 | 3 | 3 | 3 | 3 |
| successful | True | 3 | 1 | 16333 | 21 | 21 | 21 | 21 | 5 | 1 |
| validation | True | 2 | 2 | 16313 | 7 | 7 | 7 | 7 | 7 | 3 |
| keepalive | False | 0 | 0 | 16322 | 16322 | 947 | 947 | 947 | 947 | 0 |
| healthcheck | False | 0 | 0 | 16320 | 1078 | 1078 | 108 | 108 | 108 | 0 |
| nullpointerexception | False | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 |

- `unavailable` retains no grams at 200/300 KB, so it equals byte fallback (16,317). At 400 KB it retains `ble` (1,037 candidates), at 500 KB `ail/abl/ble` (12), and at 1 MB also `ava` (1 true block).
- `heartbeat` retains nothing at 200 KB (16,332), only `art` at 300 KB (1,048), and all seven distinct grams at 400 KB (3 candidates, all true).
- Absent `keepalive` retains only `ive` at 300 KB through 1 MB (947 candidates). Absent `healthcheck` retains `che` at 200/300 KB (1,078) and `hea/che` from 400 KB (108). The full references reject all blocks for both. Missing features explain the partial/full gap.
- `validation` has 2 matching terms in 2 blocks, but full trigrams retain 3 blocks. Complete remainder postings are still necessary conditions, so cross-term gram co-occurrence and prefix continuations can leave false positives.

The retention facts above come from serialized-row lookups recorded per gram in report.json. Prefix-contained matches need no remainder grams. Crossing matches check only continuation grams. Retaining every whole-query gram is therefore not a prerequisite for useful safe pruning.

## Length benefit versus byte fallback

All eight families have fewer candidates at 32 bytes than at 8 bytes in every mode. Some reductions are only 1–6 blocks and also occur in byte fallback. The following extra diagnostic counts families whose absolute candidate reduction exceeds the byte-only reduction. It is a comparison of reductions, not a causal attribution or workload-weighted score.

| Mode | Families improving 8→32 | Families reduction exceeds bytes | Families gaining retained grams |
| --- | --- | --- | --- |
| bytes | 8 | 0 | n/a |
| document_200000 | 8 | 0 | 0 |
| document_300000 | 8 | 1 | 1 |
| document_400000 | 8 | 1 | 1 |
| document_500000 | 8 | 1 | 1 |
| document_1000000 | 8 | 7 | 6 |
| full_trigrams | 8 | 8 | n/a |

| Mode | 8-byte mean candidates | 12-byte mean candidates | 16-byte mean candidates | 24-byte mean candidates | 32-byte mean candidates |
| --- | --- | --- | --- | --- | --- |
| bytes | 16424.625 | 16370.125 | 16369.125 | 16368.125 | 16367.625 |
| document_200000 | 14762.5 | 14708.0 | 14707.0 | 14706.0 | 14705.5 |
| document_300000 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_400000 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_500000 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_1000000 | 7524.375 | 7259.5 | 7258.5 | 5348.875 | 3490.0 |
| full_trigrams | 158.125 | 1.875 | 1.0 | 1.0 | 1.0 |

### Illustrative family 0

| Query | Present? | Terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| a5b557df | True | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 15 |
| a5b557dfa94b | True | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 2 |
| a5b557dfa94b1f70 | True | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 1 |
| a5b557dfa94b1f70209f6a70 | True | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 3081 | 1 |
| a5b557dfa94b1f70209f6a70fbb39a5c | True | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 3080 | 1 |

Family 0 gains the retained `702` gram at length 24 in the 1 MB index. It drops from 16,368 candidates at length 16 to 3,081 at length 24. Length 32 has 3,080 candidates, while full trigrams reach the single true block at length 16. At 200–500 KB none of this family’s grams are retained at any tested length.

Family 6 gains no retained grams even at 1 MB and length 32. Its candidates are 16,369 / 16,367 / 16,367 / 16,367 / 16,367, identical to bytes. The full reference gives 21 / 2 / 1 / 1 / 1. Longer literals help only when usable features or byte constraints add information.

Every source-derived literal matches one dictionary term in one block. All full-reference families reach exactly one candidate by length 16. This ID-like diagnostic supports a conditional length benefit and does not predict production traffic.

## Actual full-LCP-excluded remainders

7,671,280 remainders, 75,510,607 raw bytes, mean 9.8433 bytes. Minimum 0, maximum 91, empty 1. Per-block length histograms and full prefixes are in remainders.jsonl. Complete aggregate histograms are in interpretation.json.

## Run history

The preserved first attempt reported a sampled ordinal as the total term count because of a variable-shadowing bug. The complete scan and physical-boundary checks had succeeded. A regression test caught the wrong report count. The corrected rerun reports 7,671,280 terms. Its cohort.json, truth.tsv, decisions.jsonl and remainders.jsonl are byte-identical to the preserved first attempt. See failed_attempt_1/failure.json and test/run logs.
