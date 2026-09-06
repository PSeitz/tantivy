# Search benchmark game

Use `~/Development/search-benchmark-game`. `engines/tantivy-perf/Cargo.toml` points to the current folder of this repository. `engines/tantivy-main/Cargo.toml` points to the `main` branch of Tantivy.

When comparing two version of the current branch (e.g. latest changes), prefer to just change the current folder to the desired commit and run the benchmark. This avoids having to change the `Cargo.toml` files.

## Run the benchmarks

From the search-benchmark-game repository:

```bash
cd ~/Development/search-benchmark-game
cd engines/tantivy-perf/;cargo build --release;cd ../../;WARMUP_TIME=3 make bench
```

Always run the benchmark with `WARMUP_TIME=3`.

The current Makefile defaults to `tantivy-perf tantivy-main`, commands `TOP_100_COUNT TOP_100_FF COUNT`, and 10 measured iterations. Override these explicitly when needed:

```bash
WARMUP_TIME=3 make bench ENGINES="tantivy-perf tantivy-main" COMMANDS="TOP_100_COUNT"
```

## Execute only union queries

The Makefile defaults to `queries.txt`; override it with the `QUERIES` environment variable or Make argument. Create a separate JSON-lines query file to leave the original query set unchanged:

```bash
cd ~/Development/search-benchmark-game
jq -c 'select((.tags // []) | index("union"))' queries.txt > queries-union.txt

QUERIES=queries-union.txt WARMUP_TIME=3 make bench \
    ENGINES="tantivy-perf tantivy-main" COMMANDS="TOP_100_COUNT TOP_100_FF COUNT" NUM_ITER=10
```

This selects the exact `union` tag, excluding `intersection_union`. Build the engines before running the benchmark. This run also overwrites `results.json`. To measure only one collector mode, set `COMMANDS="TOP_100_COUNT"` (or another supported command).

## Analyze results

Run from the search-benchmark-game repository:

Inspect absolute timings for a single engine:

```bash
python3 tools/analyze.py tantivy-perf --command
```

Inspect absolute timings for a single engine and command:

```bash
python3 tools/analyze.py tantivy-perf --command TOP_100_COUNT --filter pure_union
```

Compare candidate and baseline on pure unions across all measured commands:
```bash
python3 tools/analyze.py tantivy-perf --compare tantivy-main --filter pure_union
```

For a full-suite run, report all query categories:

```bash
python3 tools/analyze.py tantivy-perf --compare tantivy-main --all-filters
```

Use `--filter pure_union` for exact union tags. `--filter union` uses substring matching and also includes `intersection_union`.
