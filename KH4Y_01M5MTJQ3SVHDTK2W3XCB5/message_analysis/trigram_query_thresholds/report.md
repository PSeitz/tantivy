# Block-presence cap comparison: frozen 42 queries

Exact full-term distinct-document DF ranking, descending DF then ascending u24 ties. Whole rows require block DF × 100 ≤ cap × 16,794. Nonfitting rows are skipped. The builder uses minimum skip 80 / 79 / 0 for maximum presence 20% / 21% / uncapped. No query enters selection.

Actual metadata = 5,323 fixed byte/prefix bytes + serialized partial file. Reports, truth, decisions and helper memory are excluded. 16,794 physical blocks, 7,671,280 distinct terms, 10,002,000 documents.

## Primary matrix: 34 present literals of at least three bytes

Mean skip weights each query equally. The percentage column compares actual bytes with the original 195,640 B index.

| Budget B | Presence cap | Actual B | % of original | Rows | Eligible rows | Mean skip % | Queries ≥99% skip |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 20 | 199999 | 102.23 | 304 | 47198 | 69.6226 | 19 |
| 200000 | 21 | 200000 | 102.23 | 297 | 47763 | 69.6226 | 19 |
| 200000 | uncapped | 199996 | 102.23 | 306 | 47796 | 69.6226 | 19 |
| 300000 | 20 | 300000 | 153.34 | 476 | 47198 | 83.9472 | 24 |
| 300000 | 21 | 299993 | 153.34 | 419 | 47763 | 83.9263 | 24 |
| 300000 | uncapped | 299996 | 153.34 | 424 | 47796 | 83.9044 | 24 |
| 400000 | 20 | 399990 | 204.45 | 646 | 47198 | 86.7808 | 26 |
| 400000 | 21 | 399998 | 204.46 | 597 | 47763 | 86.7798 | 26 |
| 400000 | uncapped | 399991 | 204.45 | 621 | 47796 | 86.7798 | 26 |
| 500000 | 20 | 499997 | 255.57 | 828 | 47198 | 86.8004 | 26 |
| 500000 | 21 | 499995 | 255.57 | 786 | 47763 | 86.8004 | 26 |
| 500000 | uncapped | 499996 | 255.57 | 760 | 47796 | 86.8004 | 26 |
| 1000000 | 20 | 999997 | 511.14 | 1223 | 47198 | 86.8046 | 26 |
| 1000000 | 21 | 999999 | 511.14 | 1207 | 47763 | 89.0745 | 26 |
| 1000000 | uncapped | 999999 | 511.14 | 1207 | 47796 | 89.0745 | 26 |

## Why higher caps collapse to one policy

Global maximum row block DF is **3588 / 16794 = 21.364773133262%**, gram(s) `217`.

Caps 22%, 30%, 50%, 75%, 100% each admit all 47,796 rows. Complete eligible-key lists and selected-key lists were compared with uncapped independently for all five budgets. All are identical, including serialized size. Same keys copy the same original frames, so no duplicate index files are needed. JSON records SHA-256 of each eligible/selected key list.

| Cap | Eligible rows |
| --- | --- |
| 20 | 47198 |
| 21 | 47763 |
| uncapped | 47796 |
| 22 | 47796 |
| 30 | 47796 |
| 50 | 47796 |
| 75 | 47796 |
| 100 | 47796 |

## Same-budget query changes

Negative delta improves pruning. Positive delta worsens pruning. Every count is an absolute candidate-block count.

| Budget | Before cap | After cap | Query | Before candidates | After candidates | Delta |
| --- | --- | --- | --- | --- | --- | --- |
| 300000 | 20 | 21 | *server* | 19 | 138 | 119 |
| 300000 | 20 | uncapped | *timeout* | 14 | 139 | 125 |
| 300000 | 20 | uncapped | *server* | 19 | 138 | 119 |
| 300000 | 21 | uncapped | *timeout* | 14 | 139 | 125 |
| 400000 | 20 | 21 | *connection* | 6 | 9 | 3 |
| 400000 | 20 | 21 | *exception* | 7 | 10 | 3 |
| 400000 | 20 | uncapped | *connection* | 6 | 9 | 3 |
| 400000 | 20 | uncapped | *exception* | 7 | 10 | 3 |
| 1000000 | 20 | 21 | *success* | 5 | 21 | 16 |
| 1000000 | 20 | 21 | *message* | 7 | 15 | 8 |
| 1000000 | 20 | 21 | *123* | 16794 | 3809 | -12985 |
| 1000000 | 20 | uncapped | *success* | 5 | 21 | 16 |
| 1000000 | 20 | uncapped | *message* | 7 | 15 | 8 |
| 1000000 | 20 | uncapped | *123* | 16794 | 3809 | -12985 |

