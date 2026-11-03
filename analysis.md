# Warm-up API consolidation

## Goal

Tantivy reads and decodes the metadata needed to identify payload slices. It returns those slices before downloading the payload. Quickwit can then count the planned bytes and fetch the slices through their existing cache-backed handles.

```text
Tantivy                                   Quickwit
Read necessary metadata
Decode entries
Identify payload slices
    └── return slices ──────────────────► Count planned bytes
                                          Fetch slices
                                          Execute search
```

The split footer and reader opening are prerequisites outside these APIs. Metadata bytes already fetched during preparation are separate from the returned payload bytes.

This is a proposed API design, not an implemented change. The analysis follows Tantivy `9f2f1b2f6` and the main local Quickwit checkout at `3348f4dc989d2d63ea205b814cd93473ffa736b3`.

## 1. Combine exact terms and term ranges

An exact term is an inclusive range with identical endpoints:

```text
Exact term "apple"  → ["apple", "apple"]
Term range          → ["apple", "banana")
```

Both operations have the same preparation steps:

```text
Read relevant term-dictionary blocks
    → Decode matching TermInfo entries
    → Identify first and last matching terms
    → Return postings span and optional positions span
```

One method can replace `warm_postings()` and `warm_postings_range()`:

```rust
async fn postings_slices(
    &self,
    terms: impl RangeBounds<Term>,
    limit: Option<u64>,
    with_positions: bool,
) -> io::Result<Option<Vec<FileSlice>>>;
```

The name describes the returned data: slices containing postings and, when requested, positions.

- `None` means no matching term.
- Exact-term callers still know which term they requested. Quickwit can preserve required-term absence reporting and cancellation.
- The range limit and existing first-to-last span calculation remain unchanged.
- An exact-term lookup can remain an internal optimization if needed. It does not need a separate public entry point.
- Verify that an exact-term request still reads only the necessary dictionary block.

## 2. Combine single-field and batch fast-field helpers in Quickwit

Quickwit currently has a batching helper around a single-field helper:

```text
warm_up_fastfields()
    loops over segments × requested fields
        → warm_up_fastfield()
```

These represent one operation plus batching, rather than two independent capabilities.

Use one Quickwit function to collect column slices across the requested fields and segments. The single-field body can live inside that loop. Keep a separate internal helper only if it has another caller or makes the field-resolution logic clearer. Tantivy does not need an additional batch API for this.

### Exact field and descendants

For each requested fast field:

```text
list_dynamic_column_handles(field)
    → Read relevant column-dictionary blocks
    → Decode exact-name entries into handles

If subfields are requested:
list_subpath_dynamic_column_handles(field)
    → Read relevant column-dictionary blocks
    → Decode descendant entries into handles

Collect the column FileSlices from all selected handles
    → Return slices without reading column data
```

These existing Tantivy methods already stop at the right boundary. Keep them as low-level metadata lookups and combine their results in Quickwit.

The entries are inside the downloaded SSTable blocks. Decoding them reveals each column's type and byte range. It does not require a separate request for the entries.

A logical field can have multiple typed columns, so one requested field can produce multiple payload slices.

## 3. Combine full-dictionary and full-postings helpers in Quickwit

Quickwit currently drives both operations from the same `term_dict_fields` set. One helper can collect both resources:

```text
For each selected field and segment:
    collect whole dictionary data slice
    collect whole field postings slice
```

This preparation needs no additional metadata reads. The readers already know these ranges.

- For the dictionary, reuse `TermDictionary::file_slice_for_range(.., None)`.
- For postings, expose the existing postings slice through a synchronous accessor. Include the positions slice when requested by that accessor.
- Keep the underlying dictionary and postings access separate. They are different resources, even though this Quickwit caller requests them together.

Do not express full-postings access as an unbounded term-range lookup. That would introduce dictionary reads where the direct accessor needs none.

## 4. Keep automata separate

An automaton can match terms spread across the dictionary. Its preparation differs from the single contiguous span used for term ranges:

```text
Select and fetch relevant dictionary ranges
    → Enumerate matching terms on the CPU executor
    → Coalesce their postings ranges
    → Return postings slices
```

Preserve the existing coalescing rules and CPU executor. Quickwit takes over payload downloads and must retain a download concurrency limit.

Returning a complete slice collection means finishing enumeration before downloading postings. The current implementation overlaps those operations. Knowing the complete payload size first introduces a scheduling tradeoff that needs measurement.

