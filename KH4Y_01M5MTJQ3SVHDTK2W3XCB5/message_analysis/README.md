# Message dictionary: measured block-pruning study

Input: `KH4Y_01M5MTJQ3SVHDTK2W3XCB5`, segment `4fea6c896fd045aba3cece0808f3e41e`, schema field 3 (`message`). All original physical block boundaries are preserved. No library code or source data was changed.

**Current prefix-excluded comparison:** [comparison.md](comparison.md) reports blocks skipped and actually serialized metadata costs, separating methods that need caller-supplied patterns from those that do not. Every current summary excludes the actual common prefix. The full-term summary experiments below are historical baselines.

**Budget follow-up:** the subsequent limit is +100% of existing index metadata. See [budget.md](budget.md). Sharing repeated exact byte masks brings their actually serialized size down to 20,231 bytes (+10.34%), versus the fixed-per-block layout below. The tested trigram configurations exceed that budget.

**Externally configured sequences:** [selected_sequences.md](selected_sequences.md) evaluates caller-supplied pairs, trigrams, and longer literals, including `*player*` and `*ms*`. The caller supplies the list. The indexer performs no automatic selection. The eight-sequence example plus shared byte masks serializes to 37,095 bytes (+18.96% of current index metadata).

## Main findings

- **7,671,280 distinct terms in 16,794 blocks. 1,289,229 terms (16.81%) are digit-only numbers.**
- **Zero original blocks are entirely numeric. Zero blocks have one fixed term length.** Numeric regions are interspersed with IDs and other terms.
- Existing block pruning retains every block for all 7 evaluated patterns. Exact bounds + min/max length intersection skips four blocks for `*player*` (0.03% of data bytes) and retains every block for the other six.
- `*a*` and `*ab*` genuinely match terms in most blocks. Their maximum possible data-byte skipping is only **2.35%** and **7.62%** respectively.
- Exact pairs cost about **3.4 MB** and skip **32.15%** of data bytes for `*traa*`. A free-prefix remainder pair graph raises that to **32.17%**.
- Prefix removal saves only **415 bytes across the entire dictionary** in the modeled packed-pair layout. Most blocks still have the same alphabet after removing their prefix.
- The appended `*player*` substring matches **21 terms in 13 blocks**. Pairs skip **44.14%** of data bytes. Exact trigrams retain just **14 blocks**.
- The appended `*ms*` substring matches **50,585 terms in 14,065 blocks**. Exact pairs already reach its **18.30%** data-byte skipping ceiling. A trigram filter cannot test a two-byte literal.
- Trigrams are much more selective for `*traa*` and `*player*`. Practical Bloom results and size trade-offs appear below. Raw exact trigram lists are an expensive reference, not the recommended encoding.

## Definitions and interpretation

- Percentages of terms count **distinct dictionary keys**, not token occurrences, posting counts, or documents.
- Strict numeric means the entire nonempty key is ASCII `[0-9]+`. Decimals are counted separately as `[+-]?[0-9]+[.][0-9]+`.
- Queries are **case-sensitive byte wildcards**: `*a*`, `*ab*`, `*traa*`, `*player*`, and `*ms*` are substring searches. The latter two are appended at indices 5 and 6. `*X` and diagnostic `*x` are suffix searches. `*` consumes any byte, including newline. These semantics are used consistently by the current-index baseline and exact scans.
- Uppercase `X` occurs in no term. If the production query analyzer lowercases `X`, use the **`*x` column**, not the `*X` column. This experiment does not run the production query analyzer.
- Skipped data bytes are sums of original compressed physical block sizes, **without merging holes**. They are potential avoided reads/decompression, not measured I/O or latency. Reading new metadata and caching/coalescing can change the benefit.
- MB means 1,000,000 bytes. All proposed metadata sizes are incremental, uncompressed layout estimates, with the explicit bookkeeping described below. These summaries have not been added to the on-disk SSTable format.

## 1. Extracted dictionary and numeric content

