# Same 89 queries: maximum block-presence caps 10%–80%

Each of 40 cap/budget configurations was selected, serialized, reopened, frame-compared and Rust-decoded. Caps 30–80 have identical complete eligible/selected lists and bytes at each budget. Shared computation follows this verification. See validation counts below.

Eligibility is inclusive: blockDF × 100 ≤ cap × 16,794. Ranking is descending exact distinct-document DF, then ascending u24. Whole frames that fit are retained, nonfitting rows are skipped. No queries enter selection. Missing rows mean unknown. All evaluation retains full-LCP prefix-contained/crossing safety and byte fallback.

The frozen independent full-key truth was reused after SHA-256/provenance verification. Physical boundaries, decoded byte/prefix sidecars and the eight sampled source identities were rechecked. No 7.6M-key rescan was needed.

## Actual additional metadata

Actual B = 5,323 shared byte/prefix sidecar bytes + partial file (18 + 7×rows + frame bytes). Sidecars are charged once per deployable index. Percent is additional metadata / original 195,640 B. Full-reference metadata is 38,073,845 B. Reports and decisions are excluded.

| Budget B | Cap % | Actual B | +original % | Rows | Eligible |
| --- | --- | --- | --- | --- | --- |
| 200000 | 10 | 199996 | 102.2265 | 299 | 43700 |
| 200000 | 20 | 199999 | 102.2281 | 304 | 47198 |
| 200000 | 30 | 199996 | 102.2265 | 306 | 47796 |
| 200000 | 40 | 199996 | 102.2265 | 306 | 47796 |
| 200000 | 50 | 199996 | 102.2265 | 306 | 47796 |
| 200000 | 60 | 199996 | 102.2265 | 306 | 47796 |
| 200000 | 70 | 199996 | 102.2265 | 306 | 47796 |
| 200000 | 80 | 199996 | 102.2265 | 306 | 47796 |
| 300000 | 10 | 299989 | 153.3373 | 482 | 43700 |
| 300000 | 20 | 300000 | 153.3429 | 476 | 47198 |
| 300000 | 30 | 299996 | 153.3408 | 424 | 47796 |
| 300000 | 40 | 299996 | 153.3408 | 424 | 47796 |
| 300000 | 50 | 299996 | 153.3408 | 424 | 47796 |
| 300000 | 60 | 299996 | 153.3408 | 424 | 47796 |
| 300000 | 70 | 299996 | 153.3408 | 424 | 47796 |
| 300000 | 80 | 299996 | 153.3408 | 424 | 47796 |
| 400000 | 10 | 400000 | 204.4572 | 686 | 43700 |
| 400000 | 20 | 399990 | 204.4521 | 646 | 47198 |
| 400000 | 30 | 399991 | 204.4526 | 621 | 47796 |
| 400000 | 40 | 399991 | 204.4526 | 621 | 47796 |
| 400000 | 50 | 399991 | 204.4526 | 621 | 47796 |
| 400000 | 60 | 399991 | 204.4526 | 621 | 47796 |
| 400000 | 70 | 399991 | 204.4526 | 621 | 47796 |
| 400000 | 80 | 399991 | 204.4526 | 621 | 47796 |
| 500000 | 10 | 499998 | 255.5704 | 864 | 43700 |
| 500000 | 20 | 499997 | 255.5699 | 828 | 47198 |
| 500000 | 30 | 499996 | 255.5694 | 760 | 47796 |
| 500000 | 40 | 499996 | 255.5694 | 760 | 47796 |
| 500000 | 50 | 499996 | 255.5694 | 760 | 47796 |
| 500000 | 60 | 499996 | 255.5694 | 760 | 47796 |
| 500000 | 70 | 499996 | 255.5694 | 760 | 47796 |
| 500000 | 80 | 499996 | 255.5694 | 760 | 47796 |
| 1000000 | 10 | 1000000 | 511.1429 | 2063 | 43700 |
| 1000000 | 20 | 999997 | 511.1414 | 1223 | 47198 |
| 1000000 | 30 | 999999 | 511.1424 | 1207 | 47796 |
| 1000000 | 40 | 999999 | 511.1424 | 1207 | 47796 |
| 1000000 | 50 | 999999 | 511.1424 | 1207 | 47796 |
| 1000000 | 60 | 999999 | 511.1424 | 1207 | 47796 |
| 1000000 | 70 | 999999 | 511.1424 | 1207 | 47796 |
| 1000000 | 80 | 999999 | 511.1424 | 1207 | 47796 |

Maximum remainder-block DF: 3588/16,794 = 21.364773133262%.

## Separate group means

Every query receives equal weight. Lower candidate counts are better. All false-negative counts are zero.

### natural_present: 14 queries

True-block sum: 20. Byte mean: 16316.857. Full mean: 1.500.

| Budget | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 1342.000 | 2420.143 | 2422.143 | 2422.143 | 2422.143 | 2422.143 | 2422.143 | 2422.143 |
| 300000 | 1264.643 | 1327.643 | 1327.786 | 1327.786 | 1327.786 | 1327.786 | 1327.786 | 1327.786 |
| 400000 | 17.714 | 87.643 | 1185.000 | 1185.000 | 1185.000 | 1185.000 | 1185.000 | 1185.000 |
| 500000 | 2.714 | 5.214 | 10.500 | 10.500 | 10.500 | 10.500 | 10.500 | 10.500 |
| 1000000 | 1.500 | 2.714 | 4.071 | 4.071 | 4.071 | 4.071 | 4.071 | 4.071 |

- 200000 B: best tested cap(s) [10], mean 1342.000.
- 300000 B: best tested cap(s) [10], mean 1264.643.
- 400000 B: best tested cap(s) [10], mean 17.714.
- 500000 B: best tested cap(s) [10], mean 2.714.
- 1000000 B: best tested cap(s) [10], mean 1.500.

### natural_absent: 27 queries

True-block sum: 0. Byte mean: 16311.815. Full mean: 0.000.

| Budget | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 809.222 | 810.444 | 819.889 | 819.889 | 819.889 | 819.889 | 819.889 | 819.889 |
| 300000 | 93.889 | 93.926 | 133.148 | 133.148 | 133.148 | 133.148 | 133.148 | 133.148 |
| 400000 | 45.296 | 49.148 | 85.741 | 85.741 | 85.741 | 85.741 | 85.741 | 85.741 |
| 500000 | 40.963 | 41.000 | 45.296 | 45.296 | 45.296 | 45.296 | 45.296 | 45.296 |
| 1000000 | 4.259 | 40.963 | 41.000 | 41.000 | 41.000 | 41.000 | 41.000 | 41.000 |

- 200000 B: best tested cap(s) [10], mean 809.222.
- 300000 B: best tested cap(s) [10], mean 93.889.
- 400000 B: best tested cap(s) [10], mean 45.296.
- 500000 B: best tested cap(s) [10], mean 40.963.
- 1000000 B: best tested cap(s) [10], mean 4.259.

### source_present: 40 queries

True-block sum: 40. Byte mean: 16379.925. Full mean: 32.600.

| Budget | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 16379.925 | 14717.800 | 14717.800 | 14717.800 | 14717.800 | 14717.800 | 14717.800 | 14717.800 |
| 300000 | 16379.925 | 13012.350 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 |
| 400000 | 16379.925 | 13012.350 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 |
| 500000 | 16379.925 | 13012.350 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 | 12897.225 |
| 1000000 | 16379.925 | 6176.250 | 6941.700 | 6941.700 | 6941.700 | 6941.700 | 6941.700 | 6941.700 |

- 200000 B: best tested cap(s) [20, 30, 40, 50, 60, 70, 80], mean 14717.800.
- 300000 B: best tested cap(s) [30, 40, 50, 60, 70, 80], mean 12897.225.
- 400000 B: best tested cap(s) [30, 40, 50, 60, 70, 80], mean 12897.225.
- 500000 B: best tested cap(s) [30, 40, 50, 60, 70, 80], mean 12897.225.
- 1000000 B: best tested cap(s) [20], mean 6176.250.

### controls: 8 queries

True-block sum: 112. Byte mean: 16335.125. Full mean: 43.625.

| Budget | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 6530.750 | 12510.250 | 12510.250 | 12510.250 | 12510.250 | 12510.250 | 12510.250 | 12510.250 |
| 300000 | 4134.500 | 4366.625 | 4381.500 | 4381.500 | 4381.500 | 4381.500 | 4381.500 | 4381.500 |
| 400000 | 4108.875 | 2345.875 | 2345.875 | 2345.875 | 2345.875 | 2345.875 | 2345.875 | 2345.875 |
| 500000 | 4108.875 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 |
| 1000000 | 4108.875 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 | 2332.000 |

