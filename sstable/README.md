# SSTable

The `tantivy-sstable` crate is yet another sstable crate.

It has been designed to be used in `quickwit`:
- as an alternative to the default tantivy fst dictionary.
- as a way to store the column index for dynamic fast fields.

The benefit compared to the fst crate is locality.
Searching a key in the fst crate requires downloading the entire dictionary.

Once the sstable index is downloaded, running a `get` in the sstable
crate only requires a single fetch.

Right now, the block index and the default block size have been thought
for quickwit, and the performance of a get is very bad.

# Sorted strings?

SSTable stands for Sorted String Table.
Strings have to be insert in sorted order.

That sorted order is used in different ways:
- it makes gets and streaming ranges of keys
possible.
- it allows incremental encoding of the keys
- the front compression is leveraged to optimize
the intersection with an automaton

# On disk format

Overview of the SSTable format. Unless noted otherwise, numbers are little-endian.

### SSTable
```
+-------+-------+-----+--------+
| Block | Block | ... | Footer |
+-------+-------+-----+--------+
|----( # of blocks)---|
```
- Block(`SSTBlock`): list of independent block, terminated by a single empty block.
- Footer(`SSTFooter`)

### SSTBlock
```
+----------+----------+--------+-------+-------+-----+
| BlockLen | Compress | Values | Delta | Delta | ... |
+----------+----------+--------+-------+-------+-----+
                      |        |----( # of deltas)---|
                      |------(maybe compressed)------|
```
- BlockLen(u32): length of the block, including the compress byte.
- Compress(u8): indicate whether block is compressed. 0 if not compressed, 1 if compressed.
- Values: an application defined format storing a sequence of value, capable of determining it own length
- Delta

### Delta
```
+---------+--------+
| KeepAdd | Suffix |
+---------+--------+
```
- KeepAdd
- Suffix: KeepAdd.add bytes of key suffix

### KeepAdd
KeepAdd can be represented in two different representation, a very compact 1byte one which is enough for most usage, and a longer variable-len one when required

When keep < 16 and add < 16
```
+-----+------+
| Add | Keep |
+-----+------+
```
- Add(u4): number of bytes to push
- Keep(u4): number of bytes to pop

Otherwise:
```
+------+------+-----+
| 0x01 | Keep | Add |
+------+------+-----+
```
- Add(VInt): number of bytes to push
- Keep(VInt): number of bytes to pop


Note: as the SSTable does not support redundant keys, there is no ambiguity between both representation. Add is always guaranteed to be non-zero, except for the very first key of an SSTable, where Keep is guaranteed to be zero.

### SSTFooter
```
+-----+----------------+-------------+-------------+---------+---------+
| Fst | BlockAddrStore | StoreOffset | IndexOffset | NumTerm | Version |
+-----+----------------+-------------+-------------+---------+---------+
```
- Fst(Fst): finite state transducer mapping keys to a block number
- BlockAddrStore(BlockAddrStore): store mapping a block number to its BlockAddr
- StoreOffset(u64): Offset to start of the BlockAddrStore. If zero, see the SingleBlockSStable section
- IndexOffset(u64): Offset to the start of the SSTFooter
- NumTerm(u64): number of terms in the sstable
- Version(u32): 3 for ordinary tables, 4 when an embedded pruning section is present

### Fst

Fst is in the format of tantivy\_fst

### BlockAddrStore

+---------+-----------+-----------+-----+-----------+-----------+-----+
| MetaLen | BlockMeta | BlockMeta | ... | BlockData | BlockData | ... |
+---------+-----------+-----------+-----+-----------+-----------+-----+
          |---------(N blocks)----------|---------(N blocks)----------|

- MetaLen(u64): length of the BlockMeta section
- BlockMeta(BlockAddrBlockMetadata): metadata to seek through BlockData
- BlockData(CompactedBlockAddr): bitpacked per block metadata

### BlockAddrBlockMetadata

+--------+------------+--------------+------------+--------------+-------------------+-----------------+----------+
| Offset | RangeStart | FirstOrdinal | RangeSlope | OrdinalSlope | FirstOrdinalNBits | RangeStartNBits | BlockLen |
+--------+------------+--------------+------------+--------------+-------------------+-----------------+----------+

- Offset(u64): offset of the corresponding BlockData in the datastream
- RangeStart(u64): the start position of the first block
- FirstOrdinal(u64): the first ordinal of the first block
- RangeSlope(u32): slope predicted for start range evolution (see computation in BlockData)
- OrdinalSlope(u64): slope predicted for first ordinal evolution (see computation in BlockData)
- FirstOrdinalNBits(u8): number of bits per ordinal in datastream (see computation in BlockData)
- RangeStartNBits(u8): number of bits per range start in datastream (see computation in BlockData)

### BlockData

+-----------------+-------------------+---------------+
| RangeStartDelta | FirstOrdinalDelta | FinalRangeEnd |
+-----------------+-------------------+---------------+
|------(BlockLen repetitions)---------|

- RangeStartDelta(var): RangeStartNBits *bits* of little endian number. See below for decoding
- FirstOrdinalDelta(var): FirstOrdinalNBits *bits* of little endian number. See below for decoding
- FinalRangeEnd(var): RangeStartNBits *bits* of integer. See below for decoding

