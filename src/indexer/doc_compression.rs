//! Compressed documents waiting to be indexed. This format is used only while building a segment.

mod staged_document;
pub(super) use staged_document::StagedDocument;

#[cfg(test)]
#[path = "doc_compression_tests.rs"]
mod tests;

use std::mem::size_of;

use common::VInt;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use stacker::{Addr, MemoryArena, SharedArenaHashMap};

use super::FieldPath;
use crate::TantivyError;
use crate::core::json_utils::split_json_path;
use crate::schema::document::{
    BinaryValueSerializer, ReferenceValue, ReferenceValueLeaf, type_codes,
};
use crate::schema::{Document, Field, FieldType, OwnedValue, Schema, Value};

// Staging-only value codes, outside the document store's code range.
const CUSTOM_CODE: u8 = 254;
const MESSAGE_TOKENS_CODE: u8 = 255;

const MAX_PATH_NODES: usize = 16_384;
const MAX_PATH_NAME_BYTES: usize = 1024 * 1024;

/// Lossless compression of documents waiting to be indexed.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct CompressionConfig {
    /// Text fields or JSON paths whose strings may use message token encoding.
    pub message_fields: Vec<String>,
    /// Admit up to `max_fields_per_path - 1` named keys per object path, in arrival order.
    /// Keep admitted keys shared and store additional keys inline. Must be at least one.
    pub max_fields_per_path: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            message_fields: Vec::new(),
            max_fields_per_path: 256,
        }
    }
}

// Validate supported values and estimate their owned size, including nested values.
fn estimate_owned_value_bytes<'a, V: Value<'a>>(value: V) -> crate::Result<usize> {
    let heap_bytes = match value.as_value() {
        ReferenceValue::Leaf(leaf) => match leaf {
            ReferenceValueLeaf::Str(text) | ReferenceValueLeaf::Facet(text) => text.len(),
            ReferenceValueLeaf::Bytes(bytes) | ReferenceValueLeaf::Custom(bytes) => bytes.len(),
            ReferenceValueLeaf::PreTokStr(_) => {
                return Err(TantivyError::InvalidArgument(
                    "document staging does not support pre-tokenized strings".into(),
                ));
            }
            _ => 0,
        },
        ReferenceValue::Array(values) => values
            .map(estimate_owned_value_bytes)
            .sum::<crate::Result<usize>>()?,
        ReferenceValue::Object(values) => values
            .map(|(key, value)| {
                Ok(size_of::<String>() + key.len() + estimate_owned_value_bytes(value)?)
            })
            .sum::<crate::Result<usize>>()?,
    };
    Ok(size_of::<OwnedValue>() + heap_bytes)
}

// Include bucket capacity and control bytes.
fn estimate_map_bytes<K, V>(map: &FxHashMap<K, V>) -> usize {
    if map.capacity() == 0 {
        0
    } else {
        (map.capacity() * 8).div_ceil(7) * (size_of::<(K, V)>() + 1) + 16
    }
}

fn write_vint(out: &mut Vec<u8>, value: usize) {
    VInt(value as u64).serialize_into_vec(out);
}

// Replace the reserved byte with the header and optional inline bytes.
// One-byte headers leave following bytes in place.
fn replace_header(out: &mut Vec<u8>, offset: usize, header_value: usize, inline_bytes: &[u8]) {
    let mut encoded = [0u8; 10];
    let integer_len = VInt(header_value as u64).serialize_into(&mut encoded);
    let header_len = integer_len + inline_bytes.len();
    if header_len > 1 {
        let end = out.len();
        out.resize(end + header_len - 1, 0);
        out.copy_within(offset + 1..end, offset + header_len);
    }
    out[offset..offset + integer_len].copy_from_slice(&encoded[..integer_len]);
    out[offset + integer_len..offset + header_len].copy_from_slice(inline_bytes);
}

fn read_vint(input: &mut &[u8]) -> usize {
    VInt::deserialize_u64(input).expect("staged integer") as usize
}

fn write_length_prefixed_bytes(out: &mut Vec<u8>, value: &[u8]) {
    write_vint(out, value.len());
    out.extend_from_slice(value);
}

/// A byte range identified by its arena address and length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ByteRange {
    addr: Addr,
    len: u32,
}