JSON also counts newly allowed and newly pruned block IDs, including equal-size candidate-set swaps.

## Requested examples

| Query | Budget | 20% candidates | 21% candidates | Uncapped candidates | True blocks |
| --- | --- | --- | --- | --- | --- |
| *123* | 200000 | 16794 | 16794 | 16794 | 3443 |
| *123* | 300000 | 16794 | 16794 | 16794 | 3443 |
| *123* | 400000 | 16794 | 16794 | 16794 | 3443 |
| *123* | 500000 | 16794 | 16794 | 16794 | 3443 |
| *123* | 1000000 | 16794 | 3809 | 3809 | 3443 |
| *000* | 200000 | 3708 | 3708 | 3708 | 3320 |
| *000* | 300000 | 3708 | 3708 | 3708 | 3320 |
| *000* | 400000 | 3708 | 3708 | 3708 | 3320 |
| *000* | 500000 | 3708 | 3708 | 3708 | 3320 |
| *000* | 1000000 | 3708 | 3708 | 3708 | 3320 |
| *client* | 200000 | 1070 | 1070 | 1070 | 15 |
| *client* | 300000 | 1070 | 1070 | 1070 | 15 |
| *client* | 400000 | 15 | 15 | 15 | 15 |
| *client* | 500000 | 15 | 15 | 15 | 15 |
| *client* | 1000000 | 15 | 15 | 15 | 15 |
| *error* | 200000 | 16341 | 16341 | 16341 | 6 |
| *error* | 300000 | 15 | 15 | 15 | 6 |
| *error* | 400000 | 15 | 15 | 15 | 6 |
| *error* | 500000 | 15 | 15 | 15 | 6 |
| *error* | 1000000 | 15 | 15 | 15 | 6 |
| *user* | 200000 | 16337 | 16337 | 16337 | 36 |
| *user* | 300000 | 143 | 143 | 143 | 36 |
| *user* | 400000 | 143 | 143 | 143 | 36 |
| *user* | 500000 | 143 | 143 | 143 | 36 |
| *user* | 1000000 | 143 | 143 | 143 | 36 |
| *server* | 200000 | 16334 | 16334 | 16334 | 6 |
| *server* | 300000 | 19 | 138 | 138 | 6 |
| *server* | 400000 | 19 | 19 | 19 | 6 |
| *server* | 500000 | 6 | 6 | 6 | 6 |
| *server* | 1000000 | 6 | 6 | 6 | 6 |
| *service* | 200000 | 16323 | 16323 | 16323 | 9 |
| *service* | 300000 | 9 | 9 | 9 | 9 |
| *service* | 400000 | 9 | 9 | 9 | 9 |
| *service* | 500000 | 9 | 9 | 9 | 9 |
| *service* | 1000000 | 9 | 9 | 9 | 9 |
| *trace* | 200000 | 16340 | 16340 | 16340 | 1 |
| *trace* | 300000 | 16340 | 16340 | 16340 | 1 |
| *trace* | 400000 | 2125 | 2125 | 2125 | 1 |
| *trace* | 500000 | 2125 | 2125 | 2125 | 1 |
| *trace* | 1000000 | 2125 | 2125 | 2125 | 1 |
| *traa* | 200000 | 16341 | 16341 | 16341 | 38 |
| *traa* | 300000 | 16341 | 16341 | 16341 | 38 |
| *traa* | 400000 | 16341 | 16341 | 16341 | 38 |
| *traa* | 500000 | 16341 | 16341 | 16341 | 38 |
| *traa* | 1000000 | 16341 | 16341 | 16341 | 38 |

## Retention and crowding

