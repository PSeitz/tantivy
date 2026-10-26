# Two adjacent-byte pair masks, separated by offset parity

The caller-supplied pair list is:

```json
["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms"]
```

For each full term, even-offset pairs are `[0,1], [2,3], ...`, and odd-offset pairs are `[1,2], [3,4], ...`. The block accumulates the unions of these two sets separately. Their union is the ordinary overlapping-pair mask from the earlier experiment.

For `*traa*`, the stronger condition is:

```text
(tr in even AND ra in odd AND aa in even)
OR
(tr in odd AND ra in even AND aa in odd)
```

For `*ms*`, the condition is `ms in even OR ms in odd`, exactly the ordinary pair test. All pairs of a longer query must agree on one starting parity. Pairs can still come from different terms/positions within the block, so the test can have false positives.

## Actually serialized configured-mask size

- 10 configured pairs, two bits per pair: **3 bytes/block** after byte rounding.
- Parity sidecar including definitions and header: **50,458 bytes**.
- With shared whole-byte masks: **70,689 bytes (+36.13% of existing index metadata)**.
- Same configured pairs without parity, including byte masks: **53,895 bytes**.

## No caller-supplied patterns: all-pair masks

There are 1,444 observed pairs globally. Each block receives two bits per observed pair. The complete masks are compressed in independent groups of 128 blocks using zstd level 3. The pair-phase-major layout transposes each group to place one pair/phase's block-presence bits together. Both layouts retain exact information and produce the same query decisions.

| Layout | Pair parity bytes | Including byte masks | Increase over current metadata |
| --- | --- | --- | --- |
| block | 4,935,485 | 4,955,716 | +2533.08% |
| pair_phase | 4,587,618 | 4,607,849 | +2355.27% |

These are actually serialized sizes including the global pair table, header, and compressed-group offsets. Every compressed group was decoded and verified. The all-pair vocabulary is derived from dictionary keys, without a supplied pattern list. These layouts exceed the +100% metadata budget.

## Compressed data bytes skipped

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bytes | 2.354% | 2.371% | 2.598% | 100.000% | 2.511% | 2.671% | 2.607% |
| all_pairs | 2.354% | 7.618% | 32.151% | 100.000% | 2.511% | 44.138% | 18.297% |
| all_pairs_with_parity | 2.354% | 7.618% | 45.754% | 100.000% | 2.511% | 68.123% | 18.297% |
| configured_pairs | 2.354% | 7.618% | 32.151% | 100.000% | 2.511% | 44.138% | 18.297% |
| configured_pairs_with_parity | 2.354% | 7.618% | 45.754% | 100.000% | 2.511% | 68.123% | 18.297% |
| oracle | 2.354% | 7.618% | 99.784% | 100.000% | 5.305% | 99.924% | 18.297% |

## Candidate blocks retained

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bytes | 16,376 | 16,373 | 16,341 | 0 | 16,354 | 16,331 | 16,339 |
| all_pairs | 16,376 | 15,490 | 11,802 | 0 | 16,354 | 9,800 | 14,065 |
| all_pairs_with_parity | 16,376 | 15,490 | 9,455 | 0 | 16,354 | 5,602 | 14,065 |
| configured_pairs | 16,376 | 15,490 | 11,802 | 0 | 16,354 | 9,800 | 14,065 |
| configured_pairs_with_parity | 16,376 | 15,490 | 9,455 | 0 | 16,354 | 5,602 | 14,065 |
| oracle | 16,376 | 15,490 | 38 | 0 | 16,022 | 13 | 14,065 |

## Prefix handling

This experiment starts parity at byte zero of the **full term**, without removing a common prefix. With remainder-only masks, the phase must account for prefix length. The matcher must also preserve matches wholly inside or crossing the prefix. Consuming the prefix in the query automaton and carrying the remainder parity is one safe approach. Simply testing the complete literal against remainder-only masks can cause false negatives.

## Validation and reproduction

Every serialized mask was decoded and compared with its original. Decisions from decoded masks equal build-time decisions. Exact term counts, whole-byte results, and ordinary pair results agree with the independent evaluator. All seven queries have zero false negatives. Data-byte percentages sum original compressed blocks; no physical I/O or latency is measured.

```sh
python3 -m unittest discover -s dictionary_analysis -p 'test_pair_parity.py'
python3 dictionary_analysis/pair_parity.py --config dictionary_analysis/selected_sequences.json --configuration pairs --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
```

The sidecar begins with `SP01`, u32 block count, u32 pair count, u32 row width, then `[u32 length][pair bytes]` definitions, then fixed-width per-block masks. Integers are little-endian. Pair i uses bit `2*i` for even offsets and `2*i+1` for odd offsets. Full all-pair files use `SPG1`, u32 block count, u32 observed pair count, u32 group size, u8 order, u8 zstd level, sorted u16 pair definitions, u32 compressed-group offsets including the final offset, then compressed groups. Pair i's two bits retain the same even/odd interpretation. The prior all-pair sidecar supplies the data-derived pair vocabulary, which is checked against the fresh scan. Requires the `zstd` executable. Source data and library code are unchanged.
