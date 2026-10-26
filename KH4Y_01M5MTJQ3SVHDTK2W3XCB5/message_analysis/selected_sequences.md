# Caller-configured byte sequences

The caller supplies an explicit sequence list through external configuration. The indexer does **not** choose sequences. This experiment accepts a required JSON configuration file and writes exactly those sequences into each sidecar's definition table.

Example invocation from the repository root:

```sh
python3 dictionary_analysis/selected_sequences.py --config dictionary_analysis/selected_sequences.json --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 -m unittest discover -s dictionary_analysis -p 'test_selected_sequences.py'
```

The input configuration is `dictionary_analysis/selected_sequences.json`. The result file with the same basename is under `message_analysis/`. These are separate files.

## Externally supplied examples

### `pairs`

```json
["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms"]
```

### `pairs_and_trigrams`

```json
["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms", "tra", "raa", "pla", "lay", "aye", "yer"]
```

### `pairs_trigrams_and_literals`

```json
["ab", "tr", "ra", "aa", "pl", "la", "ay", "ye", "er", "ms", "tra", "raa", "pla", "lay", "aye", "yer", "traa", "player"]
```

### `mixed_sequences`

```json
["ab", "ms", "tra", "raa", "pla", "lay", "aye", "yer"]
```

### `literals`

```json
["ab", "traa", "player", "ms"]
```

These lists illustrate the requested queries. They are deliberately workload-specific examples, not a measured vocabulary-selection strategy for all queries. Sequences may have different lengths. `player` is a six-byte substring feature, not a whole-token match.

## Exact matching results

| Query | Matching distinct terms | Blocks with real matches |
| --- | --- | --- |
| `*a*` | 1,921,269 | 16,376 |
| `*ab*` | 75,774 | 15,490 |
| `*traa*` | 42 | 38 |
| `*X` | 0 | 0 |
| `*x` | 153,990 | 16,022 |
| `*player*` | 21 | 13 |
| `*ms*` | 50,585 | 14,065 |

## Actually serialized storage

The fixed bit vector uses one bit per configured sequence and rounds up to a whole byte per block. Sequence definitions and a 16-byte header are included. Every file was read back and decoded, with all masks compared against the build-time masks.

| Configuration | Sequences | Bytes/block | Sequence sidecar bytes | Including byte masks | Increase over current metadata |
| --- | --- | --- | --- | --- | --- |
| pairs | 10 | 2 | 33,664 | 53,895 | +27.55% |
| pairs_and_trigrams | 16 | 2 | 33,706 | 53,937 | +27.57% |
| pairs_trigrams_and_literals | 18 | 3 | 50,518 | 70,749 | +36.16% |
| mixed_sequences | 8 | 1 | 16,864 | 37,095 | +18.96% |
| literals | 4 | 1 | 16,840 | 37,071 | +18.95% |

The existing shared whole-byte masks add 20,231 bytes. The +100% budget is 195,640 additional bytes. These measured sizes exclude production container integration and runtime allocation. The experiment changes no SSTable library code.

## Compressed data bytes skipped

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| byte_masks | 2.35% | 2.37% | 2.60% | 100.00% | 2.51% | 2.67% | 2.61% |
| oracle | 2.35% | 7.62% | 99.78% | 100.00% | 5.30% | 99.92% | 18.30% |
| pairs | 2.35% | 7.62% | 32.15% | 100.00% | 2.51% | 44.14% | 18.30% |
| pairs_and_trigrams | 2.35% | 7.62% | 99.34% | 100.00% | 2.51% | 99.92% | 18.30% |
| pairs_trigrams_and_literals | 2.35% | 7.62% | 99.78% | 100.00% | 2.51% | 99.92% | 18.30% |
| mixed_sequences | 2.35% | 7.62% | 99.34% | 100.00% | 2.51% | 99.92% | 18.30% |
| literals | 2.35% | 7.62% | 99.78% | 100.00% | 2.51% | 99.92% | 18.30% |

### Candidate blocks retained

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| byte_masks | 16,376 | 16,373 | 16,341 | 0 | 16,354 | 16,331 | 16,339 |
| oracle | 16,376 | 15,490 | 38 | 0 | 16,022 | 13 | 14,065 |
| pairs | 16,376 | 15,490 | 11,802 | 0 | 16,354 | 9,800 | 14,065 |
| pairs_and_trigrams | 16,376 | 15,490 | 116 | 0 | 16,354 | 14 | 14,065 |
| pairs_trigrams_and_literals | 16,376 | 15,490 | 38 | 0 | 16,354 | 13 | 14,065 |
| mixed_sequences | 16,376 | 15,490 | 116 | 0 | 16,354 | 14 | 14,065 |
| literals | 16,376 | 15,490 | 38 | 0 | 16,354 | 13 | 14,065 |

The percentage display rounds to two decimals. The candidate counts distinguish small differences such as 14 retained blocks versus the 13-block oracle for `*player*`.

Every configuration combines its sequence bits with exact whole-byte masks. No ending-byte masks are included. Consequently the suffix query `*x` can retain blocks with x elsewhere in a term. `*X` is case-sensitive uppercase X, which does not occur in this dictionary.

A configured full literal gives an exact block-level presence test for that substring. Pairs and trigrams may occur at different positions or in different terms, causing false positives. The oracle scans actual terms and shows the maximum whole-block pruning.

## API and safety implications

- The caller passes the sequence list into the indexing configuration. Membership must be accumulated from full logical keys before front coding, and reset/flushed with each physical block.
- Each sequence has a stable bit position **within the written dictionary**. Persist sequence definitions (or an immutable, verifiable configuration identity). A reader must use the writer's mapping, not the caller's current list.
- At query time, only sequences guaranteed to occur in a matching term can be required. For these simple substring/suffix patterns, any configured sequence contained in the literal is necessary. Different sequences may be satisfied by different terms in the block, so the test remains conservative.
- An untracked sequence imposes no condition. Missing metadata retains candidate blocks. General regex alternation/optional branches require conservative mandatory-sequence extraction or no pruning.
- Supplied strings are UTF-8 encoded with no normalization by this helper. Production callers must supply sequences in the indexed term byte domain after the relevant normalization. The underlying mask logic works on bytes.
- Existing dictionaries need no rewrite to remain readable. They cannot gain these summaries without a rebuild or separate scan. Merges must recompute the summary for output blocks and the output sequence configuration. Term insertion/deletion and block-boundary changes invalidate direct reuse of source block masks.

These are API/design requirements, not a production indexing API implementation.

## Validation

All 16,794 original blocks were decoded using their original term boundaries. Exact counts were calculated by direct term matching, separately from sequence-bit queries. Whole-byte masks matched a fresh scan. All serialized masks roundtripped. Every method had **zero false negatives** for all 7 queries. The tests cover unknown sequences, absent metadata, arbitrary bytes, UTF-8, cross-term boundaries, and suffix semantics. Skipped bytes sum original compressed blocks without hole coalescing; they are not measured I/O or latency.