Relaxing a cap admits more rows into the same document-DF ordering. Newly eligible high-DF rows can consume space before useful query grams. The resulting whole-row budget packing can also exchange low-cost rows. An eligible omitted row is marked budget/DF-rank omission. A row above the cap is marked cap restriction. Both omissions mean unknown at lookup.

The following changed query grams show document DF, remainder block DF and incremental serialized cost (7 B directory entry + original posting frame). JSON includes every added/removed row for each same-budget comparison.

| Budget | Caps | Gram | Doc DF | Block DF | Cost B | Before | After |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 300000 | 20 → 21 | ver | 398413 | 990 | 763 | retained | budget/DF-rank omission |
| 300000 | 20 → uncapped | out | 438109 | 921 | 723 | retained | budget/DF-rank omission |
| 300000 | 20 → uncapped | ver | 398413 | 990 | 763 | retained | budget/DF-rank omission |
| 300000 | 21 → uncapped | out | 438109 | 921 | 723 | retained | budget/DF-rank omission |
| 400000 | 20 → 21 | cti | 198873 | 1062 | 795 | retained | budget/DF-rank omission |
| 400000 | 20 → 21 | exc | 210432 | 1019 | 773 | retained | budget/DF-rank omission |
| 400000 | 20 → 21 | xce | 210419 | 981 | 755 | retained | budget/DF-rank omission |
| 400000 | 20 → 21 | pti | 212167 | 1009 | 768 | retained | budget/DF-rank omission |
| 400000 | 20 → uncapped | cti | 198873 | 1062 | 795 | retained | budget/DF-rank omission |
| 400000 | 20 → uncapped | exc | 210432 | 1019 | 773 | retained | budget/DF-rank omission |
| 400000 | 20 → uncapped | xce | 210419 | 981 | 755 | retained | budget/DF-rank omission |
| 400000 | 20 → uncapped | pti | 212167 | 1009 | 768 | retained | budget/DF-rank omission |
| 1000000 | 20 → 21 | suc | 87917 | 1069 | 802 | retained | budget/DF-rank omission |
| 1000000 | 20 → 21 | ucc | 87806 | 963 | 744 | retained | budget/DF-rank omission |
| 1000000 | 20 → 21 | ssa | 86724 | 1041 | 781 | retained | budget/DF-rank omission |
| 1000000 | 20 → 21 | sag | 86770 | 1082 | 807 | retained | budget/DF-rank omission |
| 1000000 | 20 → 21 | 123 | 134374 | 3425 | 1411 | cap restriction | retained |
| 1000000 | 20 → uncapped | suc | 87917 | 1069 | 802 | retained | budget/DF-rank omission |
| 1000000 | 20 → uncapped | ucc | 87806 | 963 | 744 | retained | budget/DF-rank omission |
| 1000000 | 20 → uncapped | ssa | 86724 | 1041 | 781 | retained | budget/DF-rank omission |
| 1000000 | 20 → uncapped | sag | 86770 | 1082 | 807 | retained | budget/DF-rank omission |
| 1000000 | 20 → uncapped | 123 | 134374 | 3425 | 1411 | cap restriction | retained |

### Newly admitted rows selected ahead of displaced query grams

For each 20% → uncapped transition: five highest-document-DF added rows that were excluded by the old cap. Their selection shows the space competition. Whole-row packing determines the exact removed rows.

| Budget | Gram | Doc DF | Block DF | Cost B |
| --- | --- | --- | --- | --- |
| 200000 | 202 | 8742968 | 3550 | 1453 |
| 200000 | 100 | 1652858 | 3532 | 1445 |
| 200000 | 158 | 989598 | 3420 | 1415 |
| 200000 | 506 | 968134 | 3388 | 1443 |
| 200000 | 850 | 967733 | 3409 | 1426 |
| 300000 | 202 | 8742968 | 3550 | 1453 |
| 300000 | 100 | 1652858 | 3532 | 1445 |
| 300000 | 158 | 989598 | 3420 | 1415 |
| 300000 | 506 | 968134 | 3388 | 1443 |
| 300000 | 850 | 967733 | 3409 | 1426 |
| 400000 | 202 | 8742968 | 3550 | 1453 |
| 400000 | 100 | 1652858 | 3532 | 1445 |
| 400000 | 158 | 989598 | 3420 | 1415 |
| 400000 | 506 | 968134 | 3388 | 1443 |
| 400000 | 850 | 967733 | 3409 | 1426 |
| 500000 | 202 | 8742968 | 3550 | 1453 |
| 500000 | 100 | 1652858 | 3532 | 1445 |
| 500000 | 158 | 989598 | 3420 | 1415 |
| 500000 | 506 | 968134 | 3388 | 1443 |
| 500000 | 850 | 967733 | 3409 | 1426 |
| 1000000 | 202 | 8742968 | 3550 | 1453 |
| 1000000 | 100 | 1652858 | 3532 | 1445 |
| 1000000 | 158 | 989598 | 3420 | 1415 |
| 1000000 | 506 | 968134 | 3388 | 1443 |
| 1000000 | 850 | 967733 | 3409 | 1426 |

