# Document clustering in Tantivy

## Purpose

Assign document IDs in cluster order so the index can be written once. The
existing approach indexes in arrival order, then permutes document IDs and
rewrites the document store. That rewrite adds disk I/O; retaining an expanded
in-memory representation instead adds memory pressure.

We retain compact documents during collection, then index them in cluster order
at finalization. The goals are to:

- Avoid the document-ID permutation and intermediate store write/rewrite.
- Reduce retained document memory, including grouping and dictionary overhead.
- Borrow retained storage during indexing instead of copying it into another
  batch representation.

Evaluate collection, draining and serialization together. Smaller staging buffers
are useful only when finalization memory and latency remain acceptable.

The [index_file example](examples/index_file.rs) indexes a JSONL file in cluster
order using a supplied schema and grouping fields.

## Lifecycle and ownership

`SingleSegmentIndexWriter` owns the clustering tree and a Stacker `MemoryArena`.
Text and layout dictionaries share that arena with encoded documents. Clusters
store document addresses and lengths. The caller decides when to finalize.

```text
add_document -> encode the document into the arena and group its byte range
finalize     -> build dictionary-ID-to-range lookups and release dictionary hash tables
             -> traverse clusters in cluster order
             -> assign document IDs and index borrowed document views
             -> release consumed groups and reuse document-view buffers
             -> release the arena and dictionary lookups after indexing the last document
             -> finish serializing the index to disk
```

Postings, fast fields and the document store are built during the drain. Docstore
serialization, compression and writes already happen while documents are indexed.
The final serialization tail completes the inverted index, fast fields and store.

Arena-backed collection and borrowed indexing are implemented. Memory
estimation and caller-budget validation remain open. Integration with the regular
multi-segment writer is separate work.

### Grouping

- Configured fields define successive levels, such as `status` then `service`.
- Grouping keys are strings. Document staging rejects pre-tokenized strings.
- Repeated fields and arrays use the first value.
- Missing values, empty arrays, non-string values and paths ending at an object
  share a group. Their documents are retained and indexed normally.
- Resolve configured paths through objects. Traverse larger subtrees first at each
  level. Equal-size siblings follow first arrival, refining the previously
  unspecified tie order. Documents within leaves retain arrival order.
- `DocIdGroups::new(&schema, &clustering.fields)` shares grouping with compact
  staging, so manual remapping reproduces the same order without retaining full
  documents. Group-size statistics describe the leaf groups.

