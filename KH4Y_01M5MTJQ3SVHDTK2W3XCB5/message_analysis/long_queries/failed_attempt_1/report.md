# Longer queries on unchanged partial trigram indexes

All counts are candidate physical blocks out of 16,794. Every mode includes byte+full-prefix fallback. Rows retain the original document-DF ranking and 20% remainder-block-presence cap. No index was rebuilt or tuned.

| Mode | Additional metadata bytes |
| --- | --- |
| bytes | 5323 |
| document_200000 | 199999 |
| document_300000 | 300000 |
| document_400000 | 399990 |
| document_500000 | 499997 |
| document_1000000 | 999997 |
| full_trigrams | 38073845 |

## Predeclared natural-word shortlist

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| authentication | present | 1 | 1 | 16309 | 4 | 4 | 1 | 1 | 1 | 1 |
| configuration | present | 4 | 3 | 16309 | 10 | 3 | 3 | 3 | 3 | 3 |
| connectiontimeout | present | 1 | 1 | 16312 | 3 | 3 | 2 | 1 | 1 | 1 |
| successful | present | 3 | 1 | 16333 | 21 | 21 | 21 | 21 | 5 | 1 |
| permissions | absent | 0 | 0 | 16313 | 1042 | 102 | 18 | 1 | 1 | 0 |
| heartbeat | present | 4 | 3 | 16332 | 16332 | 1048 | 3 | 3 | 3 | 3 |
| healthcheck | absent | 0 | 0 | 16320 | 1078 | 1078 | 108 | 108 | 108 | 0 |
| outofmemory | absent | 0 | 0 | 16315 | 954 | 102 | 102 | 6 | 6 | 0 |
| nullpointerexception | absent | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 |
| indexoutofbounds | absent | 0 | 0 | 16309 | 1061 | 6 | 6 | 0 | 0 | 0 |

## Groups

Equal query weighting. Source-derived queries are diagnostics, potentially ID-like. Empty groups have null means in JSON.