### Requested query gram omission reasons

Each policy cell gives gram=status. R=retained, C=cap restriction, B=budget/DF-rank omission, U=unobserved/unknown.

| Query | Budget | 20 | 21 | uncapped |
| --- | --- | --- | --- | --- |
| *123* | 200000 | 123=C | 123=B | 123=B |
| *123* | 300000 | 123=C | 123=B | 123=B |
| *123* | 400000 | 123=C | 123=B | 123=B |
| *123* | 500000 | 123=C | 123=B | 123=B |
| *123* | 1000000 | 123=C | 123=R | 123=R |
| *000* | 200000 | 000=R | 000=R | 000=R |
| *000* | 300000 | 000=R | 000=R | 000=R |
| *000* | 400000 | 000=R | 000=R | 000=R |
| *000* | 500000 | 000=R | 000=R | 000=R |
| *000* | 1000000 | 000=R | 000=R | 000=R |
| *client* | 200000 | cli=B, lie=B, ien=B, ent=R | cli=B, lie=B, ien=B, ent=R | cli=B, lie=B, ien=B, ent=R |
| *client* | 300000 | cli=B, lie=B, ien=B, ent=R | cli=B, lie=B, ien=B, ent=R | cli=B, lie=B, ien=B, ent=R |
| *client* | 400000 | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R |
| *client* | 500000 | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R |
| *client* | 1000000 | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R | cli=R, lie=R, ien=R, ent=R |
| *error* | 200000 | err=B, rro=B, ror=B | err=B, rro=B, ror=B | err=B, rro=B, ror=B |
| *error* | 300000 | err=R, rro=R, ror=R | err=R, rro=R, ror=R | err=R, rro=R, ror=R |
| *error* | 400000 | err=R, rro=R, ror=R | err=R, rro=R, ror=R | err=R, rro=R, ror=R |
| *error* | 500000 | err=R, rro=R, ror=R | err=R, rro=R, ror=R | err=R, rro=R, ror=R |
| *error* | 1000000 | err=R, rro=R, ror=R | err=R, rro=R, ror=R | err=R, rro=R, ror=R |
| *user* | 200000 | use=B, ser=B | use=B, ser=B | use=B, ser=B |
| *user* | 300000 | use=R, ser=R | use=R, ser=R | use=R, ser=R |
| *user* | 400000 | use=R, ser=R | use=R, ser=R | use=R, ser=R |
| *user* | 500000 | use=R, ser=R | use=R, ser=R | use=R, ser=R |
| *user* | 1000000 | use=R, ser=R | use=R, ser=R | use=R, ser=R |
| *server* | 200000 | ser=B, erv=B, rve=B, ver=B | ser=B, erv=B, rve=B, ver=B | ser=B, erv=B, rve=B, ver=B |
| *server* | 300000 | ser=R, erv=R, rve=B, ver=R | ser=R, erv=R, rve=B, ver=B | ser=R, erv=R, rve=B, ver=B |
| *server* | 400000 | ser=R, erv=R, rve=B, ver=R | ser=R, erv=R, rve=B, ver=R | ser=R, erv=R, rve=B, ver=R |
| *server* | 500000 | ser=R, erv=R, rve=R, ver=R | ser=R, erv=R, rve=R, ver=R | ser=R, erv=R, rve=R, ver=R |
| *server* | 1000000 | ser=R, erv=R, rve=R, ver=R | ser=R, erv=R, rve=R, ver=R | ser=R, erv=R, rve=R, ver=R |
| *service* | 200000 | ser=B, erv=B, rvi=B, vic=B, ice=B | ser=B, erv=B, rvi=B, vic=B, ice=B | ser=B, erv=B, rvi=B, vic=B, ice=B |
| *service* | 300000 | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R |
| *service* | 400000 | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R |
| *service* | 500000 | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R |
| *service* | 1000000 | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R | ser=R, erv=R, rvi=R, vic=R, ice=R |
| *trace* | 200000 | tra=B, rac=B, ace=B | tra=B, rac=B, ace=B | tra=B, rac=B, ace=B |
| *trace* | 300000 | tra=B, rac=B, ace=B | tra=B, rac=B, ace=B | tra=B, rac=B, ace=B |
| *trace* | 400000 | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R |
| *trace* | 500000 | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R |
| *trace* | 1000000 | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R | tra=B, rac=B, ace=R |
| *traa* | 200000 | tra=B, raa=B | tra=B, raa=B | tra=B, raa=B |
| *traa* | 300000 | tra=B, raa=B | tra=B, raa=B | tra=B, raa=B |
| *traa* | 400000 | tra=B, raa=B | tra=B, raa=B | tra=B, raa=B |
| *traa* | 500000 | tra=B, raa=B | tra=B, raa=B | tra=B, raa=B |
| *traa* | 1000000 | tra=B, raa=B | tra=B, raa=B | tra=B, raa=B |