impl ByteRange {
    fn store(arena: &mut MemoryArena, bytes: &[u8]) -> crate::Result<Self> {
        if bytes.is_empty() {
            return Ok(Self {
                addr: Addr::null_pointer(),
                len: 0,
            });
        }
        if !arena.can_allocate(bytes.len()) {
            return Err(TantivyError::InvalidArgument(format!(
                "document staging cannot allocate {} bytes (arena limits: 1 MiB per allocation, 4 GiB total)",
                bytes.len()
            )));
        }
        let addr = arena.allocate_space(bytes.len());
        arena.slice_mut(addr, bytes.len()).copy_from_slice(bytes);
        Ok(Self {
            addr,
            len: bytes.len() as u32,
        })
    }

    fn bytes(self, arena: &MemoryArena) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        arena.slice(self.addr, self.len as usize)
    }

    /// Take a prefix of the range and advance past it.
    fn take_prefix(&mut self, len: usize) -> Self {
        assert!(len <= self.len as usize, "staged byte range");
        let len = len as u32;
        let prefix = Self { len, ..*self };
        self.addr = self.addr.offset(len);
        self.len -= len;
        prefix
    }
}

// Assign IDs during encoding. Build the address lookup when decoding begins.
#[derive(Default)]
pub(super) struct Dictionary {
    ids_by_bytes: SharedArenaHashMap,
}

impl Dictionary {
    fn get_id(&self, bytes: &[u8], arena: &MemoryArena) -> Option<u32> {
        self.ids_by_bytes.get(bytes, arena)
    }

    fn get_or_insert_id(&mut self, bytes: &[u8], arena: &mut MemoryArena) -> Option<u32> {
        // Keep empty keys inline. Stacker truncates keys longer than 65,535 bytes.
        if bytes.is_empty() || bytes.len() > u16::MAX as usize {
            return None;
        }
        if !arena.can_allocate(size_of::<u16>() + bytes.len() + size_of::<u32>()) {
            return self.get_id(bytes, arena);
        }
        let id = self.ids_by_bytes.len() as u32;
        Some(
            self.ids_by_bytes
                .mutate_or_create(bytes, arena, |existing| existing.unwrap_or(id)),
        )
    }

    fn ranges(&self, arena: &MemoryArena) -> Vec<ByteRange> {
        let mut ranges = vec![
            ByteRange {
                addr: Addr::null_pointer(),
                len: 0
            };
            self.ids_by_bytes.len()
        ];
        for (bytes, value_addr) in self.ids_by_bytes.iter(arena) {
            let id = arena.read::<u32>(value_addr);
            // Stacker stores the key immediately before its value.
            ranges[id as usize] = ByteRange {
                addr: value_addr.offset(0u32.wrapping_sub(bytes.len() as u32)),
                len: bytes.len() as u32,
            };
        }
        ranges
    }

    // The low bit distinguishes a dictionary ID from inline bytes.
    fn write_id_or_bytes(&mut self, out: &mut Vec<u8>, bytes: &[u8], arena: &mut MemoryArena) {
        if let Some(id) = self.get_or_insert_id(bytes, arena) {
            write_vint(out, ((id as usize) << 1) | 1);
        } else {
            Self::write_inline_bytes(out, bytes);
        }
    }

    fn write_inline_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
        write_vint(out, bytes.len() << 1);
        out.extend_from_slice(bytes);
    }

    fn mem_usage(&self) -> usize {
        self.ids_by_bytes.mem_usage()
    }
}

#[derive(Default)]
struct KeyTreeNode {
    // Keys outside the tree are written inline with their values.
    children: FxHashMap<String, usize>,
}

pub(crate) struct DocumentCompressor {
    message_token_paths: Vec<FieldPath>,
    max_named_children: usize,

    indexing_headroom_bytes: usize,
    expanded_document_bytes: usize,
    reconstructed_text_bytes: usize,

    arena: MemoryArena,
    texts: Dictionary,
    // Ordered field IDs or object keys, with markers for keys stored beside their values.
    layouts: Dictionary,
    layout_buffer: Vec<u8>,
    document_buffer: Vec<u8>,

