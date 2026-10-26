Yes—**per-block byte sets are a good first step, and adjacent-byte sets are a natural extension.** The main challenge with generated IDs is that short substrings often genuinely occur in them. Better metadata can eliminate uncertainty, but it cannot skip a block containing a real match.

I inspected the SSTable and regex paths in this checkout. My recommendation is:

1. Add **byte presence and byte-length bounds** as cheap baseline metadata.
2. Distinguish **suffix searches** from **substring searches**. Suffix metadata can be substantially more selective.
3. Evaluate **bigrams and trigrams against actual block contents before choosing their encoding**.
4. Keep query-side pruning information optional and separate from exact automaton matching.

## 1. What the metadata must establish

For a block, you need to answer:

> Could any term in this block match the query?

A summary can conservatively answer “possibly.” It must only answer “no” when that is provable.

There are three distinct reasons a block might survive:

1. **A matching term actually exists.** Every correct block filter must retain it.
2. **The summary loses relationships.** For example, `m` and `s` exist in different terms.
3. **The representation has collisions.** For example, a Bloom filter reports an absent bigram as possibly present.

These distinctions matter when evaluating the design. Increasing Bloom-filter size fixes the third issue. Longer grams or more precise summaries address the second. Neither fixes the first.

## 2. Byte presence: cheap and useful, with an important query-side distinction

A 256-bit set is **32 bytes per block**, representing byte values. ASCII itself has 128 values. Bytes are the appropriate domain here because the SSTable and automata operate on bytes.

For:

```text
.*ms.*
```

every matching term contains both `m` and `s`. You can reject a block when either byte is absent:

```text
required_bytes ⊄ block_bytes  →  skip
```

This should work well for separating decimal numbers from alphabetic queries.

### The automaton needs requirements, not just its possible alphabet

For `.*ms.*`, the wildcard can consume a broad alphabet. Comparing the block alphabet with all bytes the automaton might consume would lose the useful information.

You need to preserve that **`m` and `s` are mandatory despite the wildcard**.

Likewise:

```text
.*(ms|kg).*
```

requires:

```text
(has m AND has s) OR (has k AND has g)
```

Requiring all four bytes would incorrectly skip valid matches.

A simple initial implementation can use bytes required by **every** accepting path. A stronger implementation retains these AND/OR conditions.

### Generated IDs still have useful alphabet restrictions

Generated IDs usually cover their encoding alphabet rather than all 256 bytes:

- Hexadecimal: roughly 16 symbols, plus separators.
- Base36: 36 symbols.
- Base62: 62 symbols.

A hex-ID block can already be excluded for `.*ms.*`. It becomes difficult for queries using bytes inside the ID alphabet, such as `.*dead.*`.

Also, a block containing numbers, IDs, and ordinary terms has their **combined** alphabet. A few diverse terms can weaken the summary considerably.

## 3. Length bounds are worth storing—but help different queries

Store minimum and maximum **full-key byte lengths**.

Reject when the block’s length interval and the query’s possible length interval do not overlap.

Examples, assuming ASCII patterns:

| Query | Possible lengths | Can reject a block containing only 32-byte IDs? |
|---|---:|---|
| `ms` | 2 | Yes |
| `[a-z]{3,12}` | 3–12 | Yes |
| `.{0,8}ms` | 2–10 | Yes |
| `.*ms` | 2–∞ | No |
| `.*ms.*` | 2–∞ | No |

**Fixed-length IDs do not help length pruning when the query permits arbitrarily long terms.**

Min/max also loses gaps. If a block contains only lengths 5 and 32, it still appears compatible with length 12. If exact-length queries are common, a length-presence bitmap can improve this—for example, bits for lengths 0–63 plus an overflow indicator.

I would start with min/max. They are inexpensive, and the benefit is straightforward to measure.

For Unicode queries, these must remain byte bounds. A Unicode character can occupy several bytes.

