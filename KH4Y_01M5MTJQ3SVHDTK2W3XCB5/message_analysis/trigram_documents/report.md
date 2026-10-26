# Exact document-frequency trigram selection: original baseline

Ranking uses exact distinct-document unions across **full raw-byte terms**. Stored block postings continue to summarize **full-LCP-excluded remainders**. Case-sensitive within-term trigrams, per-term deduplication, no interterm grams. The candidate universe remains all 47,796 original remainder rows. Source has 10,002,000 documents and no deletes. Synthetic key-only dictionaries have no document assignments and are excluded.

**Result:** caps 0–92% retain `pla`, `lay`, `aye`, `yer` and reproduce full-trigram `player` pruning: 14 candidates / 99.9166% skipped. At 94%, only `pla` and `lay` remain, giving 134 candidates / 99.2021% skipped. At 95–99%, all four are ineligible and `player` falls back to bytes (16,331 candidates). `tra` and `raa` are never retained, so `traa` remains at 16,341 byte-only candidates for every cap. The 80% cap improves none of these seven queries over the uncapped document-DF policy.

Eligible iff blockDF*100 <= N*(100-cutoff). Descending exact documentDF, ascending u24 ties. Skip nonfitting rows and continue.

The 1,000,000 B added-metadata ceiling is +511.14% of the original 195,640 B index. Each total includes 5,275 B inverted bytes + 48 B prefix extensions + the actual IU01 file. Header/directory = 18 + 7×rows. Frame bytes are copied unchanged. Omitted observed and unobserved grams are unknown. All query evaluation uses the existing prefix/continuation-safe permits callback. All eleven files have every retained posting decoded and compared. All 154 query/mode evaluations have zero false-negative blocks.

## Costs and retention

| Min block skip % | Eligible | Rows | Directory B | Payload B | Partial B | Total B | +index % | Unused B | Query grams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 47796 | 1207 | 8467 | 986209 | 994676 | 999999 | 511.1424 | 1 | aye, lay, pla, yer |
| 50 | 47796 | 1207 | 8467 | 986209 | 994676 | 999999 | 511.1424 | 1 | aye, lay, pla, yer |
| 70 | 47796 | 1207 | 8467 | 986209 | 994676 | 999999 | 511.1424 | 1 | aye, lay, pla, yer |
| 80 | 47198 | 1223 | 8579 | 986095 | 994674 | 999997 | 511.1414 | 3 | aye, lay, pla, yer |
| 85 | 46796 | 2022 | 14172 | 980505 | 994677 | 1000000 | 511.1429 | 0 | aye, lay, pla, yer |
| 90 | 43700 | 2063 | 14459 | 980218 | 994677 | 1000000 | 511.1429 | 0 | aye, lay, pla, yer |
| 92 | 43699 | 2071 | 14515 | 980162 | 994677 | 1000000 | 511.1429 | 0 | aye, lay, pla, yer |
| 94 | 21836 | 2114 | 14816 | 979859 | 994675 | 999998 | 511.1419 | 2 | lay, pla |
| 95 | 1140 | 1140 | 7998 | 44937 | 52935 | 58258 | 29.7782 | 941742 | none |
| 98 | 1122 | 1122 | 7872 | 41678 | 49550 | 54873 | 28.0479 | 945127 | none |
| 99 | 850 | 850 | 5968 | 15612 | 21580 | 26903 | 13.7513 | 973097 | none |

## Diagnostic grams

The four `player` grams have roughly 5.9–6.0 million distinct documents while occupying only 988–1,067 remainder blocks. They rise to global ranks 17–21. `tra`, `raa` and control `3ic` have roughly 1,300–1,650 documents and comparable block counts. Thus document DF separates repeated document text from these low-document-frequency background grams. `*player*` itself matches 5,884,377 documents through just 21 terms, with term-DF sum 13,655,986. That overlap illustrates why adding term DFs would misrepresent document frequency.

Global rank is before cap filtering. Per-cap eligible ranks and all rows are in selection_rows.jsonl. Term-DF sum is an upper bound and is never used for ranking.

