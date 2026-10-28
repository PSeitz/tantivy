# Inline postings

With `quickwit`, `IndexSettings.max_inline_docs` selects the maximum documents in a postings list stored in its term dictionary block. It accepts 0–32 and defaults to 0, which keeps all postings external. Longer lists, including their final blocks, retain the external postings/positions format. Documents and frequencies use the existing 128-document postings buffer. Positions are buffered until the list closes or exceeds the configured limit.

```rust
let settings = tantivy::IndexSettings {
    max_inline_docs: 32,
    ..Default::default()
};
```

Use these settings when building the index. Newly written segments and merge outputs use the destination index's setting. Existing dictionaries carry their own limit. Direct dictionary builders can use `TermDictionaryBuilder::create_with_max_inline_docs(writer, max_inline_docs)`.

## Compatibility

New Quickwit dictionaries use dictionary type 4. Their trailer is `[max_inline_docs: u8, dictionary type: u32 LE]`, following the SSTable footer. The limit is stored **once per dictionary**, including empty and external-only dictionaries. Opening reads both trailer fields together and validates the limit. Each stream copies the limit into a fresh value reader, including async and disjoint-block streams.

This reader also accepts type-2 external dictionaries and type-3 fixed-eight dictionaries. Type 3 retains its implicit limit of 8 and four-bit layout. New inline-postings blocks use codec version 2 with document-ID deltas. The reader also accepts version 1 with absolute IDs. Readers supporting only version 1 reject version 2 blocks. Dictionary type 4 identifies the dictionary metadata and variable-width layout. Older readers reject type 4. Default-feature FST dictionaries keep their existing format and external postings.

SSTables containing separately encoded block parts use SSTable version 4, with the unchanged version-3 block index. External-only SSTables continue to use version 3. The SSTable reader accepts versions 2, 3, and 4.

`TermInfo.inline_postings` owns `LoadedPostings` through an `Arc`. `LoadedPostings` holds arrays of document IDs, term frequencies, and position deltas. `LoadedPostingsReader` reads these shared arrays. `SegmentPostings` reads postings through block decoders. Its external ranges are empty at the current cumulative offsets. Callers constructing `TermInfo` literals must supply `inline_postings: None` for external terms. Fixed-size `BinarySerializable` serialization rejects inline postings.

## Block framing

Existing raw/zstd blocks retain compression flags 0/1. Flag 2 stores three parts:

1. Term metadata, independently compressed with zstd level 3 when smaller.
2. Inline-postings payload, already compressed by byte lane.
3. Delta-encoded keys, independently compressed with zstd level 3 when smaller.

Each part has `[decoded length: u32 LE, stored length: u32 LE, codec: u8, stored bytes]`. Codec 0 is raw and codec 1 is zstd. The payload part is raw to avoid nested compression. The outer u32 block length includes the flag and all three parts.

## Term metadata

Blocks with inline postings start with these VInts:

```text
0, inline-postings version (2), term count,
initial external postings offset, initial external positions offset
```

For each term:

- External: document count (>0), postings byte length, positions byte length.
- Inline: zero marker. Both external offsets stay unchanged.

After all term metadata, inline terms have packed document counts and positions flags in marker order, least-significant bits first. The reader derives `count_and_positions_bits = bit_width(max_inline_docs - 1) + 1` from dictionary metadata. The highest bit indicates positions are present. The lower bits contain document count minus one. Packed values can cross byte boundaries. The writer zeroes unused high bits in the final byte. Readers ignore them.

A limit of 1 uses one bit (only the positions flag), 8 uses four bits, 9–16 use five bits, and 17–32 use six bits. Counts exceeding the dictionary limit are rejected, including unused values when the limit is between powers of two. A limit of 0 requires external postings for every term.

External-only blocks retain the legacy metadata encoding.

## Inline-postings payload

Three u32 streams are concatenated in inline-term/document order:

1. Document-ID deltas, reset per list. The first value is the absolute first ID.
2. Frequency codes.
3. Position deltas that remain after tagging.

Each stream is transposed into four byte lanes, low byte first. Each lane is independently zstd-compressed at level 3 and preceded by its compressed length as a VInt. Empty streams have no lanes. Document counts determine the first two stream lengths. Untagged frequencies on position-bearing terms determine the third.

Frequency codes:

- Zero: this term has implicit frequencies, as for JSON numeric values. All frequency codes for the term must be zero and its positions array must be empty. Readers return a frequency of one while retaining the absence of stored frequencies for merging.
- High bit clear, nonzero: term frequency. Documents with positions contribute that many deltas to the positions stream.
- High bit set: frequency one. The low 31 bits hold its sole position. This document contributes zero values to the positions stream.

Frequencies and tagged positions must fit in 31 bits. The writer rejects larger values. Readers check framing lengths, tags, and encoding limits. Callers supply arrays whose lengths match document and frequency counts, and contiguous external ranges.

Document IDs are reconstructed with a prefix sum reset per list. Callers supply increasing document IDs within the segment and position deltas whose per-document sums fit in u32. Version-1 blocks store absolute IDs and skip reconstruction. Position deltas reset per document, as in the existing serializer. Multi-occurrence positions retain their existing deltas. Size measurements and an absolute-ID comparison are in the [inline postings results](../../../experiments/inline-postings/README.md#storage-by-inline-cutoff).

## Reads and warmup

Postings reads and resets initialize the block decoders from inline arrays. Position readers retain the shared decoded positions. Inline terms use the dictionary's postings and positions. Automaton warmup inserts their document IDs directly into its bitset, and inline-only matches still return true. Full warmup includes the dictionary.

Automaton dictionary reads merge gaps of at most **512 KiB (524,288 bytes)**. External postings start with the same streaming merge. The planner then fills the cheapest remaining gaps to minimize requests, with an additional byte allowance of `max(2 MiB, initial_read_bytes / 4)` per field, segment, and call. These later merges have no individual gap ceiling. Request counts never exceed the initial plan. Empty external ranges are omitted, and touching ranges merge without adding bytes. Existing range warmup continues to read the contiguous span selected by its bounds. Measurements and tradeoffs are in the [read-plan results](../../../experiments/inline-postings/README.md#external-reads-and-request-budget).

The value reader decodes all inline streams when loading a dictionary block, including frequencies and positions. Independent compression avoids nested compression. Stream I/O and positions decoding operate on the whole block.