## All 42 queries

Cells are candidates / skip% / false-positive blocks. All false-negative counts are zero. Matching terms and true blocks come from the same frozen independent scan.

| Query | Matching terms | True blocks | 200000/20 | 200000/21 | 200000/uncapped | 300000/20 | 300000/21 | 300000/uncapped | 400000/20 | 400000/21 | 400000/uncapped | 500000/20 | 500000/21 | 500000/uncapped | 1000000/20 | 1000000/21 | 1000000/uncapped |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 |
| *a* | 1921269 | 16376 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 |
| *ab* | 75774 | 15490 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 |
| *ms* | 50585 | 14065 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 |
| *player* | 21 | 13 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 |
| *traa* | 42 | 38 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 |
| *x | 153990 | 16022 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 |
| *error* | 7 | 6 | 16341 / 2.6974 / 16335 | 16341 / 2.6974 / 16335 | 16341 / 2.6974 / 16335 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 |
| *warning* | 0 | 0 | 1047 / 93.7656 / 1047 | 1047 / 93.7656 / 1047 | 1047 / 93.7656 / 1047 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 | 89 / 99.4700 / 89 |
| *info* | 42 | 35 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 |
| *debug* | 2 | 2 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 |
| *request* | 22 | 11 | 101 / 99.3986 / 90 | 101 / 99.3986 / 90 | 101 / 99.3986 / 90 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 |
| *response* | 11 | 4 | 97 / 99.4224 / 93 | 97 / 99.4224 / 93 | 97 / 99.4224 / 93 | 12 / 99.9285 / 8 | 12 / 99.9285 / 8 | 12 / 99.9285 / 8 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 |
| *timeout* | 6 | 5 | 139 / 99.1723 / 134 | 139 / 99.1723 / 134 | 139 / 99.1723 / 134 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 139 / 99.1723 / 134 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 | 14 / 99.9166 / 9 |
| *failed* | 2 | 1 | 996 / 94.0693 / 995 | 996 / 94.0693 / 995 | 996 / 94.0693 / 995 | 996 / 94.0693 / 995 | 996 / 94.0693 / 995 | 996 / 94.0693 / 995 | 100 / 99.4045 / 99 | 100 / 99.4045 / 99 | 100 / 99.4045 / 99 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 |
| *success* | 9 | 4 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 | 5 / 99.9702 / 1 | 21 / 99.8750 / 17 | 21 / 99.8750 / 17 |
| *user* | 42 | 36 | 16337 / 2.7212 / 16301 | 16337 / 2.7212 / 16301 | 16337 / 2.7212 / 16301 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 |
| *server* | 11 | 6 | 16334 / 2.7391 / 16328 | 16334 / 2.7391 / 16328 | 16334 / 2.7391 / 16328 | 19 / 99.8869 / 13 | 138 / 99.1783 / 132 | 138 / 99.1783 / 132 | 19 / 99.8869 / 13 | 19 / 99.8869 / 13 | 19 / 99.8869 / 13 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 |
| *client* | 25 | 15 | 1070 / 93.6287 / 1055 | 1070 / 93.6287 / 1055 | 1070 / 93.6287 / 1055 | 1070 / 93.6287 / 1055 | 1070 / 93.6287 / 1055 | 1070 / 93.6287 / 1055 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 |
| *session* | 4 | 2 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 |
| *connection* | 9 | 5 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 6 / 99.9643 / 1 | 9 / 99.9464 / 4 | 9 / 99.9464 / 4 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 |
| *login* | 0 | 0 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 |
| *trace* | 1 | 1 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 | 2125 / 87.3467 / 2124 |
| *cache* | 9 | 4 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 |
| *database* | 5 | 2 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 |
| *process* | 12 | 6 | 11 / 99.9345 / 5 | 11 / 99.9345 / 5 | 11 / 99.9345 / 5 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 |
| *message* | 13 | 7 | 146 / 99.1306 / 139 | 146 / 99.1306 / 139 | 146 / 99.1306 / 139 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 | 7 / 99.9583 / 0 | 15 / 99.9107 / 8 | 15 / 99.9107 / 8 |
| *status* | 9 | 7 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 |
| *service* | 27 | 9 | 16323 / 2.8046 / 16314 | 16323 / 2.8046 / 16314 | 16323 / 2.8046 / 16314 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 |
| *http* | 66 | 38 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 |
| *exception* | 14 | 7 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 7 / 99.9583 / 0 | 10 / 99.9405 / 3 | 10 / 99.9405 / 3 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 |
| *game* | 71 | 44 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 |
| *team* | 42 | 37 | 16336 / 2.7272 / 16299 | 16336 / 2.7272 / 16299 | 16336 / 2.7272 / 16299 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 |
| *match* | 4 | 2 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 |
| *time* | 66 | 51 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 |
| *name* | 50 | 47 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 |
| *world* | 1 | 1 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 |
| *3ic* | 1255 | 1008 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 |
| *a76* | 2871 | 2103 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 |
| *000* | 8333 | 3320 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 |
| *123* | 12372 | 3443 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 3809 / 77.3193 / 366 | 3809 / 77.3193 / 366 |
| *deadbeef* | 0 | 0 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 |