    field_roots: FxHashMap<Field, usize>,
    key_tree: Vec<KeyTreeNode>,
    path_name_bytes: usize,
    child_map_bytes: usize,
}

impl DocumentCompressor {
    pub fn new(config: CompressionConfig, schema: &Schema) -> crate::Result<Self> {
        if config.max_fields_per_path == 0 {
            return Err(TantivyError::InvalidArgument(
                "compression max_fields_per_path must be at least one".into(),
            ));
        }
        let mut message_token_paths = Vec::new();
        for name in config.message_fields {
            let (field, json_path) = schema
                .find_field(&name)
                .ok_or_else(|| TantivyError::FieldNotFound(name.clone()))?;
            let field_type = schema.get_field_entry(field).field_type();
            if (!json_path.is_empty() && !matches!(field_type, FieldType::JsonObject(_)))
                || (json_path.is_empty()
                    && !matches!(field_type, FieldType::Str(_) | FieldType::JsonObject(_)))
            {
                return Err(TantivyError::InvalidArgument(format!(
                    "compression message path `{name}` must address text or JSON"
                )));
            }
            let json_path = if json_path.is_empty() {
                Vec::new()
            } else {
                split_json_path(json_path)
            };
            message_token_paths.push(FieldPath { field, json_path });
        }
        Ok(Self {
            message_token_paths,
            max_named_children: config.max_fields_per_path - 1,
            indexing_headroom_bytes: 0,
            expanded_document_bytes: 0,
            reconstructed_text_bytes: 0,
            arena: MemoryArena::default(),
            texts: Dictionary::default(),
            layouts: Dictionary::default(),
            layout_buffer: Vec::new(),
            document_buffer: Vec::new(),
            field_roots: FxHashMap::default(),
            key_tree: Vec::new(),
            path_name_bytes: 0,
            child_map_bytes: 0,
        })
    }

    fn uses_message_tokens(&self, field: Field, json_path: &[&str]) -> bool {
        self.message_token_paths
            .iter()
            .any(|path| path.field == field && path.json_path == json_path)
    }

    fn get_or_add_root(&mut self, field: Field) -> Option<usize> {
        if let Some(&node_index) = self.field_roots.get(&field) {
            return Some(node_index);
        }
        if self.key_tree.len() >= MAX_PATH_NODES {
            return None;
        }
        let node_index = self.key_tree.len();
        self.key_tree.push(KeyTreeNode::default());
        self.field_roots.insert(field, node_index);
        Some(node_index)
    }

    fn get_or_add_child(&mut self, parent_index: Option<usize>, name: &str) -> Option<usize> {
        let parent_index = parent_index?;
        let node = &self.key_tree[parent_index];
        if let Some(&node_index) = node.children.get(name) {
            return Some(node_index);
        }
        if node.children.len() >= self.max_named_children
            || self.key_tree.len() >= MAX_PATH_NODES
            || self.path_name_bytes + name.len() > MAX_PATH_NAME_BYTES
        {
            return None;
        }
        let node_index = self.key_tree.len();
        self.key_tree.push(KeyTreeNode::default());
        let previous_bytes = estimate_map_bytes(&self.key_tree[parent_index].children);
        let name = name.to_owned();
        self.path_name_bytes += name.capacity();
        self.child_map_bytes += name.capacity();
        self.key_tree[parent_index]
            .children
            .insert(name, node_index);
        self.child_map_bytes +=
            estimate_map_bytes(&self.key_tree[parent_index].children) - previous_bytes;
        Some(node_index)
    }

    pub fn compress<D: Document>(&mut self, document: &D) -> crate::Result<ByteRange> {
        let estimated_owned_bytes = document
            .iter_fields_and_values()
            .map(|(_, value)| Ok(size_of::<Field>() + estimate_owned_value_bytes(value)?))
            .sum::<crate::Result<usize>>()?;
        // Reserve for indexing every occurrence, independent of dictionary sharing.
        self.expanded_document_bytes = self
            .expanded_document_bytes
            .saturating_add(estimated_owned_bytes);
        // Estimate extra memory for the largest document's view, reconstructed text, and plugin copy.
        self.indexing_headroom_bytes = self
            .indexing_headroom_bytes
            .max(estimated_owned_bytes.saturating_mul(3));
        self.encode(document)
    }