- 200000 B: best tested cap(s) [10], mean 6530.750.
- 300000 B: best tested cap(s) [10], mean 4134.500.
- 400000 B: best tested cap(s) [20, 30, 40, 50, 60, 70, 80], mean 2345.875.
- 500000 B: best tested cap(s) [20, 30, 40, 50, 60, 70, 80], mean 2332.000.
- 1000000 B: best tested cap(s) [20, 30, 40, 50, 60, 70, 80], mean 2332.000.

## Source-derived lengths

All 40 queries match one term in one block. Eight families per length. Nested candidate containment was checked on every physical block for every distinct index. Full-reference means are 8: 158.125, 12: 1.875, 16: 1, 24: 1, 32: 1.

| Budget | Length | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 200000 | 8 | 16424.625 | 14762.500 | 14762.500 | 14762.500 | 14762.500 | 14762.500 | 14762.500 | 14762.500 |
| 200000 | 12 | 16370.125 | 14708.000 | 14708.000 | 14708.000 | 14708.000 | 14708.000 | 14708.000 | 14708.000 |
| 200000 | 16 | 16369.125 | 14707.000 | 14707.000 | 14707.000 | 14707.000 | 14707.000 | 14707.000 | 14707.000 |
| 200000 | 24 | 16368.125 | 14706.000 | 14706.000 | 14706.000 | 14706.000 | 14706.000 | 14706.000 | 14706.000 |
| 200000 | 32 | 16367.625 | 14705.500 | 14705.500 | 14705.500 | 14705.500 | 14705.500 | 14705.500 | 14705.500 |
| 300000 | 8 | 16424.625 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 |
| 300000 | 12 | 16370.125 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 |
| 300000 | 16 | 16369.125 | 13045.375 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 |
| 300000 | 24 | 16368.125 | 13044.375 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 |
| 300000 | 32 | 16367.625 | 12845.250 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 |
| 400000 | 8 | 16424.625 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 |
| 400000 | 12 | 16370.125 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 |
| 400000 | 16 | 16369.125 | 13045.375 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 |
| 400000 | 24 | 16368.125 | 13044.375 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 |
| 400000 | 32 | 16367.625 | 12845.250 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 |
| 500000 | 8 | 16424.625 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 | 13080.375 |
| 500000 | 12 | 16370.125 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 | 13046.375 |
| 500000 | 16 | 16369.125 | 13045.375 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 | 12853.500 |
| 500000 | 24 | 16368.125 | 13044.375 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 | 12852.500 |
| 500000 | 32 | 16367.625 | 12845.250 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 | 12653.375 |
| 1000000 | 8 | 16424.625 | 7524.375 | 9349.250 | 9349.250 | 9349.250 | 9349.250 | 9349.250 | 9349.250 |
| 1000000 | 12 | 16370.125 | 7259.500 | 9129.750 | 9129.750 | 9129.750 | 9129.750 | 9129.750 | 9129.750 |
| 1000000 | 16 | 16369.125 | 7258.500 | 9080.125 | 9080.125 | 9080.125 | 9080.125 | 9080.125 | 9080.125 |
| 1000000 | 24 | 16368.125 | 5348.875 | 3678.500 | 3678.500 | 3678.500 | 3678.500 | 3678.500 | 3678.500 |
| 1000000 | 32 | 16367.625 | 3490.000 | 3470.875 | 3470.875 | 3470.875 | 3470.875 | 3470.875 | 3470.875 |

## Same-budget changes

Caps 30–80 are byte-identical. The following adjacent-cap changes include newly allowed/pruned block IDs, including equal-count swaps. JSON records full added/removed row details with block DF, document DF and exact costs.