## Presence and literal-length groups

| Group | Queries | Budget/cap | Mean skip % | ≥99% skip | Candidate sum | FP sum |
| --- | --- | --- | --- | --- | --- | --- |
| present/one_two_bytes | 4 | 200000/20 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 200000/21 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 200000/uncapped | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 300000/20 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 300000/21 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 300000/uncapped | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 400000/20 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 400000/21 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 400000/uncapped | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 500000/20 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 500000/21 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 500000/uncapped | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 1000000/20 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 1000000/21 | 2.5813 | 0 | 65442 | 3489 |
| present/one_two_bytes | 4 | 1000000/uncapped | 2.5813 | 0 | 65442 | 3489 |
| present/at_least_three_bytes | 34 | 200000/20 | 69.6226 | 19 | 173454 | 163134 |
| present/at_least_three_bytes | 34 | 200000/21 | 69.6226 | 19 | 173454 | 163134 |
| present/at_least_three_bytes | 34 | 200000/uncapped | 69.6226 | 19 | 173454 | 163134 |
| present/at_least_three_bytes | 34 | 300000/20 | 83.9472 | 24 | 91661 | 81341 |
| present/at_least_three_bytes | 34 | 300000/21 | 83.9263 | 24 | 91780 | 81460 |
| present/at_least_three_bytes | 34 | 300000/uncapped | 83.9044 | 24 | 91905 | 81585 |
| present/at_least_three_bytes | 34 | 400000/20 | 86.7808 | 26 | 75481 | 65161 |
| present/at_least_three_bytes | 34 | 400000/21 | 86.7798 | 26 | 75487 | 65167 |
| present/at_least_three_bytes | 34 | 400000/uncapped | 86.7798 | 26 | 75487 | 65167 |
| present/at_least_three_bytes | 34 | 500000/20 | 86.8004 | 26 | 75369 | 65049 |
| present/at_least_three_bytes | 34 | 500000/21 | 86.8004 | 26 | 75369 | 65049 |
| present/at_least_three_bytes | 34 | 500000/uncapped | 86.8004 | 26 | 75369 | 65049 |
| present/at_least_three_bytes | 34 | 1000000/20 | 86.8046 | 26 | 75345 | 65025 |
| present/at_least_three_bytes | 34 | 1000000/21 | 89.0745 | 26 | 62384 | 52064 |
| present/at_least_three_bytes | 34 | 1000000/uncapped | 89.0745 | 26 | 62384 | 52064 |
| absent/one_two_bytes | 1 | 200000/20 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 200000/21 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 200000/uncapped | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 300000/20 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 300000/21 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 300000/uncapped | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 400000/20 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 400000/21 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 400000/uncapped | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 500000/20 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 500000/21 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 500000/uncapped | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 1000000/20 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 1000000/21 | 100.0000 | 1 | 0 | 0 |
| absent/one_two_bytes | 1 | 1000000/uncapped | 100.0000 | 1 | 0 | 0 |
| absent/at_least_three_bytes | 3 | 200000/20 | 91.1238 | 0 | 4472 | 4472 |
| absent/at_least_three_bytes | 3 | 200000/21 | 91.1238 | 0 | 4472 | 4472 |
| absent/at_least_three_bytes | 3 | 200000/uncapped | 91.1238 | 0 | 4472 | 4472 |
| absent/at_least_three_bytes | 3 | 300000/20 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 300000/21 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 300000/uncapped | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 400000/20 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 400000/21 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 400000/uncapped | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 500000/20 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 500000/21 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 500000/uncapped | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 1000000/20 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 1000000/21 | 93.0253 | 1 | 3514 | 3514 |
| absent/at_least_three_bytes | 3 | 1000000/uncapped | 93.0253 | 1 | 3514 | 3514 |