## 4. Your adjacent-tuple idea is a bigram index

For each term, collect its adjacent byte pairs:

```text
"terms" → te, er, rm, ms
```

Then `.*ms.*` can reject a block where `ms` is absent, even when both `m` and `s` occur separately.

That is a meaningful improvement.

### Exact bigram storage

A full byte-pair bitmap costs:

```text
256 × 256 bits = 8 KiB per block
```

That is too large as a default here. Your current SSTable targets **4,000 bytes of delta-encoded keys per block**, excluding values.

However, there is a useful exact encoding:

### Use the block’s alphabet to pack the pair matrix

If a block contains `k` distinct bytes:

1. Store its 256-bit byte set.
2. Assign the present bytes dense ranks `0..k-1`.
3. Store a `k × k` pair bitmap.

A pair is addressed by:

```text
rank(first_byte) * k + rank(second_byte)
```

Absent bytes immediately reject the pair.

| Distinct bytes in block | Exact pair bitmap | Including 32-byte alphabet |
|---:|---:|---:|
| 10 | 13 B | 45 B |
| 16 | 32 B | 64 B |
| 36 | 162 B | 194 B |
| 62 | 481 B | 513 B |
| 95 | 1,129 B | 1,161 B |

This is a promising way to store your tuples **without probabilistic collisions**.

The remaining question is whether enough pairs are absent to make it useful.

## 5. The fundamental limitation: random IDs really contain short substrings

Suppose a block has:

- `N` random IDs,
- each of length `L`,
- drawn uniformly from an alphabet of size `A`.

For a specific `q`-byte substring within that alphabet, a useful approximation is:

```text
P(substring occurs somewhere in block)
    ≈ 1 − exp(−N × (L − q + 1) / A^q)
```

For **100 IDs of length 32**:

| ID alphabet | Specific bigram present | Specific trigram present | Specific 4-gram present |
|---|---:|---:|---:|
| Hex, 16 symbols | ~100% | 51.9% | 4.3% |
| Base36 | 90.9% | 6.2% | 0.17% |
| Base62 | 55.4% | 1.25% | 0.020% |

These are illustrative random-data estimates, not measurements of your lexicographically sorted blocks. Overlapping windows and sorting affect the details.

The important result is:

> For `.*ms.*` over base36 IDs, even a perfect bigram filter may retain about 91% of these blocks, because they really contain `ms`.

For a three-character substring, the situation changes dramatically.

Therefore:

- Bigrams are necessary to accelerate arbitrary two-byte substring queries.
- Trigrams are much more promising for random IDs when the query has a three-byte literal.
- Longer grams improve selectivity further, but cannot directly serve shorter literals.

You cannot use only a trigram filter to answer the bigram-membership question for `ms`.

## 6. A leading wildcard may offer a much stronger signal: the end of the term

This distinction is especially important for your workload:

```text
.*ms       ends with "ms"
.*ms.*     contains "ms" anywhere
```

The current regex implementation matches whole keys, so those are different queries.

For a suffix query, store the **last two or three bytes of each term**, rather than relying only on anywhere-in-term grams.

With 100 random base36 IDs:

- Probability that some ID contains `ms` anywhere: approximately **90.9%**.
- Probability that some ID ends in `ms`: approximately **7.4%**.

That is a large improvement using the same two-byte literal.

Suffix summaries also have fewer insertions:

```text
anywhere bigrams: up to L−1 per term
suffix bigram:   one per term
```

This makes a small Bloom filter much more effective too.

**If many of your expensive queries are “ends with,” I would prioritize suffix summaries over a general trigram filter.**

Possible representations:

- Exact alphabet-packed suffix-pair bitmap.
- Bloom filter of fixed-length suffixes.
- A small suffix trie for more variable-length queries.

For a workload dominated by suffix search, a secondary dictionary of reversed terms is a larger architectural option. It turns suffix lookup into prefix lookup, at the cost of another index and a mapping to original term ordinals.

