# Revised budget: at most +100% of existing index metadata

Existing index: **195,640 bytes**. Additional budget: **195,640 bytes**, or **11.65 bytes/block**. This treats 100% as an increase, so the final total index budget is 391,280 bytes.

## Repeated byte masks make exact summaries fit

Across 16,794 original blocks, only **107 distinct whole-key byte masks** occur. Store each 256-bit mask once and reference it with a one-byte ID per block.

- Shared full-mask table: 3,424 bytes.
- Block IDs: 16,794 bytes.
- Header: 13 bytes.
- **Actually serialized size: 20,231 bytes.**

The ending-byte summary has 866 unique masks and uses two-byte IDs. It costs 61,313 bytes including its header.

| Summary | Added bytes | Increase over current index metadata |
|---|---:|---:|
| Whole-key byte masks | 20,231 | +10.34% |
| Ending-byte masks | 61,313 | +31.34% |
| Both tables | 81,544 | +41.68% |
| Both + optional min/max lengths | 115,132 | +58.85% |

The optional length fields are sized at two bytes/block for this dataset. They were not serialized in these mask files. Bounds plus min/max length skip four blocks for `*player*` and none for the other six queries. Whole-byte masks already reject those four blocks, so lengths add no selectivity to these masks and need not consume this budget initially.

These are lossless encodings of the original exact byte masks, not smaller Bloom filters. Every block's mask was decoded from the generated file and compared with the original. Candidate decisions reproduce the previous experiment exactly:

| Query | Data-byte skipping with whole + ending masks |
|---|---:|
| `*a*` | 2.35% |
| `*ab*` | 2.37% |
| `*traa*` | 2.60% |
| `*X` | 100.00% |
| `*x` | 5.30% |
| `*player*` | 2.67% |
| `*ms*` | 2.61% |

Queries remain case-sensitive raw byte wildcards. `*player*` and `*ms*` are appended substrings. Only `*X` and `*x` use ending-byte tests. If production normalizes uppercase X, use the lowercase result. A dictionary-wide alphabet also rejects uppercase X on this dataset.

## Format and reproduction

The two files are standalone experimental sidecars, not a new production SSTable version:

```text
4-byte magic BM01
u32 LE block count
u32 LE distinct mask count
u8 ID width in bytes
32-byte masks, sorted by integer value, little-endian bit order
one ID per original block, little-endian
```

Run `python3 dictionary_analysis/budget_masks.py` from the repository root. Outputs:

- `byte_masks.interned`
- `ending_byte_masks.interned`
- `budget_masks.json`
- This report, `budget.md`

A further encoding using the measured dictionary-wide alphabet of 41 bytes and bit-packed IDs could be smaller; estimates are in the JSON, but that encoding was not implemented. The simple full-mask tables already meet the budget without that complexity.

## Implication

The tested fixed-size trigram filters exceed the budget. Exact byte and ending-byte masks fit comfortably once shared across blocks. Their selectivity remains modest for the lowercase substring queries. The adjacent-pair follow-up is in [budget_pairs.md](budget_pairs.md): the tested exact compressed pair layouts exceed the budget. Sharing pair unions across groups fits at a selectivity cost. Selectively tracking a limited set of pairs is another option to evaluate.
