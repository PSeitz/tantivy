# FST term dictionary: singleton metadata groups

Disabled dictionaries retain version 1 and its original bytes. Enabled dictionaries
use version 2. Readers accept versions 1 and 2. The outer dictionary type remains
FST (1).

## Eligibility

With `max_inline_docs > 0`, the FST writer inlines only a term with exactly one
document, explicit frequency one, and one position. Both frequencies are implicit
in the inline encoding. Repeated occurrences and fields without frequencies and
positions retain external postings. Quickwit/SSTable eligibility is unchanged.

## FST and footer

FST outputs remain consecutive sorted term ordinals, including inline terms.
Point lookups, ordinal lookups, streams, ranges, reverse streams, and merging use
the original FST paths. There is no ordinal scan or second index.

The FST is followed by `TermInfoStore`, then its byte length (`u64`) and the
format version (`u32`). The outer dictionary appends its type (`u32`).

## TermInfoStore

The store begins with two `u64`s: block-header byte length and total term count.
Fixed-size block headers follow, then packed block payloads. Each block covers
up to 256 consecutive term ordinals.

Version 1 uses its original 39-byte header and external metadata encoding.
Version 2 extends every block header by 42 bytes:

- 32-byte inline bitmap, four little-endian `u64`s. Bit `slot` is set for an inline term.
- Document-ID delta bit width (`u8`, 0–31).
- Position delta bit width (`u8`, 0–32).
- Minimum inline document ID (`u32`, little-endian).
- Minimum inline position (`u32`, little-endian).

Unused bitmap bits in a partial final block must be zero. A bit selects the
record group. Popcount of preceding bits gives the inline record index. The
external record index is `slot - preceding_inline_count`. This costs at most
four word popcounts and requires no payload scanning.

### External group

External terms retain their relative order. They use the existing encoding:

- The 39-byte block header contains the payload offset, a complete reference
  `TermInfo` for the first external term, and three external bit widths.
- Remaining external records contain postings-start and positions-start offsets
  relative to that reference, followed by document frequency.
- End offsets come from the next external record's start. The final external
  term's end offsets are appended explicitly.
- The group is padded to a byte boundary.

Inline terms consume no external postings or positions bytes, so consecutive
external ranges remain adjacent. An all-inline block has a default zero
reference and zero external widths, hence no external payload bytes.

### Inline group

Immediately after the external group's byte-aligned data, inline records store
`document ID - min_doc` and `position - min_position`, interleaved and bitpacked
with independent block widths. Each width depends on the corresponding inline
maximum minus minimum. Decoding adds each minimum back, checking for overflow
and invalid document IDs. External records do not affect these minima or widths.
The group is padded to a byte boundary. Empty groups use zero minima and widths.
A group of identical pairs needs no payload bytes, including a singleton group.

The inline group's location is computed from the external record count and
external bit widths. Neither per-term offsets nor payload-length frames are
stored. Document frequency and term frequency are implicitly one.

Decoded inline values currently construct `Arc<LoadedPostings>` and boxed
singleton arrays, retaining the existing postings reader interface.