| Measurement | Value |
|---|---:|
| Original composite `.term` file | 445,626,343 B |
| Extracted `message.sstable` | 51,677,990 B (51.678 MB) |
| Compressed data blocks | 51,482,326 B |
| Existing index, including its FST-length field | 195,640 B |
| Existing SSTable footer | 20 B |
| Data terminator | 4 B |
| Distinct terms | 7,671,280 |
| Original data blocks | 16,794 |
| Mean terms/block | 456.8 |
| Mean compressed bytes/block | 3065.5 |
| Strict numeric terms | 1,289,229 (16.805918%) |
| Decimal terms, disjoint from strict numeric | 161,380 (2.103691%) |
| Strict numeric + these decimals | 1,450,609 (18.909608%) |
| Signed integer terms | 0 |
| Blocks containing any strict numeric terms | 1,652 |
| Blocks with at least 50% strict numeric terms | 1,387 |
| Entirely numeric blocks | 0 |
| Blocks with equal min/max term length | 0 |
| Overall min/max term byte length | 1 / 93 |

The extracted dictionary is an unchanged slice of the source file: source range `[143, 51678133)`, after removing only the surrounding managed/composite containers and trailing dictionary-type tag. `terms.bin` expands front-coded keys for analysis; it is not a replacement SSTable.

### A concrete numeric-heavy block

The most numeric-dense original block is block **2738**:

| Property | Value |
| --- | --- |
| Stored block ID | 2738 |
| Decoded first term ordinal | 1614217 |
| Decoded data byte range | [8941120, 8945719) |
| Previous separator (exclusive lower fence, derived) | `4142561` |
| Stored current separator (inclusive upper fence) | `414874` |
| Stored compression flag | 1 |
| Stored frame length (excludes its own 4 bytes) | 4595 |
| Stored TermInfo value count | 1335 |
| Actual first term (derived) | `414257` |
| Actual last term (derived) | `414874` |
| Strict numeric terms (derived) | 1,298 / 1,335 (97.23%) |
| Minimum / maximum key bytes (derived) | 4 / 32 |

Its nonnumeric terms include:

```text
41428353791a
4142a760a2900a19fbfb1cd823e1e210
4142buuzifpmn
4142c5e41fe39dfeb7c2478ee6745ef0
414304c8852lx
41439bda
4143b1e388b3ae644d920cab6f972bf1
4143dxv3bqhwf
```

These actual IDs explain why numeric-looking fences do not imply a numeric-only block. Full keys for this block are in [block_2738_terms.txt](block_2738_terms.txt). [numeric_blocks.tsv](numeric_blocks.tsv) ranks all original blocks by numeric fraction.

## 2. What is stored in current block metadata

The **existing index** stores:

1. An FST mapping a lexicographic upper separator to a block ID.
2. A compressed address store giving block byte addresses and first term ordinals.
3. An 8-byte field identifying the FST length.

Separators can be synthetic. For example, block 0 ends at actual key `00.451z`, while its stored upper separator is `00.451{`. The lower fence for a block is the previous separator; it is derived rather than separately stored.

| Existing index component | Bytes |
|---|---:|
| Separator FST | 146,797 |
| Address store | 48,835 |
| FST-length field | 8 |
| **Total** | **195,640** |

The address store has 132 groups, each normally covering 128 blocks. Its 4,752 bytes of fixed headers contain reference addresses/ordinals, slopes, residual offsets, bit widths, and group counts. Packed residuals use 44,075 bytes, plus an 8-byte header-length field.

Each physical data block also has a 4-byte frame length and a 1-byte compression flag. All 16,794 blocks use zstd. The TermInfo value count is inside the block payload, so it is available after decoding, rather than as a pre-read pruning summary.

**Current metadata has no term-length bounds, byte masks, pair sets, trigram sets, suffix masks, or numeric counts.**

Readable complete dumps:

- [existing_metadata.tsv](existing_metadata.tsv): one row per block. Actual stored/decoded fields come first. Computed observations are explicitly named `derived_*`.
- [address_store_metadata.tsv](address_store_metadata.tsv): actual address-store group headers.
- [blocks.jsonl](blocks.jsonl): machine-readable blocks, exact terms/fences in hex, sizes, original current-index candidate decisions, and derived observations.

## 3. Exact query results: the ceiling on block pruning

| Query | Matching distinct terms | Blocks with a real match | Maximum block skipping | Maximum data-byte skipping |
| --- | --- | --- | --- | --- |
| `*a*` | 1,921,269 | 16,376 | 2.49% | 2.35% |
| `*ab*` | 75,774 | 15,490 | 7.76% | 7.62% |
| `*traa*` | 42 | 38 | 99.77% | 99.78% |
| `*X` | 0 | 0 | 100.00% | 100.00% |
| `*x` | 153,990 | 16,022 | 4.60% | 5.30% |
| `*player*` | 21 | 13 | 99.92% | 99.92% |
| `*ms*` | 50,585 | 14,065 | 16.25% | 18.30% |

For example, `*ab*` matches only 75,774 distinct terms, but they are spread across 15,490 blocks. A block index must retain each of those blocks. More accurate metadata cannot exceed this ceiling without finer-grained addressing or a secondary term-level index.

## 4. Comparing bounds, lengths, bytes, pairs, and trigrams

### Percentage of compressed data bytes skipped

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Existing index | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% |
| Bounds + min/max lengths | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.03% | 0.00% |
| + whole-key byte mask | 2.35% | 2.37% | 2.60% | 100.00% | 2.51% | 2.67% | 2.61% |
| + exact whole-key pairs | 2.35% | 7.62% | 32.15% | 100.00% | 2.51% | 44.14% | 18.30% |
| Whole-key pair graph | 2.35% | 7.62% | 32.15% | 100.00% | 2.51% | 44.14% | 18.30% |
| Free-prefix remainder bytes | 2.35% | 2.37% | 2.60% | 100.00% | 2.51% | 2.67% | 2.61% |
| Free-prefix remainder pair graph | 2.35% | 7.62% | 32.17% | 100.00% | 2.51% | 44.20% | 18.30% |
| Stored-term-prefix remainder bytes | 2.35% | 2.37% | 2.60% | 100.00% | 2.51% | 2.67% | 2.61% |
| Stored-term-prefix remainder pair graph | 2.35% | 7.62% | 32.17% | 100.00% | 2.51% | 44.20% | 18.30% |
| + exact whole-key trigram lists | 2.35% | 7.62% | 99.34% | 100.00% | 2.51% | 99.92% | 18.30% |
| Exact block-match oracle | 2.35% | 7.62% | 99.78% | 100.00% | 5.30% | 99.92% | 18.30% |

### Number of candidate blocks retained, out of 16,794

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Existing index | 16,794 | 16,794 | 16,794 | 16,794 | 16,794 | 16,794 | 16,794 |
| Bounds + min/max lengths | 16,794 | 16,794 | 16,794 | 16,794 | 16,794 | 16,790 | 16,794 |
| + whole-key byte mask | 16,376 | 16,373 | 16,341 | 0 | 16,354 | 16,331 | 16,339 |
| + exact whole-key pairs | 16,376 | 15,490 | 11,802 | 0 | 16,354 | 9,800 | 14,065 |
| Whole-key pair graph | 16,376 | 15,490 | 11,802 | 0 | 16,354 | 9,800 | 14,065 |
| Free-prefix remainder bytes | 16,376 | 16,373 | 16,341 | 0 | 16,354 | 16,331 | 16,339 |
| Free-prefix remainder pair graph | 16,376 | 15,490 | 11,799 | 0 | 16,354 | 9,789 | 14,065 |
| Stored-term-prefix remainder bytes | 16,376 | 16,373 | 16,341 | 0 | 16,354 | 16,331 | 16,339 |
| Stored-term-prefix remainder pair graph | 16,376 | 15,490 | 11,799 | 0 | 16,354 | 9,789 | 14,065 |
| + exact whole-key trigram lists | 16,376 | 15,490 | 116 | 0 | 16,354 | 14 | 14,065 |
| Exact block-match oracle | 16,376 | 15,490 | 38 | 0 | 16,022 | 13 | 14,065 |