| Group | Queries | Mode | Mean candidates | Skip % | True blocks sum | FP sum |
| --- | --- | --- | --- | --- | --- | --- |
| natural_present | 14 | bytes | 16316.857 | 2.8412 | 20 | 228416 |
| natural_present | 14 | document_200000 | 2420.143 | 85.5892 | 20 | 33862 |
| natural_present | 14 | document_300000 | 1327.643 | 92.0945 | 20 | 18567 |
| natural_present | 14 | document_400000 | 87.643 | 99.4781 | 20 | 1207 |
| natural_present | 14 | document_500000 | 5.214 | 99.9690 | 20 | 53 |
| natural_present | 14 | document_1000000 | 2.714 | 99.9838 | 20 | 18 |
| natural_present | 14 | full_trigrams | 1.500 | 99.9911 | 20 | 1 |
| natural_absent | 27 | bytes | 16311.815 | 2.8712 | 0 | 440419 |
| natural_absent | 27 | document_200000 | 810.444 | 95.1742 | 0 | 21882 |
| natural_absent | 27 | document_300000 | 93.926 | 99.4407 | 0 | 2536 |
| natural_absent | 27 | document_400000 | 49.148 | 99.7073 | 0 | 1327 |
| natural_absent | 27 | document_500000 | 41.000 | 99.7559 | 0 | 1107 |
| natural_absent | 27 | document_1000000 | 40.963 | 99.7561 | 0 | 1106 |
| natural_absent | 27 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| source_present | 40 | bytes | 16379.925 | 2.4656 | 40 | 655157 |
| source_present | 40 | document_200000 | 14717.800 | 12.3627 | 40 | 588672 |
| source_present | 40 | document_300000 | 13012.350 | 22.5179 | 40 | 520454 |
| source_present | 40 | document_400000 | 13012.350 | 22.5179 | 40 | 520454 |
| source_present | 40 | document_500000 | 13012.350 | 22.5179 | 40 | 520454 |
| source_present | 40 | document_1000000 | 6176.250 | 63.2235 | 40 | 247010 |
| source_present | 40 | full_trigrams | 32.600 | 99.8059 | 40 | 1264 |
| controls | 8 | bytes | 16335.125 | 2.7324 | 112 | 130569 |
| controls | 8 | document_200000 | 12510.250 | 25.5076 | 112 | 99970 |
| controls | 8 | document_300000 | 4366.625 | 73.9989 | 112 | 34821 |
| controls | 8 | document_400000 | 2345.875 | 86.0315 | 112 | 18655 |
| controls | 8 | document_500000 | 2332.000 | 86.1141 | 112 | 18544 |
| controls | 8 | document_1000000 | 2332.000 | 86.1141 | 112 | 18544 |
| controls | 8 | full_trigrams | 43.625 | 99.7402 | 112 | 237 |
| natural/absent/8_bytes | 1 | bytes | 16316.000 | 2.8463 | 0 | 16316 |
| natural/absent/8_bytes | 1 | document_200000 | 139.000 | 99.1723 | 0 | 139 |
| natural/absent/8_bytes | 1 | document_300000 | 14.000 | 99.9166 | 0 | 14 |
| natural/absent/8_bytes | 1 | document_400000 | 14.000 | 99.9166 | 0 | 14 |
| natural/absent/8_bytes | 1 | document_500000 | 14.000 | 99.9166 | 0 | 14 |
| natural/absent/8_bytes | 1 | document_1000000 | 14.000 | 99.9166 | 0 | 14 |
| natural/absent/8_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/9_bytes | 1 | bytes | 16332.000 | 2.7510 | 3 | 16329 |
| natural/present/9_bytes | 1 | document_200000 | 16332.000 | 2.7510 | 3 | 16329 |
| natural/present/9_bytes | 1 | document_300000 | 1048.000 | 93.7597 | 3 | 1045 |
| natural/present/9_bytes | 1 | document_400000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/9_bytes | 1 | document_500000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/9_bytes | 1 | document_1000000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/9_bytes | 1 | full_trigrams | 3.000 | 99.9821 | 3 | 0 |
| natural/absent/9_bytes | 1 | bytes | 16322.000 | 2.8105 | 0 | 16322 |
| natural/absent/9_bytes | 1 | document_200000 | 16322.000 | 2.8105 | 0 | 16322 |
| natural/absent/9_bytes | 1 | document_300000 | 947.000 | 94.3611 | 0 | 947 |
| natural/absent/9_bytes | 1 | document_400000 | 947.000 | 94.3611 | 0 | 947 |
| natural/absent/9_bytes | 1 | document_500000 | 947.000 | 94.3611 | 0 | 947 |
| natural/absent/9_bytes | 1 | document_1000000 | 947.000 | 94.3611 | 0 | 947 |
| natural/absent/9_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/10_bytes | 4 | bytes | 16320.250 | 2.8209 | 6 | 65275 |
| natural/present/10_bytes | 4 | document_200000 | 28.250 | 99.8318 | 6 | 107 |
| natural/present/10_bytes | 4 | document_300000 | 28.250 | 99.8318 | 6 | 107 |
| natural/present/10_bytes | 4 | document_400000 | 28.250 | 99.8318 | 6 | 107 |
| natural/present/10_bytes | 4 | document_500000 | 9.750 | 99.9419 | 6 | 33 |
| natural/present/10_bytes | 4 | document_1000000 | 5.750 | 99.9658 | 6 | 17 |
| natural/present/10_bytes | 4 | full_trigrams | 1.750 | 99.9896 | 6 | 1 |
| natural/absent/10_bytes | 2 | bytes | 16312.500 | 2.8671 | 0 | 32625 |
| natural/absent/10_bytes | 2 | document_200000 | 524.000 | 96.8798 | 0 | 1048 |
| natural/absent/10_bytes | 2 | document_300000 | 54.000 | 99.6785 | 0 | 108 |
| natural/absent/10_bytes | 2 | document_400000 | 54.000 | 99.6785 | 0 | 108 |
| natural/absent/10_bytes | 2 | document_500000 | 9.500 | 99.9434 | 0 | 19 |
| natural/absent/10_bytes | 2 | document_1000000 | 9.500 | 99.9434 | 0 | 19 |
| natural/absent/10_bytes | 2 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/11_bytes | 2 | bytes | 16316.500 | 2.8433 | 2 | 32631 |
| natural/present/11_bytes | 2 | document_200000 | 8693.500 | 48.2345 | 2 | 17385 |
| natural/present/11_bytes | 2 | document_300000 | 8693.500 | 48.2345 | 2 | 17385 |
| natural/present/11_bytes | 2 | document_400000 | 546.500 | 96.7459 | 2 | 1091 |
| natural/present/11_bytes | 2 | document_500000 | 9.000 | 99.9464 | 2 | 16 |
| natural/present/11_bytes | 2 | document_1000000 | 1.000 | 99.9940 | 2 | 0 |
| natural/present/11_bytes | 2 | full_trigrams | 1.000 | 99.9940 | 2 | 0 |
| natural/absent/11_bytes | 6 | bytes | 16317.500 | 2.8373 | 0 | 97905 |
| natural/absent/11_bytes | 6 | document_200000 | 538.667 | 96.7925 | 0 | 3232 |
| natural/absent/11_bytes | 6 | document_300000 | 238.667 | 98.5789 | 0 | 1432 |
| natural/absent/11_bytes | 6 | document_400000 | 40.333 | 99.7598 | 0 | 242 |
| natural/absent/11_bytes | 6 | document_500000 | 19.833 | 99.8819 | 0 | 119 |
| natural/absent/11_bytes | 6 | document_1000000 | 19.667 | 99.8829 | 0 | 118 |
| natural/absent/11_bytes | 6 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/12_bytes | 2 | bytes | 16321.500 | 2.8135 | 2 | 32641 |
| natural/present/12_bytes | 2 | document_200000 | 12.500 | 99.9256 | 2 | 23 |
| natural/present/12_bytes | 2 | document_300000 | 12.500 | 99.9256 | 2 | 23 |
| natural/present/12_bytes | 2 | document_400000 | 4.500 | 99.9732 | 2 | 7 |
| natural/present/12_bytes | 2 | document_500000 | 3.000 | 99.9821 | 2 | 4 |
| natural/present/12_bytes | 2 | document_1000000 | 1.500 | 99.9911 | 2 | 1 |
| natural/present/12_bytes | 2 | full_trigrams | 1.000 | 99.9940 | 2 | 0 |
| natural/absent/12_bytes | 1 | bytes | 16318.000 | 2.8343 | 0 | 16318 |
| natural/absent/12_bytes | 1 | document_200000 | 8.000 | 99.9524 | 0 | 8 |
| natural/absent/12_bytes | 1 | document_300000 | 3.000 | 99.9821 | 0 | 3 |
| natural/absent/12_bytes | 1 | document_400000 | 1.000 | 99.9940 | 0 | 1 |
| natural/absent/12_bytes | 1 | document_500000 | 1.000 | 99.9940 | 0 | 1 |
| natural/absent/12_bytes | 1 | document_1000000 | 1.000 | 99.9940 | 0 | 1 |
| natural/absent/12_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/13_bytes | 1 | bytes | 16309.000 | 2.8879 | 3 | 16306 |
| natural/present/13_bytes | 1 | document_200000 | 10.000 | 99.9405 | 3 | 7 |
| natural/present/13_bytes | 1 | document_300000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/13_bytes | 1 | document_400000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/13_bytes | 1 | document_500000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/13_bytes | 1 | document_1000000 | 3.000 | 99.9821 | 3 | 0 |
| natural/present/13_bytes | 1 | full_trigrams | 3.000 | 99.9821 | 3 | 0 |
| natural/absent/13_bytes | 4 | bytes | 16309.000 | 2.8879 | 0 | 65236 |
| natural/absent/13_bytes | 4 | document_200000 | 10.500 | 99.9375 | 0 | 42 |
| natural/absent/13_bytes | 4 | document_300000 | 2.250 | 99.9866 | 0 | 9 |
| natural/absent/13_bytes | 4 | document_400000 | 0.500 | 99.9970 | 0 | 2 |
| natural/absent/13_bytes | 4 | document_500000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/13_bytes | 4 | document_1000000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/13_bytes | 4 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/14_bytes | 2 | bytes | 16310.500 | 2.8790 | 2 | 32619 |
| natural/present/14_bytes | 2 | document_200000 | 4.500 | 99.9732 | 2 | 7 |
| natural/present/14_bytes | 2 | document_300000 | 3.000 | 99.9821 | 2 | 4 |
| natural/present/14_bytes | 2 | document_400000 | 1.500 | 99.9911 | 2 | 1 |
| natural/present/14_bytes | 2 | document_500000 | 1.000 | 99.9940 | 2 | 0 |
| natural/present/14_bytes | 2 | document_1000000 | 1.000 | 99.9940 | 2 | 0 |
| natural/present/14_bytes | 2 | full_trigrams | 1.000 | 99.9940 | 2 | 0 |
| natural/absent/14_bytes | 2 | bytes | 16309.000 | 2.8879 | 0 | 32618 |
| natural/absent/14_bytes | 2 | document_200000 | 5.500 | 99.9673 | 0 | 11 |
| natural/absent/14_bytes | 2 | document_300000 | 4.000 | 99.9762 | 0 | 8 |
| natural/absent/14_bytes | 2 | document_400000 | 0.500 | 99.9970 | 0 | 1 |
| natural/absent/14_bytes | 2 | document_500000 | 0.500 | 99.9970 | 0 | 1 |
| natural/absent/14_bytes | 2 | document_1000000 | 0.500 | 99.9970 | 0 | 1 |
| natural/absent/14_bytes | 2 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/15_bytes | 4 | bytes | 16308.000 | 2.8939 | 0 | 65232 |
| natural/absent/15_bytes | 4 | document_200000 | 3.000 | 99.9821 | 0 | 12 |
| natural/absent/15_bytes | 4 | document_300000 | 1.500 | 99.9911 | 0 | 6 |
| natural/absent/15_bytes | 4 | document_400000 | 1.500 | 99.9911 | 0 | 6 |
| natural/absent/15_bytes | 4 | document_500000 | 1.500 | 99.9911 | 0 | 6 |
| natural/absent/15_bytes | 4 | document_1000000 | 1.500 | 99.9911 | 0 | 6 |
| natural/absent/15_bytes | 4 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/16_bytes | 3 | bytes | 16310.333 | 2.8800 | 0 | 48931 |
| natural/absent/16_bytes | 3 | document_200000 | 355.000 | 97.8861 | 0 | 1065 |
| natural/absent/16_bytes | 3 | document_300000 | 3.000 | 99.9821 | 0 | 9 |
| natural/absent/16_bytes | 3 | document_400000 | 2.000 | 99.9881 | 0 | 6 |
| natural/absent/16_bytes | 3 | document_500000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/16_bytes | 3 | document_1000000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/16_bytes | 3 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/17_bytes | 1 | bytes | 16312.000 | 2.8701 | 1 | 16311 |
| natural/present/17_bytes | 1 | document_200000 | 3.000 | 99.9821 | 1 | 2 |
| natural/present/17_bytes | 1 | document_300000 | 3.000 | 99.9821 | 1 | 2 |
| natural/present/17_bytes | 1 | document_400000 | 2.000 | 99.9881 | 1 | 1 |
| natural/present/17_bytes | 1 | document_500000 | 1.000 | 99.9940 | 1 | 0 |
| natural/present/17_bytes | 1 | document_1000000 | 1.000 | 99.9940 | 1 | 0 |
| natural/present/17_bytes | 1 | full_trigrams | 1.000 | 99.9940 | 1 | 0 |
| natural/absent/19_bytes | 1 | bytes | 16308.000 | 2.8939 | 0 | 16308 |
| natural/absent/19_bytes | 1 | document_200000 | 3.000 | 99.9821 | 0 | 3 |
| natural/absent/19_bytes | 1 | document_300000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/19_bytes | 1 | document_400000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/19_bytes | 1 | document_500000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/19_bytes | 1 | document_1000000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/19_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | bytes | 16305.000 | 2.9118 | 0 | 16305 |
| natural/absent/20_bytes | 1 | document_200000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | document_300000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | document_400000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | document_500000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | document_1000000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/20_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| natural/present/21_bytes | 1 | bytes | 16305.000 | 2.9118 | 1 | 16304 |
| natural/present/21_bytes | 1 | document_200000 | 3.000 | 99.9821 | 1 | 2 |
| natural/present/21_bytes | 1 | document_300000 | 2.000 | 99.9881 | 1 | 1 |
| natural/present/21_bytes | 1 | document_400000 | 1.000 | 99.9940 | 1 | 0 |
| natural/present/21_bytes | 1 | document_500000 | 1.000 | 99.9940 | 1 | 0 |
| natural/present/21_bytes | 1 | document_1000000 | 1.000 | 99.9940 | 1 | 0 |
| natural/present/21_bytes | 1 | full_trigrams | 1.000 | 99.9940 | 1 | 0 |
| natural/absent/21_bytes | 1 | bytes | 16303.000 | 2.9237 | 0 | 16303 |
| natural/absent/21_bytes | 1 | document_200000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/21_bytes | 1 | document_300000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/21_bytes | 1 | document_400000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/21_bytes | 1 | document_500000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/21_bytes | 1 | document_1000000 | 0.000 | 100.0000 | 0 | 0 |
| natural/absent/21_bytes | 1 | full_trigrams | 0.000 | 100.0000 | 0 | 0 |
| source/present/8_bytes | 8 | bytes | 16424.625 | 2.1994 | 8 | 131389 |
| source/present/8_bytes | 8 | document_200000 | 14762.500 | 12.0966 | 8 | 118092 |
| source/present/8_bytes | 8 | document_300000 | 13080.375 | 22.1128 | 8 | 104635 |
| source/present/8_bytes | 8 | document_400000 | 13080.375 | 22.1128 | 8 | 104635 |
| source/present/8_bytes | 8 | document_500000 | 13080.375 | 22.1128 | 8 | 104635 |
| source/present/8_bytes | 8 | document_1000000 | 7524.375 | 55.1961 | 8 | 60187 |
| source/present/8_bytes | 8 | full_trigrams | 158.125 | 99.0584 | 8 | 1257 |
| source/present/12_bytes | 8 | bytes | 16370.125 | 2.5240 | 8 | 130953 |
| source/present/12_bytes | 8 | document_200000 | 14708.000 | 12.4211 | 8 | 117656 |
| source/present/12_bytes | 8 | document_300000 | 13046.375 | 22.3153 | 8 | 104363 |
| source/present/12_bytes | 8 | document_400000 | 13046.375 | 22.3153 | 8 | 104363 |
| source/present/12_bytes | 8 | document_500000 | 13046.375 | 22.3153 | 8 | 104363 |
| source/present/12_bytes | 8 | document_1000000 | 7259.500 | 56.7733 | 8 | 58068 |
| source/present/12_bytes | 8 | full_trigrams | 1.875 | 99.9888 | 8 | 7 |
| source/present/16_bytes | 8 | bytes | 16369.125 | 2.5299 | 8 | 130945 |
| source/present/16_bytes | 8 | document_200000 | 14707.000 | 12.4271 | 8 | 117648 |
| source/present/16_bytes | 8 | document_300000 | 13045.375 | 22.3212 | 8 | 104355 |
| source/present/16_bytes | 8 | document_400000 | 13045.375 | 22.3212 | 8 | 104355 |
| source/present/16_bytes | 8 | document_500000 | 13045.375 | 22.3212 | 8 | 104355 |
| source/present/16_bytes | 8 | document_1000000 | 7258.500 | 56.7792 | 8 | 58060 |
| source/present/16_bytes | 8 | full_trigrams | 1.000 | 99.9940 | 8 | 0 |
| source/present/24_bytes | 8 | bytes | 16368.125 | 2.5359 | 8 | 130937 |
| source/present/24_bytes | 8 | document_200000 | 14706.000 | 12.4330 | 8 | 117640 |
| source/present/24_bytes | 8 | document_300000 | 13044.375 | 22.3272 | 8 | 104347 |
| source/present/24_bytes | 8 | document_400000 | 13044.375 | 22.3272 | 8 | 104347 |
| source/present/24_bytes | 8 | document_500000 | 13044.375 | 22.3272 | 8 | 104347 |
| source/present/24_bytes | 8 | document_1000000 | 5348.875 | 68.1501 | 8 | 42783 |
| source/present/24_bytes | 8 | full_trigrams | 1.000 | 99.9940 | 8 | 0 |
| source/present/32_bytes | 8 | bytes | 16367.625 | 2.5389 | 8 | 130933 |
| source/present/32_bytes | 8 | document_200000 | 14705.500 | 12.4360 | 8 | 117636 |
| source/present/32_bytes | 8 | document_300000 | 12845.250 | 23.5129 | 8 | 102754 |
| source/present/32_bytes | 8 | document_400000 | 12845.250 | 23.5129 | 8 | 102754 |
| source/present/32_bytes | 8 | document_500000 | 12845.250 | 23.5129 | 8 | 102754 |
| source/present/32_bytes | 8 | document_1000000 | 3490.000 | 79.2188 | 8 | 27912 |
| source/present/32_bytes | 8 | full_trigrams | 1.000 | 99.9940 | 8 | 0 |