## Cohort-only recommendation

- 200,000 B: best mean skip in this cohort is **20% / 21% / uncapped**, 69.6226% (tie). Use the same-budget query changes above to assess individual regressions.
- 300,000 B: best mean skip in this cohort is **20%**, 83.9472%. Use the same-budget query changes above to assess individual regressions.
- 400,000 B: best mean skip in this cohort is **20%**, 86.7808%. Use the same-budget query changes above to assess individual regressions.
- 500,000 B: best mean skip in this cohort is **20% / 21% / uncapped**, 86.8004% (tie). Use the same-budget query changes above to assess individual regressions.
- 1,000,000 B: best mean skip in this cohort is **21% / uncapped**, 89.0745% (tie). Use the same-budget query changes above to assess individual regressions.

These rankings apply to this frozen cohort and equal query weighting. A less restrictive cap can improve numeric queries and worsen other queries at the same budget. Choose a production policy only after measuring its workload and latency.

## Verification and limitations

- SHA-256 authenticated the frozen cohort, independent truth, inherited reports, indexes and source inputs. Rechecked all physical boundaries and decoded prefix/byte sidecars. Recomputed inherited four-mode decisions.
- Read query rows from all 15 actual serialized partial indexes. Rust verify-subset decoded every retained row. Recomputed generic selections, exact 5,323 B fixed accounting and actual file sizes. Existing 20% results match their prior report exactly.
- Persisted six-byte masks for all 42 queries at every budget/policy. Reloaded and recomputed every decision. Truth ⊆ full ⊆ partial ⊆ byte on every block/query. Prefix-contained, remainder-contained and crossing matches use the existing full-prefix callback.
- Every pre-existing snapshotted source/report/index file remained hash-identical. Missing indexes were built only via budget.run in normal budget directories.
- Frozen diagnostic cohort, not a representative production workload.
- No production latency claim. Matching counts are dictionary terms, not document-query DF.
- The row cap does not guarantee query-wide pruning. Missing partial rows are unknown.
- Budget/DF-rank omission includes skipped nonfitting rows. Selection need not be nested.
- Independent full-key truth was reused after SHA-256 verification, not rescanned.
- Caps 22/30/50/75/100 were checked by complete eligible/selected key equality at every budget; duplicate indexes were unnecessary.

Reproduction and tests: `dictionary_analysis/trigram_queries/thresholds.md`.