The oracle checks actual terms. Other methods operate on summaries. Every method was checked against the oracle for false negatives.

- `Bounds + min/max lengths` performs automaton/interval/length intersection, rather than merely comparing global length intervals.
- Whole-key pair tests require all literal pairs. For `*traa*`, these are `tr`, `ra`, and `aa`. They can occur in different terms or positions, explaining the remaining false positives.
- Exact trigrams additionally require `tra` and `raa`. They reduce `*traa*` to 116 candidate blocks; only 38 contain a true match.
- The free-prefix pair graph consumes the exact prefix guaranteed by the stored fences, then restricts the remainder using its byte/pair sets. Prefix-crossing matches are preserved. For `*traa*`, this eliminates three more blocks than whole-key pairs. The tables also show the appended substring queries and both stored-prefix models.
- Whole-key pair-graph intersection and cheap literal-pair tests are listed separately so their measured decisions can be compared for every query.

### Estimated additional storage for these models

| Additional metadata | Total bytes | Mean bytes/block | % of current compressed data |
| --- | --- | --- | --- |
| Existing index | 0 | 0.00 | 0.00% |
| Bounds + min/max lengths | 33,588 | 2.00 | 0.07% |
| + whole-key byte mask | 570,996 | 34.00 | 1.11% |
| + exact whole-key pairs | 3,426,394 | 204.02 | 6.66% |
| Whole-key pair graph | 3,426,394 | 204.02 | 6.66% |
| Free-prefix remainder bytes | 570,996 | 34.00 | 1.11% |
| Free-prefix remainder pair graph | 3,425,979 | 204.00 | 6.65% |
| Stored-term-prefix remainder bytes | 621,144 | 36.99 | 1.21% |
| Stored-term-prefix remainder pair graph | 3,476,124 | 206.99 | 6.75% |
| + exact whole-key trigram lists | 166,323,120 | 9903.72 | 323.07% |

Size accounting:

- Two unsigned varints per block for min/max full byte length: 33,588 B. Every observed length fits in one byte.
- A fixed byte mask is 32 B/block: 537,408 B.
- Pair payload chooses `min(2 × distinct_pairs, ceil(alphabet_size² / 8))`. All blocks choose the alphabet-packed matrix on this dataset.
- Pair layout additionally charges 1 format byte/block, a varint distinct-pair count, and `(block_count + 1)` 64-bit offsets. The alphabet mask provides byte ranks.
- Actual-term-prefix models additionally charge a varint prefix length and raw prefix bytes. Fence-derived prefixes cost zero new bytes.
- The exact-trigram reference adds 3 bytes/distinct trigram, varint counts, and separate 64-bit offsets. It includes the existing byte/pair/length layers. This is a simple raw-list model, not an assertion that 166 MB is the smallest possible exact encoding.
- Global section framing, alignment, compression benefits, and runtime data-structure allocation are excluded. Other layouts may reduce overhead. The current index is shared and excluded from incremental sizes.

## 5. Does excluding the common prefix save much here?

| Measurement | Whole key | Remainder after free prefix |
|---|---:|---:|
| Median alphabet size | 36 | 36 |
| Median distinct pairs/block | 1214 | 1214 |
| Median distinct trigrams/block | 3467 | 3356 |
| Sum of distinct pair counts | 18,632,754 | 18,626,814 |
| Packed pair payload bytes | 2,671,020 | 2,670,605 |

The median free prefix is **2 bytes**, and the maximum is 5. Median remainder min/max lengths are 3/11 bytes. Prefix removal leaves most of the alphabet and pairs present elsewhere in each block.