## Nested source families

Each family uses the first 8/12/16/24/32 bytes of one centered 32-byte window. Source terms were selected solely by seeded SHA-256 before truth/index evaluation.

| Mode | Families improving 8→32 | 8-byte mean candidates | 12-byte mean candidates | 16-byte mean candidates | 24-byte mean candidates | 32-byte mean candidates |
| --- | --- | --- | --- | --- | --- | --- |
| bytes | 8 | 16424.625 | 16370.125 | 16369.125 | 16368.125 | 16367.625 |
| document_200000 | 8 | 14762.5 | 14708.0 | 14707.0 | 14706.0 | 14705.5 |
| document_300000 | 8 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_400000 | 8 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_500000 | 8 | 13080.375 | 13046.375 | 13045.375 | 13044.375 | 12845.25 |
| document_1000000 | 8 | 7524.375 | 7259.5 | 7258.5 | 5348.875 | 3490.0 |
| full_trigrams | 8 | 158.125 | 1.875 | 1.0 | 1.0 | 1.0 |

### Family 0: `a5b557dfa94b1f70209f6a70fbb39a5c` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| a5b557df | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 15 |
| a5b557dfa94b | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 2 |
| a5b557dfa94b1f70 | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 1 |
| a5b557dfa94b1f70209f6a70 | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 3081 | 1 |
| a5b557dfa94b1f70209f6a70fbb39a5c | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 3080 | 1 |