| Before | After | Query | Candidates before | After | Delta | Newly allowed | Newly pruned |
| --- | --- | --- | --- | --- | --- | --- | --- |
| budget_200000_presence10 | budget_200000_presence20 | *serialization* | 6 | 34 | 28 | 28 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *deserialization* | 0 | 3 | 3 | 3 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *successful* | 116 | 21 | -95 | 0 | 95 |
| budget_200000_presence10 | budget_200000_presence20 | *successfully* | 116 | 21 | -95 | 0 | 95 |
| budget_200000_presence10 | budget_200000_presence20 | *heartbeat* | 1048 | 16332 | 15284 | 15284 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *internalservererror* | 1 | 3 | 2 | 2 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *c000283a* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_200000_presence10 | budget_200000_presence20 | *c000283a31c6* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_200000_presence10 | budget_200000_presence20 | *c000283a31c604eb* | 16370 | 3073 | -13297 | 0 | 13297 |
| budget_200000_presence10 | budget_200000_presence20 | *c000283a31c604eb3d8d670f* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_200000_presence10 | budget_200000_presence20 | *c000283a31c604eb3d8d670f16332953* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_200000_presence10 | budget_200000_presence20 | *user* | 1011 | 16337 | 15326 | 15326 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *service* | 9 | 16323 | 16314 | 16314 | 0 |
| budget_200000_presence10 | budget_200000_presence20 | *server* | 138 | 16334 | 16196 | 16196 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *configuration* | 10 | 29 | 19 | 19 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *initialization* | 7 | 13 | 6 | 6 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *serialization* | 34 | 139 | 105 | 105 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *deserialization* | 3 | 139 | 136 | 136 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *synchronization* | 6 | 14 | 8 | 8 | 0 |
| budget_200000_presence20 | budget_200000_presence30 | *validation* | 7 | 16 | 9 | 9 | 0 |
| budget_300000_presence10 | budget_300000_presence20 | *authentication* | 1 | 4 | 3 | 3 | 0 |
| budget_300000_presence10 | budget_300000_presence20 | *requestfailed* | 2 | 3 | 1 | 1 | 0 |
| budget_300000_presence10 | budget_300000_presence20 | *successful* | 116 | 21 | -95 | 0 | 95 |
| budget_300000_presence10 | budget_300000_presence20 | *successfully* | 11 | 21 | 10 | 16 | 6 |
| budget_300000_presence10 | budget_300000_presence20 | *heartbeat* | 84 | 1048 | 964 | 964 | 0 |
| budget_300000_presence10 | budget_300000_presence20 | *23504150* | 16794 | 3337 | -13457 | 0 | 13457 |
| budget_300000_presence10 | budget_300000_presence20 | *23504150bf03* | 16374 | 3081 | -13293 | 0 | 13293 |
| budget_300000_presence10 | budget_300000_presence20 | *23504150bf03afe2* | 16371 | 3078 | -13293 | 0 | 13293 |
| budget_300000_presence10 | budget_300000_presence20 | *23504150bf03afe28db8f861* | 16368 | 3075 | -13293 | 0 | 13293 |
| budget_300000_presence10 | budget_300000_presence20 | *23504150bf03afe28db8f8610f728305* | 16368 | 1486 | -14882 | 0 | 14882 |
| budget_300000_presence10 | budget_300000_presence20 | *c000283a* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_300000_presence10 | budget_300000_presence20 | *c000283a31c6* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_300000_presence10 | budget_300000_presence20 | *c000283a31c604eb* | 16370 | 3073 | -13297 | 0 | 13297 |
| budget_300000_presence10 | budget_300000_presence20 | *c000283a31c604eb3d8d670f* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_300000_presence10 | budget_300000_presence20 | *c000283a31c604eb3d8d670f16332953* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_300000_presence10 | budget_300000_presence20 | *client* | 109 | 1070 | 961 | 961 | 0 |
| budget_300000_presence10 | budget_300000_presence20 | *failed* | 100 | 996 | 896 | 896 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *configuration* | 3 | 4 | 1 | 1 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *connectionfailed* | 0 | 1 | 1 | 1 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *requesttimeout* | 1 | 3 | 2 | 2 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *timedout* | 14 | 139 | 125 | 125 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *certificate* | 0 | 1 | 1 | 1 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *outofmemory* | 102 | 954 | 852 | 852 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *illegalstateexception* | 2 | 3 | 1 | 1 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *indexoutofbounds* | 6 | 84 | 78 | 78 | 0 |
| budget_300000_presence20 | budget_300000_presence30 | *c000283a31c604eb* | 3073 | 1538 | -1535 | 0 | 1535 |
| budget_300000_presence20 | budget_300000_presence30 | *c000283a31c604eb3d8d670f* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_300000_presence20 | budget_300000_presence30 | *c000283a31c604eb3d8d670f16332953* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_300000_presence20 | budget_300000_presence30 | *server* | 19 | 138 | 119 | 119 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *disconnected* | 1 | 4 | 3 | 3 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *connectiontimeout* | 1 | 2 | 1 | 1 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *requestfailed* | 0 | 2 | 2 | 2 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *connecttimeout* | 1 | 2 | 1 | 1 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *unavailable* | 12 | 1037 | 1025 | 1025 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *successful* | 116 | 21 | -95 | 0 | 95 |
| budget_400000_presence10 | budget_400000_presence20 | *successfully* | 11 | 5 | -6 | 0 | 6 |
| budget_400000_presence10 | budget_400000_presence20 | *credentials* | 6 | 56 | 50 | 50 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *outofmemory* | 6 | 102 | 96 | 96 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *indexoutofbounds* | 0 | 6 | 6 | 6 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *23504150* | 16794 | 3337 | -13457 | 0 | 13457 |
| budget_400000_presence10 | budget_400000_presence20 | *23504150bf03* | 16374 | 3081 | -13293 | 0 | 13293 |
| budget_400000_presence10 | budget_400000_presence20 | *23504150bf03afe2* | 16371 | 3078 | -13293 | 0 | 13293 |
| budget_400000_presence10 | budget_400000_presence20 | *23504150bf03afe28db8f861* | 16368 | 3075 | -13293 | 0 | 13293 |
| budget_400000_presence10 | budget_400000_presence20 | *23504150bf03afe28db8f8610f728305* | 16368 | 1486 | -14882 | 0 | 14882 |
| budget_400000_presence10 | budget_400000_presence20 | *c000283a* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_400000_presence10 | budget_400000_presence20 | *c000283a31c6* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_400000_presence10 | budget_400000_presence20 | *c000283a31c604eb* | 16370 | 3073 | -13297 | 0 | 13297 |
| budget_400000_presence10 | budget_400000_presence20 | *c000283a31c604eb3d8d670f* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_400000_presence10 | budget_400000_presence20 | *c000283a31c604eb3d8d670f16332953* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_400000_presence10 | budget_400000_presence20 | *server* | 6 | 19 | 13 | 13 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *failed* | 2 | 100 | 98 | 98 | 0 |
| budget_400000_presence10 | budget_400000_presence20 | *trace* | 16340 | 2125 | -14215 | 0 | 14215 |
| budget_400000_presence20 | budget_400000_presence30 | *reconnection* | 1 | 3 | 2 | 2 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *connectiontimeout* | 2 | 3 | 1 | 1 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *unavailable* | 1037 | 16317 | 15280 | 15280 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *transaction* | 3 | 16 | 13 | 13 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *heartbeat* | 3 | 84 | 81 | 81 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *healthcheck* | 108 | 1078 | 970 | 970 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *exceptionhandler* | 0 | 3 | 3 | 3 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *illegalstateexception* | 1 | 2 | 1 | 1 | 0 |
| budget_400000_presence20 | budget_400000_presence30 | *c000283a31c604eb* | 3073 | 1538 | -1535 | 0 | 1535 |
| budget_400000_presence20 | budget_400000_presence30 | *c000283a31c604eb3d8d670f* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_400000_presence20 | budget_400000_presence30 | *c000283a31c604eb3d8d670f16332953* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_500000_presence10 | budget_500000_presence20 | *unavailable* | 1 | 12 | 11 | 11 | 0 |
| budget_500000_presence10 | budget_500000_presence20 | *successful* | 5 | 21 | 16 | 16 | 0 |
| budget_500000_presence10 | budget_500000_presence20 | *successfully* | 2 | 5 | 3 | 3 | 0 |
| budget_500000_presence10 | budget_500000_presence20 | *transaction* | 2 | 3 | 1 | 1 | 0 |
| budget_500000_presence10 | budget_500000_presence20 | *credentials* | 1 | 6 | 5 | 5 | 0 |
| budget_500000_presence10 | budget_500000_presence20 | *23504150* | 16794 | 3337 | -13457 | 0 | 13457 |
| budget_500000_presence10 | budget_500000_presence20 | *23504150bf03* | 16374 | 3081 | -13293 | 0 | 13293 |
| budget_500000_presence10 | budget_500000_presence20 | *23504150bf03afe2* | 16371 | 3078 | -13293 | 0 | 13293 |
| budget_500000_presence10 | budget_500000_presence20 | *23504150bf03afe28db8f861* | 16368 | 3075 | -13293 | 0 | 13293 |
| budget_500000_presence10 | budget_500000_presence20 | *23504150bf03afe28db8f8610f728305* | 16368 | 1486 | -14882 | 0 | 14882 |
| budget_500000_presence10 | budget_500000_presence20 | *c000283a* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_500000_presence10 | budget_500000_presence20 | *c000283a31c6* | 16371 | 3074 | -13297 | 0 | 13297 |
| budget_500000_presence10 | budget_500000_presence20 | *c000283a31c604eb* | 16370 | 3073 | -13297 | 0 | 13297 |
| budget_500000_presence10 | budget_500000_presence20 | *c000283a31c604eb3d8d670f* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_500000_presence10 | budget_500000_presence20 | *c000283a31c604eb3d8d670f16332953* | 16367 | 3070 | -13297 | 0 | 13297 |
| budget_500000_presence10 | budget_500000_presence20 | *trace* | 16340 | 2125 | -14215 | 0 | 14215 |
| budget_500000_presence20 | budget_500000_presence30 | *unexpected* | 8 | 82 | 74 | 74 | 0 |
| budget_500000_presence20 | budget_500000_presence30 | *unsupported* | 1 | 11 | 10 | 10 | 0 |
| budget_500000_presence20 | budget_500000_presence30 | *permission* | 13 | 102 | 89 | 89 | 0 |
| budget_500000_presence20 | budget_500000_presence30 | *permissions* | 1 | 18 | 17 | 17 | 0 |
| budget_500000_presence20 | budget_500000_presence30 | *c000283a31c604eb* | 3073 | 1538 | -1535 | 0 | 1535 |
| budget_500000_presence20 | budget_500000_presence30 | *c000283a31c604eb3d8d670f* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_500000_presence20 | budget_500000_presence30 | *c000283a31c604eb3d8d670f16332953* | 3070 | 1535 | -1535 | 0 | 1535 |
| budget_1000000_presence10 | budget_1000000_presence20 | *synchronization* | 0 | 6 | 6 | 6 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *requesttimeout* | 0 | 1 | 1 | 1 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *timedout* | 1 | 14 | 13 | 13 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *unexpected* | 1 | 8 | 7 | 7 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *successful* | 1 | 5 | 4 | 4 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *successfully* | 1 | 2 | 1 | 1 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *processing* | 2 | 3 | 1 | 1 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *deployment* | 1 | 6 | 5 | 5 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *validation* | 3 | 7 | 4 | 4 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *permission* | 0 | 13 | 13 | 13 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *permissions* | 0 | 1 | 1 | 1 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *keepalive* | 1 | 947 | 946 | 946 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *outofmemory* | 0 | 6 | 6 | 6 | 0 |
| budget_1000000_presence10 | budget_1000000_presence20 | *a5b557dfa94b1f70209f6a70* | 16368 | 3081 | -13287 | 0 | 13287 |
| budget_1000000_presence10 | budget_1000000_presence20 | *a5b557dfa94b1f70209f6a70fbb39a5c* | 16367 | 3080 | -13287 | 0 | 13287 |
| budget_1000000_presence10 | budget_1000000_presence20 | *6ed6a787d02af80e84c5ea014d267a2a* | 16367 | 3065 | -13302 | 0 | 13302 |
| budget_1000000_presence10 | budget_1000000_presence20 | *793c3564* | 16375 | 3086 | -13289 | 0 | 13289 |
| budget_1000000_presence10 | budget_1000000_presence20 | *793c3564223a* | 16371 | 1213 | -15158 | 0 | 15158 |
| budget_1000000_presence10 | budget_1000000_presence20 | *793c3564223ab74c* | 16370 | 1212 | -15158 | 0 | 15158 |
| budget_1000000_presence10 | budget_1000000_presence20 | *793c3564223ab74c29f651a1* | 16369 | 1211 | -15158 | 0 | 15158 |
| budget_1000000_presence10 | budget_1000000_presence20 | *793c3564223ab74c29f651a1f4f41756* | 16369 | 1211 | -15158 | 0 | 15158 |
| budget_1000000_presence10 | budget_1000000_presence20 | *e668977a* | 16373 | 3080 | -13293 | 0 | 13293 |
| budget_1000000_presence10 | budget_1000000_presence20 | *e668977af5cc* | 16369 | 3076 | -13293 | 0 | 13293 |
| budget_1000000_presence10 | budget_1000000_presence20 | *e668977af5cc0a19* | 16369 | 3076 | -13293 | 0 | 13293 |
| budget_1000000_presence10 | budget_1000000_presence20 | *e668977af5cc0a1953aa1431* | 16369 | 1518 | -14851 | 0 | 14851 |
| budget_1000000_presence10 | budget_1000000_presence20 | *e668977af5cc0a1953aa143191d6379b* | 16367 | 904 | -15463 | 0 | 15463 |
| budget_1000000_presence10 | budget_1000000_presence20 | *3447ff7f* | 16377 | 1694 | -14683 | 0 | 14683 |
| budget_1000000_presence10 | budget_1000000_presence20 | *3447ff7f43d9* | 16371 | 1688 | -14683 | 0 | 14683 |
| budget_1000000_presence10 | budget_1000000_presence20 | *3447ff7f43d94cd8* | 16370 | 1687 | -14683 | 0 | 14683 |
| budget_1000000_presence10 | budget_1000000_presence20 | *3447ff7f43d94cd814477f31* | 16370 | 1676 | -14694 | 0 | 14694 |
| budget_1000000_presence10 | budget_1000000_presence20 | *3447ff7f43d94cd814477f31c3ccdb66* | 16369 | 1675 | -14694 | 0 | 14694 |
| budget_1000000_presence10 | budget_1000000_presence20 | *23504150* | 16794 | 1749 | -15045 | 0 | 15045 |
| budget_1000000_presence10 | budget_1000000_presence20 | *23504150bf03* | 16374 | 1515 | -14859 | 0 | 14859 |
| budget_1000000_presence10 | budget_1000000_presence20 | *23504150bf03afe2* | 16371 | 1512 | -14859 | 0 | 14859 |
| budget_1000000_presence10 | budget_1000000_presence20 | *23504150bf03afe28db8f861* | 16368 | 1509 | -14859 | 0 | 14859 |
| budget_1000000_presence10 | budget_1000000_presence20 | *23504150bf03afe28db8f8610f728305* | 16368 | 832 | -15536 | 0 | 15536 |
| budget_1000000_presence10 | budget_1000000_presence20 | *c000283a* | 16371 | 1479 | -14892 | 0 | 14892 |
| budget_1000000_presence10 | budget_1000000_presence20 | *c000283a31c6* | 16371 | 1479 | -14892 | 0 | 14892 |
| budget_1000000_presence10 | budget_1000000_presence20 | *c000283a31c604eb* | 16370 | 1478 | -14892 | 0 | 14892 |
| budget_1000000_presence10 | budget_1000000_presence20 | *c000283a31c604eb3d8d670f* | 16367 | 1062 | -15305 | 0 | 15305 |
| budget_1000000_presence10 | budget_1000000_presence20 | *c000283a31c604eb3d8d670f16332953* | 16367 | 786 | -15581 | 0 | 15581 |
| budget_1000000_presence10 | budget_1000000_presence20 | *trace* | 16340 | 2125 | -14215 | 0 | 14215 |
| budget_1000000_presence20 | budget_1000000_presence30 | *successful* | 5 | 21 | 16 | 16 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *successfully* | 2 | 5 | 3 | 3 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *transaction* | 2 | 3 | 1 | 1 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *a5b557dfa94b1f70209f6a70* | 3081 | 3036 | -45 | 1507 | 1552 |
| budget_1000000_presence20 | budget_1000000_presence30 | *a5b557dfa94b1f70209f6a70fbb39a5c* | 3080 | 3035 | -45 | 1507 | 1552 |
| budget_1000000_presence20 | budget_1000000_presence30 | *6ed6a787d02af80e84c5ea014d267a2a* | 3065 | 16367 | 13302 | 13302 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *793c3564* | 3086 | 3133 | 47 | 1520 | 1473 |
| budget_1000000_presence20 | budget_1000000_presence30 | *793c3564223a* | 1213 | 1610 | 397 | 708 | 311 |
| budget_1000000_presence20 | budget_1000000_presence30 | *793c3564223ab74c* | 1212 | 1609 | 397 | 708 | 311 |
| budget_1000000_presence20 | budget_1000000_presence30 | *793c3564223ab74c29f651a1* | 1211 | 1608 | 397 | 708 | 311 |
| budget_1000000_presence20 | budget_1000000_presence30 | *793c3564223ab74c29f651a1f4f41756* | 1211 | 1216 | 5 | 408 | 403 |
| budget_1000000_presence20 | budget_1000000_presence30 | *e668977a* | 3080 | 16373 | 13293 | 13293 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *e668977af5cc* | 3076 | 16369 | 13293 | 13293 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *e668977af5cc0a19* | 3076 | 16369 | 13293 | 13293 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *e668977af5cc0a1953aa1431* | 1518 | 1624 | 106 | 697 | 591 |
| budget_1000000_presence20 | budget_1000000_presence30 | *e668977af5cc0a1953aa143191d6379b* | 904 | 986 | 82 | 292 | 210 |
| budget_1000000_presence20 | budget_1000000_presence30 | *3447ff7f* | 1694 | 3428 | 1734 | 1734 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *3447ff7f43d9* | 1688 | 3422 | 1734 | 1734 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *3447ff7f43d94cd8* | 1687 | 3421 | 1734 | 1734 | 0 |
| budget_1000000_presence20 | budget_1000000_presence30 | *3447ff7f43d94cd814477f31* | 1676 | 1623 | -53 | 411 | 464 |
| budget_1000000_presence20 | budget_1000000_presence30 | *3447ff7f43d94cd814477f31c3ccdb66* | 1675 | 1622 | -53 | 411 | 464 |
| budget_1000000_presence20 | budget_1000000_presence30 | *23504150* | 1749 | 1274 | -475 | 0 | 475 |
| budget_1000000_presence20 | budget_1000000_presence30 | *23504150bf03* | 1515 | 1053 | -462 | 0 | 462 |
| budget_1000000_presence20 | budget_1000000_presence30 | *23504150bf03afe2* | 1512 | 1051 | -461 | 0 | 461 |
| budget_1000000_presence20 | budget_1000000_presence30 | *23504150bf03afe28db8f861* | 1509 | 1050 | -459 | 0 | 459 |
| budget_1000000_presence20 | budget_1000000_presence30 | *23504150bf03afe28db8f8610f728305* | 832 | 743 | -89 | 0 | 89 |
| budget_1000000_presence20 | budget_1000000_presence30 | *0b3da49b0ecf04c0127fae58* | 16367 | 3034 | -13333 | 0 | 13333 |
| budget_1000000_presence20 | budget_1000000_presence30 | *0b3da49b0ecf04c0127fae58532caf18* | 16367 | 3034 | -13333 | 0 | 13333 |
| budget_1000000_presence20 | budget_1000000_presence30 | *c000283a31c604eb* | 1478 | 1088 | -390 | 0 | 390 |
| budget_1000000_presence20 | budget_1000000_presence30 | *c000283a31c604eb3d8d670f* | 1062 | 1086 | 24 | 180 | 156 |
| budget_1000000_presence20 | budget_1000000_presence30 | *c000283a31c604eb3d8d670f16332953* | 786 | 764 | -22 | 147 | 169 |