| Gram | Document DF | Term-DF sum | Max term DF | Matching terms | Block DF | Row B | Global rank | Retained caps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| pla | 6026979 | 13801702 | 4817876 | 1259 | 988 | 758 | 17 | 0, 50, 70, 80, 85, 90, 92, 94 |
| lay | 5892760 | 13667504 | 4817876 | 1286 | 992 | 762 | 19 | 0, 50, 70, 80, 85, 90, 92, 94 |
| aye | 5915284 | 13695710 | 4817876 | 1301 | 1032 | 772 | 18 | 0, 50, 70, 80, 85, 90, 92 |
| yer | 5884479 | 13657298 | 4817876 | 1333 | 1067 | 804 | 21 | 0, 50, 70, 80, 85, 90, 92 |
| tra | 1653 | 1670 | 68 | 1325 | 1036 | 779 | 6213 | none |
| raa | 1292 | 1292 | 1 | 1292 | 1020 | 773 | 23974 | none |
| 3ic | 1256 | 1256 | 2 | 1255 | 974 | 755 | 38839 | none |

## All queries and references

| Query | Distinct documents | Distinct terms | Matching blocks | Bytes candidates | Full T candidates | Full T+B candidates |
| --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 | 0 | 16794 | 0 |
| *a* | 8993283 | 1921269 | 16376 | 16376 | 16794 | 16376 |
| *ab* | 1557602 | 75774 | 15490 | 16373 | 16794 | 16373 |
| *ms* | 5505831 | 50585 | 14065 | 16339 | 16794 | 16339 |
| *player* | 5884377 | 21 | 13 | 16331 | 14 | 14 |
| *traa* | 42 | 42 | 38 | 16341 | 141 | 141 |
| *x | 271995 | 153990 | 16022 | 16354 | 16794 | 16354 |

| Reference | Bytes | +index % |
| --- | --- | --- |
| bytes | 5323 | 2.7208 |
| trigrams | 38068570 | 19458.4799 |
| trigrams_bytes | 38073845 | 19461.1761 |

Cells below give candidates / skipped blocks / skip percent. Both partial modes are charged the mandatory-byte budget.

### trigrams

| Cap | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 50 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 70 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 80 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 85 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 90 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 92 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 14 / 16780 / 99.9166% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 94 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 134 / 16660 / 99.2021% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 95 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 98 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |
| 99 | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% | 16794 / 0 / 0.0000% |

### trigrams_bytes

| Cap | *X | *a* | *ab* | *ms* | *player* | *traa* | *x |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 50 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 70 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 80 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 85 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 90 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 92 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 14 / 16780 / 99.9166% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 94 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 134 / 16660 / 99.2021% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 95 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 16331 / 463 / 2.7569% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 98 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 16331 / 463 / 2.7569% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |
| 99 | 0 / 16794 / 100.0000% | 16376 / 418 / 2.4890% | 16373 / 421 / 2.5068% | 16339 / 455 / 2.7093% | 16331 / 463 / 2.7569% | 16341 / 453 / 2.6974% | 16354 / 440 / 2.6200% |

## Actual previous block-DF policy comparison

The previous policy retained none of the six query grams at all eleven baseline caps. Every combined query result equaled bytes-only. These are the recorded previous runs, rather than a hypothetical ordering.

| Cap | Old rows | Old total B | Old player candidates | New player candidates | Old traa candidates | New traa candidates |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 691 | 1000000 | 16331 | 14 | 16341 | 16341 |
| 50 | 691 | 1000000 | 16331 | 14 | 16341 | 16341 |
| 70 | 691 | 1000000 | 16331 | 14 | 16341 | 16341 |
| 80 | 733 | 999999 | 16331 | 14 | 16341 | 16341 |
| 85 | 792 | 999992 | 16331 | 14 | 16341 | 16341 |
| 90 | 1230 | 1000000 | 16331 | 14 | 16341 | 16341 |
| 92 | 1229 | 999997 | 16331 | 14 | 16341 | 16341 |
| 94 | 1298 | 999997 | 16331 | 134 | 16341 | 16341 |
| 95 | 1140 | 58258 | 16331 | 16331 | 16341 | 16341 |
| 98 | 1122 | 54873 | 16331 | 16331 | 16341 | 16341 |
| 99 | 850 | 26903 | 16331 | 16331 | 16341 | 16341 |

## Top-ranked retained examples