### Family 1: `6ed6a787d02af80e84c5ea014d267a2a` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 6ed6a787 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 19 |
| 6ed6a787d02a | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 1 |
| 6ed6a787d02af80e | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 1 |
| 6ed6a787d02af80e84c5ea01 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 6ed6a787d02af80e84c5ea014d267a2a | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 3065 | 1 |

### Family 2: `793c3564223ab74c29f651a1f4f41756` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 793c3564 | present | 1 | 1 | 16375 | 16375 | 16375 | 16375 | 16375 | 3086 | 29 |
| 793c3564223a | present | 1 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 1213 | 3 |
| 793c3564223ab74c | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1212 | 1 |
| 793c3564223ab74c29f651a1 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1211 | 1 |
| 793c3564223ab74c29f651a1f4f41756 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1211 | 1 |

### Family 3: `e668977af5cc0a1953aa143191d6379b` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| e668977a | present | 1 | 1 | 16373 | 16373 | 16373 | 16373 | 16373 | 3080 | 60 |
| e668977af5cc | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 3076 | 1 |
| e668977af5cc0a19 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 3076 | 1 |
| e668977af5cc0a1953aa1431 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1518 | 1 |
| e668977af5cc0a1953aa143191d6379b | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 904 | 1 |

### Family 4: `3447ff7f43d94cd814477f31c3ccdb66` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 3447ff7f | present | 1 | 1 | 16377 | 16377 | 16377 | 16377 | 16377 | 1694 | 16 |
| 3447ff7f43d9 | present | 1 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 1688 | 1 |
| 3447ff7f43d94cd8 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1687 | 1 |
| 3447ff7f43d94cd814477f31 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1676 | 1 |
| 3447ff7f43d94cd814477f31c3ccdb66 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1675 | 1 |