## Budget regressions and membership changes

| Before | After | Query | Before candidates | After candidates | Newly allowed | Newly pruned |
| --- | --- | --- | --- | --- | --- | --- |

JSON includes every adjacent-budget selection change, every changed candidate set, every group/length summary and each family.

## Every query at every cap and budget

Cells are absolute candidate blocks. Per-query FP, FN and skip percentages are in JSON.

### 200000 B

| Literal | Matching terms | True blocks | Bytes | Full | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *authentication* | 1 | 1 | 16309 | 1 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *authorization* | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *configuration* | 4 | 3 | 16309 | 3 | 10 | 10 | 29 | 29 | 29 | 29 | 29 | 29 |
| *initialization* | 0 | 0 | 16316 | 0 | 7 | 7 | 13 | 13 | 13 | 13 | 13 | 13 |
| *serialization* | 0 | 0 | 16311 | 0 | 6 | 34 | 139 | 139 | 139 | 139 | 139 | 139 |
| *deserialization* | 0 | 0 | 16311 | 0 | 0 | 3 | 139 | 139 | 139 | 139 | 139 | 139 |
| *synchronization* | 0 | 0 | 16305 | 0 | 6 | 6 | 14 | 14 | 14 | 14 | 14 | 14 |
| *communication* | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *reconnection* | 0 | 0 | 16318 | 0 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| *disconnected* | 1 | 1 | 16318 | 1 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *connectionfailed* | 0 | 0 | 16318 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *connectiontimeout* | 1 | 1 | 16312 | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *requesttimeout* | 0 | 0 | 16302 | 0 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *requestfailed* | 0 | 0 | 16307 | 0 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| *connectionreset* | 0 | 0 | 16315 | 0 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *connecttimeout* | 1 | 1 | 16312 | 1 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| *timedout* | 0 | 0 | 16316 | 0 | 139 | 139 | 139 | 139 | 139 | 139 | 139 | 139 |
| *unexpected* | 1 | 1 | 16322 | 1 | 82 | 82 | 82 | 82 | 82 | 82 | 82 | 82 |
| *unsupported* | 0 | 0 | 16316 | 0 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 |
| *unavailable* | 1 | 1 | 16317 | 1 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 |
| *successful* | 3 | 1 | 16333 | 1 | 116 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *successfully* | 1 | 1 | 16325 | 1 | 116 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *processing* | 2 | 2 | 16313 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *transaction* | 0 | 0 | 16315 | 0 | 139 | 139 | 139 | 139 | 139 | 139 | 139 | 139 |
| *deployment* | 0 | 0 | 16312 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *validation* | 2 | 2 | 16313 | 3 | 7 | 7 | 16 | 16 | 16 | 16 | 16 | 16 |
| *permission* | 0 | 0 | 16313 | 0 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 |
| *permissions* | 0 | 0 | 16313 | 0 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 | 1042 |
| *certificate* | 0 | 0 | 16326 | 0 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| *credentials* | 1 | 1 | 16316 | 1 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 |
| *heartbeat* | 4 | 3 | 16332 | 3 | 1048 | 16332 | 16332 | 16332 | 16332 | 16332 | 16332 | 16332 |
| *keepalive* | 0 | 0 | 16322 | 0 | 16322 | 16322 | 16322 | 16322 | 16322 | 16322 | 16322 | 16322 |
| *healthcheck* | 0 | 0 | 16320 | 0 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 |
| *exceptionhandler* | 0 | 0 | 16304 | 0 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *internalservererror* | 0 | 0 | 16308 | 0 | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *invalidargument* | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *outofmemory* | 0 | 0 | 16315 | 0 | 954 | 954 | 954 | 954 | 954 | 954 | 954 | 954 |
| *nullpointerexception* | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *numberformatexception* | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *illegalstateexception* | 1 | 1 | 16305 | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *indexoutofbounds* | 0 | 0 | 16309 | 0 | 1061 | 1061 | 1061 | 1061 | 1061 | 1061 | 1061 | 1061 |
| *a5b557df* | 1 | 1 | 16368 | 15 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b* | 1 | 1 | 16368 | 2 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70fbb39a5c* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787* | 1 | 1 | 16370 | 19 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02a* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02af80e* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *6ed6a787d02af80e84c5ea01* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787d02af80e84c5ea014d267a2a* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *793c3564* | 1 | 1 | 16375 | 29 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 |
| *793c3564223a* | 1 | 1 | 16371 | 3 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *793c3564223ab74c* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *793c3564223ab74c29f651a1* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *793c3564223ab74c29f651a1f4f41756* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977a* | 1 | 1 | 16373 | 60 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 |
| *e668977af5cc* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a19* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa1431* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa143191d6379b* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *3447ff7f* | 1 | 1 | 16377 | 16 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 |
| *3447ff7f43d9* | 1 | 1 | 16371 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *3447ff7f43d94cd8* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31c3ccdb66* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *23504150* | 1 | 1 | 16794 | 1053 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 | 16794 |
| *23504150bf03* | 1 | 1 | 16374 | 4 | 16374 | 16374 | 16374 | 16374 | 16374 | 16374 | 16374 | 16374 |
| *23504150bf03afe2* | 1 | 1 | 16371 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *23504150bf03afe28db8f861* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *23504150bf03afe28db8f8610f728305* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *0b3da49b* | 1 | 1 | 16369 | 21 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *0b3da49b0ecf* | 1 | 1 | 16367 | 2 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58532caf18* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *c000283a* | 1 | 1 | 16371 | 52 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c6* | 1 | 1 | 16371 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c604eb* | 1 | 1 | 16370 | 1 | 16370 | 3073 | 3073 | 3073 | 3073 | 3073 | 3073 | 3073 |
| *c000283a31c604eb3d8d670f* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 3070 | 3070 | 3070 |
| *c000283a31c604eb3d8d670f16332953* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 3070 | 3070 | 3070 | 3070 | 3070 | 3070 |
| *error* | 7 | 6 | 16341 | 15 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |
| *user* | 42 | 36 | 16337 | 143 | 1011 | 16337 | 16337 | 16337 | 16337 | 16337 | 16337 | 16337 |
| *service* | 27 | 9 | 16323 | 9 | 9 | 16323 | 16323 | 16323 | 16323 | 16323 | 16323 | 16323 |
| *client* | 25 | 15 | 16325 | 15 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 |
| *server* | 11 | 6 | 16334 | 6 | 138 | 16334 | 16334 | 16334 | 16334 | 16334 | 16334 | 16334 |
| *failed* | 2 | 1 | 16340 | 2 | 996 | 996 | 996 | 996 | 996 | 996 | 996 | 996 |
| *trace* | 1 | 1 | 16340 | 18 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 |
| *traa* | 42 | 38 | 16341 | 141 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |

### 300000 B

| Literal | Matching terms | True blocks | Bytes | Full | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *authentication* | 1 | 1 | 16309 | 1 | 1 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *authorization* | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *configuration* | 4 | 3 | 16309 | 3 | 3 | 3 | 4 | 4 | 4 | 4 | 4 | 4 |
| *initialization* | 0 | 0 | 16316 | 0 | 7 | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| *serialization* | 0 | 0 | 16311 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *deserialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *synchronization* | 0 | 0 | 16305 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *communication* | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *reconnection* | 0 | 0 | 16318 | 0 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *disconnected* | 1 | 1 | 16318 | 1 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *connectionfailed* | 0 | 0 | 16318 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 1 |
| *connectiontimeout* | 1 | 1 | 16312 | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *requesttimeout* | 0 | 0 | 16302 | 0 | 1 | 1 | 3 | 3 | 3 | 3 | 3 | 3 |
| *requestfailed* | 0 | 0 | 16307 | 0 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *connectionreset* | 0 | 0 | 16315 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connecttimeout* | 1 | 1 | 16312 | 1 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| *timedout* | 0 | 0 | 16316 | 0 | 14 | 14 | 139 | 139 | 139 | 139 | 139 | 139 |
| *unexpected* | 1 | 1 | 16322 | 1 | 82 | 82 | 82 | 82 | 82 | 82 | 82 | 82 |
| *unsupported* | 0 | 0 | 16316 | 0 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 |
| *unavailable* | 1 | 1 | 16317 | 1 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 |
| *successful* | 3 | 1 | 16333 | 1 | 116 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *successfully* | 1 | 1 | 16325 | 1 | 11 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *processing* | 2 | 2 | 16313 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *transaction* | 0 | 0 | 16315 | 0 | 139 | 139 | 139 | 139 | 139 | 139 | 139 | 139 |
| *deployment* | 0 | 0 | 16312 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *validation* | 2 | 2 | 16313 | 3 | 7 | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| *permission* | 0 | 0 | 16313 | 0 | 102 | 102 | 102 | 102 | 102 | 102 | 102 | 102 |
| *permissions* | 0 | 0 | 16313 | 0 | 102 | 102 | 102 | 102 | 102 | 102 | 102 | 102 |
| *certificate* | 0 | 0 | 16326 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 1 |
| *credentials* | 1 | 1 | 16316 | 1 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 |
| *heartbeat* | 4 | 3 | 16332 | 3 | 84 | 1048 | 1048 | 1048 | 1048 | 1048 | 1048 | 1048 |
| *keepalive* | 0 | 0 | 16322 | 0 | 947 | 947 | 947 | 947 | 947 | 947 | 947 | 947 |
| *healthcheck* | 0 | 0 | 16320 | 0 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 |
| *exceptionhandler* | 0 | 0 | 16304 | 0 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *internalservererror* | 0 | 0 | 16308 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *invalidargument* | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *outofmemory* | 0 | 0 | 16315 | 0 | 102 | 102 | 954 | 954 | 954 | 954 | 954 | 954 |
| *nullpointerexception* | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *numberformatexception* | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *illegalstateexception* | 1 | 1 | 16305 | 1 | 2 | 2 | 3 | 3 | 3 | 3 | 3 | 3 |
| *indexoutofbounds* | 0 | 0 | 16309 | 0 | 6 | 6 | 84 | 84 | 84 | 84 | 84 | 84 |
| *a5b557df* | 1 | 1 | 16368 | 15 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b* | 1 | 1 | 16368 | 2 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70fbb39a5c* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787* | 1 | 1 | 16370 | 19 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02a* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02af80e* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *6ed6a787d02af80e84c5ea01* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787d02af80e84c5ea014d267a2a* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *793c3564* | 1 | 1 | 16375 | 29 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 |
| *793c3564223a* | 1 | 1 | 16371 | 3 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *793c3564223ab74c* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *793c3564223ab74c29f651a1* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *793c3564223ab74c29f651a1f4f41756* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977a* | 1 | 1 | 16373 | 60 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 |
| *e668977af5cc* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a19* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa1431* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa143191d6379b* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *3447ff7f* | 1 | 1 | 16377 | 16 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 |
| *3447ff7f43d9* | 1 | 1 | 16371 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *3447ff7f43d94cd8* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31c3ccdb66* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *23504150* | 1 | 1 | 16794 | 1053 | 16794 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 |
| *23504150bf03* | 1 | 1 | 16374 | 4 | 16374 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 |
| *23504150bf03afe2* | 1 | 1 | 16371 | 1 | 16371 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 |
| *23504150bf03afe28db8f861* | 1 | 1 | 16368 | 1 | 16368 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 |
| *23504150bf03afe28db8f8610f728305* | 1 | 1 | 16368 | 1 | 16368 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 |
| *0b3da49b* | 1 | 1 | 16369 | 21 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *0b3da49b0ecf* | 1 | 1 | 16367 | 2 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58532caf18* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *c000283a* | 1 | 1 | 16371 | 52 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c6* | 1 | 1 | 16371 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c604eb* | 1 | 1 | 16370 | 1 | 16370 | 3073 | 1538 | 1538 | 1538 | 1538 | 1538 | 1538 |
| *c000283a31c604eb3d8d670f* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *c000283a31c604eb3d8d670f16332953* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *error* | 7 | 6 | 16341 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *user* | 42 | 36 | 16337 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 |
| *service* | 27 | 9 | 16323 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 |
| *client* | 25 | 15 | 16325 | 15 | 109 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 | 1070 |
| *server* | 11 | 6 | 16334 | 6 | 19 | 19 | 138 | 138 | 138 | 138 | 138 | 138 |
| *failed* | 2 | 1 | 16340 | 2 | 100 | 996 | 996 | 996 | 996 | 996 | 996 | 996 |
| *trace* | 1 | 1 | 16340 | 18 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 | 16340 |
| *traa* | 42 | 38 | 16341 | 141 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |

### 400000 B

| Literal | Matching terms | True blocks | Bytes | Full | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *authentication* | 1 | 1 | 16309 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *authorization* | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *configuration* | 4 | 3 | 16309 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *initialization* | 0 | 0 | 16316 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *serialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *deserialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *synchronization* | 0 | 0 | 16305 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *communication* | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *reconnection* | 0 | 0 | 16318 | 0 | 1 | 1 | 3 | 3 | 3 | 3 | 3 | 3 |
| *disconnected* | 1 | 1 | 16318 | 1 | 1 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| *connectionfailed* | 0 | 0 | 16318 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connectiontimeout* | 1 | 1 | 16312 | 1 | 1 | 2 | 3 | 3 | 3 | 3 | 3 | 3 |
| *requesttimeout* | 0 | 0 | 16302 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *requestfailed* | 0 | 0 | 16307 | 0 | 0 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| *connectionreset* | 0 | 0 | 16315 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connecttimeout* | 1 | 1 | 16312 | 1 | 1 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| *timedout* | 0 | 0 | 16316 | 0 | 14 | 14 | 14 | 14 | 14 | 14 | 14 | 14 |
| *unexpected* | 1 | 1 | 16322 | 1 | 82 | 82 | 82 | 82 | 82 | 82 | 82 | 82 |
| *unsupported* | 0 | 0 | 16316 | 0 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 |
| *unavailable* | 1 | 1 | 16317 | 1 | 12 | 1037 | 16317 | 16317 | 16317 | 16317 | 16317 | 16317 |
| *successful* | 3 | 1 | 16333 | 1 | 116 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *successfully* | 1 | 1 | 16325 | 1 | 11 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| *processing* | 2 | 2 | 16313 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *transaction* | 0 | 0 | 16315 | 0 | 3 | 3 | 16 | 16 | 16 | 16 | 16 | 16 |
| *deployment* | 0 | 0 | 16312 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *validation* | 2 | 2 | 16313 | 3 | 7 | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| *permission* | 0 | 0 | 16313 | 0 | 102 | 102 | 102 | 102 | 102 | 102 | 102 | 102 |
| *permissions* | 0 | 0 | 16313 | 0 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 |
| *certificate* | 0 | 0 | 16326 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *credentials* | 1 | 1 | 16316 | 1 | 6 | 56 | 56 | 56 | 56 | 56 | 56 | 56 |
| *heartbeat* | 4 | 3 | 16332 | 3 | 3 | 3 | 84 | 84 | 84 | 84 | 84 | 84 |
| *keepalive* | 0 | 0 | 16322 | 0 | 947 | 947 | 947 | 947 | 947 | 947 | 947 | 947 |
| *healthcheck* | 0 | 0 | 16320 | 0 | 108 | 108 | 1078 | 1078 | 1078 | 1078 | 1078 | 1078 |
| *exceptionhandler* | 0 | 0 | 16304 | 0 | 0 | 0 | 3 | 3 | 3 | 3 | 3 | 3 |
| *internalservererror* | 0 | 0 | 16308 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *invalidargument* | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *outofmemory* | 0 | 0 | 16315 | 0 | 6 | 102 | 102 | 102 | 102 | 102 | 102 | 102 |
| *nullpointerexception* | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *numberformatexception* | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *illegalstateexception* | 1 | 1 | 16305 | 1 | 1 | 1 | 2 | 2 | 2 | 2 | 2 | 2 |
| *indexoutofbounds* | 0 | 0 | 16309 | 0 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *a5b557df* | 1 | 1 | 16368 | 15 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b* | 1 | 1 | 16368 | 2 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70fbb39a5c* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787* | 1 | 1 | 16370 | 19 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02a* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02af80e* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *6ed6a787d02af80e84c5ea01* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787d02af80e84c5ea014d267a2a* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *793c3564* | 1 | 1 | 16375 | 29 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 |
| *793c3564223a* | 1 | 1 | 16371 | 3 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *793c3564223ab74c* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *793c3564223ab74c29f651a1* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *793c3564223ab74c29f651a1f4f41756* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977a* | 1 | 1 | 16373 | 60 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 |
| *e668977af5cc* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a19* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa1431* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa143191d6379b* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *3447ff7f* | 1 | 1 | 16377 | 16 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 |
| *3447ff7f43d9* | 1 | 1 | 16371 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *3447ff7f43d94cd8* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31c3ccdb66* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *23504150* | 1 | 1 | 16794 | 1053 | 16794 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 |
| *23504150bf03* | 1 | 1 | 16374 | 4 | 16374 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 |
| *23504150bf03afe2* | 1 | 1 | 16371 | 1 | 16371 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 |
| *23504150bf03afe28db8f861* | 1 | 1 | 16368 | 1 | 16368 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 |
| *23504150bf03afe28db8f8610f728305* | 1 | 1 | 16368 | 1 | 16368 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 |
| *0b3da49b* | 1 | 1 | 16369 | 21 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *0b3da49b0ecf* | 1 | 1 | 16367 | 2 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58532caf18* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *c000283a* | 1 | 1 | 16371 | 52 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c6* | 1 | 1 | 16371 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c604eb* | 1 | 1 | 16370 | 1 | 16370 | 3073 | 1538 | 1538 | 1538 | 1538 | 1538 | 1538 |
| *c000283a31c604eb3d8d670f* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *c000283a31c604eb3d8d670f16332953* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *error* | 7 | 6 | 16341 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *user* | 42 | 36 | 16337 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 |
| *service* | 27 | 9 | 16323 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 |
| *client* | 25 | 15 | 16325 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *server* | 11 | 6 | 16334 | 6 | 6 | 19 | 19 | 19 | 19 | 19 | 19 | 19 |
| *failed* | 2 | 1 | 16340 | 2 | 2 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| *trace* | 1 | 1 | 16340 | 18 | 16340 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 |
| *traa* | 42 | 38 | 16341 | 141 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |

### 500000 B

| Literal | Matching terms | True blocks | Bytes | Full | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *authentication* | 1 | 1 | 16309 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *authorization* | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *configuration* | 4 | 3 | 16309 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *initialization* | 0 | 0 | 16316 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *serialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *deserialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *synchronization* | 0 | 0 | 16305 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *communication* | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *reconnection* | 0 | 0 | 16318 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *disconnected* | 1 | 1 | 16318 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *connectionfailed* | 0 | 0 | 16318 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connectiontimeout* | 1 | 1 | 16312 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *requesttimeout* | 0 | 0 | 16302 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *requestfailed* | 0 | 0 | 16307 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connectionreset* | 0 | 0 | 16315 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connecttimeout* | 1 | 1 | 16312 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *timedout* | 0 | 0 | 16316 | 0 | 14 | 14 | 14 | 14 | 14 | 14 | 14 | 14 |
| *unexpected* | 1 | 1 | 16322 | 1 | 8 | 8 | 82 | 82 | 82 | 82 | 82 | 82 |
| *unsupported* | 0 | 0 | 16316 | 0 | 1 | 1 | 11 | 11 | 11 | 11 | 11 | 11 |
| *unavailable* | 1 | 1 | 16317 | 1 | 1 | 12 | 12 | 12 | 12 | 12 | 12 | 12 |
| *successful* | 3 | 1 | 16333 | 1 | 5 | 21 | 21 | 21 | 21 | 21 | 21 | 21 |
| *successfully* | 1 | 1 | 16325 | 1 | 2 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| *processing* | 2 | 2 | 16313 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *transaction* | 0 | 0 | 16315 | 0 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *deployment* | 0 | 0 | 16312 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *validation* | 2 | 2 | 16313 | 3 | 7 | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| *permission* | 0 | 0 | 16313 | 0 | 13 | 13 | 102 | 102 | 102 | 102 | 102 | 102 |
| *permissions* | 0 | 0 | 16313 | 0 | 1 | 1 | 18 | 18 | 18 | 18 | 18 | 18 |
| *certificate* | 0 | 0 | 16326 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *credentials* | 1 | 1 | 16316 | 1 | 1 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *heartbeat* | 4 | 3 | 16332 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *keepalive* | 0 | 0 | 16322 | 0 | 947 | 947 | 947 | 947 | 947 | 947 | 947 | 947 |
| *healthcheck* | 0 | 0 | 16320 | 0 | 108 | 108 | 108 | 108 | 108 | 108 | 108 | 108 |
| *exceptionhandler* | 0 | 0 | 16304 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *internalservererror* | 0 | 0 | 16308 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *invalidargument* | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *outofmemory* | 0 | 0 | 16315 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *nullpointerexception* | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *numberformatexception* | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *illegalstateexception* | 1 | 1 | 16305 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *indexoutofbounds* | 0 | 0 | 16309 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *a5b557df* | 1 | 1 | 16368 | 15 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b* | 1 | 1 | 16368 | 2 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70fbb39a5c* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787* | 1 | 1 | 16370 | 19 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02a* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02af80e* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *6ed6a787d02af80e84c5ea01* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787d02af80e84c5ea014d267a2a* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *793c3564* | 1 | 1 | 16375 | 29 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 | 16375 |
| *793c3564223a* | 1 | 1 | 16371 | 3 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *793c3564223ab74c* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *793c3564223ab74c29f651a1* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *793c3564223ab74c29f651a1f4f41756* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977a* | 1 | 1 | 16373 | 60 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 |
| *e668977af5cc* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a19* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa1431* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa143191d6379b* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *3447ff7f* | 1 | 1 | 16377 | 16 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 | 16377 |
| *3447ff7f43d9* | 1 | 1 | 16371 | 1 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 | 16371 |
| *3447ff7f43d94cd8* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *3447ff7f43d94cd814477f31c3ccdb66* | 1 | 1 | 16369 | 1 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *23504150* | 1 | 1 | 16794 | 1053 | 16794 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 | 3337 |
| *23504150bf03* | 1 | 1 | 16374 | 4 | 16374 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 | 3081 |
| *23504150bf03afe2* | 1 | 1 | 16371 | 1 | 16371 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 | 3078 |
| *23504150bf03afe28db8f861* | 1 | 1 | 16368 | 1 | 16368 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 | 3075 |
| *23504150bf03afe28db8f8610f728305* | 1 | 1 | 16368 | 1 | 16368 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 | 1486 |
| *0b3da49b* | 1 | 1 | 16369 | 21 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *0b3da49b0ecf* | 1 | 1 | 16367 | 2 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58532caf18* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *c000283a* | 1 | 1 | 16371 | 52 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c6* | 1 | 1 | 16371 | 1 | 16371 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 | 3074 |
| *c000283a31c604eb* | 1 | 1 | 16370 | 1 | 16370 | 3073 | 1538 | 1538 | 1538 | 1538 | 1538 | 1538 |
| *c000283a31c604eb3d8d670f* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *c000283a31c604eb3d8d670f16332953* | 1 | 1 | 16367 | 1 | 16367 | 3070 | 1535 | 1535 | 1535 | 1535 | 1535 | 1535 |
| *error* | 7 | 6 | 16341 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *user* | 42 | 36 | 16337 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 |
| *service* | 27 | 9 | 16323 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 |
| *client* | 25 | 15 | 16325 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *server* | 11 | 6 | 16334 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *failed* | 2 | 1 | 16340 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| *trace* | 1 | 1 | 16340 | 18 | 16340 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 |
| *traa* | 42 | 38 | 16341 | 141 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |

### 1000000 B