- 0%: `202` (docDF 8742968, blockDF 3550, 1453 B), `026` (docDF 8739631, blockDF 3342, 1442 B), `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B)
- 50%: `202` (docDF 8742968, blockDF 3550, 1453 B), `026` (docDF 8739631, blockDF 3342, 1442 B), `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B)
- 70%: `202` (docDF 8742968, blockDF 3550, 1453 B), `026` (docDF 8739631, blockDF 3342, 1442 B), `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B)
- 80%: `026` (docDF 8739631, blockDF 3342, 1442 B), `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B), `ate` (docDF 7681620, blockDF 1028, 782 B)
- 85%: `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B), `ate` (docDF 7681620, blockDF 1028, 782 B), `ame` (docDF 7193625, blockDF 998, 762 B)
- 90%: `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B), `ate` (docDF 7681620, blockDF 1028, 782 B), `ame` (docDF 7193625, blockDF 998, 762 B)
- 92%: `eve` (docDF 8175411, blockDF 943, 735 B), `vel` (docDF 8132820, blockDF 1039, 784 B), `lev` (docDF 8132817, blockDF 974, 752 B), `ate` (docDF 7681620, blockDF 1028, 782 B), `ame` (docDF 7193625, blockDF 998, 762 B)
- 94%: `eve` (docDF 8175411, blockDF 943, 735 B), `lev` (docDF 8132817, blockDF 974, 752 B), `ame` (docDF 7193625, blockDF 998, 762 B), `her` (docDF 6190075, blockDF 1003, 762 B), `las` (docDF 6092295, blockDF 972, 755 B)
- 95%: `.la` (docDF 6090565, blockDF 7, 26 B), `c.y` (docDF 5635089, blockDF 1, 14 B), `.y.` (docDF 5635087, blockDF 1, 14 B), `y.s` (docDF 5635049, blockDF 3, 18 B), `.s.` (docDF 5634701, blockDF 1, 14 B)
- 98%: `.la` (docDF 6090565, blockDF 7, 26 B), `c.y` (docDF 5635089, blockDF 1, 14 B), `.y.` (docDF 5635087, blockDF 1, 14 B), `y.s` (docDF 5635049, blockDF 3, 18 B), `.s.` (docDF 5634701, blockDF 1, 14 B)
- 99%: `.la` (docDF 6090565, blockDF 7, 26 B), `c.y` (docDF 5635089, blockDF 1, 14 B), `.y.` (docDF 5635087, blockDF 1, 14 B), `y.s` (docDF 5635049, blockDF 3, 18 B), `.s.` (docDF 5634701, blockDF 1, 14 B)

## Validation and resources

The helper uses the unchanged public Tantivy reader. A disposable isolated directory contains byte-identical copies of the original term/postings and generated auxiliary files solely to satisfy SegmentReader opening. Store, fast fields, field norms and positions are never queried or claimed as source data. The real position-record schema governs postings decoding. Basic requests skip term-frequency/position retrieval. Both composites pass provenance SHA-256 and managed CRC validation. The original SSTable slice is byte-identical. All physical addresses, ordinals, first/last keys, all 7,671,280 full keys, term posting counts, document ordering/bounds, contiguous posting ranges and the 8 B token-count prefix are checked. All source input hashes are rechecked after extraction and selection.

Document memberships are decoded once and cached as u32 IDs. Gram-to-term lists use per-term deduplication. One reusable document bitset computes exact unions, including overlap across different terms. Document DF is checked against max matching term DF, wide term-DF sum and max_doc. Diagnostic outputs are excluded from deployable cost. External /usr/bin/time resource logs record peak RSS.

Extraction resource counters:

```json
{
  "block_posting_lists": 3061,
  "decoded_document_memberships": 298159215,
  "docs_capacity_bytes": 2147483648,
  "elapsed_seconds": 20.282874,
  "gram_index_bytes": 67108864,
  "gram_term_capacity_bytes": 476578192,
  "gram_term_memberships": 75901063,
  "offsets_capacity_bytes": 67108864,
  "short_posting_lists": 7668219,
  "source_validation_and_decode_seconds": 10.709950792,
  "union_bitset_bytes": 1250256,
  "union_document_visits": 1399637604
}
```

Selection/readback/evaluation: 7.521 seconds.

External resources and artifact storage (populated by --report-only after timed processes exit):

```json
{
  "extract": {
    "wall_seconds": 20.79,
    "peak_rss_bytes": 3126657024
  },
  "selection": {
    "wall_seconds": 7.6,
    "peak_rss_bytes": 352387072
  },
  "baseline_artifact_bytes": 10726912,
  "document_frequencies_bytes": 5708172,
  "selection_rows_bytes": 20763663,
  "temporary_source_copy_bytes": 1021100307,
  "temporary_reader_directory_removed": true
}
```

Two complete extraction/selection runs reproduced 26 identical frequency/selection/serialized-decision artifacts. See reproducibility.json.

## Limits and failures

This is single-segment field DF. No deleted-document ambiguity exists. These seven queries support workload-specific comparison, rather than a universal optimal cap. No query information enters construction. Short predicates still depend on bytes. A large document DF can still represent a random-ID background. A strict cap can exclude useful grams or exhaust eligibility. Resource/run logs and tests are retained beside this report. See failures.md for any implementation/run failures.
