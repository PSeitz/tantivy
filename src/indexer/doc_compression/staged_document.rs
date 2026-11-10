//! Indexes documents whose bytes and dictionaries share a memory arena.

use std::fmt;
use std::net::Ipv6Addr;

use stacker::MemoryArena;

use super::{ByteRange, CUSTOM_CODE, MESSAGE_TOKENS_CODE, read_vint};
use crate::schema::document::{
    BinaryValueDeserializer, ReferenceValue, ReferenceValueLeaf, ValueDeserializer, type_codes,
};
use crate::schema::{Document, Field, Value};
use crate::store::DOC_STORE_VERSION;
use crate::{DateTime, TantivyDocument};

enum NodeValue {
    Array,
    Object,
    Text(ByteRange),
    Facet(ByteRange),
    Bytes(ByteRange),
    Custom(ByteRange),
    Null,
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    Date(DateTime),
    IpAddr(Ipv6Addr),
}

struct ValueNode {
    value: NodeValue,
    key: Option<ByteRange>,
    subtree_end: usize,
}

/// Owns the segment's staging arena and the parsed values of the document being indexed.
pub(crate) struct StagedDocument {
    arena: MemoryArena,
    texts: Vec<ByteRange>,
    layouts: Vec<ByteRange>,
    nodes: Vec<ValueNode>,
    field_nodes: Vec<(Field, usize)>,
    message_buffer: Vec<u8>,
}

impl StagedDocument {
    pub(super) fn new(arena: MemoryArena, texts: Vec<ByteRange>, layouts: Vec<ByteRange>) -> Self {
        for range in &texts {
            std::str::from_utf8(range.bytes(&arena)).expect("staged UTF-8");
        }
        Self {
            arena,
            texts,
            layouts,
            nodes: Vec::new(),
            field_nodes: Vec::new(),
            message_buffer: Vec::new(),
        }
    }

    /// Parse a compressed document in the arena for indexing.
    pub(crate) fn load(&mut self, mut input: ByteRange) -> crate::Result<()> {
        self.clear();
        let mut layout = self.read_byte_range(&mut input, &self.layouts);
        while layout.len != 0 {
            let field = Field::from_field_id(self.read_vint(&mut layout) as u32);
            let node_index = self.read_value_node(&mut input, None)?;
            self.field_nodes.push((field, node_index));
        }
        debug_assert_eq!(input.len, 0);
        Ok(())
    }