| Literal | Matching terms | True blocks | Bytes | Full | 10% | 20% | 30% | 40% | 50% | 60% | 70% | 80% |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| *authentication* | 1 | 1 | 16309 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *authorization* | 0 | 0 | 16306 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *configuration* | 4 | 3 | 16309 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *initialization* | 0 | 0 | 16316 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *serialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *deserialization* | 0 | 0 | 16311 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *synchronization* | 0 | 0 | 16305 | 0 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *communication* | 0 | 0 | 16312 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *reconnection* | 0 | 0 | 16318 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *disconnected* | 1 | 1 | 16318 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *connectionfailed* | 0 | 0 | 16318 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connectiontimeout* | 1 | 1 | 16312 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *requesttimeout* | 0 | 0 | 16302 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *requestfailed* | 0 | 0 | 16307 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connectionreset* | 0 | 0 | 16315 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *connecttimeout* | 1 | 1 | 16312 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *timedout* | 0 | 0 | 16316 | 0 | 1 | 14 | 14 | 14 | 14 | 14 | 14 | 14 |
| *unexpected* | 1 | 1 | 16322 | 1 | 1 | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| *unsupported* | 0 | 0 | 16316 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *unavailable* | 1 | 1 | 16317 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *successful* | 3 | 1 | 16333 | 1 | 1 | 5 | 21 | 21 | 21 | 21 | 21 | 21 |
| *successfully* | 1 | 1 | 16325 | 1 | 1 | 2 | 5 | 5 | 5 | 5 | 5 | 5 |
| *processing* | 2 | 2 | 16313 | 2 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *transaction* | 0 | 0 | 16315 | 0 | 2 | 2 | 3 | 3 | 3 | 3 | 3 | 3 |
| *deployment* | 0 | 0 | 16312 | 0 | 1 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *validation* | 2 | 2 | 16313 | 3 | 3 | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| *permission* | 0 | 0 | 16313 | 0 | 0 | 13 | 13 | 13 | 13 | 13 | 13 | 13 |
| *permissions* | 0 | 0 | 16313 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *certificate* | 0 | 0 | 16326 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *credentials* | 1 | 1 | 16316 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *heartbeat* | 4 | 3 | 16332 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| *keepalive* | 0 | 0 | 16322 | 0 | 1 | 947 | 947 | 947 | 947 | 947 | 947 | 947 |
| *healthcheck* | 0 | 0 | 16320 | 0 | 108 | 108 | 108 | 108 | 108 | 108 | 108 | 108 |
| *exceptionhandler* | 0 | 0 | 16304 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *internalservererror* | 0 | 0 | 16308 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *invalidargument* | 0 | 0 | 16301 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *outofmemory* | 0 | 0 | 16315 | 0 | 0 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *nullpointerexception* | 0 | 0 | 16305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *numberformatexception* | 0 | 0 | 16303 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *illegalstateexception* | 1 | 1 | 16305 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| *indexoutofbounds* | 0 | 0 | 16309 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| *a5b557df* | 1 | 1 | 16368 | 15 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b* | 1 | 1 | 16368 | 2 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *a5b557dfa94b1f70209f6a70* | 1 | 1 | 16368 | 1 | 16368 | 3081 | 3036 | 3036 | 3036 | 3036 | 3036 | 3036 |
| *a5b557dfa94b1f70209f6a70fbb39a5c* | 1 | 1 | 16367 | 1 | 16367 | 3080 | 3035 | 3035 | 3035 | 3035 | 3035 | 3035 |
| *6ed6a787* | 1 | 1 | 16370 | 19 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02a* | 1 | 1 | 16370 | 1 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 | 16370 |
| *6ed6a787d02af80e* | 1 | 1 | 16368 | 1 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 | 16368 |
| *6ed6a787d02af80e84c5ea01* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *6ed6a787d02af80e84c5ea014d267a2a* | 1 | 1 | 16367 | 1 | 16367 | 3065 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *793c3564* | 1 | 1 | 16375 | 29 | 16375 | 3086 | 3133 | 3133 | 3133 | 3133 | 3133 | 3133 |
| *793c3564223a* | 1 | 1 | 16371 | 3 | 16371 | 1213 | 1610 | 1610 | 1610 | 1610 | 1610 | 1610 |
| *793c3564223ab74c* | 1 | 1 | 16370 | 1 | 16370 | 1212 | 1609 | 1609 | 1609 | 1609 | 1609 | 1609 |
| *793c3564223ab74c29f651a1* | 1 | 1 | 16369 | 1 | 16369 | 1211 | 1608 | 1608 | 1608 | 1608 | 1608 | 1608 |
| *793c3564223ab74c29f651a1f4f41756* | 1 | 1 | 16369 | 1 | 16369 | 1211 | 1216 | 1216 | 1216 | 1216 | 1216 | 1216 |
| *e668977a* | 1 | 1 | 16373 | 60 | 16373 | 3080 | 16373 | 16373 | 16373 | 16373 | 16373 | 16373 |
| *e668977af5cc* | 1 | 1 | 16369 | 1 | 16369 | 3076 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a19* | 1 | 1 | 16369 | 1 | 16369 | 3076 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *e668977af5cc0a1953aa1431* | 1 | 1 | 16369 | 1 | 16369 | 1518 | 1624 | 1624 | 1624 | 1624 | 1624 | 1624 |
| *e668977af5cc0a1953aa143191d6379b* | 1 | 1 | 16367 | 1 | 16367 | 904 | 986 | 986 | 986 | 986 | 986 | 986 |
| *3447ff7f* | 1 | 1 | 16377 | 16 | 16377 | 1694 | 3428 | 3428 | 3428 | 3428 | 3428 | 3428 |
| *3447ff7f43d9* | 1 | 1 | 16371 | 1 | 16371 | 1688 | 3422 | 3422 | 3422 | 3422 | 3422 | 3422 |
| *3447ff7f43d94cd8* | 1 | 1 | 16370 | 1 | 16370 | 1687 | 3421 | 3421 | 3421 | 3421 | 3421 | 3421 |
| *3447ff7f43d94cd814477f31* | 1 | 1 | 16370 | 1 | 16370 | 1676 | 1623 | 1623 | 1623 | 1623 | 1623 | 1623 |
| *3447ff7f43d94cd814477f31c3ccdb66* | 1 | 1 | 16369 | 1 | 16369 | 1675 | 1622 | 1622 | 1622 | 1622 | 1622 | 1622 |
| *23504150* | 1 | 1 | 16794 | 1053 | 16794 | 1749 | 1274 | 1274 | 1274 | 1274 | 1274 | 1274 |
| *23504150bf03* | 1 | 1 | 16374 | 4 | 16374 | 1515 | 1053 | 1053 | 1053 | 1053 | 1053 | 1053 |
| *23504150bf03afe2* | 1 | 1 | 16371 | 1 | 16371 | 1512 | 1051 | 1051 | 1051 | 1051 | 1051 | 1051 |
| *23504150bf03afe28db8f861* | 1 | 1 | 16368 | 1 | 16368 | 1509 | 1050 | 1050 | 1050 | 1050 | 1050 | 1050 |
| *23504150bf03afe28db8f8610f728305* | 1 | 1 | 16368 | 1 | 16368 | 832 | 743 | 743 | 743 | 743 | 743 | 743 |
| *0b3da49b* | 1 | 1 | 16369 | 21 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 | 16369 |
| *0b3da49b0ecf* | 1 | 1 | 16367 | 2 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 | 16367 |
| *0b3da49b0ecf04c0127fae58* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 3034 | 3034 | 3034 | 3034 | 3034 | 3034 |
| *0b3da49b0ecf04c0127fae58532caf18* | 1 | 1 | 16367 | 1 | 16367 | 16367 | 3034 | 3034 | 3034 | 3034 | 3034 | 3034 |
| *c000283a* | 1 | 1 | 16371 | 52 | 16371 | 1479 | 1479 | 1479 | 1479 | 1479 | 1479 | 1479 |
| *c000283a31c6* | 1 | 1 | 16371 | 1 | 16371 | 1479 | 1479 | 1479 | 1479 | 1479 | 1479 | 1479 |
| *c000283a31c604eb* | 1 | 1 | 16370 | 1 | 16370 | 1478 | 1088 | 1088 | 1088 | 1088 | 1088 | 1088 |
| *c000283a31c604eb3d8d670f* | 1 | 1 | 16367 | 1 | 16367 | 1062 | 1086 | 1086 | 1086 | 1086 | 1086 | 1086 |
| *c000283a31c604eb3d8d670f16332953* | 1 | 1 | 16367 | 1 | 16367 | 786 | 764 | 764 | 764 | 764 | 764 | 764 |
| *error* | 7 | 6 | 16341 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *user* | 42 | 36 | 16337 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 | 143 |
| *service* | 27 | 9 | 16323 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 | 9 |
| *client* | 25 | 15 | 16325 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| *server* | 11 | 6 | 16334 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| *failed* | 2 | 1 | 16340 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| *trace* | 1 | 1 | 16340 | 18 | 16340 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 | 2125 |
| *traa* | 42 | 38 | 16341 | 141 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 | 16341 |

## Omission versus exclusion

Every query gram has measured document DF, remainder-block DF, serialized row cost, global DF rank and per-configuration status in JSON. A cap restriction differs from an eligible row omitted by budget/rank (including nonfitting rows). Both read as unknown. Relaxing the cap can admit common hex grams and displace word grams under the same budget. Whole-query gram retention is a diagnostic. Prefix-contained matches require no remainder grams and crossings require only continuation grams.

## Validation

```json
{
  "logical_partial_configurations": 40,
  "serialized_partial_files": 40,
  "distinct_partial_indexes": 15,
  "distinct_partial_computations": 15,
  "reference_computations": 2,
  "partial_logical_query_block_decisions": 59786640,
  "partial_distinct_query_block_decisions_per_pass": 22419990,
  "total_distinct_query_block_decisions_per_pass": 25409322,
  "recomputation_passes": 2,
  "retained_rows_decoded_across_40_files": 27779,
  "decision_width_bytes": 12,
  "inherited_truth_hash_verified": true,
  "independent_truth_reused": true,
  "physical_boundaries_verified": true,
  "sampled_source_terms_verified": true,
  "decoded_sidecars_equal": true,
  "higher_caps_eligible_selected_and_bytes_equal": true,
  "cap20_per_block_and_totals_equal": true,
  "masks_reloaded_recomputed_recounted": true,
  "aliases_hash_bytes_and_masks_equal": true,
  "truth_subset_full_subset_partial_subset_bytes": true,
  "zero_false_negatives": true,
  "nested_candidate_sets_monotone": true,
  "source_files": 362,
  "all_source_hashes_unchanged": true
}
```

## Scope and reproduction

Diagnostic frozen cohort, equal query weighting. The natural-present, natural-absent, source-ID and control groups answer separate questions. Best tested caps describe these groups rather than a universal optimum. No production latency, document-query DF, Chinese-data or UUID-rebuild claims.

Runner and test commands: `dictionary_analysis/long_queries/thresholds.md`. Parent-ready interpretation and run/test history: `parent_report.md` in this directory.