    fn encode<D: Document>(&mut self, document: &D) -> crate::Result<ByteRange> {
        let mut json_path = SmallVec::new();
        let mut layout = std::mem::take(&mut self.layout_buffer);
        let mut out = std::mem::take(&mut self.document_buffer);
        layout.clear();
        out.clear();
        out.push(0); // Filled once the field sequence is known.
        for (field, value) in document.iter_fields_and_values() {
            write_vint(&mut layout, field.field_id() as usize);
            let path_node_index = self.get_or_add_root(field);
            self.encode_value(
                &mut out,
                &mut layout,
                value,
                path_node_index,
                field,
                &mut json_path,
            );
        }
        self.replace_layout_header(&mut out, 0, &layout, true);
        // Store the encoded document in the arena so both buffers can be reused.
        let document = ByteRange::store(&mut self.arena, &out);
        self.layout_buffer = layout;
        self.document_buffer = out;
        document
    }

    fn replace_layout_header(
        &mut self,
        out: &mut Vec<u8>,
        offset: usize,
        layout: &[u8],
        use_dictionary: bool,
    ) {
        let id = if use_dictionary {
            self.layouts.get_or_insert_id(layout, &mut self.arena)
        } else {
            None
        };
        if let Some(id) = id {
            replace_header(out, offset, ((id as usize) << 1) | 1, &[]);
        } else {
            replace_header(out, offset, layout.len() << 1, layout);
        }
    }

    fn encode_text(&mut self, out: &mut Vec<u8>, text: &str) {
        if text.len() < 4 {
            Dictionary::write_inline_bytes(out, text.as_bytes());
        } else {
            self.texts
                .write_id_or_bytes(out, text.as_bytes(), &mut self.arena);
        }
    }

    // Preserve word and separator tokens byte-for-byte. Digit-only tokens stay inline.
    // Compare token bytes with an estimate of inline text size.
    // Dictionary allocation is accounted for separately.
    fn encode_message(&mut self, out: &mut Vec<u8>, text: &str) {
        if let Some(id) = self.texts.get_id(text.as_bytes(), &self.arena) {
            out.push(type_codes::TEXT_CODE);
            write_vint(out, ((id as usize) << 1) | 1);
            return;
        }
        // Keep oversized text inline so the document allocation reports the arena limit.
        if text.len() > 1024 * 1024 {
            out.push(type_codes::TEXT_CODE);
            self.encode_text(out, text);
            return;
        }
        let start = out.len();
        out.extend_from_slice(&[MESSAGE_TOKENS_CODE, 0]);
        let token_count = self.write_message_tokens(out, text);
        replace_header(out, start + 1, token_count, &[]);
        if out.len() - start >= text.len() + 2 {
            out.truncate(start);
            out.push(type_codes::TEXT_CODE);
            self.encode_text(out, text);
        } else {
            self.reconstructed_text_bytes += text.len();
        }
    }

    fn write_message_tokens(&mut self, out: &mut Vec<u8>, text: &str) -> usize {
        let mut token_count = 0;
        let mut start = 0;
        let mut previous_is_word_char = None;
        for (offset, ch) in text.char_indices() {
            let is_word_char = ch.is_alphanumeric() || ch == '_';
            if previous_is_word_char.is_some_and(|previous| previous != is_word_char) {
                self.write_message_token(out, &text[start..offset]);
                token_count += 1;
                start = offset;
            }
            previous_is_word_char = Some(is_word_char);
        }
        if !text.is_empty() {
            self.write_message_token(out, &text[start..]);
            token_count += 1;
        }
        token_count
    }

    fn write_message_token(&mut self, out: &mut Vec<u8>, text: &str) {
        if text.bytes().all(|byte| byte.is_ascii_digit()) {
            Dictionary::write_inline_bytes(out, text.as_bytes());
        } else {
            self.encode_text(out, text);
        }
    }