So the earlier hypothesis is valid in principle, but the measured gain is small for this dictionary: **415 B less packed-pair metadata**, and **three additional blocks skipped for `*traa*` and eleven for `*player*`**, relative to whole-key literal pairs. A stored actual-term prefix costs more. Its per-query decisions are shown above.

## 6. Practical Bloom-filter experiments

Filters were actually constructed from each original block's terms. Three deterministic hash seeds were tested as alternatives. The range below shows those three outcomes for this particular query, not a statistical confidence interval. No independence approximation is used for skipping results.

### Trigram Bloom filters for `*traa*` and `*player*`

| Configuration | Query | Bloom bytes/block | Hashes | Total extra metadata | % of data | Data-byte skipping | Candidate blocks |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Byte mask + trigram Bloom | `*traa*` | 256 | 1 | 4.837 MB | 9.39% | 26.92%–34.31% | 10,813–12,457 |
| Byte mask + trigram Bloom | `*player*` | 256 | 1 | 4.837 MB | 9.39% | 56.59%–60.52% | 6,806–7,394 |
| Byte mask + trigram Bloom | `*traa*` | 512 | 1 | 9.136 MB | 17.75% | 66.61%–73.20% | 4,508–5,838 |
| Byte mask + trigram Bloom | `*player*` | 512 | 1 | 9.136 MB | 17.75% | 85.44%–91.81% | 1,426–2,396 |
| Byte mask + trigram Bloom | `*traa*` | 1024 | 2 | 17.734 MB | 34.45% | 78.40%–88.04% | 2,083–3,754 |
| Byte mask + trigram Bloom | `*player*` | 1024 | 2 | 17.734 MB | 34.45% | 97.44%–98.78% | 215–447 |
| Byte mask + trigram Bloom | `*traa*` | 2048 | 4 | 34.932 MB | 67.85% | 96.23%–97.80% | 386–651 |
| Byte mask + trigram Bloom | `*player*` | 2048 | 4 | 34.932 MB | 67.85% | 99.88%–99.90% | 17–21 |
| Byte mask + exact pairs + trigram Bloom | `*traa*` | 256 | 1 | 7.692 MB | 14.94% | 46.58%–56.93% | 7,448–9,317 |
| Byte mask + exact pairs + trigram Bloom | `*player*` | 256 | 1 | 7.692 MB | 14.94% | 72.40%–73.98% | 4,570–4,851 |
| Byte mask + exact pairs + trigram Bloom | `*traa*` | 512 | 1 | 11.991 MB | 23.29% | 72.95%–81.08% | 3,280–4,748 |
| Byte mask + exact pairs + trigram Bloom | `*player*` | 512 | 1 | 11.991 MB | 23.29% | 90.11%–94.19% | 1,020–1,740 |
| Byte mask + exact pairs + trigram Bloom | `*traa*` | 1024 | 2 | 20.590 MB | 39.99% | 82.91%–90.71% | 1,627–2,993 |
| Byte mask + exact pairs + trigram Bloom | `*player*` | 1024 | 2 | 20.590 MB | 39.99% | 98.07%–99.05% | 166–339 |
| Byte mask + exact pairs + trigram Bloom | `*traa*` | 2048 | 4 | 37.787 MB | 73.40% | 97.18%–98.03% | 346–492 |
| Byte mask + exact pairs + trigram Bloom | `*player*` | 2048 | 4 | 37.787 MB | 73.40% | 99.88%–99.90% | 17–21 |

`Byte mask + trigram Bloom` has no pair layer, so two-byte queries `*ab*` and `*ms*` get only byte-mask pruning. `Byte mask + exact pairs + trigram Bloom` preserves exact pair pruning for short literals and uses both layers for longer literals. These configurations omit length metadata because membership tests do not use it. Adding bounds and lengths to the byte/pair tests did not improve their decisions here.

Trigram Bloom sizes are per block. Seeds are alternative encodings, not three filters stored together. Hash counts are selected once per configuration from mean distinct-gram counts, then every bit array is built and queried. See [the helper README](../../dictionary_analysis/filters/README.md) for hashing and accounting details.

