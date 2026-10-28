# Naming review

Scope: inline postings, their readers and serializers, SSTable block encoding,
and the postings read-plan experiment. Descriptions were written before source
edits. The tables preserve the descriptions used to choose names.

## Postings

| Description | Name |
| --- | --- |
| Loaded postings arrays holding document IDs, term frequencies, and position deltas. | `LoadedPostings` |
| Reader over shared loaded postings. | `LoadedPostingsReader` |
| Postings stored inline in the term dictionary. | `inline_postings` |
| Maximum documents per postings list stored inline in dictionary. | `max_inline_docs` |
| Decode the remaining documents, frequencies, and positions from segment postings. | `decode_remaining` |
| Position deltas buffered until the term's document count determines where its postings are stored. | `buffered_position_deltas` |
| Document IDs and frequencies buffered for a compression block. | `PostingsBlock` |
| Index of the document being read. | `doc_index` |
| Shared loaded postings read by this reader. | `postings` |
| Position reconstructed from successive deltas. | `position` |
| Length of the output before appending position deltas. | `output_len` |
| A serializer for external postings used as the test comparison. | `external_serializer` |

## Dictionary encoding

| Description | Name |
| --- | --- |
| Number of inline terms in a block. | `inline_term_count` |
| Packed document count minus one and a positions flag. | `count_and_positions` |
| Byte slice storing packed counts and positions flags. | `counts_and_positions` |
| Byte length of the packed counts and positions flags. | `counts_and_positions_len` |
| Iterator returning document counts and positions flags for inline terms. | `doc_counts_and_positions` |
| Document deltas written to the block. | `doc_deltas` |
| Document codes: absolute IDs in version 1, deltas in version 2. | `doc_codes` |
| Frequency codes: frequencies or tagged positions. | `frequency_codes` |
| Previous document ID used to encode or decode the next ID. | `previous_doc_id` |
| Compressed bytes for a byte lane. | `compressed_lane` |
| Position deltas stored separately from the frequency codes. | `separate_position_deltas` |
| Offset of the dictionary trailer. | `trailer_offset` |
| Dictionary type read from a legacy trailer. | `legacy_dictionary_type` |

## SSTable blocks

| Description | Name |
| --- | --- |
| Encoding flag selecting raw bytes, whole-block zstd, or separately encoded parts. | `encoding_flag` |
| The block was serialized with keys. | `block_serialized_with_keys` |
| Expected output length after appending a decoded part. | `expected_output_len` |
| Prepends the block length to a block body. | `prepend_block_length` |
| Decodes a block body through the block reader. | `decode_block_body` |
| Stored length of a part. | `stored_len` |
| Index of the part within metadata, payload, and keys. | `part_index` |

## Read plans

| Description | Name |
| --- | --- |
| Merge and send postings ranges. | `merge_and_send_postings_ranges` |
| Encoded postings ranges and document counts needed to decode them. | `encoded_postings_ranges_and_doc_counts` |
| End of the previous postings range. | `previous_postings_end` |
| Extra bytes remaining for merging gaps. | `remaining_bytes` |
| Last gap selected for merging, ordered by byte length and index. | `last_gap_to_merge` |
| Postings lists matching a query, with their inline count and external ranges. | `MatchingPostings` |
| External postings ranges for matching terms. | `external_ranges` |
| Read ranges after merging gaps within the byte budget. | `byte_budget_ranges` |
| Read ranges after merging gaps up to the fixed gap limit. | `fixed_gap_ranges` |
| Bytes in matching external postings lists, excluding gaps. | `postings_bytes` |
| Validates and summarizes read ranges. | `validate_and_summarize_reads` |
| Reads the field dictionary after validating the footer and CRC. | `read_field_dictionary` |
| Index of the inline cutoff being evaluated. | `cutoff_index` |
| Index of the query pattern being evaluated. | `pattern_index` |
| The term matches this query pattern. | `matches_pattern` |

## Prose and compatibility

- Inline describes storage location. Document counts describe list size.
- `LoadedPostings` holds document ID, term frequency, and position delta arrays.
  `LoadedPostingsReader` reads those shared arrays. `SegmentPostings` reads
  postings through block decoders.
- Position deltas and frequency codes remain distinct from reconstructed positions
  and frequencies.
- Stored and decoded lengths distinguish the measurements in block headers.
- Whole-block compression, full scans, cold reads, and external ranges retain
  qualifiers that distinguish their behavior or measurements.
- Read-plan bytes distinguish postings from gaps included in requests.
- Rust APIs use the names above. The experiment retains its JSON keys and policy name.
- Renaming preserves encoded formats and algorithms.
