# Benchmarking

For search-benchmark-game runs, union-only queries, and result analysis, see [Search benchmark game](SEARCH_BENCHMARK_GAME.md).

Use the dedicated Tantivy query benchmark repository. Its manifest is:

```text
~/Development/tantivy-query-benchmark/Cargo.toml
```

Change that `Cargo.toml` to switch the Tantivy dependency between this candidate branch and `main`. Benchmark the two configurations in separate runs so only one benchmark process is active at a time.

Run targeted benchmarks for the affected changes. If you are unsure which benchmarks to run, ask.

```bash
cd ~/Development/tantivy-query-benchmark
cargo bench --bench query_bench "top_docs_100+count AND group_name:flat_intersection"
```

```bash
cd ~/Development/tantivy-query-benchmark
cargo bench --bench query_bench "top_docs_100+count AND group_name:flat_union"
```

Compare absolute timings from the candidate and `main` runs; the tool will always compare to the previous run. The benchmark repository does not need to be inspected as part of the implementation.

If its existing query set does not sufficiently cover the query shape, density, or term count, report that insufficient coverage explicitly rather than silently treating the benchmark run as conclusive.
