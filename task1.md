# Task: Add Built-In Reader Downcast Helper For Query Construction

## Summary

Implement a first version of query-reader downcasting in Tantivy with the smallest possible design:

- query code can call a helper from Tantivy to try a built-in concrete reader downcast
- if the downcast succeeds, the callback is invoked with the concrete reader type
- otherwise the code falls back to the existing dynamic reader path
- this mechanism replaces the current query-creator entrypoints that live on `SegmentReader` or `InvertedIndexReader`
- there is no query-type registry
- there is no consumer/custom-reader registration in this first version

This first version intentionally does only this:

- built-in static dispatch only
- consumer-defined readers continue to use the dynamic fallback path for now
- support for external registration can be added later without changing the core query-side calling pattern

## Why This Version First

This version keeps the design easy to follow and implement.

It avoids:

- type-level registries
- query registries
- runtime erased callback registration
- threading consumer reader types through the full query-construction stack

It still gives a useful improvement:

- Tantivy query code can statically specialize for Tantivy's own standard reader type

## Constraints

The target behavior should satisfy these constraints for v1:

- a query impl can call a `try_downcast` helper and get a typed callback invocation
- there is no registry for query types
- queries may live in Tantivy or in a downstream consumer
- the downcast code itself lives in Tantivy, not in downstream code
- the default reader to try is Tantivy's standard inverted index reader
- there is no requirement in v1 to support downstream custom readers with static dispatch

Constraints intentionally not satisfied in v1:

- downstream consumers cannot override the reader type for static dispatch without future extension work
- there is no exhaustive registration mechanism for all query/callback combinations

## Core Design

Expose one helper with this shape:

```rust
pub fn try_downcast_and_call<R, C>(
    reader: &dyn DynInvertedIndexReader,
    callback: &mut C,
) -> R
where
    C: TypedInvertedIndexReaderCb<R>
```

Behavior:

1. Check whether `reader` is Tantivy's built-in concrete reader type.
2. If yes, invoke `callback.call(concrete_reader)`.
3. If no, invoke `callback.call(reader)` on the dynamic fallback path.

This means query code only needs to do:

```rust
let mut cb = BuildSomethingQuery { ... };
try_downcast_and_call(reader, &mut cb)
```

The query does not need to know how the downcast works.

## Required Traits / Types

The design should use two reader traits:

```rust
pub trait InvertedIndexReader {
    type Postings: PostingList;
    fn as_any(&self) -> &dyn Any;
    fn posting_list(&self) -> Self::Postings;
}

pub trait DynInvertedIndexReader {
    fn as_any(&self) -> &dyn Any;
    fn posting_list_boxed(&self) -> Box<dyn PostingList>;
}
```

The intent is:

- `InvertedIndexReader` is the properly typed version
- `DynInvertedIndexReader` is the dyn-erased object-safe version used through the stack
- query callbacks are generic over `InvertedIndexReader`, not over `DynInvertedIndexReader`

The usual bridge is:

- blanket-implement `DynInvertedIndexReader` for any `T: InvertedIndexReader`
- implement `InvertedIndexReader` for `dyn DynInvertedIndexReader` with `Postings = Box<dyn PostingList>`

That gives:

- concrete static dispatch when the helper successfully downcasts to a built-in concrete reader
- dynamic fallback through `dyn DynInvertedIndexReader` when it does not

The callback trait should look like:

trait TypedInvertedIndexReaderCb<R> {
    fn call<I: InvertedIndexReader + ?Sized>(&mut self, reader: &I) -> R;
}
```

That callback shape is the key API boundary.

## Implementation Outline

Inside Tantivy:

1. Identify the standard built-in inverted index reader type that queries should specialize for.
2. Make sure `DynInvertedIndexReader` exposes `as_any()` or equivalent downcast support.
3. Implement `try_downcast_and_call(...)` in the place that should replace the current query-creation mechanism on `SegmentReader` or `InvertedIndexReader`.
4. Migrate the existing query-construction path so query creators go through this helper instead of the old direct mechanism.
5. Start with one query, for example term query construction, then keep behavior unchanged.

Minimal pseudocode:

```rust
pub fn try_downcast_and_call<R, C>(
    reader: &dyn DynInvertedIndexReader,
    callback: &mut C,
) -> R
where
    C: TypedInvertedIndexReaderCb<R>,
{
    if let Some(reader) = reader.as_any().downcast_ref::<BuiltInReader>() {
        return callback.call(reader);
    }

    callback.call(reader)
}
```

## Query-Side Usage

A query implementation should use the helper through a callback object, and this should become the replacement for the current query-construction entrypoints on the reader/segment abstractions.

For example:

```rust
struct BuildTermQuery { /* fields */ }

