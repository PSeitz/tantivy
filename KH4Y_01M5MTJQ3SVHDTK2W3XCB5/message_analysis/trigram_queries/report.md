# Frozen-cohort query evaluation

16,794 physical blocks, 7,671,280 distinct terms. All four modes include byte+full-prefix fallback. Costs: {'bytes': 5323, 'full_trigrams': 38073845, 'document_195640': 195640, 'document_999997': 999997}.

Zero false negatives. All source hashes unchanged. New truth is exact case-sensitive key matching, not document query DF.

| Query | Matching terms | True blocks | bytes candidates / skip% / FP | document_195640 candidates / skip% / FP | document_999997 candidates / skip% / FP | full_trigrams candidates / skip% / FP | Retained grams 195640 / 999997 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| *X | 0 | 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | 0 / 100.0000 / 0 | none / none |
| *a* | 1921269 | 16376 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | 16376 / 2.4890 / 0 | none / none |
| *ab* | 75774 | 15490 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | 16373 / 2.5068 / 883 | none / none |
| *ms* | 50585 | 14065 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | 16339 / 2.7093 / 2274 | none / none |
| *player* | 21 | 13 | 16331 / 2.7569 / 16318 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | 14 / 99.9166 / 1 | pla,lay,aye,yer / pla,lay,aye,yer |
| *traa* | 42 | 38 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 16341 / 2.6974 / 16303 | 141 / 99.1604 / 103 | none / none |
| *x | 153990 | 16022 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | 16354 / 2.6200 / 332 | none / none |
| *error* | 7 | 6 | 16341 / 2.6974 / 16335 | 16341 / 2.6974 / 16335 | 15 / 99.9107 / 9 | 15 / 99.9107 / 9 | none / err,rro,ror |
| *warning* | 0 | 0 | 16316 / 2.8463 / 16316 | 1047 / 93.7656 / 1047 | 89 / 99.4700 / 89 | 0 / 100.0000 / 0 | ing / nin,ing |
| *info* | 42 | 35 | 16327 / 2.7808 / 16292 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | 146 / 99.1306 / 111 | inf,nfo / inf,nfo |
| *debug* | 2 | 2 | 16339 / 2.7093 / 16337 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | 12 / 99.9285 / 10 | deb,ebu,bug / deb,ebu,bug |
| *request* | 22 | 11 | 16317 / 2.8403 / 16306 | 101 / 99.3986 / 90 | 11 / 99.9345 / 0 | 11 / 99.9345 / 0 | que,est / req,equ,que,ues,est |
| *response* | 11 | 4 | 16325 / 2.7927 / 16321 | 97 / 99.4224 / 93 | 4 / 99.9762 / 0 | 4 / 99.9762 / 0 | res,spo / res,esp,spo,pon,ons |
| *timeout* | 6 | 5 | 16316 / 2.8463 / 16311 | 139 / 99.1723 / 134 | 14 / 99.9166 / 9 | 5 / 99.9702 / 0 | tim,ime / tim,ime,out |
| *failed* | 2 | 1 | 16340 / 2.7033 / 16339 | 996 / 94.0693 / 995 | 2 / 99.9881 / 1 | 2 / 99.9881 / 1 | ile / fai,ail,ile,led |
| *success* | 9 | 4 | 16341 / 2.6974 / 16337 | 21 / 99.8750 / 17 | 5 / 99.9702 / 1 | 5 / 99.9702 / 1 | cce,ces,ess / suc,ucc,cce,ces,ess |
| *user* | 42 | 36 | 16337 / 2.7212 / 16301 | 16337 / 2.7212 / 16301 | 143 / 99.1485 / 107 | 143 / 99.1485 / 107 | none / use,ser |
| *server* | 11 | 6 | 16334 / 2.7391 / 16328 | 16334 / 2.7391 / 16328 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | none / ser,erv,rve,ver |
| *client* | 25 | 15 | 16325 / 2.7927 / 16310 | 1070 / 93.6287 / 1055 | 15 / 99.9107 / 0 | 15 / 99.9107 / 0 | ent / cli,lie,ien,ent |
| *session* | 4 | 2 | 16324 / 2.7986 / 16322 | 87 / 99.4820 / 85 | 87 / 99.4820 / 85 | 2 / 99.9881 / 0 | ess,ion / ess,ion |
| *connection* | 9 | 5 | 16322 / 2.8105 / 16317 | 9 / 99.9464 / 4 | 5 / 99.9702 / 0 | 5 / 99.9702 / 0 | con,ect,tio,ion / con,onn,nne,nec,ect,cti,tio,ion |
| *login* | 0 | 0 | 16317 / 2.8403 / 16317 | 1396 / 91.6875 / 1396 | 1396 / 91.6875 / 1396 | 17 / 99.8988 / 17 | log / log |
| *trace* | 1 | 1 | 16340 / 2.7033 / 16339 | 16340 / 2.7033 / 16339 | 2125 / 87.3467 / 2124 | 18 / 99.8928 / 17 | none / ace |
| *cache* | 9 | 4 | 16350 / 2.6438 / 16346 | 1078 / 93.5810 / 1074 | 1078 / 93.5810 / 1074 | 19 / 99.8869 / 15 | che / che |
| *database* | 5 | 2 | 16337 / 2.7212 / 16335 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | 2 / 99.9881 / 0 | dat,ata,tab,aba,bas,ase / dat,ata,tab,aba,bas,ase |
| *process* | 12 | 6 | 16328 / 2.7748 / 16322 | 11 / 99.9345 / 5 | 6 / 99.9643 / 0 | 6 / 99.9643 / 0 | pro,ces,ess / pro,roc,oce,ces,ess |
| *message* | 13 | 7 | 16330 / 2.7629 / 16323 | 146 / 99.1306 / 139 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | mes,ess / mes,ess,ssa,sag,age |
| *status* | 9 | 7 | 16330 / 2.7629 / 16323 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | 8 / 99.9524 / 1 | sta,tat,atu,tus / sta,tat,atu,tus |
| *service* | 27 | 9 | 16323 / 2.8046 / 16314 | 16323 / 2.8046 / 16314 | 9 / 99.9464 / 0 | 9 / 99.9464 / 0 | none / ser,erv,rvi,vic,ice |
| *http* | 66 | 38 | 16331 / 2.7569 / 16293 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | 130 / 99.2259 / 92 | htt,ttp / htt,ttp |
| *exception* | 14 | 7 | 16315 / 2.8522 / 16308 | 10 / 99.9405 / 3 | 7 / 99.9583 / 0 | 7 / 99.9583 / 0 | cep,ept,tio,ion / exc,xce,cep,ept,pti,tio,ion |
| *game* | 71 | 44 | 16334 / 2.7391 / 16290 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | 126 / 99.2497 / 82 | gam,ame / gam,ame |
| *team* | 42 | 37 | 16336 / 2.7272 / 16299 | 16336 / 2.7272 / 16299 | 128 / 99.2378 / 91 | 128 / 99.2378 / 91 | none / tea,eam |
| *match* | 4 | 2 | 16325 / 2.7927 / 16323 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | 23 / 99.8630 / 21 | mat,atc,tch / mat,atc,tch |
| *time* | 66 | 51 | 16328 / 2.7748 / 16277 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | 139 / 99.1723 / 88 | tim,ime / tim,ime |
| *name* | 50 | 47 | 16338 / 2.7153 / 16291 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | 148 / 99.1187 / 101 | nam,ame / nam,ame |
| *world* | 1 | 1 | 16323 / 2.8046 / 16322 | 1362 / 91.8900 / 1361 | 1362 / 91.8900 / 1361 | 12 / 99.9285 / 11 | wor / wor |
| *3ic* | 1255 | 1008 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 16349 / 2.6498 / 15341 | 1534 / 90.8658 / 526 | none / none |
| *a76* | 2871 | 2103 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 16376 / 2.4890 / 14273 | 2469 / 85.2983 / 366 | none / none |
| *000* | 8333 | 3320 | 16794 / 0.0000 / 13474 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 3708 / 77.9207 / 388 | 000 / 000 |
| *123* | 12372 | 3443 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 16794 / 0.0000 / 13351 | 3809 / 77.3193 / 366 | none / none |
| *deadbeef* | 0 | 0 | 16368 / 2.5366 / 16368 | 2029 / 87.9183 / 2029 | 2029 / 87.9183 / 2029 | 16 / 99.9047 / 16 | ead / ead |

