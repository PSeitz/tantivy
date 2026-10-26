# Coalescing-gap sweep

Split `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, physical `message` field 3. Each cutoff removes entire inline lists and compacts the remaining external postings before calculating request ranges. The sweep covers gaps from 0 through 1 MiB, including inline cutoffs of one, two, three, four, seven, and eight documents.

Each cell is **request MB / request count**. MB = 1,000,000 bytes. KiB = 1,024 bytes and MiB = 1,048,576 bytes. The gap is the maximum distance between successive matching external ranges that are merged. A zero gap merges touching ranges only. Request bytes include unrelated postings inside merged gaps.

The baseline stores every list externally. Document cutoffs refer to documents per list. Dictionary and positions reads are excluded. These are offline plans with no cache, retry, concurrency, request-size cap, or latency model.

`CUTOFFS.md` includes dictionary sizes and a focused comparison around 512 KiB. Exact values for the complete sweep are in `coalescing.json`.

## `*a*`

1,921,269 matching lists at every cutoff.

| Maximum gap | No inline lists | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents | ≤7 documents | ≤8 documents |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 43.987 / 961,079 | 34.848 / 2,974 | 34.821 / 1,814 | 34.804 / 1,392 | 34.789 / 1,173 | 34.728 / 823 | 34.697 / 717 |
| 64 KiB | 155.330 / 253 | 38.135 / 731 | 36.956 / 557 | 36.887 / 481 | 36.685 / 435 | 36.220 / 319 | 36.048 / 290 |
| 128 KiB | 164.038 / 155 | 75.296 / 326 | 64.244 / 254 | 61.324 / 210 | 59.884 / 178 | 51.439 / 156 | 49.117 / 152 |
| 256 KiB | 173.890 / 98 | 107.270 / 158 | 81.488 / 160 | 75.533 / 132 | 73.814 / 101 | 69.350 / 63 | 65.421 / 68 |
| 512 KiB | 195.244 / 48 | 135.102 / 83 | 103.480 / 101 | 89.582 / 94 | 83.993 / 73 | 82.032 / 30 | 79.121 / 33 |
| 1 MiB | 215.940 / 18 | 175.967 / 22 | 152.279 / 35 | 123.962 / 49 | 103.126 / 47 | 92.635 / 16 | 92.343 / 15 |

## `*ab*`

75,774 matching lists at every cutoff.

| Maximum gap | No inline lists | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents | ≤7 documents | ≤8 documents |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0.657 / 57,681 | 0.297 / 217 | 0.296 / 148 | 0.295 / 124 | 0.294 / 101 | 0.290 / 78 | 0.288 / 58 |
| 64 KiB | 99.576 / 519 | 1.088 / 102 | 0.968 / 76 | 0.940 / 62 | 0.859 / 49 | 0.740 / 39 | 0.621 / 34 |
| 128 KiB | 127.515 / 199 | 1.176 / 101 | 0.968 / 76 | 0.940 / 62 | 0.859 / 49 | 0.740 / 39 | 0.621 / 34 |
| 256 KiB | 138.807 / 134 | 1.734 / 98 | 1.525 / 73 | 1.497 / 59 | 1.265 / 47 | 1.147 / 37 | 1.027 / 32 |
| 512 KiB | 162.545 / 76 | 5.189 / 89 | 3.498 / 68 | 2.838 / 56 | 2.665 / 44 | 2.559 / 34 | 1.948 / 30 |
| 1 MiB | 196.082 / 28 | 30.960 / 60 | 24.483 / 45 | 20.895 / 36 | 18.422 / 27 | 9.881 / 26 | 4.321 / 27 |

## `*aba*`

2,817 matching lists at every cutoff.

| Maximum gap | No inline lists | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents | ≤7 documents | ≤8 documents |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0.158 / 2,384 | 0.144 / 11 | 0.144 / 9 | 0.144 / 9 | 0.144 / 9 | 0.144 / 6 | 0.144 / 6 |
| 64 KiB | 20.960 / 497 | 0.152 / 10 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 | 0.144 / 6 | 0.144 / 6 |
| 128 KiB | 34.813 / 351 | 0.152 / 10 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 | 0.144 / 6 | 0.144 / 6 |
| 256 KiB | 57.017 / 237 | 0.303 / 9 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 | 0.144 / 6 | 0.144 / 6 |
| 512 KiB | 93.638 / 140 | 0.303 / 9 | 0.152 / 8 | 0.152 / 8 | 0.152 / 8 | 0.144 / 6 | 0.657 / 5 |
| 1 MiB | 163.219 / 44 | 0.875 / 8 | 0.722 / 7 | 0.721 / 7 | 0.716 / 7 | 0.677 / 5 | 0.657 / 5 |

At 512 KiB, increasing the inline cutoff from seven to eight documents compacts a gap enough to trigger an additional merge. This increases fetched bytes while reducing requests from six to five.

## `*player*`

21 matching lists. None is inline at the displayed cutoffs. Removing other terms' postings still compacts the gaps between these lists, changing coalescing decisions.

| Maximum gap | No inline lists | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents | ≤7 documents | ≤8 documents |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 4.222 / 15 | 4.222 / 14 | 4.222 / 14 | 4.222 / 14 | 4.222 / 14 | 4.222 / 14 | 4.222 / 14 |
| 64 KiB | 4.224 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 |
| 128 KiB | 4.224 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 | 4.222 / 13 |
| 256 KiB | 4.442 / 12 | 4.701 / 11 | 4.700 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 |
| 512 KiB | 4.883 / 11 | 4.701 / 11 | 4.700 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 |
| 1 MiB | 4.883 / 11 | 4.701 / 11 | 4.700 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 | 4.699 / 11 |

## `*ms*`

50,585 matching lists. Of these, 50,538 are inline at ≤1 document, 50,556 at each cutoff from ≤2 through ≤4 documents, and 50,558 at both ≤7 and ≤8 documents.

| Maximum gap | No inline lists | ≤1 document | ≤2 documents | ≤3 documents | ≤4 documents | ≤7 documents | ≤8 documents |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 1.067 / 38,838 | 0.825 / 39 | 0.825 / 22 | 0.825 / 22 | 0.825 / 22 | 0.825 / 20 | 0.825 / 20 |
| 64 KiB | 37.978 / 673 | 0.889 / 33 | 0.859 / 21 | 0.851 / 21 | 0.847 / 21 | 0.845 / 19 | 0.845 / 19 |
| 128 KiB | 62.895 / 411 | 1.108 / 31 | 0.955 / 20 | 0.948 / 20 | 0.944 / 20 | 0.941 / 18 | 0.941 / 18 |
| 256 KiB | 100.837 / 210 | 1.298 / 30 | 1.145 / 19 | 1.137 / 19 | 1.133 / 19 | 1.131 / 17 | 1.131 / 17 |
| 512 KiB | 140.639 / 101 | 2.070 / 28 | 1.571 / 18 | 1.563 / 18 | 1.559 / 18 | 1.557 / 16 | 1.557 / 16 |
| 1 MiB | 191.696 / 30 | 5.140 / 24 | 3.790 / 15 | 3.781 / 15 | 3.772 / 15 | 2.923 / 14 | 2.923 / 14 |

## Other patterns

For `*traaa*`, the baseline reads 10 bytes in two requests at every tested gap. Both matching lists become inline at every nonzero tested inline cutoff, yielding zero external requests. `traaa*` matches no lists and has zero external requests throughout.