### Family 5: `23504150bf03afe28db8f8610f728305` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 23504150 | present | 1 | 1 | 16794 | 16794 | 3337 | 3337 | 3337 | 1749 | 1053 |
| 23504150bf03 | present | 1 | 1 | 16374 | 16374 | 3081 | 3081 | 3081 | 1515 | 4 |
| 23504150bf03afe2 | present | 1 | 1 | 16371 | 16371 | 3078 | 3078 | 3078 | 1512 | 1 |
| 23504150bf03afe28db8f861 | present | 1 | 1 | 16368 | 16368 | 3075 | 3075 | 3075 | 1509 | 1 |
| 23504150bf03afe28db8f8610f728305 | present | 1 | 1 | 16368 | 16368 | 1486 | 1486 | 1486 | 832 | 1 |

### Family 6: `0b3da49b0ecf04c0127fae58532caf18` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0b3da49b | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 21 |
| 0b3da49b0ecf | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 2 |
| 0b3da49b0ecf04c0 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 0b3da49b0ecf04c0127fae58 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 0b3da49b0ecf04c0127fae58532caf18 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |

### Family 7: `c000283a31c604eb3d8d670f16332953` (hex)

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c000283a | present | 1 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 1479 | 52 |
| c000283a31c6 | present | 1 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 1479 | 1 |
| c000283a31c604eb | present | 1 | 1 | 16370 | 3073 | 3073 | 3073 | 3073 | 1478 | 1 |
| c000283a31c604eb3d8d670f | present | 1 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 1062 | 1 |
| c000283a31c604eb3d8d670f16332953 | present | 1 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 786 | 1 |

## Every query