converting a BlockData of index Index and a BlockAddrBlockMetadata to an actual block address is done as follow:
range\_prediction := RangeStart + Index * RangeSlop;
range\_derivation := RangeStartDelta - (1 << (RangeStartNBits-1));
range\_start := range\_prediction + range\_derivation

The same computation can be done for ordinal.

Note that `range_derivation` can take negative value. `RangeStartDelta` is just its translation to a positive range.


## Optional literal block pruning

Pruning is disabled by default. Enable it before insertion, choosing a serialized
byte budget, a maximum block-presence fraction, and a maximum block count per trigram:

```rust
use common::OwnedBytes;
use tantivy_sstable::{Dictionary, MonotonicU64SSTable, PruningOptions};

let mut writer = Dictionary::<MonotonicU64SSTable>::builder(Vec::new())?;
writer.enable_pruning(
    PruningOptions {
        budget_bytes: 200_000,
        max_block_presence: 0.2,
        max_blocks_per_trigram: 1_024,
    },
    |value| *value,
)?;
writer.insert(b"player", &42)?;
let mut dictionary = Dictionary::<MonotonicU64SSTable>::from_bytes(
    OwnedBytes::new(writer.finish()?),
)?;
dictionary.load_pruning()?;
let mut stream = dictionary
    .search(tantivy_fst::Regex::new(".*player.*").unwrap())
    .prune_with_substring(b"player")
    .into_stream()?;
# Ok::<(), std::io::Error>(())
```

Choose these limits for the dictionary and queries being indexed. With Tantivy's
`quickwit` feature, `TermDictionaryBuilder::enable_pruning` and
`TermDictionary::load_pruning` forward these operations. A term-info callback can
use `|info| u64::from(info.doc_freq)`.

The callback supplies a score per inserted value. Each distinct trigram in the
**full term** receives that score once. Scores are summed with checked `u64`
arithmetic. They rank trigrams and can exceed the document count.

When a block is flushed, the builder removes its longest common prefix from each
term and collects the block ID for each byte and trigram in the remainders. Once a
trigram occurs in more than `max_blocks_per_trigram` blocks, the builder frees its
block-ID vector and stores `None` to stop collection for that trigram. The limit is
an absolute count, independent of the final block count. Zero skips collection for
all trigrams; `usize::MAX` disables the collection cap. Skipped trigrams remain
unknown during queries, preserving correctness while potentially reducing pruning
effectiveness. Collection continues for every byte. At finish, trigrams above the
configured block-presence fraction are excluded. The remaining trigrams are
considered by descending score, then ascending numeric key. Each encoded block-ID
list is included if it fits the budget, otherwise selection continues with the
next trigram.

`prune_with_substring` is a **required-substring hint**. Every term accepted by the
automaton must contain those case-sensitive raw bytes. The caller supplies this
guarantee. An incorrect hint can exclude valid results. The automaton still checks
exact matches, and an absent or unloaded section leaves search behavior unchanged.
Prefix-contained matches and prefix-crossing continuations are handled
conservatively. Missing trigram block-ID lists mean unknown. Trigram lookups are
prepared per query, and blocks are pruned before read coalescing. Coalescing can still read across
rejected gaps. Always-matching automata retain the existing limited range-read path.

### Version 4 layout and budget

Version 4 keeps the data blocks and ordinary v3 index encoding unchanged:

```text
Blocks | EmptyBlock | Fst | BlockAddrStore | StoreOffset
       | PruningSection | PruningOffset | IndexOffset | NumTerm | Version=4
```

`PruningOffset` is an absolute `u64` offset. The final 20 bytes retain their existing
field positions. Ordinary open reads the extra offset and the ordinary index,
without reading pruning bytes. `load_pruning` reads and validates the section once.
Clones made after loading share it. The reader also accepts versions 2 and 3.
Older readers cannot open version 4 files.

The `BP01` section stores each block's longest common prefix, block-ID lists for
all 256 bytes, and selected trigram block-ID lists. Each list chooses the smallest
of fixed-width IDs, delta varints, runs, and a bitmap, using IDs of blocks where the
byte or trigram is present or absent. Lists are independently addressable. The
encodings are implemented in this crate. The section format is documented in
`src/pruning.rs` and `src/pruning/codec.rs`.

The budget charges the entire section and the additional eight-byte footer field.
If the prefixes and byte block-ID lists do not fit, the writer emits the ordinary
v3 format. Counts, lengths, and section offsets are limited to `u32`.

**The serialized budget applies to file size.** During construction, the builder
keeps trigram scores, retained block-ID vectors, map entries for discarded trigrams,
block prefixes, and current-block keys. The collection cap limits each trigram
vector's length. Total memory also includes spare vector capacity and these other
allocations. Prefixes are stored as bytes. Encoding a block-ID list also constructs
the IDs of blocks where its byte or trigram is absent. These allocations can make
construction expensive for large dictionaries.

## SingleBlockSStable

The format used for the index is meant to be compact, however it has a constant cost of around 70
bytes, which isn't negligible for a table containing very few keys.
To limit the impact of that constant cost, single block sstable omit the Fst and BlockAddrStore from
their index. Instead a block with first ordinal of 0, range start of 0 and range end of IndexOffset
is implicitly used for every operations.