impl TypedInvertedIndexReaderCb<Box<dyn Query>> for BuildTermQuery {
    fn call<I: InvertedIndexReader + ?Sized>(&mut self, reader: &I) -> Box<dyn Query> {
        Box::new(TermQuery::new(reader))
    }
}

pub fn build_term_query(reader: &dyn DynInvertedIndexReader) -> Box<dyn Query> {
    let mut callback = BuildTermQuery { /* fields */ };
    try_downcast_and_call(reader, &mut callback)
}
```

This keeps the query code simple:

- no type registry
- no consumer hooks
- no repeated downcast logic inside each query

It also makes the direction explicit:

- query construction no longer lives as bespoke logic directly on `SegmentReader` or `InvertedIndexReader`
- instead those existing entrypoints should delegate to a single callback-based downcast helper, or be replaced by it

## Non-Goals For V1

Do not implement any of this in the first version:

- registration of downstream custom reader types
- a registry keyed by query type
- a registry keyed by callback type
- macros for reader selection
- build-script code generation
- workspace-level external code generation

Those can be evaluated later if downstream custom-reader static dispatch becomes important.

## Why Not Solve Consumer Overrides Now

A downstream crate can define a custom reader type, but Tantivy as an upstream dependency cannot directly generate static-dispatch code for that reader type unless one of these is added later:

- the custom type is threaded through generics
- the consumer registers itself at runtime
- an external code-generation pipeline ties the crates together

That is intentionally out of scope for v1.

## Future Extension Path

If later needed, add a second mechanism on top of the same callback pattern:

- keep `try_downcast_and_call(...)` as the built-in default path
- add optional runtime registration for external reader types
- keep the query-side callsite unchanged or nearly unchanged

Possible later shape:

```rust
try_downcast_and_call(reader, &mut callback)
```

where the helper first tries:

1. registered external reader hooks
2. built-in Tantivy reader downcast
3. dynamic fallback

This is why the first version should keep the callback-based entrypoint narrow and central.

## Acceptance Criteria

- Tantivy exposes one helper for built-in reader downcast plus dynamic fallback.
- The new helper replaces the current query-construction mechanism on `SegmentReader` or `InvertedIndexReader`, rather than being added as a parallel path.
- The downcast code lives in Tantivy, not in query implementations.
- Every query constructor that is currently on the SegmentReader or InvertedIndexReader entrypoints is migrated to use the new helper and removed.
- If the reader is Tantivy's built-in concrete reader type, the callback is invoked with that concrete type.
- If not, the callback is invoked with the dynamic reader type. (this will be later changed)
- No query-type registry exists.
- No downstream registration mechanism exists in this first version.

## Suggested Task Breakdown

1. Add or reuse `as_any()` support on the dynamic reader abstraction.
2. Introduce or align the two reader traits: `InvertedIndexReader` and `DynInvertedIndexReader`.
3. Add the bridge between them so the dynamic path can still be used as an `InvertedIndexReader`.
4. Migrate term query construction first.
5. Add tests covering built-in concrete-reader dispatch and dynamic fallback.

## Suggested Tests

- A test where the reader is the built-in concrete Tantivy reader and the callback records that the concrete path was used.
- A test where the reader is only available as a dynamic reader and the fallback path is used.
- A test showing the query result is unchanged compared with the old behavior.

## Decision

For v1, choose the smallest design:

- built-in static dispatch only
- no consumer override mechanism yet
- leave room to add registration later if it becomes necessary

