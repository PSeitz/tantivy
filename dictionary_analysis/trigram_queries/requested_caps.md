# Requested maximum block-presence limits

The user requested explicit **40%, 60%, 70%, and 80% maximum block-presence limits**. These are presence percentages, rather than minimum skipping percentages.

`requested_caps.py` separately constructs all four policies at each of five decimal-byte budgets (200000, 300000, 400000, 500000, 1000000). It uses the existing exact document-frequency ranking and serialization, then decodes every retained posting through `verify-subset`.

```sh
python3 dictionary_analysis/trigram_queries/requested_caps.py \
  KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

Outputs: `message_analysis/trigram_requested_caps/report.json` and 20 actual partial index files. The 5,323-byte byte/prefix fallback is charged once per deployment and reused from the existing baseline.

## Equivalence verification

The largest observed remainder-block frequency is **3,588 / 16,794 = 21.364773133262%**, for `217`. Consequently every requested limit admits all **47,796** observed rows. At each budget:

- Complete eligible and selected key lists agree across all four requested limits.
- The four serialized files are byte-identical.
- Every retained posting matches the complete trigram index.
- The files also match the independently evaluated references in `trigram_query_thresholds/report.json` byte-for-byte. Its full 42-query results therefore apply to each requested limit.

The recorded reference-report SHA identifies the query evaluation used. Existing source hashes are rechecked before and after construction. Missing rows remain unknown, and the reference evaluation preserves full-prefix continuations.

## Results

Average block skipping, equally weighted over the **34 present queries with at least three bytes**:

| Budget | Existing 20% presence limit | Each of 40%, 60%, 70%, 80% | Actual added B at each requested limit | +original index % |
| --- | ---: | ---: | ---: | ---: |
| 200 KB | 69.6226% | 69.6226% | 199,996 | 102.23% |
| 300 KB | 83.9472% | 83.9044% | 299,996 | 153.34% |
| 400 KB | 86.7808% | 86.7798% | 399,991 | 204.45% |
| 500 KB | 86.8004% | 86.8004% | 499,996 | 255.57% |
| 1 MB | 86.8046% | 89.0745% | 999,999 | 511.14% |

At 1 MB, `123` improves from 16,794 to 3,809 candidate blocks. It is excluded by the 20% cap, remains budget-omitted at smaller relaxed budgets, and is retained at 1 MB. At that same budget, `success` regresses from 5 to 21 candidates and `message` from 7 to 15. At 300 KB, `server` regresses from 19 to 138 and `timeout` from 14 to 139. These changes arise because newly eligible high-document-frequency rows displace other rows under the fixed byte allowance.

`client` is unchanged: 1,070 candidates at 200–300 KB and 15 from 400 KB. All measured decisions have zero false negatives. Present/absent and short/long query groups are separated in the reference report.

This establishes equivalence and the measured trade-offs on the original dictionary and frozen cohort. Different dictionaries may have trigrams in more than 40–80% of blocks, so these limits need not be equivalent elsewhere. No production-latency or universal-threshold recommendation follows from these measurements.