## 7. Bloom filters make grams compact, but sizing matters

For trigrams, an exact bitmap over all byte values costs 2 MiB per block. A Bloom filter or static approximate-membership structure is more practical.

At write time:

```text
for each term:
    insert all q-byte windows
```

At query time:

```text
if a required qgram is definitely absent:
    skip block
```

### A tiny filter can saturate quickly

The relevant quantity is **distinct grams per block**, not the number of terms.

Random IDs can contribute thousands of distinct trigrams. For an illustrative block with 2,500 distinct grams, standard Bloom estimates with suitable integer hash counts give:

| Filter size | False-positive probability for an absent gram |
|---:|---:|
| 256 B | ~70% |
| 512 B | ~46% |
| 1 KiB | ~21% |
| 2 KiB | ~4.3% |

A 256-byte filter can still help. It is simply much weaker than a “1% Bloom filter” intuition would suggest.

Longer query literals supply several grams:

```text
timeout → tim, ime, meo, eou, out
```

Any definitely absent gram rejects the block. This can make a modest filter effective for longer literals. The probes are correlated, so measure the combined result rather than assuming independent false-positive probabilities.

Also, even exact gram membership loses term identity and ordering. Different required grams can occur in different terms.

### My representation preference

- **Byte presence:** exact bitmap.
- **Bigrams:** benchmark alphabet-packed exact bitmaps against small hashed filters.
- **Trigrams or longer grams:** benchmark Bloom filters first.
- Consider XOR/binary-fuse-style filters later if their space savings justify more complicated construction.

If a filter exceeds its budget, the safe fallback is “summary unavailable” or another conservative representation. Arbitrarily dropping grams introduces false negatives.

## 8. How to connect this safely to regex automata

I would separate two levels of sophistication.

### Level 1: compile necessary conditions once per query

The query carries optional constraints such as:

- Minimum/maximum byte length.
- Required bytes.
- Required grams.
- Required suffix grams.
- Eventually, AND/OR combinations.

Examples:

```text
.*timeout.*
    requires all trigrams of "timeout"

.*(timeout|failure).*
    requires grams(timeout) OR grams(failure)

.*ab(cd)?.*
    requires "ab"
    does not require "cd"

.*ab.*cd.*
    requires "ab" AND "cd"
    does not require "bc"
```

Keep exact automaton matching as the final check.

