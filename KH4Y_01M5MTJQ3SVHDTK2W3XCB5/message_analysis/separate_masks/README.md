# Deduplicate the two masks separately

**Actual common prefixes excluded. No neighboring-block unions. No caller-provided pattern list.** There are 16,794 blocks and 195,640 bytes of existing index metadata.

## Two different meanings of a mask

1. **Byte-position masks:** 256-bit set of bytes at even remainder offsets, plus a separate set at odd offsets. A query's bytes must fit one consistent starting phase. Their union supplies the ordinary remainder-byte mask, so no third mask or ID is needed. This does not preserve adjacent-byte correlation: terms `ac` and `db` allow the candidate `ab`, because `a` occurs in the even mask and `b` in the odd mask.
2. **Adjacent-pair masks:** even starts `[0,1],[2,3],...` and odd starts `[1,2],[3,4],...`. Each mask has one bit per observed ordered pair (1,443 globally). Deduplicating the two masks independently preserves the original pair-parity pruning exactly. These rows also include shared remainder-byte masks to cover single-byte remainders and a global pair-definition table stored once.

The **joint** layout references one combined-mask table entry per block. The **independent** layout references two independently deduplicated tables, one ID per mask per block. Both use the same lossless information for a given mask kind. Every row includes the 48-byte prefix-extension table once.

## Actually serialized sizes

| Mask kind | Deduplication | Unique masks per table | ID bytes/block per table | Added bytes | Increase | With whole-sidecar zstd3 (B) |
| --- | --- | --- | --- | --- | --- | --- |
| byte_positions | joint | 405 | 2 | 59,581 | +30.45% | 4,804 |
| byte_positions | independent | 276, 245 | 2, 1 | 67,152 | +34.32% | 5,050 |
| adjacent_pairs | joint | 16513 | 2 | 6,018,051 | +3076.08% | 5,062,450 |
| adjacent_pairs | independent | 16488, 16416 | 2, 2 | 6,046,095 | +3090.42% | 5,064,041 |

The last column compresses each entire interned sidecar (table plus ID stream), unlike the earlier 128-block independently compressed groups. It needs whole-sidecar decompression. Runtime allocation, physical I/O and latency are unmeasured. Sizes include framing and every necessary table/ID stream.

## Blocks skipped

Joint and independent layouts have identical decisions within each row:

| Mask kind | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| byte_positions | 2.49% | 2.51% | 2.78% | 100.00% | 2.62% | 3.00% | 2.79% |
| adjacent_pairs | 2.49% | 7.63% | 43.58% | 100.00% | 2.62% | 66.55% | 16.14% |

All decisions have zero false negatives against the full-term per-block oracle. Every raw and compressed sidecar roundtrips exactly. Adjacent-pair results equal the preceding prefix-excluded all-pair parity study.

```sh
python3 dictionary_analysis/separate_masks.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```