| Literal | Presence | Matching terms | True blocks | bytes | document_200000 | document_300000 | document_400000 | document_500000 | document_1000000 | full_trigrams |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| authentication | present | 1 | 1 | 16309 | 4 | 4 | 1 | 1 | 1 | 1 |
| authorization | absent | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 |
| configuration | present | 4 | 3 | 16309 | 10 | 3 | 3 | 3 | 3 | 3 |
| initialization | absent | 0 | 0 | 16316 | 7 | 7 | 0 | 0 | 0 | 0 |
| serialization | absent | 0 | 0 | 16311 | 34 | 6 | 0 | 0 | 0 | 0 |
| deserialization | absent | 0 | 0 | 16311 | 3 | 0 | 0 | 0 | 0 | 0 |
| synchronization | absent | 0 | 0 | 16305 | 6 | 6 | 6 | 6 | 6 | 0 |
| communication | absent | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 |
| reconnection | absent | 0 | 0 | 16318 | 8 | 3 | 1 | 1 | 1 | 0 |
| disconnected | present | 1 | 1 | 16318 | 4 | 4 | 4 | 1 | 1 | 1 |
| connectionfailed | absent | 0 | 0 | 16318 | 1 | 0 | 0 | 0 | 0 | 0 |
| connectiontimeout | present | 1 | 1 | 16312 | 3 | 3 | 2 | 1 | 1 | 1 |
| requesttimeout | absent | 0 | 0 | 16302 | 4 | 1 | 1 | 1 | 1 | 0 |
| requestfailed | absent | 0 | 0 | 16307 | 8 | 3 | 2 | 0 | 0 | 0 |
| connectionreset | absent | 0 | 0 | 16315 | 3 | 0 | 0 | 0 | 0 | 0 |
| connecttimeout | present | 1 | 1 | 16312 | 5 | 2 | 2 | 1 | 1 | 1 |
| timedout | absent | 0 | 0 | 16316 | 139 | 14 | 14 | 14 | 14 | 0 |
| unexpected | present | 1 | 1 | 16322 | 82 | 82 | 82 | 8 | 8 | 1 |
| unsupported | absent | 0 | 0 | 16316 | 11 | 11 | 11 | 1 | 1 | 0 |
| unavailable | present | 1 | 1 | 16317 | 16317 | 16317 | 1037 | 12 | 1 | 1 |
| successful | present | 3 | 1 | 16333 | 21 | 21 | 21 | 21 | 5 | 1 |
| successfully | present | 1 | 1 | 16325 | 21 | 21 | 5 | 5 | 2 | 1 |
| processing | present | 2 | 2 | 16313 | 3 | 3 | 3 | 3 | 3 | 2 |
| transaction | absent | 0 | 0 | 16315 | 139 | 139 | 3 | 3 | 2 | 0 |
| deployment | absent | 0 | 0 | 16312 | 6 | 6 | 6 | 6 | 6 | 0 |
| validation | present | 2 | 2 | 16313 | 7 | 7 | 7 | 7 | 7 | 3 |
| permission | absent | 0 | 0 | 16313 | 1042 | 102 | 102 | 13 | 13 | 0 |
| permissions | absent | 0 | 0 | 16313 | 1042 | 102 | 18 | 1 | 1 | 0 |
| certificate | absent | 0 | 0 | 16326 | 8 | 0 | 0 | 0 | 0 | 0 |
| credentials | present | 1 | 1 | 16316 | 1070 | 1070 | 56 | 6 | 1 | 1 |
| heartbeat | present | 4 | 3 | 16332 | 16332 | 1048 | 3 | 3 | 3 | 3 |
| keepalive | absent | 0 | 0 | 16322 | 16322 | 947 | 947 | 947 | 947 | 0 |
| healthcheck | absent | 0 | 0 | 16320 | 1078 | 1078 | 108 | 108 | 108 | 0 |
| exceptionhandler | absent | 0 | 0 | 16304 | 3 | 3 | 0 | 0 | 0 | 0 |
| internalservererror | absent | 0 | 0 | 16308 | 3 | 0 | 0 | 0 | 0 | 0 |
| invalidargument | absent | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 |
| outofmemory | absent | 0 | 0 | 16315 | 954 | 102 | 102 | 6 | 6 | 0 |
| nullpointerexception | absent | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 |
| numberformatexception | absent | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 |
| illegalstateexception | present | 1 | 1 | 16305 | 3 | 2 | 1 | 1 | 1 | 1 |
| indexoutofbounds | absent | 0 | 0 | 16309 | 1061 | 6 | 6 | 0 | 0 | 0 |
| a5b557df | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 15 |
| a5b557dfa94b | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 2 |
| a5b557dfa94b1f70 | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 1 |
| a5b557dfa94b1f70209f6a70 | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 3081 | 1 |
| a5b557dfa94b1f70209f6a70fbb39a5c | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 3080 | 1 |
| 6ed6a787 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 19 |
| 6ed6a787d02a | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 1 |
| 6ed6a787d02af80e | present | 1 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 1 |
| 6ed6a787d02af80e84c5ea01 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 6ed6a787d02af80e84c5ea014d267a2a | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 3065 | 1 |
| 793c3564 | present | 1 | 1 | 16375 | 16375 | 16375 | 16375 | 16375 | 3086 | 29 |
| 793c3564223a | present | 1 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 1213 | 3 |
| 793c3564223ab74c | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1212 | 1 |
| 793c3564223ab74c29f651a1 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1211 | 1 |
| 793c3564223ab74c29f651a1f4f41756 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1211 | 1 |
| e668977a | present | 1 | 1 | 16373 | 16373 | 16373 | 16373 | 16373 | 3080 | 60 |
| e668977af5cc | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 3076 | 1 |
| e668977af5cc0a19 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 3076 | 1 |
| e668977af5cc0a1953aa1431 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1518 | 1 |
| e668977af5cc0a1953aa143191d6379b | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 904 | 1 |
| 3447ff7f | present | 1 | 1 | 16377 | 16377 | 16377 | 16377 | 16377 | 1694 | 16 |
| 3447ff7f43d9 | present | 1 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 1688 | 1 |
| 3447ff7f43d94cd8 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1687 | 1 |
| 3447ff7f43d94cd814477f31 | present | 1 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 1676 | 1 |
| 3447ff7f43d94cd814477f31c3ccdb66 | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 1675 | 1 |
| 23504150 | present | 1 | 1 | 16794 | 16794 | 3337 | 3337 | 3337 | 1749 | 1053 |
| 23504150bf03 | present | 1 | 1 | 16374 | 16374 | 3081 | 3081 | 3081 | 1515 | 4 |
| 23504150bf03afe2 | present | 1 | 1 | 16371 | 16371 | 3078 | 3078 | 3078 | 1512 | 1 |
| 23504150bf03afe28db8f861 | present | 1 | 1 | 16368 | 16368 | 3075 | 3075 | 3075 | 1509 | 1 |
| 23504150bf03afe28db8f8610f728305 | present | 1 | 1 | 16368 | 16368 | 1486 | 1486 | 1486 | 832 | 1 |
| 0b3da49b | present | 1 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 21 |
| 0b3da49b0ecf | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 2 |
| 0b3da49b0ecf04c0 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 0b3da49b0ecf04c0127fae58 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| 0b3da49b0ecf04c0127fae58532caf18 | present | 1 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 1 |
| c000283a | present | 1 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 1479 | 52 |
| c000283a31c6 | present | 1 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 1479 | 1 |
| c000283a31c604eb | present | 1 | 1 | 16370 | 3073 | 3073 | 3073 | 3073 | 1478 | 1 |
| c000283a31c604eb3d8d670f | present | 1 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 1062 | 1 |
| c000283a31c604eb3d8d670f16332953 | present | 1 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 786 | 1 |
| error | present | 7 | 6 | 16341 | 16341 | 15 | 15 | 15 | 15 | 15 |
| user | present | 42 | 36 | 16337 | 16337 | 143 | 143 | 143 | 143 | 143 |
| service | present | 27 | 9 | 16323 | 16323 | 9 | 9 | 9 | 9 | 9 |
| client | present | 25 | 15 | 16325 | 1070 | 1070 | 15 | 15 | 15 | 15 |
| server | present | 11 | 6 | 16334 | 16334 | 19 | 19 | 6 | 6 | 6 |
| failed | present | 2 | 1 | 16340 | 996 | 996 | 100 | 2 | 2 | 2 |
| trace | present | 1 | 1 | 16340 | 16340 | 16340 | 2125 | 2125 | 2125 | 18 |
| traa | present | 42 | 38 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 141 |