For `warm_postings_automaton_with_bitset()`, reuse automaton preparation and separate the final bitset construction:

```text
Prepare postings slices
    → Quickwit counts and fetches them
    → Decode cached postings into a bitset
```

The bitset cannot be built from dictionary metadata alone.

## 5. Keep field norms as direct slice collection

Quickwit already obtains fieldnorm slices from the opened composite file:

```text
fieldnorms_readers().get_inner_file().open_read(field)
    → Existing FileSlice, if present
```

Collect these slices instead of immediately reading them. No new Tantivy metadata lookup or warm-up API is needed.

## Resulting Quickwit preparation groups

| Group | Existing helpers covered | Returned payload |
|---|---|---|
| Terms and term ranges | `warm_up_terms`, `warm_up_term_ranges` | Postings spans and optional positions spans |
| Automata | `warm_up_automatons` | Selected, coalesced postings slices |
| Fast fields | `warm_up_fastfield`, `warm_up_fastfields` | Selected column slices |
| Whole indexed fields | `warm_up_full_term_dictionaries`, `warm_up_all_postings` | Whole dictionary and postings slices |
| Field norms | `warm_up_fieldnorms` | Existing norm slices |

The outer Quickwit `warmup()` coordinates these groups:

```text
Prepare metadata and collect payload slices
    → Stop if a required term proves the split empty
    → Report planned payload bytes
    → Fetch through the returned FileSlices
    → Complete warm-up
```

This describes caller orchestration. Each Tantivy method retains its own preparation steps and return contract.

## Migration requirements

1. **Use names that describe returning slices.** Changing existing warm-up return types in place is unsafe: several Quickwit callers discard successful results, so they could compile while dropping the download plans. Migrate every caller explicitly, then remove obsolete methods.
2. **Preserve absence separately from cancellation.** An absent required term can prove a split empty. A cancelled operation cannot. Preserve the current single-segment restriction on split-wide cancellation from term absence.
3. **Preserve cache-backed handles.** `FileSlice` already supports byte counting and asynchronous reads. Download through those handles so later synchronous search reads use the populated caches.
4. **Preserve scheduling controls.** Keep automaton enumeration on the CPU executor. Keep a download concurrency limit and hold the warm-up permit through preparation and fetching.
5. **Keep the outer completion contract.** Quickwit's `warmup()` must finish required downloads before returning successfully for a split that will be searched.
6. **Define the byte quantity.** Summed slice lengths are planned payload bytes. They are not exact remote traffic or unique retained memory. Cache hits, overlapping ranges, coalescing, and retries affect those quantities. Metadata already downloaded is accounted for separately.
7. **Separate reporting from memory admission.** `SearchPermit::update_memory_usage()` updates accounting without waiting for permission to grow. Moving it before downloads does not turn it into a memory-budget gate.

## Tests required

Use a directory that records reads and rejects uncached synchronous reads. Test the boundary, rather than only query results:

- Preparation reads metadata but no payload.
- Returned slices cover the data required by the query.
- Fetching those slices makes the subsequent synchronous query succeed.
- Exact-term and equivalent inclusive-range preparation produce equivalent coverage.
- Missing terms preserve absence reporting and cancellation behavior.
- Optional positions, range limits, and empty results retain their behavior.
- Fast-field descendants and multiple typed columns return all required slices.
- Bitset construction runs after postings are fetched and preserves matching document IDs.

## Source locations

Tantivy:

- `src/index/inverted_index_reader.rs`: postings warm-up variants, range merging, CPU enumeration, and download limits.
- `src/fastfield/readers.rs`: exact-name and descendant column-handle listing.
- `columnar/src/columnar/reader/mod.rs`: column-dictionary reads and handle construction.
- `src/termdict/mod.rs`: `file_slice_for_range()` and the dictionary warm-up wrapper.
- `sstable/src/dictionary.rs`: dictionary block/range reads and full-body slice access.
- `common/src/file_slice.rs`: slice ownership, `num_bytes()`, and asynchronous reads.

Quickwit paths relative to its `quickwit/` directory:

- `quickwit-search/src/leaf.rs`: warm-up orchestration and all helper callers.
- `quickwit-search/src/search_permit_provider.rs`: memory accounting and warm-up permits.
- `quickwit-doc-mapper/src/doc_mapper/mod.rs`: warm-up requirements and simplification.
- `quickwit-directories/src/debug_proxy_directory.rs`: existing read-recording support for tests.