More skipped data is not automatically better: the 2 KiB + exact-pair configuration adds nearly 38 MB of metadata to a dictionary with 51.5 MB of compressed data. Metadata read/residency cost must be included in a later end-to-end benchmark.

### Smaller pair Bloom filters

| Configuration | Total extra metadata | `*ab*` data-byte skipping | `*traa*` data-byte skipping | `*player*` data-byte skipping | `*ms*` data-byte skipping |
| --- | --- | --- | --- | --- | --- |
| Byte mask + 64 B pair Bloom | 1.612 MB | 2.37%–7.62% | 9.01%–19.11% | 7.12%–18.23% | 5.34%–6.35% |
| Byte mask + 128 B pair Bloom | 2.687 MB | 2.37%–7.62% | 11.28%–28.48% | 20.84%–29.36% | 6.35%–8.90% |
| Byte mask + exact packed pairs | 3.393 MB | 7.62% | 32.15% | 44.14% | 18.30% |

These results include only byte masks plus the stated pair representation. They omit length metadata, unlike the earlier solver table. Exact pair storage is therefore 3,392,806 B instead of 3,426,394 B.

### Suffix metadata for `*X` and `*x`

An exact bitmap of term-ending bytes costs **537,408 B** (32 B/block). It reaches the one-byte suffix oracle:

- `*X`: 100% of data bytes skipped.
- `*x`: **5.30%** skipped, compared with 2.51% using anywhere-in-term byte masks.

When packed-pair records already contain byte ranks and variable-record offsets, an alphabet-ranked ending-byte bitmap would need an estimated **82,671 additional payload bytes**. This compact encoding was sized, not implemented. The tested last-byte filters use 32 B/block.

The `*X` result alone does not justify per-block metadata: because `X` occurs nowhere in this dictionary, a single dictionary-wide byte mask could reject that case too. Production normalization must be resolved before treating it as a representative optimization.

## 7. Numeric-heavy blocks specifically

There are 1,387 blocks with at least half their terms numeric, containing 1,483,943 total terms and 6,254,188 compressed bytes. They are still mixed blocks.

Data-byte skipping within this subset:

| Method | `*a*` | `*ab*` | `*traa*` | `*X` | `*x` | `*player*` | `*ms*` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Existing index | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% |
| Bounds + min/max lengths | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% | 0.00% |
| + whole-key byte mask | 0.00% | 0.00% | 0.93% | 100.00% | 0.41% | 1.20% | 0.94% |
| + exact whole-key pairs | 0.00% | 4.21% | 84.72% | 100.00% | 0.41% | 97.36% | 62.81% |
| Whole-key pair graph | 0.00% | 4.21% | 84.72% | 100.00% | 0.41% | 97.36% | 62.81% |
| Free-prefix remainder bytes | 0.00% | 0.00% | 0.93% | 100.00% | 0.41% | 1.20% | 0.94% |
| Stored-term-prefix remainder bytes | 0.00% | 0.00% | 0.93% | 100.00% | 0.41% | 1.20% | 0.94% |
| Stored-term-prefix remainder pair graph | 0.00% | 4.21% | 84.72% | 100.00% | 0.41% | 97.36% | 62.81% |
| + exact whole-key trigram lists | 0.00% | 4.21% | 100.00% | 100.00% | 0.41% | 100.00% | 62.81% |
| Exact block-match oracle | 0.00% | 4.21% | 100.00% | 100.00% | 21.30% | 100.00% | 62.81% |

Every numeric-majority block genuinely contains `a`. Consequently, no correct whole-block filter can skip any of them for `*a*`. Conversely, none contains `traa`: the pair summary skips 84.72% of their bytes, and exact trigrams skip all of them.

## 8. Conclusion and next experiments

