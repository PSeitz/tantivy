# Generic prefix-excluded trigrams

16,794 original blocks, 7,671,280 terms. No caller patterns, query-driven features, or neighbor unions. All grams come from individual terms after removing each block's full actual common prefix.

## Outcome

Tested layouts meeting both ≥90% `*player*` block skip and ≤195,640 added bytes: **0**.
Exact trigram baseline: **99.92%** player skip, 46,778,329 added B with whole-file zstd3. Its 14 candidate blocks include 1 false-positive block.
Query-independent budget choice: **runs_all_512**, 141,059 added B, 5.48% player skip.
Highest observed player skip within budget (post-evaluation comparison, not feature selection): **bloom_128_k2_seed2**, 150,800 added B, 7.43%.
Smallest tested ≥90% candidate: **bloom_512_k2_seed0**, 6,443,859 added B (32.94× allowance), 91.12% player skip.
Smallest tested Bloom setting reaching 90% for **all three tested seeds**: 1024 B/block, k=1, 15,334,985–15,339,021 added B, 97.55–98.61% skip. This remains a finite-seed observation.
At the cheapest successful Bloom setting, the measured seed range is 83.73–91.12%. A successful seed does not establish the same skip rate for every seed.
Without whole-stream compression, the highest observed budget-fitting result is **runs_longest_2**, 171,859 added B, 2.83% player skip.
The two targets are incompatible in these tested layouts. This is a measured result, not an impossibility claim about other generic encodings.

## Accounting and construction

Every total includes the 48 B sparse actual-prefix extension table and 20,307 B remainder-byte sidecar once. Prefix reconstruction also uses existing block bounds. The alphabet has 41 bytes and its full cubic product has 68,921 ranks. Alphabet, GT01 header, run framing, intern IDs, and Bloom seed/k/width are serialized and charged. No external trigram vocabulary is required.
Exact membership records the union of grams within each block. Grams from different terms or positions can still combine into false-positive candidates.
Run thresholds fill only zero intervals of length ≤ threshold, including edges. Longest-K retains K maximal absent intervals, ties by lower rank. Independent interning shares identical records without unioning blocks. The budget choice minimizes filled bits, then storage, before loading the queries.
Bloom filters insert every distinct remainder trigram. GT01 specifies SplitMix64, two hash salts, and modulo bit placement. Tested sizes are 16–2048 B/block, k=1/2, seeds 0/1/2. These are finite-seed measured outcomes, not universal skip guarantees. Bloom false positives are conservative. Exact-set, gap-fill and inserted-gram checks establish no missing inserted grams.
Sizes come from files written and read back. Raw files above 2 MB are removed after measurement to limit artifacts. Their retained whole-file zstd3 versions reproduce the measured raw bytes and SHA-256 exactly. JSON records every raw/compressed layout and its retention status. Whole-stream compression describes disk metadata only. It requires stream decompression, offers no charged per-block offsets, and does not establish resident-memory or latency performance. Runtime/latency were not measured.

## Best serialized layout for each construction

