# Fewer postings requests within an extra-byte budget

**Priority: minimize request count, then minimize bytes.** For each **field +
segment + call**, first form the historical baseline by merging every gap of at
most **512 KiB**. Use that plan and allow additional fetched bytes of
`max(2 MiB, floor(baseline_fetched_bytes / 4))`.

`common::merge_ranges_with_byte_budget` merges the globally cheapest remaining
gaps until the allowance cannot cover the next gap. Each merge removes one
request. This minimizes requests within the allowance, then minimizes bytes for
that request count. Subsequent merges have **no per-gap ceiling**.

The guarantees are:

- New requests **never exceed the fixed-512-KiB baseline**.
- Total fetched bytes are at most baseline bytes plus 25%, rounded down, with a
  **minimum 2 MiB extra allowance** per field/segment/call.
- The allowance is a ceiling, not a requirement to consume all available bytes.

The bandwidth cost is deliberate: fewer requests may fetch more bytes. Small
plans may grow by more than 25% because of the 2 MiB minimum. A fixed gap threshold
already minimizes bytes for its request count. Improving that request count
requires spending bytes. The prior code allowed more requests, contrary to this
objective, and was removed. Dictionary planning is unchanged. These measurements
establish read plans and make **no observed latency claim**.

## Reproduce

Historical recipe, requiring the experimental source (including the unpublished `examples/postings_read_plans.rs`) and the original local split's non-inline dictionary:

```sh
cargo run --release --example postings_read_plans --features quickwit -- \
  /tmp/wildcard-data/splits/KH4Y_01M5MTJQ3SVHDTK2W3XCB5/4fea6c896fd045aba3cece0808f3e41e.term \
  > experiments/inline-postings/request-budget.json

cargo run --release --example postings_read_plans --features quickwit -- \
  /tmp/wildcard-data/splits/KH4Y_01M5MTJQ3SVHDTK2W3XCB5/4fea6c896fd045aba3cece0808f3e41e.term \
  > /tmp/request-budget-repeat.json
cmp experiments/inline-postings/request-budget.json /tmp/request-budget-repeat.json

cargo clippy --release --example postings_read_plans --features quickwit -- -D warnings
```

The release example and clippy passed after wiring the planner into the experimental warmup call site.
Two release runs produced byte-identical JSON (`cmp` passed). All 35 plans passed
request-count, byte-budget, and coverage assertions. Their results also match an
independent cheapest-gap calculation from the source dictionary's range extraction.
The common tests include exhaustive comparison against every gap subset for small
inputs. The full Quickwit library suite passed with 1,361 tests.

`examples/postings_read_plans.rs` calls
`common::merge_ranges_with_byte_budget` on a clone of the fixed-512-KiB plan.
The unpublished compact summary `request-budget.json` records all
35 cutoff/pattern combinations, matching/inline/external input counts, useful
bytes, both plans, and each extra-byte allowance. It contains no individual ranges.

The example verifies footer compatibility and CRC, opens composite physical
field 3, and enumerates all source terms **once**. Case-sensitive byte
substring/prefix matching follows the seven patterns below. Cutoffs 0, 8, 32, 64,
and 128 remove **every** list with `doc_freq <= cutoff`, including nonmatching
lists, before remapping matching external offsets. This models compaction without
rebuilding postings or dictionary. Only the `.term` file is needed.

**Cutoffs 64 and 128 are hypothetical.** The experimental implementation accepts at
most 32 documents per inline list. These larger cutoffs model external postings
removal only. An isolated encoding experiment measures their dictionary storage
sizes in [`SIZE_UP_TO_128.md`](SIZE_UP_TO_128.md). Cold dictionary read costs
remain unmeasured.

The source contains **7,671,280 terms** and **253,765,218 postings bytes**:

| Cutoff | Removed lists, all terms | Removed bytes | Remaining postings bytes |
|---:|---:|---:|---:|
| 0 | 0 | 0 | 253,765,218 |
| 8 | 6,597,536 | 34,311,754 | 219,453,464 |
| 32 | 7,537,196 | 99,733,706 | 154,031,512 |
| 64 | 7,644,231 | 118,772,219 | 134,992,999 |
| 128 | 7,668,224 | 125,484,921 | 128,280,297 |

Assertions verify nonempty contiguous original ranges, positive frequencies,
absence of source inline lists, dictionary count, exact coverage of every input
range, ordered disjoint output, requests no greater than the fixed baseline, and
bytes between baseline and baseline plus allowance. The 512 KiB gap rule applies
only when constructing the baseline. Coverage validation permits larger gaps in
the new plan.

## Inline cutoff 8 results

Exact bytes. Each policy cell is **requests / planned external bytes**, including
holes. Matching includes inline lists. Useful bytes include only external lists.
The allowance is additional to baseline fetched bytes, including baseline holes.

| Pattern | Matching lists | External lists | Useful bytes | Fixed 512 KiB | Request budget | Allowed extra bytes |
|---|---:|---:|---:|---:|---:|---:|
| `*a*` | 1,921,269 | 3,870 | 34,697,432 | 33 / 79,121,102 | 11 / 97,597,707 | 19,780,275 |
| `*ab*` | 75,774 | 203 | 288,249 | 30 / 1,947,701 | 28 / 3,401,299 | 2,097,152 |
| `*aba*` | 2,817 | 16 | 143,930 | 5 / 657,307 | 4 / 2,015,570 | 2,097,152 |
| `*traaa*` | 2 | 0 | 0 | 0 / 0 | 0 / 0 | 2,097,152 |
| `traaa*` | 0 | 0 | 0 | 0 / 0 | 0 / 0 | 2,097,152 |
| `*player*` | 21 | 21 | 4,221,761 | 11 / 4,699,006 | 10 / 5,943,196 | 2,097,152 |
| `*ms*` | 50,585 | 27 | 825,018 | 16 / 1,556,712 | 14 / 2,922,609 | 2,097,152 |