## Retained and missing grams for shortlist

These rows are read from serialized files. Omitted partial rows mean unknown. A prefix-contained match needs no remainder grams. A prefix crossing checks only its continuation, so retaining all whole-query grams is not required.

| Literal | Budget | Retained grams | Missing/unknown grams |
| --- | --- | --- | --- |
| authentication | 200000 | ent, ica, cat, ati, tio, ion | aut, uth, the, hen, nti, tic |
| authentication | 300000 | ent, ica, cat, ati, tio, ion | aut, uth, the, hen, nti, tic |
| authentication | 400000 | the, hen, ent, ica, cat, ati, tio, ion | aut, uth, nti, tic |
| authentication | 500000 | the, hen, ent, tic, ica, cat, ati, tio, ion | aut, uth, nti |
| authentication | 1000000 | the, hen, ent, tic, ica, cat, ati, tio, ion | aut, uth, nti |
| configuration | 200000 | con, ati, tio, ion | onf, nfi, fig, igu, gur, ura, rat |
| configuration | 300000 | con, onf, nfi, fig, ura, rat, ati, tio, ion | igu, gur |
| configuration | 400000 | con, onf, nfi, fig, igu, gur, ura, rat, ati, tio, ion | none |
| configuration | 500000 | con, onf, nfi, fig, igu, gur, ura, rat, ati, tio, ion | none |
| configuration | 1000000 | con, onf, nfi, fig, igu, gur, ura, rat, ati, tio, ion | none |
| connectiontimeout | 200000 | con, ect, tio, ion, tim, ime | onn, nne, nec, cti, ont, nti, meo, eou, out |
| connectiontimeout | 300000 | con, ect, tio, ion, tim, ime, out | onn, nne, nec, cti, ont, nti, meo, eou |
| connectiontimeout | 400000 | con, ect, cti, tio, ion, tim, ime, out | onn, nne, nec, ont, nti, meo, eou |
| connectiontimeout | 500000 | con, nne, ect, cti, tio, ion, ont, tim, ime, out | onn, nec, nti, meo, eou |
| connectiontimeout | 1000000 | con, onn, nne, nec, ect, cti, tio, ion, ont, tim, ime, out | nti, meo, eou |
| successful | 200000 | cce, ces, ess | suc, ucc, ssf, sfu, ful |
| successful | 300000 | cce, ces, ess | suc, ucc, ssf, sfu, ful |
| successful | 400000 | cce, ces, ess | suc, ucc, ssf, sfu, ful |
| successful | 500000 | cce, ces, ess | suc, ucc, ssf, sfu, ful |
| successful | 1000000 | suc, ucc, cce, ces, ess | ssf, sfu, ful |
| permissions | 200000 | ion | per, erm, rmi, mis, iss, ssi, sio, ons |
| permissions | 300000 | per, ion | erm, rmi, mis, iss, ssi, sio, ons |
| permissions | 400000 | per, ion, ons | erm, rmi, mis, iss, ssi, sio |
| permissions | 500000 | per, erm, ion, ons | rmi, mis, iss, ssi, sio |
| permissions | 1000000 | per, erm, ion, ons | rmi, mis, iss, ssi, sio |
| heartbeat | 200000 | none | hea, ear, art, rtb, tbe, bea, eat |
| heartbeat | 300000 | art | hea, ear, rtb, tbe, bea, eat |
| heartbeat | 400000 | hea, ear, art, rtb, tbe, bea, eat | none |
| heartbeat | 500000 | hea, ear, art, rtb, tbe, bea, eat | none |
| heartbeat | 1000000 | hea, ear, art, rtb, tbe, bea, eat | none |
| healthcheck | 200000 | che | hea, eal, alt, lth, thc, hch, hec, eck |
| healthcheck | 300000 | che | hea, eal, alt, lth, thc, hch, hec, eck |
| healthcheck | 400000 | hea, che | eal, alt, lth, thc, hch, hec, eck |
| healthcheck | 500000 | hea, che | eal, alt, lth, thc, hch, hec, eck |
| healthcheck | 1000000 | hea, che | eal, alt, lth, thc, hch, hec, eck |
| outofmemory | 200000 | ory | out, uto, tof, ofm, fme, mem, emo, mor |
| outofmemory | 300000 | out, ory | uto, tof, ofm, fme, mem, emo, mor |
| outofmemory | 400000 | out, ory | uto, tof, ofm, fme, mem, emo, mor |
| outofmemory | 500000 | out, uto, ory | tof, ofm, fme, mem, emo, mor |
| outofmemory | 1000000 | out, uto, ory | tof, ofm, fme, mem, emo, mor |
| nullpointerexception | 200000 | oin, int, nte, ter, ere, cep, ept, tio, ion | nul, ull, llp, lpo, poi, rex, exc, xce, pti |
| nullpointerexception | 300000 | oin, int, nte, ter, ere, cep, ept, tio, ion | nul, ull, llp, lpo, poi, rex, exc, xce, pti |
| nullpointerexception | 400000 | nul, ull, oin, int, nte, ter, ere, exc, xce, cep, ept, pti, tio, ion | llp, lpo, poi, rex |
| nullpointerexception | 500000 | nul, ull, oin, int, nte, ter, ere, exc, xce, cep, ept, pti, tio, ion | llp, lpo, poi, rex |
| nullpointerexception | 1000000 | nul, ull, oin, int, nte, ter, ere, exc, xce, cep, ept, pti, tio, ion | llp, lpo, poi, rex |
| indexoutofbounds | 200000 | oun | ind, nde, dex, exo, xou, out, uto, tof, ofb, fbo, bou, und, nds |
| indexoutofbounds | 300000 | nde, out, oun | ind, dex, exo, xou, uto, tof, ofb, fbo, bou, und, nds |
| indexoutofbounds | 400000 | nde, out, oun | ind, dex, exo, xou, uto, tof, ofb, fbo, bou, und, nds |
| indexoutofbounds | 500000 | nde, out, uto, oun, und | ind, dex, exo, xou, tof, ofb, fbo, bou, nds |
| indexoutofbounds | 1000000 | nde, out, uto, oun, und | ind, dex, exo, xou, tof, ofb, fbo, bou, nds |