| Construction | Added B | Raw stream B | Disk layout | Filled zero bits | player skip | Budget |
|---|---:|---:|---|---:|---:|:---:|
| exact_bitset | 46,778,329 | 144,697,169 | zstd3 | 0 | 99.92% | no |
| runs_exact_0 | 46,998,229 | 93,940,464 | zstd3 | 0 | 99.92% | no |
| runs_all_1 | 45,646,783 | 86,585,415 | zstd3 | 3677513 | 99.92% | no |
| runs_all_2 | 44,171,740 | 80,448,104 | zstd3 | 9847383 | 99.92% | no |
| runs_all_4 | 41,060,221 | 70,067,292 | zstd3 | 27803342 | 99.92% | no |
| runs_all_8 | 35,144,561 | 54,563,618 | zstd3 | 77347219 | 99.80% | no |
| runs_all_16 | 25,274,221 | 34,738,633 | zstd3 | 197774701 | 98.88% | no |
| runs_all_32 | 13,746,141 | 16,785,370 | zstd3 | 418641690 | 88.11% | no |
| runs_all_64 | 6,070,200 | 7,403,625 | zstd3 | 632536987 | 49.60% | no |
| runs_all_128 | 2,363,421 | 3,523,784 | zstd3 | 774632723 | 27.22% | no |
| runs_all_256 | 323,913 | 436,711 | zstd3 | 902640231 | 10.16% | no |
| runs_all_512 | 141,059 | 228,278 | zstd3 | 921422386 | 5.48% | yes |
| runs_all_1024 | 103,527 | 185,113 | zstd3 | 929526379 | 4.22% | yes |
| runs_longest_0 | 20,432 | 16,859 | zstd3 | 1104673460 | 2.76% | yes |
| runs_longest_1 | 40,789 | 39,589 | zstd3 | 1001462037 | 2.80% | yes |
| runs_longest_2 | 83,587 | 151,504 | zstd3 | 942032687 | 2.83% | yes |
| runs_longest_4 | 221,088 | 289,989 | zstd3 | 929956476 | 5.53% | no |
| runs_longest_8 | 442,140 | 548,600 | zstd3 | 911429048 | 9.02% | no |
| runs_longest_16 | 837,434 | 1,069,825 | zstd3 | 881976445 | 13.98% | no |
| bloom_16_k1_seed0 | 20,821 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k1_seed0 | 24,289 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k1_seed0 | 84,369 | 1,074,881 | zstd3 | — | 3.38% | yes |
| bloom_128_k1_seed0 | 805,923 | 2,149,697 | zstd3 | — | 15.12% | no |
| bloom_256_k1_seed0 | 3,232,404 | 4,299,329 | zstd3 | — | 52.70% | no |
| bloom_512_k1_seed0 | 8,431,996 | 8,598,593 | zstd3 | — | 95.10% | no |
| bloom_1024_k1_seed0 | 15,337,667 | 17,197,121 | zstd3 | — | 98.61% | no |
| bloom_2048_k1_seed0 | 23,485,039 | 34,394,177 | zstd3 | — | 99.75% | no |
| bloom_16_k2_seed0 | 20,446 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k2_seed0 | 20,798 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k2_seed0 | 27,674 | 1,074,881 | zstd3 | — | 2.78% | yes |
| bloom_128_k2_seed0 | 151,837 | 2,149,697 | zstd3 | — | 4.11% | yes |
| bloom_256_k2_seed0 | 1,588,615 | 4,299,329 | zstd3 | — | 31.98% | no |
| bloom_512_k2_seed0 | 6,443,859 | 8,598,593 | zstd3 | — | 91.12% | no |
| bloom_1024_k2_seed0 | 16,838,612 | 17,197,121 | zstd3 | — | 99.40% | no |
| bloom_2048_k2_seed0 | 30,660,769 | 34,394,177 | zstd3 | — | 99.88% | no |
| bloom_16_k1_seed1 | 20,672 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k1_seed1 | 24,210 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k1_seed1 | 84,172 | 1,074,881 | zstd3 | — | 4.69% | yes |
| bloom_128_k1_seed1 | 807,580 | 2,149,697 | zstd3 | — | 17.15% | no |
| bloom_256_k1_seed1 | 3,234,028 | 4,299,329 | zstd3 | — | 51.18% | no |
| bloom_512_k1_seed1 | 8,440,356 | 8,598,593 | zstd3 | — | 85.81% | no |
| bloom_1024_k1_seed1 | 15,339,021 | 17,197,121 | zstd3 | — | 97.84% | no |
| bloom_2048_k1_seed1 | 23,483,458 | 34,394,177 | zstd3 | — | 99.68% | no |
| bloom_16_k2_seed1 | 20,452 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k2_seed1 | 20,912 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k2_seed1 | 28,203 | 1,074,881 | zstd3 | — | 2.77% | yes |
| bloom_128_k2_seed1 | 152,239 | 2,149,697 | zstd3 | — | 6.13% | yes |
| bloom_256_k2_seed1 | 1,590,031 | 4,299,329 | zstd3 | — | 28.60% | no |
| bloom_512_k2_seed1 | 6,447,691 | 8,598,593 | zstd3 | — | 83.82% | no |
| bloom_1024_k2_seed1 | 16,839,830 | 17,197,121 | zstd3 | — | 98.05% | no |
| bloom_2048_k2_seed1 | 30,652,085 | 34,394,177 | zstd3 | — | 99.82% | no |
| bloom_16_k1_seed2 | 20,713 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k1_seed2 | 24,814 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k1_seed2 | 84,640 | 1,074,881 | zstd3 | — | 3.39% | yes |
| bloom_128_k1_seed2 | 808,918 | 2,149,697 | zstd3 | — | 24.72% | no |
| bloom_256_k1_seed2 | 3,237,168 | 4,299,329 | zstd3 | — | 63.30% | no |
| bloom_512_k1_seed2 | 8,442,131 | 8,598,593 | zstd3 | — | 89.75% | no |
| bloom_1024_k1_seed2 | 15,334,985 | 17,197,121 | zstd3 | — | 97.55% | no |
| bloom_2048_k1_seed2 | 23,482,778 | 34,394,177 | zstd3 | — | 99.51% | no |
| bloom_16_k2_seed2 | 20,447 | 268,769 | zstd3 | — | 2.76% | yes |
| bloom_32_k2_seed2 | 20,780 | 537,473 | zstd3 | — | 2.76% | yes |
| bloom_64_k2_seed2 | 27,272 | 1,074,881 | zstd3 | — | 2.76% | yes |
| bloom_128_k2_seed2 | 150,800 | 2,149,697 | zstd3 | — | 7.43% | yes |
| bloom_256_k2_seed2 | 1,587,399 | 4,299,329 | zstd3 | — | 36.55% | no |
| bloom_512_k2_seed2 | 6,446,376 | 8,598,593 | zstd3 | — | 83.73% | no |
| bloom_1024_k2_seed2 | 16,839,551 | 17,197,121 | zstd3 | — | 98.48% | no |
| bloom_2048_k2_seed2 | 30,657,105 | 34,394,177 | zstd3 | — | 99.84% | no |