    /// Clear parsed values and reuse their buffers. Arena bytes remain until finalization ends.
    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
        self.field_nodes.clear();
        self.message_buffer.clear();
    }

    pub(crate) fn to_tantivy_document(&self) -> TantivyDocument {
        let mut document = TantivyDocument::with_capacity(0);
        for (field, value) in self.iter_fields_and_values() {
            document.add_field_value(field, value);
        }
        document
    }

    fn bytes(&self, range: ByteRange) -> &[u8] {
        range.bytes(&self.arena)
    }

    fn validate_text(&self, range: ByteRange) -> ByteRange {
        std::str::from_utf8(self.bytes(range)).expect("staged UTF-8");
        range
    }

    fn text(&self, range: ByteRange) -> &str {
        // SAFETY: Only text, facet, and object-key ranges reach this accessor.
        // Dictionary text is validated in new(), inline text and keys during load().
        // Reconstructed messages concatenate validated text. These ranges remain whole
        // and refer to this append-only arena, so their validated bytes never change.
        unsafe { std::str::from_utf8_unchecked(self.bytes(range)) }
    }

    fn read_vint(&self, input: &mut ByteRange) -> usize {
        let mut bytes = self.bytes(*input);
        let value = read_vint(&mut bytes);
        input.take_prefix(input.len as usize - bytes.len());
        value
    }

    /// Read a byte range from an inline length or dictionary ID.
    fn read_byte_range(&self, input: &mut ByteRange, dictionary: &[ByteRange]) -> ByteRange {
        let header = self.read_vint(input);
        if header & 1 == 0 {
            input.take_prefix(header >> 1)
        } else {
            dictionary[header >> 1]
        }
    }

    fn read_text_range(&self, input: &mut ByteRange) -> ByteRange {
        let header = self.read_vint(input);
        if header & 1 == 0 {
            self.validate_text(input.take_prefix(header >> 1))
        } else {
            self.texts[header >> 1]
        }
    }

    fn read_value_node(
        &mut self,
        input: &mut ByteRange,
        key: Option<ByteRange>,
    ) -> crate::Result<usize> {
        let node_index = self.nodes.len();
        self.nodes.push(ValueNode {
            value: NodeValue::Null,
            key,
            subtree_end: 0,
        });
        let input_before_value = *input;
        let tag = self.bytes(input.take_prefix(1))[0];
        let value = match tag {
            type_codes::NULL_CODE => NodeValue::Null,
            type_codes::TEXT_CODE => NodeValue::Text(self.read_text_range(input)),
            type_codes::HIERARCHICAL_FACET_CODE => NodeValue::Facet(self.read_text_range(input)),
            type_codes::BYTES_CODE | CUSTOM_CODE => {
                let len = self.read_vint(input);
                let range = input.take_prefix(len);
                if tag == CUSTOM_CODE {
                    NodeValue::Custom(range)
                } else {
                    NodeValue::Bytes(range)
                }
            }
            type_codes::ARRAY_CODE => {
                let element_count = self.read_vint(input);
                for _ in 0..element_count {
                    self.read_value_node(input, None)?;
                }
                NodeValue::Array
            }
            type_codes::OBJECT_CODE => {
                let mut layout = self.read_byte_range(input, &self.layouts);
                while layout.len != 0 {
                    let key_header = self.read_vint(&mut layout);
                    let key = if key_header == 0 {
                        let key_len = self.read_vint(input);
                        input.take_prefix(key_len)
                    } else {
                        layout.take_prefix(key_header - 1)
                    };
                    self.read_value_node(input, Some(self.validate_text(key)))?;
                }
                NodeValue::Object
            }
            MESSAGE_TOKENS_CODE => {
                let token_count = self.read_vint(input);
                self.message_buffer.clear();
                for _ in 0..token_count {
                    let token_range = self.read_text_range(input);
                    self.message_buffer
                        .extend_from_slice(token_range.bytes(&self.arena));
                }
                let range = ByteRange::store(&mut self.arena, &self.message_buffer)?;
                NodeValue::Text(range)
            }
            _ => {
                let mut bytes = self.bytes(input_before_value);
                let decoder = BinaryValueDeserializer::from_reader(&mut bytes, DOC_STORE_VERSION)
                    .expect("staged value type");
                let value = match tag {
                    type_codes::U64_CODE => decoder.deserialize_u64().map(NodeValue::U64),
                    type_codes::I64_CODE => decoder.deserialize_i64().map(NodeValue::I64),
                    type_codes::F64_CODE => decoder.deserialize_f64().map(NodeValue::F64),
                    type_codes::BOOL_CODE => decoder.deserialize_bool().map(NodeValue::Bool),
                    type_codes::DATE_CODE => decoder.deserialize_datetime().map(NodeValue::Date),
                    type_codes::IP_CODE => decoder.deserialize_ip_address().map(NodeValue::IpAddr),
                    _ => unreachable!("unsupported staged value type"),
                }
                .expect("staged value");
                *input = input_before_value;
                input.take_prefix(input_before_value.len as usize - bytes.len());
                value
            }
        };
        self.nodes[node_index].value = value;
        self.nodes[node_index].subtree_end = self.nodes.len();
        Ok(node_index)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StagedValue<'a> {
    document: &'a StagedDocument,
    node_index: usize,
}

impl fmt::Debug for StagedValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StagedValue")
            .field("node_index", &self.node_index)
            .finish()
    }
}

impl<'a> Value<'a> for StagedValue<'a> {
    type ArrayIter = ChildValues<'a>;
    type ObjectIter = ObjectIter<'a>;

    fn as_value(&self) -> ReferenceValue<'a, Self> {
        let document = self.document;
        let node = &document.nodes[self.node_index];
        let children = ChildValues {
            document,
            next_child_index: self.node_index + 1,
            subtree_end: node.subtree_end,
        };
        let leaf = match &node.value {
            NodeValue::Array => return ReferenceValue::Array(children),
            NodeValue::Object => return ReferenceValue::Object(ObjectIter(children)),
            NodeValue::Text(range) => ReferenceValueLeaf::Str(document.text(*range)),
            NodeValue::Facet(range) => ReferenceValueLeaf::Facet(document.text(*range)),
            NodeValue::Bytes(range) => ReferenceValueLeaf::Bytes(document.bytes(*range)),
            NodeValue::Custom(range) => ReferenceValueLeaf::Custom(document.bytes(*range)),
            NodeValue::Null => ReferenceValueLeaf::Null,
            NodeValue::U64(value) => ReferenceValueLeaf::U64(*value),
            NodeValue::I64(value) => ReferenceValueLeaf::I64(*value),
            NodeValue::F64(value) => ReferenceValueLeaf::F64(*value),
            NodeValue::Bool(value) => ReferenceValueLeaf::Bool(*value),
            NodeValue::Date(value) => ReferenceValueLeaf::Date(*value),
            NodeValue::IpAddr(value) => ReferenceValueLeaf::IpAddr(*value),
        };
        ReferenceValue::Leaf(leaf)
    }
}