## Validation

```json
{
  "partial_indexes": {
    "document_200000": {
      "every_retained_posting_equal": true,
      "memberships": 264895,
      "missing_is_unknown": true,
      "rows": 304
    },
    "document_300000": {
      "every_retained_posting_equal": true,
      "memberships": 402310,
      "missing_is_unknown": true,
      "rows": 476
    },
    "document_400000": {
      "every_retained_posting_equal": true,
      "memberships": 541444,
      "missing_is_unknown": true,
      "rows": 646
    },
    "document_500000": {
      "every_retained_posting_equal": true,
      "memberships": 676211,
      "missing_is_unknown": true,
      "rows": 828
    },
    "document_1000000": {
      "every_retained_posting_equal": true,
      "memberships": 1797473,
      "missing_is_unknown": true,
      "rows": 1223
    }
  },
  "source_files": 117,
  "all_source_hashes_unchanged": true,
  "frozen_manifests_unchanged": true,
  "fresh_exact_truth": true,
  "all_physical_boundaries_checked": true,
  "full_lcp_remainder_summaries": true,
  "decoded_prefix_and_byte_equality": true,
  "identical_retained_frames": true,
  "zero_false_negatives": true,
  "truth_subset_full_subset_partial_subset_byte": true,
  "nested_candidate_sets_monotone": true,
  "controls_per_block_and_totals_equal": true,
  "decisions_reloaded_recomputed_recounted": true,
  "audited_blocks": 16794,
  "decision_width_bytes": 12,
  "query_mode_block_decisions": 10462662,
  "retained_rows_verified": 3477
}
```

Full-reference excess above true blocks measures residual gram co-occurrence/prefix false positives. Partial excess above full measures missing-row coverage. JSON includes per-query FP/skip counts and per-gram remainder block DF.

- Diagnostic cohort, not representative production traffic.
- Source-derived families may be ID-like. Length32 is not a universal query-length recommendation.
- Counts are matching dictionary terms and physical candidate blocks, not document query DF or latency.
- Missing partial rows are unknown. Full missing rows are absent. Prefix continuations can use fewer grams.
- Budgets are additional decimal bytes. Charge byte/prefix sidecars once. Reports and audit artifacts are excluded.
