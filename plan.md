# Block-oriented scoring plan

## Decision

Tantivy uses two execution styles behind one scoring model:

- sparse scored traversal stays on scalar `DocSet`/`Scorer` operations (`seek`,
  `advance`, and `score`);
- dense scored traversal uses wide match windows with eagerly aligned scores;
- counting stays on existing non-scoring `DocSet` and bitset paths.

Deferred scoring is not part of the API. A scorer completes matching and any
required scoring within each block call; no score-selection or discard lifecycle
crosses calls. Score-only TopN keeps the existing Block-WAND/BMW paths. Combined
`(TopDocs, Count)` uses threshold-aware scored batches: every match is returned for
Count, while scores that are provably noncompetitive may be placeholders.

## Design basis: SereneDB/IResearch

The [SereneDB block-scoring article](https://blog.serenedb.com/search-optimization-2)
and IResearch implementation motivate an end-to-end batch contract rather than
bulk posting decode alone:

- [`score_function.hpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/search/score_function.hpp)
  defines block scoring;
- [`iterators.hpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/index/iterators.hpp)
  gathers scorer arguments and passes aligned document/score arrays onward;
- [`bm25.cpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/search/bm25.cpp)
  computes BM25 over batches;
- [`block_disjunction.hpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/search/block_disjunction.hpp)
  combines child masks and dense score windows;
- [`max_score_iterator.hpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/search/max_score_iterator.hpp)
  applies score bounds to candidate processing;
- [`doc_collector.hpp`](https://github.com/serenedb/serenedb/blob/main/libs/iresearch/include/iresearch/search/doc_collector.hpp)
  separates exact TopK-with-count execution from ordinary score pruning.

Tantivy adopts the separation of posting decode, scoring batches, Boolean match
windows, and collector batches, but not the C++ API verbatim. These granularities
remain independent, and exact Count continues to observe every live match.

## Scoring API

```rust
struct BlockMaxScore {
    /// Exclusive end of the natural range covered by `max_score`.
    end: DocId,
    max_score: Score,
}

trait Scorer: DocSet {
    fn score(&mut self) -> Score;

    /// Adds matches and eagerly writes aligned scores. Without a threshold all
    /// matching scores are exact. With a threshold, only competitive scores
    /// are required to be exact.
    fn fill_matches(
        &mut self,
        requested: Range<DocId>,
        matches: &mut MatchMask,
        scores: &mut [Score],
        threshold: Option<Score>,
    ) -> DocId;

    fn max_score(&self) -> Option<Score> {
        None
    }

    fn block_max_score(&mut self, start: DocId) -> Option<BlockMaxScore> {
        None
    }
}
```

Scalar and scored-buffer methods remain available. The block operation is an
optimization for dense execution, not the canonical representation of sparse
data. Do not add capability flags, request/output enums, optional score outputs,
or sparse traversal operations to this contract.

## Block contract

`MatchMask` is reusable sliding storage with an absolute start and bounded
capacity. Requests start at `matches.start()` and must fit in its capacity.
`scores.len()` equals `matches.capacity()`, and `scores[i]` corresponds to
`matches.start() + i`.

For every document in `requested`, `fill_matches` overwrites both outputs: a
match sets its mask bit and a non-match has a clear bit and score `0.0`. Data
outside `requested` is unchanged. After processing, the caller advances both
buffers with `consume(end)`.

With `threshold = None`, every matching score is exact. Passing `Some(threshold)`
to a union enables pruning: every score strictly greater than the threshold is
exact, while a score less than or equal to it may be exact or a placeholder. Use
the supplied threshold itself as the placeholder. This is safe because TopDocs
accepts only strict improvements (`score > threshold`) and avoids assuming that
scores or thresholds are nonnegative.

The return value is at least `requested.end`, with no matches between the end of
the request and the returned document. `TERMINATED` means no later match exists.
Returning the request end is valid; the value need not identify the next match.
Implementations may decode ahead into ordinary traversal scratch, but keep no
scoring state for a completed call.

Boolean scorers eagerly request child scores and combine them in query order.
They may discard scores removed by Boolean matching and may own reusable masks
and score arrays, but must finish each internal block before advancing. This
preserves score accumulation order and avoids hidden cross-call state.

## Execution paths

### Sparse scored traversal

Use scalar `DocSet` traversal and scored buffers. Do not model sparse execution
with one-document block calls, sparse masks, or an optional block mode. Union
and intersection strategy selection remains density-sensitive.

### Dense scored traversal

Use wide `fill_matches` requests with aligned dense score arrays. Request width
may follow implementation details such as posting or union horizons. Dense sum
unions copy their dense score window directly instead of scattering scores per
match.

Term scoring consumes decoded posting batches, gathers field norms, and computes
BM25 in batches. Wrappers and Boolean nodes eagerly transform or combine aligned
child scores. Collectors receive parallel document and score slices; deletes
must filter them together.

### Counting and pruning

Count-only collection remains score-free. Score-only TopDocs continues to use
Block-WAND/BMW where supported. Combined `(TopDocs, Count)` is a distinct
threshold-aware mode:

1. TopDocs supplies its current strict competitive threshold after its heap is
   full. Before that, use exact scoring.
2. The scorer returns every match in the ordinary match mask/document batch. The
   executor applies the alive-doc filter, so Count remains exact after deletes.
3. Exact scores are computed only for documents whose safe upper bound is strictly
   greater than the supplied threshold.
4. Provably noncompetitive matches stay in the batch with `score = threshold`.
   Count sees them, while TopDocs rejects them using its normal strict comparison.
5. A threshold may increase while a batch is collected. The threshold captured at
   the start of the call remains safe and merely causes extra exact scoring.

No second candidate mask or separate match/scored collector channel is required.
The presence of the threshold is the contract: whenever a threshold is passed to
the union, it may prune and return placeholders for noncompetitive matches. A
caller requiring every exact score passes no threshold.

Prune at two granularities where useful:

- if the root block upper bound is less than or equal to the threshold, enumerate
  and count the whole block without exact scoring;
- inside a buffered union, build a per-document bound from the safe bounds of the
  children that actually matched that document. Score only documents whose
  composed bound is greater than the threshold.

A threshold-aware `BufferedUnionScorer` first gathers all child matches for its
horizon without BM25, retaining bounded posting arguments such as sparse
`(doc, term_freq)` slices only until that same call completes. It then composes
per-document bounds, batch-scores candidate child hits, writes exact combined
scores for candidates, and writes the threshold placeholder for pruned matches.
This is local processing within one eager block call, not a cross-call deferred
scoring lifecycle.

`max_score` and `block_max_score` are optional; `None` disables the associated
pruning. Bounds must be safe for the actual query-time statistics and score
semantics. Sum adds matching child bounds, DisMax uses
`max + tie_breaker * (sum - max)`, and `MUST_NOT` contributes nothing. Disable
pruning for unsupported combiners, negative or non-monotone scoring, NaN, or any
scorer without a proven bound.

Current postings metadata chooses a `(fieldnorm_id, term_freq)` pair using the
segment average field length. With custom statistics, query-time averages may
differ and the stored block value may not be a safe upper bound. Do not add new
pruning based on it without proving correctness or changing the metadata. Until
then, use a looser safe global term bound or perform exact scoring.

## Implementation sequence

- [x] Change `Scorer::fill_matches` in place to produce eager aligned scores. Do
  not add a transitional method, capability flag, request enum, or optional
  score output.
- [x] Convert leaves and wrappers to finish scoring inside each `fill_matches`
  call. Reuse caller-aligned output where possible and complete all scratch
  processing before returning.
- [x] Convert Boolean scorers to request eager child scores, apply their match
  operation, and combine surviving scores in query order. Use separate scratch
  where direct accumulation would change floating-point behavior.
- [x] Keep sparse unions and intersections on scalar `DocSet` traversal. Route
  only dense execution through wide match masks and score arrays.
- [x] Keep buffered dense unions on their 4,096-document horizon and copy dense
  sum-score windows directly to the caller. Avoid a per-match score scatter.
- [x] Collect all matches with exact scores through aligned document/score batches.
  Keep Count on non-scoring paths and score-only TopDocs on Block-WAND/BMW.
- [x] Delete deferred score-selection methods, lifecycle state, buffers, tests,
  and terminology once all implementations are eager.
- [ ] Pass an optional threshold through `fill_matches`. A buffered union prunes
  whenever `Some(threshold)` is passed and remains exact for `None`; initially
  pass the TopDocs threshold for combined `(TopDocs, Count)` collection.
- [ ] When given a threshold, `BufferedUnionScorer` scores only documents whose
  upper bound exceeds it and writes the threshold for all other matches.
- [ ] Add whole-block pruning for other scorers where safe bounds and score-free
  advancement already exist. `None` bounds retain exact eager scoring.
- [ ] Verify exact score, TopDocs, Count, delete, threshold-tie, and traversal
  parity. Benchmark sparse and dense flat unions/intersections against `main`,
  including competitive and noncompetitive `(TopDocs, Count)` blocks, and fix
  regressions.

## Correctness requirements

Tests should cover:

- union, intersection, minimum-should-match, DisMax, required/optional, and
  exclusion;
- nested Boolean trees, boosts, constants, phrases, and custom-scorer fallback;
- sparse, dense, disjoint, overlapping, empty, and terminated inputs;
- partial blocks, deletes, posting options, and fieldnorm variants;
- interleaved scalar traversal and block fills;
- exact Count parity and identical TopDocs hits for standalone and combined
  collection;
- threshold-aware batches containing every match, exact competitive scores, and
  threshold placeholders for pruned matches;
- heap-not-full behavior, rising thresholds within a batch, scores equal to the
  threshold, deletes in pruned slots, missing bounds, and unsafe-bound fallback;
- stable query-order score accumulation where floating-point ties affect TopN.

## Benchmarking

See [BENCHMARK.md](BENCHMARK.md) for the benchmark procedure and targeted commands.