1. **Bounds + lengths have little selectivity for these specific queries.** They skip four blocks for `*player*` (0.03% of bytes) and none for the other six. Whole-byte masks already reject those four blocks. They can still help other workloads.
2. **A byte mask is cheap, but common one-/two-byte queries have little block-level headroom.** Even ideal summaries cannot avoid reading most blocks for `*a*` or `*ab*`.
3. **Exact alphabet-packed pairs are the simplest useful candidate here:** roughly 3.4 MB additional metadata and 32.15% data-byte skipping for `*traa*`. Removing the prefix contributes little on this data.
4. **For longer rare literals, benchmark trigram Bloom filters.** A 512 B/block filter combined with bytes and exact pairs skips 72.95–81.08% of bytes for `*traa*`, at about 12.0 MB of extra metadata. This is a candidate for latency/memory testing, not a selected production configuration.
5. **For broad short patterns, investigate finer-grained indexing rather than larger block summaries.** Gram-to-term postings or smaller independently readable blocks can avoid the original-block ceiling, with their own storage costs.
6. Evaluate a larger real query sample, confirm normalization, then measure warm/cold latency, metadata residency, bytes read after hole coalescing, decompression CPU, and ingestion/merge overhead.

## Validation and reproduction

- Original source SHA-256: `d1225bc8fc56126acba7ca6cd3494865668a0a1a9ad781599595683355e04060`.
- Extracted SSTable SHA-256: `54c3c6b4ec99d1e4cddd74ed9695e74cd0b1697d141b371cdb58fa0ff8ac531d`.
- Source managed CRC32 verified. Extracted bytes match the source slice. Source hashes remained unchanged.
- All 7,671,280 keys checked for sorting, block/fence membership, counts, ordinals, and export byte identity. A separate audit independently decoded the compressed blocks.
- The exact evaluator reports **zero false negatives across 11 methods × 7 queries**, with no work-budget fallbacks.
- The filter evaluator reports **zero false negatives across 59 configurations × 7 queries**. Every inserted gram is found by its Bloom filter. Last-byte candidates equal the suffix oracle.
- Tests include brute-force small-alphabet comparisons, random prefix/boundary cases, empty/variable-length terms, arbitrary bytes, and conservative work-budget fallback. Repeated full runs produced identical outputs.

Reproduce from the repository root:

```sh
cargo test --release --locked --manifest-path dictionary_analysis/extract/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/extract/Cargo.toml -- KH4Y_01M5MTJQ3SVHDTK2W3XCB5
python3 dictionary_analysis/extract/verify.py KH4Y_01M5MTJQ3SVHDTK2W3XCB5
cargo test --release --locked --manifest-path dictionary_analysis/evaluate/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/evaluate/Cargo.toml -- KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
cargo test --release --locked --manifest-path dictionary_analysis/filters/Cargo.toml
cargo run --release --locked --manifest-path dictionary_analysis/filters/Cargo.toml -- KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis
python3 dictionary_analysis/report.py
```

## Artifact map

| File | Contents |
|---|---|
| [message.sstable](message.sstable) | Original message dictionary, unchanged |
| [terms.bin](terms.bin) | Expanded keys, `[u32 LE length][raw bytes]` records |
| [extraction.json](extraction.json) | Source provenance, exact sizes, term counts, definitions |
| [validation.json](validation.json) | Independent export checks |
| [existing_metadata.tsv](existing_metadata.tsv) | Readable current metadata, all blocks |
| [address_store_metadata.tsv](address_store_metadata.tsv) | Existing address-store group headers |
| [blocks.jsonl](blocks.jsonl) | Original block boundaries and observations |
| [numeric_blocks.tsv](numeric_blocks.tsv) | Blocks ranked by numeric-term percentage |
| [evaluation.json](evaluation.json) | Exact-model totals, size models, distributions |
| [evaluate_blocks.jsonl](evaluate_blocks.jsonl) | Per-block candidates, counts, and cardinalities |
| [filters.json](filters.json) | Bloom/suffix configuration results and sizes |
| [filters_blocks.jsonl](filters_blocks.jsonl) | Per-block actual filter decisions |