pub(crate) struct ChildValues<'a> {
    document: &'a StagedDocument,
    next_child_index: usize,
    subtree_end: usize,
}

impl<'a> Iterator for ChildValues<'a> {
    type Item = StagedValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_child_index == self.subtree_end {
            return None;
        }
        let node_index = self.next_child_index;
        self.next_child_index = self.document.nodes[node_index].subtree_end;
        Some(StagedValue {
            document: self.document,
            node_index,
        })
    }
}

pub(crate) struct ObjectIter<'a>(ChildValues<'a>);

impl<'a> Iterator for ObjectIter<'a> {
    type Item = (&'a str, StagedValue<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.0.next()?;
        let key = value.document.nodes[value.node_index]
            .key
            .expect("staged object key");
        Some((value.document.text(key), value))
    }
}

pub(crate) struct FieldValuesIter<'a> {
    document: &'a StagedDocument,
    field_nodes: std::slice::Iter<'a, (Field, usize)>,
}

impl<'a> Iterator for FieldValuesIter<'a> {
    type Item = (Field, StagedValue<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let &(field, node_index) = self.field_nodes.next()?;
        Some((
            field,
            StagedValue {
                document: self.document,
                node_index,
            },
        ))
    }
}

impl Document for StagedDocument {
    type Value<'a>
        = StagedValue<'a>
    where
        Self: 'a;
    type FieldsValuesIter<'a>
        = FieldValuesIter<'a>
    where
        Self: 'a;

    fn iter_fields_and_values(&self) -> Self::FieldsValuesIter<'_> {
        FieldValuesIter {
            document: self,
            field_nodes: self.field_nodes.iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompressionConfig, DocumentCompressor};
    use super::*;
    use crate::schema::{Facet, STORED, Schema};

    #[test]
    #[should_panic(expected = "staged UTF-8")]
    fn dictionary_text_is_checked_before_access() {
        let mut arena = MemoryArena::default();
        let invalid = ByteRange::store(&mut arena, &[0xff]).unwrap();
        StagedDocument::new(arena, vec![invalid], Vec::new());
    }

    #[test]
    fn inline_text_and_keys_are_checked_during_load() {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        // One-byte vints have their stop bit set.
        let vint = |value: u8| value | 0x80;
        // Each value contains an invalid UTF-8 byte. Lengths and markers follow the
        // staging format, including both object-key encodings and message tokens.
        for value in [
            vec![type_codes::TEXT_CODE, vint(2), 0xff],
            vec![type_codes::HIERARCHICAL_FACET_CODE, vint(2), 0xff],
            vec![
                type_codes::OBJECT_CODE,
                vint(2),
                vint(0),
                vint(1),
                0xff,
                type_codes::NULL_CODE,
            ],
            vec![
                type_codes::OBJECT_CODE,
                vint(4),
                vint(2),
                0xff,
                type_codes::NULL_CODE,
            ],
            vec![MESSAGE_TOKENS_CODE, vint(1), vint(2), 0xff],
        ] {
            let mut arena = MemoryArena::default();
            // Inline root layout containing field 0.
            let mut encoded = vec![vint(2), vint(0)];
            encoded.extend(value);
            let range = ByteRange::store(&mut arena, &encoded).unwrap();
            let mut document = StagedDocument::new(arena, Vec::new(), Vec::new());
            let panic = catch_unwind(AssertUnwindSafe(|| document.load(range))).unwrap_err();
            assert!(
                panic
                    .downcast_ref::<String>()
                    .unwrap()
                    .contains("staged UTF-8")
            );
        }
    }

