# SSTable

The `tantivy-sstable` crate stores sorted byte-string keys and typed values.

It was designed for use in `quickwit`:
- as an alternative to the default tantivy fst dictionary.
- as a way to store the column index for dynamic fast fields.

The benefit compared to the fst crate is locality.
Searching for a key with the fst crate requires downloading the entire dictionary.

Once the sstable index is downloaded, running a `get` in the sstable
crate only requires a single fetch.

The block index and default block size were chosen for Quickwit.
Point lookups can be slow with these settings.

# Sorted strings?

SSTable stands for Sorted String Table.
Keys must be inserted in sorted order.

That sorted order supports:
- point lookups and streaming ranges of keys
- incremental encoding using shared key prefixes
- automaton intersection that skips shared key prefixes

# On disk format

Overview of the SSTable format. Unless noted otherwise, numbers are little-endian.

### SSTable
```
+-------+-------+-----+--------+
| Block | Block | ... | Footer |
+-------+-------+-----+--------+
|----( # of blocks)---|
```
- Block(`SSTBlock`): a sequence of independent blocks, terminated by a zero block length.
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
- Values: an application-defined sequence of values whose decoder determines its length.

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
KeepAdd uses one byte when both lengths fit in four bits, and variable-length integers otherwise.

When keep < 16 and add < 16
```
+-----+------+
| Add | Keep |
+-----+------+
```
- Add(u4): number of bytes to push
- Keep(u4): number of prefix bytes to retain from the previous key

Otherwise:
```
+------+------+-----+
| 0x01 | Keep | Add |
+------+------+-----+
```
- Add(VInt): number of bytes to push
- Keep(VInt): number of prefix bytes to retain from the previous key


Keys are unique, so Add is positive after the first key. The first key has Keep equal to zero.
These constraints distinguish the one-byte representation from the 0x01 marker.

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

Convert BlockData at index Index and its BlockAddrBlockMetadata to a block address as follows:
range\_prediction := RangeStart + Index * RangeSlope;
range\_derivation := RangeStartDelta - (1 << (RangeStartNBits-1));
range\_start := range\_prediction + range\_derivation

The same computation applies to ordinals.

`range_derivation` can be negative. `RangeStartDelta` shifts it into an unsigned range.


## SingleBlockSStable

The index has a fixed cost of around 70 bytes, which can be significant for a table with few keys.
Single-block SSTables omit Fst and BlockAddrStore to save that space.
Lookups use an implicit block with first ordinal 0, range start 0, and range end IndexOffset.