Optional within-group sorting and small-group pooling have been removed to simplify
clustering. Historical comparisons and the rationale are preserved in
[the benchmark review](DOC_CLUSTERING_BENCHMARKS.md#timestamp-sorting-and-small-group-pooling).

## Compact document representation

Implemented in `src/indexer/doc_compression.rs` and used through the clustering
tree in `src/indexer/doc_clustering.rs`.

```rust
writer.with_clustering(ClusteringConfig {
    fields: vec!["status".into(), "service".into()],
    compression: CompressionConfig {
        message_fields: vec!["message".into()],
        max_fields_per_path: 256,
    },
})?;
```

Compression applies to clustered documents. Message token encoding is opt-in;
message paths default to empty.

### Values, text and layouts

Encoded documents preserve types, repeated fields, arrays, empty containers,
object order, duplicate keys and custom bytes. Pre-tokenized strings are rejected,
including inside arrays and objects. Lengths and scalars use Tantivy's serializers. The original document is released
after encoding.

Text and layout dictionaries use `SharedArenaHashMap` with the same `MemoryArena`.
The text dictionary shares strings across documents and fields, assigning
sequential `u32` IDs. Strings shorter than four bytes remain inline. The layout
dictionary shares ordered field-ID sequences and object-key layouts. Values stay
in each document's encoding. Empty or oversized dictionary keys remain inline,
because Stacker truncates keys longer than 65,535 bytes.

Encoding reuses document/layout scratch buffers. Nested values stream into the
output buffer, with lengths and layout IDs filled afterward. Each encoded document
is copied into the arena and represented by an eight-byte `ByteRange`: an `Addr`
and a `u32` length. Account for scratch capacities, hash tables and arena pages.

The arena supports allocations up to 1 MiB and a 4 GiB address space. Encoded
documents and individual reconstructed messages must fit within 1 MiB. Staging
reports an error when an allocation exceeds these limits. Dictionary admission
checks space before inserting a new key and can fall back to inline bytes.

Dictionary lookup retains Stacker's hashing semantics. With `compare_hash_only`,
equal hashes are treated as equal keys.

### Key admission

Admit up to `max_fields_per_path - 1` names at each object path in arrival order,
starting with the first document. The default is 256: 255 shared names plus an
inline fallback. The same rule applies to objects inside arrays.

Global admission budgets are 16,384 path nodes and 1 MiB of owned key-name
capacity. Once a budget fills, existing names and paths remain usable. Additional
names stay inline: a zero marker in the layout means the key name is stored beside
its value. Descendants of an untracked key also remain inline.

Admitted keys, IDs and buffered documents remain unchanged throughout collection.
There is no eviction, frequency-selection pass or buffer rewrite. The trade-off
is arrival-order bias: early names keep their slots even when later names occur
more often.

These budgets constrain the key tree. Text and layout dictionaries continue to
grow, so the caller must still bound total segment memory.

### Message tokens

Selected text/JSON paths split strings into alphanumeric/underscore runs and runs
of other characters, preserving whitespace and punctuation. Message tokens use the
text dictionary; digit-only tokens remain inline. Reuse an existing whole-text
ID when available. Otherwise choose message tokens when their encoded size beats
the inline-size estimate, falling back to whole-text encoding.

These tokens preserve the literal message text. Index tokenization still happens
during the drain using the configured analyzer. Dictionary overhead can outweigh
savings on unique text.

## Borrowing during finalization

Build ID-indexed `Vec<ByteRange>` lookups by iterating the dictionary hash tables
and reading their stored IDs. These ranges address the key bytes in the arena.
Release the hash tables, key tree and encoding scratch, then move the arena and
lookups into the staged-document view.

Parsing converts inline bytes and dictionary IDs to byte ranges. Text dictionary
entries are checked for UTF-8 once when the view is constructed. Inline text,
facets and object keys are checked during parsing. Reconstructed messages concatenate
validated text. All ranges use `ByteRange` directly.
Subsequent text access borrows directly from the arena through one documented
unchecked conversion. The arena only appends allocations, preserving the checked
bytes across document loads and page growth. Message reconstruction joins tokens
in a reusable buffer and copies the result into the same arena.

Reuse the view's parsed-value and message buffers for each document. Consume group
storage incrementally. Encoded documents, dictionary bytes and reconstructed
messages remain in the arena until all documents have been indexed. Release the
arena before the index's serialization tail.

Account for traversal metadata, reconstruction scratch and retained reconstructed
messages.
Custom plugins receive a materialized `TantivyDocument` to preserve type-downcast
compatibility. Validate that path separately from built-in borrowed indexing.

## Memory accounting

The estimate must cover allocations that coexist during collection and
finalization:

```text
staging arena (dictionaries, encoded documents and reconstructed messages)
+ remaining cluster storage and dictionary-ID-to-range lookups
+ document-view/reconstruction scratch
+ growing inverted-index and fast-field structures
+ store/compression/write buffers
+ serialization scratch
```

Measure capacities and arena slack. Ensure upstream batches and queues release
original documents after encoding. Include concurrent segments and queued input
in the caller's budget.

### Current reservation

The implementation currently uses a compression-independent reservation:

```text
segment writer's current memory
+ retained staging
+ max(retained staging, cumulative estimated owned-document bytes)
+ ID-indexed byte-range metadata
+ 3 × largest estimated owned-document size
+ allowance for retained reconstructed messages and arena page fragmentation
```

The cumulative term reuses the size calculation performed during encoding. It
counts repeated keys, values and owned-value structure before sharing. It reserves
future work rather than representing simultaneous document materialization. The
largest-document allowance covers the indexing view, reconstruction buffer and
optional plugin copy. Reconstructed-message allocations are retained across
documents. Their allowance is twice their cumulative byte length plus one 1 MiB
page when message reconstruction is used, covering page fragmentation.

This heuristic is overly conservative and can cause early commits and small
segments. It remains the implemented policy while a schema-aware replacement is
validated; it is not a universal memory bound.

### Schema-aware attribution to implement

Estimate the structures selected by the schema and configured tokenizers:

- **Inverted index:** unique encoded term bytes, recorder structs, hash-table
  capacity, posting-list blocks and arena slack. Count term/document pairs and
  optional frequencies/positions. Term identity includes field, JSON path and
  type. Include path metadata and enabled field norms.
- **Fast fields:** encoded values and populated document/column pairs, column
  metadata, and string/bytes dictionaries. Charge only fields configured as fast.
- **Docstore:** current blocks, oversized documents, compression scratch,
  queued/in-flight blocks and checkpoint metadata.
- **Serialization:** term-sorting references, posting-list decoding buffers,
  fast-field value/row-index buffers, dictionary ordinal maps and any remapping
  scratch. Account for reusable buffers by their high-water capacities.

Schema determines which costs apply. Analyzer output and term cardinality
determine their sizes. Avoid duplicating a full index merely to estimate it.

Respect release order when estimating the peak. The staging arena is released
before term sorting. Inverted-index, fast-field and store writers are then
serialized and released in sequence. Their independent maxima are not additive.

## Disk serialization and writing costs

**Production serializes to disk.** Serialized file contents belong to disk usage,
rather than a cumulative output-size allowance in the writer's heap estimate.
Serialization and I/O still require working buffers and CPU time.

Measure indexing, serialization/compression, writes and flush/sync separately
where possible. Record bytes written and elapsed/CPU time. Store compression and
writes overlap ingestion/drain, so measuring only the final serialization tail
misses some of that work. Distinguish buffered-write completion from the caller's
required durability or visibility boundary.

Track requested Rust allocations separately from process RSS/container memory.
Native codec workspaces, allocator residency and OS page-cache/writeback pressure
also affect memory limits.

## Commit policy and validation

The caller owns memory admission and timeout policy. `SingleSegmentIndexWriter`
has no timer or automatic flush. Validate batch overshoot, unusually large
documents, partially filled segments, idle-input timeouts and overlapping
finalizations. Measure from the commit decision through serialization and required
visibility, rather than treating the timer firing as commit completion.

Use the org2 staging export at `~/Development/moshiki/staging_logs.jsonl` as the
primary workload. Preserve every record and use status/service grouping. Varying
`custom.org` attributes remain data rather than export filters. Use
`~/Development/pomsky/config/cloudprem/datadog-logs.yaml` as the schema reference,
with the actual mapper, tokenizers and production compression settings. Document
input adaptations and missing fields. Other log/event and synthetic datasets
provide regression coverage.

Validate:

- Document contents and index results, including types, repeated values and order.
- Stable dictionary IDs and admitted keys across growth and exhausted budgets.
- Borrowed references pointing into the original arenas and remaining valid
  throughout the drain.
- Incremental group release, document-view buffer reuse and arena release after indexing.
- Built-in borrowed indexing and custom-plugin materialization separately.
- Memory estimates against complete lifecycle allocation/RSS peaks and caller
  commit decisions.

Current measurements and their limitations are in
[DOC_CLUSTERING_BENCHMARKS.md](DOC_CLUSTERING_BENCHMARKS.md).

## Follow-up: reuse indexing work

After memory accounting and caller behavior are validated, profile repeated
hashing, dictionary insertion, tokenization and layout parsing. Reuse compatible
Stacker maps or IDs where that removes demonstrated work.

A stored whole string, a posting term and a columnar dictionary key have different
identities and ordering requirements. Check compatibility before sharing them.
Keep ownership/API changes separate and justified; avoid a universal dictionary
abstraction in advance.