## Bloom seed ranges

| B/block | k | Added B range | player skip range |
|---:|---:|---:|---:|
| 16 | 1 | 20,672–20,821 | 2.76–2.76% |
| 16 | 2 | 20,446–20,452 | 2.76–2.76% |
| 32 | 1 | 24,210–24,814 | 2.76–2.76% |
| 32 | 2 | 20,780–20,912 | 2.76–2.76% |
| 64 | 1 | 84,172–84,640 | 3.38–4.69% |
| 64 | 2 | 27,272–28,203 | 2.76–2.78% |
| 128 | 1 | 805,923–808,918 | 15.12–24.72% |
| 128 | 2 | 150,800–152,239 | 4.11–7.43% |
| 256 | 1 | 3,232,404–3,237,168 | 51.18–63.30% |
| 256 | 2 | 1,587,399–1,590,031 | 28.60–36.55% |
| 512 | 1 | 8,431,996–8,442,131 | 85.81–95.10% |
| 512 | 2 | 6,443,859–6,447,691 | 83.73–91.12% |
| 1024 | 1 | 15,334,985–15,339,021 | 97.55–98.61% |
| 1024 | 2 | 16,838,612–16,839,830 | 98.05–99.40% |
| 2048 | 1 | 23,482,778–23,485,039 | 99.51–99.75% |
| 2048 | 2 | 30,652,085–30,660,769 | 99.82–99.88% |

## All queries for key candidates

| Construction | *a* | *ab* | *traa* | *X | *x | *player* | *ms* |
|---|---:|---:|---:|---:|---:|---:|---:|
| exact_bitset | 2.49% | 2.51% | 99.16% | 100.00% | 2.62% | 99.92% | 2.71% |
| runs_all_512 | 2.49% | 2.51% | 5.10% | 100.00% | 2.62% | 5.48% | 2.71% |
| bloom_128_k2_seed2 | 2.49% | 2.51% | 3.23% | 100.00% | 2.62% | 7.43% | 2.71% |
| bloom_512_k2_seed0 | 2.49% | 2.51% | 61.15% | 100.00% | 2.62% | 91.12% | 2.71% |

## Safety and reproduction

Fresh full-term truth was scanned after construction/selection and matched the existing per-block and aggregate reference counts. Prefixes were checked against first/last terms, all remainders against the byte sidecar, and term ordering/offsets against extraction. Every encoded layout roundtripped. All run masks were checked as supersets of exact masks. Every large Bloom insertion was checked through direct hashing and every smaller filter reconstructed independently. Every evaluated method retained every exact-positive block and every block permitted by the exact trigram baseline.
The generic callback receives only the viable continuation when a literal crosses a prefix. Prefix-contained matches and zero/one/two-byte continuations remain conservative. Short queries continue to use remainder byte masks. Tests cover exhaustive small-alphabet prefix/crossing/suffix cases, unknown bytes, random Bloom insertion, serialized formats, gap-fill supersets, and malformed/truncated framing.
Run `python3 -m unittest discover -s dictionary_analysis -p "test_generic_trigrams.py" -v` and `python3 dictionary_analysis/generic_trigrams.py --directory KH4Y_01M5MTJQ3SVHDTK2W3XCB5/message_analysis`. Use `--stage evaluate` to re-evaluate retained serialized artifacts without rebuilding. See `build.json`, `generic_trigrams.json`, `run.log`, `tests.log`, and `regression_tests.log` in this directory.