## Aggregates

Each query has equal weight. Candidate sums count query/block pairs. Empty groups are null in JSON.

| Group | Queries | Mode | Candidates sum | Mean candidates | Skip% | True blocks sum | FP sum |
| --- | --- | --- | --- | --- | --- | --- | --- |
| all/all | 42 | bytes | 670684 | 15968.67 | 4.9145 | 72273 | 598411 |
| all/all | 42 | document_195640 | 243368 | 5794.48 | 65.4967 | 72273 | 171095 |
| all/all | 42 | document_999997 | 144301 | 3435.74 | 79.5419 | 72273 | 72028 |
| all/all | 42 | full_trigrams | 78303 | 1864.36 | 88.8987 | 72273 | 6030 |
| all/one_two_bytes | 5 | bytes | 65442 | 13088.40 | 22.0650 | 61953 | 3489 |
| all/one_two_bytes | 5 | document_195640 | 65442 | 13088.40 | 22.0650 | 61953 | 3489 |
| all/one_two_bytes | 5 | document_999997 | 65442 | 13088.40 | 22.0650 | 61953 | 3489 |
| all/one_two_bytes | 5 | full_trigrams | 65442 | 13088.40 | 22.0650 | 61953 | 3489 |
| all/at_least_three_bytes | 37 | bytes | 605242 | 16357.89 | 2.5968 | 10320 | 594922 |
| all/at_least_three_bytes | 37 | document_195640 | 177926 | 4808.81 | 71.3659 | 10320 | 167606 |
| all/at_least_three_bytes | 37 | document_999997 | 78859 | 2131.32 | 87.3090 | 10320 | 68539 |
| all/at_least_three_bytes | 37 | full_trigrams | 12861 | 347.59 | 97.9302 | 10320 | 2541 |
| present/all | 38 | bytes | 621683 | 16360.08 | 2.5838 | 72273 | 549410 |
| present/all | 38 | document_195640 | 238896 | 6286.74 | 62.5656 | 72273 | 166623 |
| present/all | 38 | document_999997 | 140787 | 3704.92 | 77.9390 | 72273 | 68514 |
| present/all | 38 | full_trigrams | 78270 | 2059.74 | 87.7353 | 72273 | 5997 |
| present/one_two_bytes | 4 | bytes | 65442 | 16360.50 | 2.5813 | 61953 | 3489 |
| present/one_two_bytes | 4 | document_195640 | 65442 | 16360.50 | 2.5813 | 61953 | 3489 |
| present/one_two_bytes | 4 | document_999997 | 65442 | 16360.50 | 2.5813 | 61953 | 3489 |
| present/one_two_bytes | 4 | full_trigrams | 65442 | 16360.50 | 2.5813 | 61953 | 3489 |
| present/at_least_three_bytes | 34 | bytes | 556241 | 16360.03 | 2.5841 | 10320 | 545921 |
| present/at_least_three_bytes | 34 | document_195640 | 173454 | 5101.59 | 69.6226 | 10320 | 163134 |
| present/at_least_three_bytes | 34 | document_999997 | 75345 | 2216.03 | 86.8046 | 10320 | 65025 |
| present/at_least_three_bytes | 34 | full_trigrams | 12828 | 377.29 | 97.7534 | 10320 | 2508 |
| absent/all | 4 | bytes | 49001 | 12250.25 | 27.0558 | 0 | 49001 |
| absent/all | 4 | document_195640 | 4472 | 1118.00 | 93.3429 | 0 | 4472 |
| absent/all | 4 | document_999997 | 3514 | 878.50 | 94.7690 | 0 | 3514 |
| absent/all | 4 | full_trigrams | 33 | 8.25 | 99.9509 | 0 | 33 |
| absent/one_two_bytes | 1 | bytes | 0 | 0.00 | 100.0000 | 0 | 0 |
| absent/one_two_bytes | 1 | document_195640 | 0 | 0.00 | 100.0000 | 0 | 0 |
| absent/one_two_bytes | 1 | document_999997 | 0 | 0.00 | 100.0000 | 0 | 0 |
| absent/one_two_bytes | 1 | full_trigrams | 0 | 0.00 | 100.0000 | 0 | 0 |
| absent/at_least_three_bytes | 3 | bytes | 49001 | 16333.67 | 2.7411 | 0 | 49001 |
| absent/at_least_three_bytes | 3 | document_195640 | 4472 | 1490.67 | 91.1238 | 0 | 4472 |
| absent/at_least_three_bytes | 3 | document_999997 | 3514 | 1171.33 | 93.0253 | 0 | 3514 |
| absent/at_least_three_bytes | 3 | full_trigrams | 33 | 11.00 | 99.9345 | 0 | 33 |

## Interpretation and limitations

- The frozen cohort is diagnostic, not a representative production workload. All aggregates weight queries equally. Present and absent values are separated above.
- At 195640 bytes, error/user/server/service/team/trace retain no grams and fall back to bytes. The 999997-byte index improves these, while traa/3ic/a76 remain poorly pruned in both.
- The 123 row has remainder block DF 3425, exceeding the cap-80 limit 3358.8. Both partial indexes omit it and retain every block.
- The retained 000 row has block DF 3320, but prefix-contained/crossing cases yield 3708 candidates (77.9207% skip). A row cutoff does not guarantee query-wide skip.
- Complete postings can still have false positives through cross-term gram co-occurrence and prefix continuations. The fresh full-key truth identifies every false positive.
- One/two-byte values cannot benefit from trigram rows. No document query DF or production latency was measured.

See report.json for every gram's full-term distinct-document DF, remainder block DF, and retention in both indexes. See dictionary_analysis/trigram_queries/README.md for semantics, limitations and reproduction.