For `*a*`, requests fall from **33 to 11**, spending **18,476,605 additional
bytes (23.4%)**. `*ab*` saves two requests for 1,453,598 additional bytes.
`*aba*` saves one for 1,358,263 bytes. `*player*` saves one for 1,244,190 bytes.
`*ms*` saves two for 1,365,897 bytes. The two `*traaa*` matches are inline at
cutoff 8. The prefix `traaa*` has no matches.

Sums over seven independent queries, with equal weight and no cross-query caching:

| Cutoff | Useful bytes | Fixed requests / planned bytes | Request-budget requests / planned bytes | Allowed extra bytes |
|---:|---:|---:|---:|---:|
| 0 | 50,089,825 | 378 / 596,950,546 | 185 / 743,579,954 | 154,308,223 |
| 8 | 40,176,390 | 95 / 87,981,828 | 67 / 111,880,381 | 32,363,187 |
| 32 | 39,993,334 | 73 / 59,797,495 | 53 / 77,019,020 | 25,666,533 |
| 64 | 39,959,502 | 54 / 54,016,768 | 37 / 69,984,821 | 24,497,194 |
| 128 | 39,950,059 | 47 / 53,577,891 | 32 / 67,019,221 | 24,494,177 |

Each query gets its own allowance, including the two empty queries at cutoff 8.
Empty plans consume none of that allowance. Aggregate allowances are sums of the
per-call ceilings. These totals illustrate cutoff sensitivity rather than a
workload forecast.

## Hypothetical cutoffs 64 and 128

External-postings requests and decimal MB, using the same byte-budget policy.
Useful bytes contain only matching external lists. Downloaded bytes also include
coalesced gaps. Inline lists live in the dictionary and are excluded from both.

| Query | Requests ≤32 | Requests ≤64 | Requests ≤128 | ≤64 downloaded / useful MB | ≤128 downloaded / useful MB |
|---|---:|---:|---:|---:|---:|
| `*a*` | 16 | 11 | 8 | 59.057 / 34.494 | 59.044 / 34.486 |
| `*ab*` | 12 | 7 | 6 | 2.239 / 0.277 | 0.277 / 0.277 |
| `*aba*` | 1 | 1 | 1 | 0.143 / 0.143 | 0.143 / 0.143 |
| `*player*` | 10 | 8 | 8 | 5.725 / 4.221 | 5.725 / 4.221 |
| `*ms*` | 14 | 10 | 9 | 2.821 / 0.824 | 1.831 / 0.823 |
| **Total** | **53** | **37** | **32** | **69.985 / 39.960** | **67.019 / 39.950** |

Both `*traaa*` and `traaa*` have zero external requests at these cutoffs.
These totals exclude the unmeasured dictionary cost of the larger inline lists.

## Dictionary costs, planner cost, and scope

Coalescing changes external postings reads only. **Dictionary bytes are unchanged**
by this policy. The historical absolute-ID inline-8 measurement has
**74,303,555 bytes (74.304 MB)** of cold dictionary-block reads for each
leading-wildcard query. Adding that measured dictionary cost to `*a*` gives
153,424,657 bytes with fixed merging and 171,901,262 with the request budget.
See [`ABSOLUTE_ID_READS.md`](ABSOLUTE_ID_READS.md) for the independently measured
dictionary plans. This example does not remeasure dictionary I/O. New dictionaries
use document-ID deltas, whose storage sizes are in [`SIZE_UP_TO_128.md`](SIZE_UP_TO_128.md).
Their cold dictionary read costs require a new measurement. The external-postings
plans here apply to both encodings because their external ranges are identical.

The experimental warmup path merges gaps up to 512 KiB while streaming, before collecting
ranges. Thus cutoff-0 `*a*` reduces 1,921,269 matching ranges to **48** planner
inputs. At cutoff 8, 3,870 external ranges become **33**. The helper sorts only
the remaining baseline gaps: **O(r log r) CPU time and O(r) scratch memory**, where
`r` is the fixed baseline request count. The initial stream merge takes O(n) time.
The example retains matching input ranges to validate exact coverage. Its memory
use therefore differs from the warmup path's small baseline-range collection.

Actual request overhead, storage throughput, concurrency, cache states, and
end-to-end latency require measurement. This run reads dictionary metadata and
validates plans. It measures neither postings storage execution nor planner CPU
time or allocator peaks. Fewer requests and higher bandwidth can affect latency
in opposite directions.

Each coalesced contiguous range counts as one request. Measurements exclude
positions, index-opening reads, hit/document retrieval, other fields, cache reuse,
retries, alignment, and storage-specific maximum request sizes. Offsets are
relative to the field postings payload, excluding its token-count header.
No postings payloads are read or decoded, and no split is rewritten or deployed.
Historical fixed-gap measurement artifacts remain unchanged. The existing
`short_postings_size reads` mode retains the fixed-gap baseline.
