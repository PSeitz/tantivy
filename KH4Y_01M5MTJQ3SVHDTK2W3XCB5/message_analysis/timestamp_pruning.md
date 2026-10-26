# Numeric timestamps and byte pruning

Original message dictionary and unchanged physical blocks. Complete common prefixes excluded. No caller patterns.

Canonical ASCII decimal integer in [2000-01-01 UTC, 2035-01-01 UTC), interpreted as Unix seconds, milliseconds, microseconds or nanoseconds. Numeric IDs may qualify. Timestamps outside this interval or in other representations are not counted.

Scanned **7,671,280 terms** in **16,794 blocks**. Found **22,635 timestamp candidates (0.2951%)**, among **1,289,229 digit-only terms**.

Timestamp semantics are not available from dictionary keys alone. Counts are distinct terms, not documents or occurrences. The separate timestamp schema field is not evidence that the message dictionary contains its values.

| Interpretation | Candidate terms | Blocks containing candidates |
|---|---:|---:|
| seconds | 22,433 | 271 |
| milliseconds | 201 | 18 |
| microseconds | 1 | 1 |
| nanoseconds | 0 | 0 |

## Blocks skipped by remainder-byte masks

Percentages use the blocks in each row as denominator. Original boundaries are unchanged. A dash means no blocks qualify.

| Subset | Blocks | *a* | *ab* | *traa* | *X | *x | *player* | *ms* |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| all | 16,794 | 2.49% | 2.51% | 2.70% | 100.00% | 2.62% | 2.76% | 2.71% |
| has_candidate | 271 | 0.37% | 0.74% | 5.17% | 100.00% | 2.58% | 5.90% | 5.90% |
| no_candidate | 16,523 | 2.52% | 2.54% | 2.66% | 100.00% | 2.62% | 2.71% | 2.66% |
| candidate_majority | 0 | — | — | — | — | — | — | — |
| candidate_90_percent | 0 | — | — | — | — | — | — | — |
| candidate_only | 0 | — | — | — | — | — | — | — |
| numeric_majority | 1,387 | 0.00% | 0.00% | 1.08% | 100.00% | 0.43% | 1.37% | 1.08% |
| numeric_only | 0 | — | — | — | — | — | — | — |
| seconds | 271 | 0.37% | 0.74% | 5.17% | 100.00% | 2.58% | 5.90% | 5.90% |
| milliseconds | 18 | 0.00% | 0.00% | 0.00% | 100.00% | 0.00% | 0.00% | 0.00% |
| microseconds | 1 | 0.00% | 0.00% | 0.00% | 100.00% | 0.00% | 0.00% | 0.00% |
| nanoseconds | 0 | — | — | — | — | — | — | — |

## Comparison with exact block matches

| Subset | player: byte mask | player: oracle | traa: byte mask | traa: oracle |
|---|---:|---:|---:|---:|
| has_candidate | 5.90% | 100.00% | 5.17% | 100.00% |
| candidate_majority | — | — | — | — |
| candidate_90_percent | — | — | — | — |
| numeric_majority | 1.37% | 100.00% | 1.08% | 100.00% |

## Most candidate-dense blocks

### Block 958: 491/1,262 candidates (38.91%)

```json
{
  "candidates": [
    {
      "term": "1792540679",
      "unit": "seconds"
    },
    {
      "term": "1792577531",
      "unit": "seconds"
    },
    {
      "term": "1792584752",
      "unit": "seconds"
    },
    {
      "term": "1792586987",
      "unit": "seconds"
    },
    {
      "term": "1792612773",
      "unit": "seconds"
    },
    {
      "term": "1792633460",
      "unit": "seconds"
    }
  ],
  "other_terms": [
    "179232f6641261e4e9049052e667c001",
    "179236d16e8a51b2a8f45bbe240506fb",
    "1792b6a7b73edc7d90ebe5cdbadb16dd",
    "1792e487c98a",
    "1792e9cee94f",
    "17930ts9lc5sq"
  ]
}
```

### Block 748: 180/1,206 candidates (14.93%)

```json
{
  "candidates": [
    {
      "term": "1200014056",
      "unit": "seconds"
    },
    {
      "term": "1200131776",
      "unit": "seconds"
    },
    {
      "term": "1200257186",
      "unit": "seconds"
    },
    {
      "term": "1200264266",
      "unit": "seconds"
    },
    {
      "term": "1200375584",
      "unit": "seconds"
    },
    {
      "term": "1200407975",
      "unit": "seconds"
    }
  ],
  "other_terms": [
    "120.99676434259696",
    "120.99684205067926",
    "120.99954177474979",
    "12002dc76a38d1646830adc6467f0cc0",
    "12002f053388",
    "120096c4"
  ]
}
```

### Block 1022: 160/1,142 candidates (14.01%)

```json
{
  "candidates": [
    {
      "term": "1992953986",
      "unit": "seconds"
    },
    {
      "term": "1993041267",
      "unit": "seconds"
    },
    {
      "term": "1993049121",
      "unit": "seconds"
    },
    {
      "term": "1993061525",
      "unit": "seconds"
    },
    {
      "term": "1993104688",
      "unit": "seconds"
    },
    {
      "term": "1993119981",
      "unit": "seconds"
    }
  ],
  "other_terms": [
    "19928ba705knx",
    "1992911b",
    "1992c6b5dfaeb5ef74d66d323db1dae1",
    "1992c93a",
    "1992e891",
    "1992f4kwvab4n"
  ]
}
```

## Interpretation

A block containing only canonical numeric timestamp terms can reject all seven current letter-based queries using byte presence. Prefix exclusion preserves this result because both prefix and remainder contain only digits. This is a property of numeric-only blocks, not a measured reblocking or timestamp-injection experiment.
Adding numeric terms to a fixed block does not remove letters already present. Improvement requires the resulting dictionary layout to contain more blocks with restricted alphabets. Repeated timestamp occurrences add no distinct keys. Distinct timestamps may create new blocks when rebuilding, and lexical interleaving with IDs determines whether those blocks stay numeric.

Existing shared remainder-byte masks and prefix extensions cost **20,355 B (+10.40%)**. This experiment adds no new pruning metadata. No changes were made to dictionary terms, boundaries, or production code.

Rescanned all terms. Numeric counts match extraction per block. Rebuilt remainder byte masks and verified all seven per-block decisions against prior validated results.

## Reproduce

```sh
python3 dictionary_analysis/timestamp_pruning.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```
