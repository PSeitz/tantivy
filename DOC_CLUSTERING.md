# Document clustering in Tantivy

Retain documents during collection, then assign document IDs and index them in
cluster order. This avoids indexing in arrival order and subsequently permuting
document IDs and rewriting the document store.

## Usage

```rust
let mut writer = index_builder
    .single_segment_index_writer::<TantivyDocument>(directory, memory_budget)?
    .with_clustering(ClusteringConfig {
        fields: vec!["status".into(), "json.service".into()],
    })?;
writer.add_document(document)?;
let index = writer.finalize()?;
```

Configure clustering before adding documents. The
[index_file example](examples/index_file.rs) accepts a schema, JSONL input and
grouping fields.

## Lifecycle

```text
add_document -> retain the CompactDoc in its leaf group
finalize     -> visit groups in cluster order
             -> index each retained document and release it
             -> finish serializing the index
```

`TantivyDocument` is an alias for `CompactDoc`. Incoming CompactDocs move into
leaf vectors without copying or re-encoding. Other `Document` implementations
convert once through `into_tantivy_document`. Built-in indexing and custom
plugins receive the retained CompactDoc directly.

Clustering adds no document compression layer, dictionaries or shared arena.
The normal CompactDoc representation and final docstore compression remain.
Indexing errors are reported during finalization.

## Grouping

- Configured paths define successive levels, such as `status` then `json.service`.
- String values define groups. Pre-tokenized strings group by their text.
- Repeated fields and arrays use the first value.
- Missing values, empty arrays, non-string values and paths ending at an object
  share an ignored bucket, separate from the empty string.
- At every level, the ignored bucket comes first, followed by string groups in
  ascending lexicographic order. Documents within each leaf retain arrival order.
- `clustering_group_sizes()` returns leaf counts in traversal order.
- `DocIdGroups` provides the same grouping for caller-supplied IDs without
  retaining documents, for callers using manual remapping instead.

Clustering and manual document-ID mapping are mutually exclusive. An index's
`sort_by_field` setting still applies after grouping and can change the final
order. Grouping leaves document contents unchanged.

## Memory and scope

`mem_usage()` estimates retained document capacities, grouping allocations and
the segment writer's current memory. It performs no recursive document-size
walk. It counts reserved capacity even when unused and does not shrink documents.

The caller decides when to finalize and must leave headroom for growing indexing
structures, traversal scratch and output buffers. The retained-memory estimate
is not a peak-memory bound. Documents and consumed groups are released during
the drain. There is no automatic flush or segment rotation.

This API builds one segment. Partition routing, split management and integration
with the regular multi-segment writer remain caller responsibilities.

Historical compressed-staging measurements are preserved in
[DOC_CLUSTERING_BENCHMARKS.md](DOC_CLUSTERING_BENCHMARKS.md). CPU and memory changes
from retaining CompactDocs have not yet been benchmarked.