This is an established technique. Russ Cox’s [description of regex search using trigram indexes](https://swtch.com/~rsc/regexp/regexp4.html) explains the AND/OR extraction and how to weaken constraints safely when expression analysis becomes too large.

### Level 2: intersect the automaton with the summary

Your metadata can itself describe a permissive language:

> All strings using the block’s bytes, permitted adjacent pairs, and permitted lengths.

Every actual block term belongs to that language. If it has no intersection with the query language, the block can be skipped.

For byte presence alone:

1. Take the query automaton.
2. Disallow transitions on bytes absent from the block.
3. Check whether an accepting state remains reachable.

For bigrams, the search additionally remembers the previous byte.

This handles classes and alternatives that simple mandatory-literal extraction misses. However, per-block graph traversal can cost significantly more than several bit tests.

I would treat this as a later stage, with:

- Query analysis cached once.
- Cheap length/byte/gram checks first.
- A work budget.
- “May match” on budget exhaustion.

The current generic `Automaton` trait does not expose all the state-identity information needed for convenient graph exploration. Concrete regex states support it, so a specialized helper is possible. Merely wrapping automata with the existing intersection combinator does **not** establish intersection emptiness.

## 9. Where this fits in your implementation

There is a clean integration point.

### Build summaries from inserted full keys

[`sstable/src/lib.rs`](sstable/src/lib.rs), `Writer::insert_key`, receives the original key before delta encoding.

Accumulate there and finalize when the physical block flushes.

This avoids several mistakes:

- Measuring suffix length instead of full term length.
- Missing grams crossing the retained-prefix/suffix boundary.
- Deriving metadata from synthetic index separators.
- Missing the split `insert_key` / `insert_value` production path.

For example:

```text
previous term: abc
next term:     abd
encoded suffix: d
```

The next term introduces `bd`. Processing only the suffix would miss it.

### Store summaries alongside the index

The dictionary currently loads its index region when opened. A section keyed by block ID would make summaries available before data-block I/O.

Putting them inside compressed data blocks would lose much of the benefit.

The trade-off is resident index size. Per million blocks:

- 32 bytes/block: 32 MB.
- 512 bytes/block: 512 MB.

That makes the metadata budget consequential even when disk overhead looks acceptable.

### Filter before reads and range coalescing

In [`sstable/src/dictionary.rs`](sstable/src/dictionary.rs), `get_block_iterator_for_range_and_automaton` currently does:

```text
lexical-bound candidates
→ requested block-range filter
→ discard block IDs
→ coalesce byte ranges
```

Insert summary checks before block IDs are discarded.

This shares the pruning logic between synchronous and asynchronous reads and can save:

- Data reads.
- Decompression.
- Value decoding.
- Term-by-term automaton execution.

Two details matter:

1. Async hole coalescing can merge across a rejected block and read/decode it again. Measure **bytes actually avoided**, separately from blocks logically rejected.
2. V3 currently scans the separator index to find candidates. Summary pruning saves data work while still leaving that index scan.

A format extension also needs explicit versioning, optional-summary handling, and the zero-/one-block cases.

### Keep the pruning API optional

I would preserve `search(automaton)` and add optional query constraints through its builder/query-weight path.

That keeps:

- Existing custom automata working.
- Old dictionaries searchable.
- Exact matching unchanged.
- The non-SSTable dictionary backend compatible.

Missing constraints or missing metadata means “retain candidate.”

## 10. What I would measure before implementing the on-disk format

**First build an offline evaluator over real block boundaries, using exact sets.** This separates the usefulness of the information from its storage encoding.

For representative queries, compare:

1. Existing lexical pruning.
2. + Byte presence.
3. + Length bounds.
4. + Exact bigram membership.
5. + Exact trigram membership.
6. + Exact suffix-bigram/trigram membership.
7. Exact matching: does any term in the block actually match?

The last measurement gives the **maximum possible block-skipping rate**.

If exact bigrams barely improve pruning, a Bloom encoding will not rescue them. If exact trigrams eliminate 99% of candidates, then testing memory-efficient representations is worthwhile.

Collect:

- Distinct bytes and grams per block.
- Terms and full-key bytes per block.
- Metadata size.
- Blocks retained.
- Compressed bytes read.
- Decompression and matching CPU.
- End-to-end latency, warm and cold.
- Query literal lengths and suffix-versus-substring frequency.

For correctness, differential-test with pruning enabled and disabled, including values and term ordinals. Cover Unicode, alternation, optional expressions, empty terms, prefix compression, and async coalescing.

## Recommendation

I would implement this incrementally:

**First:** byte presence + min/max byte length + optional query constraints. This is small and directly addresses numeric blocks.

**Next:** choose based on workload:

- Mostly `.*literal`: suffix summaries.
- Mostly `.*literal.*`, with literals of length ≥3: trigram filters.
- Important two-byte substring queries: exact alphabet-packed bigrams, with realistic expectations for random IDs.

**If short substring queries must be fast even when almost every block contains a match:** block metadata has reached its limit. You need finer-grained candidate identification, such as gram-to-term postings, or smaller independently readable blocks.

The most important design decision is therefore **which information is selective on your actual blocks**, followed by how to encode it. Your byte-set proposal is a sound baseline. Suffix awareness and longer grams are the strongest next candidates.

No repository files were changed.
