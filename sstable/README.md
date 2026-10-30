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
+----------+--------------+------+
| BlockLen | EncodingFlag | Body |
+----------+--------------+------+
```
- BlockLen(u32): length of EncodingFlag and Body. Zero marks the end of the blocks.
- EncodingFlag(u8): 0 for raw bytes, 1 for whole-block zstd, 2 for separately encoded parts.
- Body: for flags 0 and 1, a sequence of Values followed by key Deltas, stored raw or compressed with zstd.
- Values: an application defined format storing a sequence of value, capable of determining it own length

Flag 2 stores three parts in order: metadata, payload, and keys. Each part has this layout:
```
+------------+-----------+-------+-------+
| DecodedLen | StoredLen | Codec | Bytes |
+------------+-----------+-------+-------+
```
- DecodedLen(u32): byte length after decoding.
- StoredLen(u32): byte length of Bytes.
- Codec(u8): 0 for raw bytes, 1 for zstd.
- Bytes: StoredLen bytes of encoded data.

Metadata and keys use zstd level 3 when the `zstd-compression` feature is enabled
and compression saves space. The payload uses codec 0 and is stored verbatim,
so callers can supply values that are already compressed.
Decoding concatenates metadata, payload, and keys for the value and key readers.

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
- Version(u32): 4 when the value writer serializes a block with keys, otherwise 3.
  Version 4 supports encoding flag 2 and uses the same index layout as version 3.

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


## SingleBlockSStable

The format used for the index is meant to be compact, however it has a constant cost of around 70
bytes, which isn't negligible for a table containing very few keys.
To limit the impact of that constant cost, single block sstable omit the Fst and BlockAddrStore from
their index. Instead a block with first ordinal of 0, range start of 0 and range end of IndexOffset
is implicitly used for every operations.