    fn encode_value<'a, V: Value<'a>>(
        &mut self,
        out: &mut Vec<u8>,
        layout: &mut Vec<u8>,
        value: V,
        path_node_index: Option<usize>,
        field: Field,
        json_path: &mut SmallVec<[&'a str; 8]>,
    ) {
        match value.as_value() {
            ReferenceValue::Array(values) => {
                out.push(type_codes::ARRAY_CODE);
                let offset = out.len();
                out.push(0);
                let mut element_count = 0;
                for value in values {
                    self.encode_value(out, layout, value, path_node_index, field, json_path);
                    element_count += 1;
                }
                replace_header(out, offset, element_count, &[]);
            }
            ReferenceValue::Object(values) => {
                out.push(type_codes::OBJECT_CODE);
                let offset = out.len();
                out.push(0);
                // Nested objects append their layout, then truncate back to this one.
                let layout_start = layout.len();
                for (key, value) in values {
                    let child_index = self.get_or_add_child(path_node_index, key);
                    if child_index.is_some() {
                        write_vint(layout, key.len() + 1);
                        layout.extend_from_slice(key.as_bytes());
                    } else {
                        // Zero means the key is stored inline beside its value.
                        write_vint(layout, 0);
                        write_length_prefixed_bytes(out, key.as_bytes());
                    }
                    json_path.push(key);
                    self.encode_value(out, layout, value, child_index, field, json_path);
                    json_path.pop();
                }
                self.replace_layout_header(
                    out,
                    offset,
                    &layout[layout_start..],
                    path_node_index.is_some(),
                );
                layout.truncate(layout_start);
            }
            ReferenceValue::Leaf(leaf) => match leaf {
                ReferenceValueLeaf::Str(text) => {
                    if self.uses_message_tokens(field, json_path) {
                        self.encode_message(out, text);
                    } else {
                        out.push(type_codes::TEXT_CODE);
                        self.encode_text(out, text);
                    }
                }
                ReferenceValueLeaf::Facet(value) => {
                    out.push(type_codes::HIERARCHICAL_FACET_CODE);
                    self.encode_text(out, value);
                }
                ReferenceValueLeaf::Custom(value) => {
                    out.push(CUSTOM_CODE);
                    write_length_prefixed_bytes(out, value);
                }
                leaf => {
                    BinaryValueSerializer::new(out)
                        .serialize_value(ReferenceValue::Leaf::<V>(leaf))
                        .expect("staging buffer write");
                }
            },
        }
    }

    // Release the hash tables, collection scratch and key tree before indexing.
    pub fn into_staged_document(self) -> StagedDocument {
        let texts = self.texts.ranges(&self.arena);
        let layouts = self.layouts.ranges(&self.arena);
        StagedDocument::new(self.arena, texts, layouts)
    }

    /// Sum of estimated owned document sizes, used to reserve deferred indexing memory.
    pub fn expanded_document_bytes(&self) -> usize {
        self.expanded_document_bytes
    }

    /// Estimated address lookups, largest document's view and plugin copy, and retained
    /// reconstructed text. Allow for arena page fragmentation when appending that text.
    pub fn indexing_headroom_bytes(&self) -> usize {
        self.indexing_headroom_bytes
            + (self.texts.ids_by_bytes.len() + self.layouts.ids_by_bytes.len())
                * size_of::<ByteRange>()
            + if self.reconstructed_text_bytes == 0 {
                0
            } else {
                2 * self.reconstructed_text_bytes + 1024 * 1024
            }
    }

    /// Estimate arena, dictionary hash-table, key-tree, and scratch-buffer memory.
    /// The arena contains dictionaries and encoded documents.
    /// Excludes allocator bookkeeping and temporary encode/decode allocations.
    pub fn mem_usage(&self) -> usize {
        size_of::<Self>()
            + self.arena.mem_usage()
            + self.texts.mem_usage()
            + self.layouts.mem_usage()
            + self.layout_buffer.capacity()
            + self.document_buffer.capacity()
            + estimate_map_bytes(&self.field_roots)
            + self.key_tree.capacity() * size_of::<KeyTreeNode>()
            + self.child_map_bytes
            + self.message_token_paths.capacity() * size_of::<FieldPath>()
            + self
                .message_token_paths
                .iter()
                .map(|path| {
                    path.json_path.capacity() * size_of::<String>()
                        + path.json_path.iter().map(String::capacity).sum::<usize>()
                })
                .sum::<usize>()
    }
}