    #[test]
    fn values_borrow_arena_and_loads_reuse_buffers() {
        let mut schema = Schema::builder();
        let json = schema.add_json_field("json", STORED);
        let message = schema.add_text_field("message", STORED);
        let payload = schema.add_bytes_field("payload", STORED);
        let facet = schema.add_facet_field("facet", STORED);
        let mut compressor = DocumentCompressor::new(
            CompressionConfig {
                message_fields: vec!["message".into()],
                ..Default::default()
            },
            &schema.build(),
        )
        .unwrap();
        let input = doc!(
            json => serde_json::json!({"鍵": ["shared 東京", "é", ""]}),
            message => "request for acme corp on web-12 at 東京支社 succeeded",
            payload => vec![0u8, 255, 42],
            facet => Facet::from_path(["東京", "🦆"])
        );
        let first = compressor.compress(&input).unwrap();
        let encoding_buffers = (
            compressor.layout_buffer.as_ptr(),
            compressor.document_buffer.as_ptr(),
        );
        let second = compressor.compress(&input).unwrap();
        assert_eq!(
            encoding_buffers,
            (
                compressor.layout_buffer.as_ptr(),
                compressor.document_buffer.as_ptr()
            )
        );
        assert_eq!(std::mem::size_of::<ByteRange>(), 8);
        let text_id = compressor
            .texts
            .get_id("shared 東京".as_bytes(), &compressor.arena)
            .unwrap();
        let text_address = compressor.texts.ranges(&compressor.arena)[text_id as usize]
            .bytes(&compressor.arena)
            .as_ptr();
        let key_address = compressor
            .layouts
            .ids_by_bytes
            .iter(&compressor.arena)
            .find_map(|(layout, _)| {
                layout
                    .windows(3)
                    .find(|bytes| *bytes == "鍵".as_bytes())
                    .map(|bytes| bytes.as_ptr())
            })
            .unwrap();
        let mut document = compressor.into_staged_document();
        document.load(first).unwrap();
        let NodeValue::Text(first_message) = document.nodes[document.field_nodes[1].1].value else {
            panic!("message text range");
        };
        // Grow the arena before reusing the view. Earlier addresses must keep their bytes.
        ByteRange::store(&mut document.arena, &vec![0; 1024 * 1024]).unwrap();
        let node_buffer = (document.nodes.as_ptr(), document.nodes.capacity());
        let field_buffer = (
            document.field_nodes.as_ptr(),
            document.field_nodes.capacity(),
        );
        let text_buffer = (
            document.message_buffer.as_ptr(),
            document.message_buffer.capacity(),
        );
        let mut encoded = first;
        for buffer in [None, Some(second)] {
            if let Some(buffer) = buffer {
                encoded = buffer;
                document.load(buffer).unwrap();
            }
            let mut values = document.iter_fields_and_values();
            let (key, array) = values
                .next()
                .unwrap()
                .1
                .as_object()
                .unwrap()
                .next()
                .unwrap();
            assert_eq!(key, "鍵");
            assert_eq!(key.as_ptr(), key_address);
            let mut array = array.as_array().unwrap();
            let text = array.next().unwrap().as_str().unwrap();
            assert_eq!(text, "shared 東京");
            assert_eq!(text.as_ptr(), text_address);
            let inline = array.next().unwrap().as_str().unwrap();
            assert_eq!(inline, "é");
            assert_eq!(array.next().unwrap().as_str(), Some(""));
            assert!(
                document
                    .bytes(encoded)
                    .as_ptr_range()
                    .contains(&inline.as_ptr())
            );
            // Repeated traversals borrow the checked text ranges, including facets.
            for _ in 0..3 {
                assert_eq!(document.to_tantivy_document(), input);
            }
            let message = values.next().unwrap().1.as_str().unwrap();
            assert_eq!(
                message,
                "request for acme corp on web-12 at 東京支社 succeeded"
            );
            assert!(!document.message_buffer.is_empty());
            assert_ne!(message.as_ptr(), document.message_buffer.as_ptr());
            let NodeValue::Text(range) = document.nodes[document.field_nodes[1].1].value else {
                panic!("message text range");
            };
            assert_eq!(message.as_ptr(), document.bytes(range).as_ptr());
            let bytes = values.next().unwrap().1.as_bytes().unwrap();
            assert!(
                document
                    .bytes(encoded)
                    .as_ptr_range()
                    .contains(&bytes.as_ptr())
            );
            assert_eq!(
                node_buffer,
                (document.nodes.as_ptr(), document.nodes.capacity())
            );
            assert_eq!(
                field_buffer,
                (
                    document.field_nodes.as_ptr(),
                    document.field_nodes.capacity()
                )
            );
            assert_eq!(
                text_buffer,
                (
                    document.message_buffer.as_ptr(),
                    document.message_buffer.capacity()
                )
            );
        }
        assert_eq!(
            document.text(first_message),
            "request for acme corp on web-12 at 東京支社 succeeded"
        );
        let arena_bytes = document.arena.len();
        document.clear();
        assert_eq!(document.arena.len(), arena_bytes);
        assert!(document.nodes.is_empty());
        assert!(document.field_nodes.is_empty());
        assert!(document.message_buffer.is_empty());
        assert_eq!(
            node_buffer,
            (document.nodes.as_ptr(), document.nodes.capacity())
        );
        assert_eq!(text_buffer.1, document.message_buffer.capacity());
    }
}
