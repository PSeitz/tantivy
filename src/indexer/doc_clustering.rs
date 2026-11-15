//! Buffer documents in nested groups before assigning document IDs.

#[cfg(test)]
#[path = "doc_clustering_tests.rs"]
mod tests;

use std::borrow::Cow;
use std::cmp::Reverse;
use std::mem::size_of;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::{AddOperation, SegmentWriter};
use crate::core::json_utils::split_json_path;
use crate::schema::document::{Document, ReferenceValue, ReferenceValueLeaf, Value};
use crate::schema::{FieldType, Schema};
use crate::{DocId, TantivyDocument, TantivyError};

struct FieldPath {
    field: crate::schema::Field,
    json_path: Vec<String>,
}

/// Fields used to group documents, from outermost to innermost.
///
/// For example, `["status", "json.service"]` groups by status, then by service
/// within each status. Paths may address schema fields or nested JSON fields.
/// Groups use strings. Missing values and non-string endpoints share an ignored bucket.
/// Objects are only traversed to resolve paths. Repeated fields and arrays use their
/// first value. Grouping never changes the document.
/// At each level, larger subtrees come first. Equal-size siblings follow their
/// first arrival, refining the previously unspecified tie order. Documents in
/// each leaf retain arrival order.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClusteringConfig {
    /// One field path per level. Must not be empty.
    pub fields: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum GroupKey<'a> {
    Ignored,
    Str(Cow<'a, str>),
}

impl<'a> GroupKey<'a> {
    fn from_value<V: Value<'a>>(value: V, json_path: &[String]) -> Self {
        match value.as_value() {
            ReferenceValue::Array(mut values) => values
                .next()
                .map(|value| Self::from_value(value, json_path))
                .unwrap_or(Self::Ignored),
            ReferenceValue::Object(mut values) => {
                let Some((json_path_segment, remaining_path)) = json_path.split_first() else {
                    return Self::Ignored;
                };
                values
                    .find(|(key, _)| *key == json_path_segment)
                    .map(|(_, value)| Self::from_value(value, remaining_path))
                    .unwrap_or(Self::Ignored)
            }
            ReferenceValue::Leaf(_) if !json_path.is_empty() => Self::Ignored,
            ReferenceValue::Leaf(leaf) => match leaf {
                ReferenceValueLeaf::Str(text) => Self::Str(Cow::Borrowed(text)),
                ReferenceValueLeaf::PreTokStr(value) => Self::Str(Cow::Owned(value.text)),
                _ => Self::Ignored,
            },
        }
    }
}

struct GroupNode<T> {
    children: FxHashMap<String, GroupNode<T>>,
    // Keep ignored values separate so even the empty string has its own group.
    ignored: Option<Box<GroupNode<T>>>,
    documents: Vec<T>,
    num_docs: usize,
    first_arrival: usize,
}

impl<T> Default for GroupNode<T> {
    fn default() -> Self {
        Self {
            children: FxHashMap::default(),
            ignored: None,
            documents: Vec::new(),
            num_docs: 0,
            first_arrival: 0,
        }
    }
}

impl<T> GroupNode<T> {
    // Locate the leaf before moving the document whose keys we borrow.
    fn leaf<'a>(
        &mut self,
        mut group_keys: impl Iterator<Item = GroupKey<'a>>,
        mem_usage: &mut usize,
    ) -> &mut Vec<T> {
        self.num_docs += 1;
        let Some(key) = group_keys.next() else {
            return &mut self.documents;
        };
        let first_arrival = self.children.len() + usize::from(self.ignored.is_some());
        let GroupKey::Str(key) = key else {
            return self
                .ignored
                .get_or_insert_with(|| {
                    *mem_usage += size_of::<Self>();
                    Box::new(Self {
                        first_arrival,
                        ..Self::default()
                    })
                })
                .leaf(group_keys, mem_usage);
        };
        if !self.children.contains_key(key.as_ref()) {
            let capacity = self.children.capacity();
            let key = key.clone().into_owned();
            *mem_usage += key.capacity();
            self.children.insert(
                key,
                Self {
                    first_arrival,
                    ..Self::default()
                },
            );
            // Include empty buckets and the trailing control-byte group.
            let bucket_growth =
                (self.children.capacity() * 8).div_ceil(7) - (capacity * 8).div_ceil(7);
            *mem_usage += bucket_growth * (size_of::<(String, Self)>() + 1)
                + if capacity == 0 { 16 } else { 0 };
        }
        self.children
            .get_mut(key.as_ref())
            .unwrap()
            .leaf(group_keys, mem_usage)
    }

    fn order_key(&self) -> (Reverse<usize>, usize) {
        (Reverse(self.num_docs), self.first_arrival)
    }

    fn group_sizes(&self) -> Vec<usize> {
        let mut sizes = Vec::new();
        self.collect_group_sizes(&mut sizes);
        sizes
    }

    fn collect_group_sizes(&self, sizes: &mut Vec<usize>) {
        if !self.documents.is_empty() {
            sizes.push(self.documents.len());
        }
        let mut children: Vec<_> = self
            .children
            .values()
            .chain(self.ignored.as_deref())
            .collect();
        children.sort_unstable_by_key(|child| child.order_key());
        for child in children {
            child.collect_group_sizes(sizes);
        }
    }

    fn drain<E>(self, visit: &mut impl FnMut(T) -> Result<(), E>) -> Result<(), E> {
        for document in self.documents {
            visit(document)?;
        }
        // Collecting drops group keys and the hash table before visiting children.
        let mut children: Vec<_> = self
            .children
            .into_values()
            .chain(self.ignored.map(|child| *child))
            .collect();
        children.sort_unstable_by_key(Self::order_key);
        for child in children {
            child.drain(visit)?;
        }
        Ok(())
    }
}

fn grouping_paths(schema: &Schema, fields: &[String]) -> crate::Result<Vec<FieldPath>> {
    if fields.is_empty() {
        return Err(TantivyError::InvalidArgument(
            "clustering requires at least one field".to_owned(),
        ));
    }
    fields
        .iter()
        .map(|name| {
            let (field, json_path) = schema
                .find_field(name)
                .ok_or_else(|| TantivyError::FieldNotFound(name.clone()))?;
            if !json_path.is_empty()
                && !matches!(
                    schema.get_field_entry(field).field_type(),
                    FieldType::JsonObject(_)
                )
            {
                return Err(TantivyError::InvalidArgument(format!(
                    "clustering path `{name}` is not inside a JSON field"
                )));
            }
            Ok(FieldPath {
                field,
                json_path: if json_path.is_empty() {
                    Vec::new()
                } else {
                    split_json_path(json_path)
                },
            })
        })
        .collect()
}

fn group_keys<'a, D: Document>(
    paths: &'a [FieldPath],
    document: &'a D,
) -> impl Iterator<Item = GroupKey<'a>> + 'a {
    paths.iter().map(|path| {
        document
            .iter_fields_and_values()
            .find(|(field, _)| *field == path.field)
            .map(|(_, value)| GroupKey::from_value(value, &path.json_path))
            .unwrap_or(GroupKey::Ignored)
    })
}

/// Groups caller-supplied document IDs using the same paths and order as clustering.
///
/// Only grouping keys and IDs are retained. Grouping follows [`ClusteringConfig`],
/// preserving arrival order within each leaf. An index's `sort_by_field` setting
/// may subsequently change the order produced by clustered indexing.
pub struct DocIdGroups {
    grouping_paths: Vec<FieldPath>,
    root: GroupNode<DocId>,
}

impl DocIdGroups {
    /// Resolve nonempty grouping field paths against the schema.
    pub fn new(schema: &Schema, fields: &[String]) -> crate::Result<Self> {
        Ok(Self {
            grouping_paths: grouping_paths(schema, fields)?,
            root: GroupNode::default(),
        })
    }

    /// Add an ID without retaining the document.
    pub fn push<D: Document>(&mut self, document: &D, doc_id: DocId) -> crate::Result<()> {
        self.root
            .leaf(group_keys(&self.grouping_paths, document), &mut 0)
            .push(doc_id);
        Ok(())
    }

    /// Counts of leaf groups in traversal order.
    pub fn group_sizes(&self) -> Vec<usize> {
        self.root.group_sizes()
    }

    /// Consume the groups and return IDs in clustering order (new ID to supplied ID).
    ///
    /// Supply each original ID exactly once to obtain a permutation suitable for
    /// [`crate::indexer::DocIdMapping::new_permutation`].
    pub fn into_order(self) -> Vec<DocId> {
        let mut order = Vec::with_capacity(self.root.num_docs);
        let result: Result<(), std::convert::Infallible> = self.root.drain(&mut |doc_id| {
            order.push(doc_id);
            Ok(())
        });
        match result {
            Ok(()) => order,
            Err(never) => match never {},
        }
    }
}

pub(crate) struct DocGroups {
    grouping_paths: Vec<FieldPath>,
    root: GroupNode<TantivyDocument>,
    mem_usage: usize,
}

impl DocGroups {
    pub fn new(config: ClusteringConfig, schema: &Schema) -> crate::Result<Self> {
        let grouping_paths = grouping_paths(schema, &config.fields)?;
        let mem_usage = size_of::<Self>()
            + grouping_paths.capacity() * size_of::<FieldPath>()
            + grouping_paths
                .iter()
                .map(|path| {
                    path.json_path.capacity() * size_of::<String>()
                        + path.json_path.iter().map(String::capacity).sum::<usize>()
                })
                .sum::<usize>();
        Ok(Self {
            grouping_paths,
            root: GroupNode::default(),
            mem_usage,
        })
    }

    pub fn push(&mut self, document: TantivyDocument) {
        self.mem_usage += document.mem_usage() - size_of::<TantivyDocument>();
        let leaf = self.root.leaf(
            group_keys(&self.grouping_paths, &document),
            &mut self.mem_usage,
        );
        let capacity = leaf.capacity();
        leaf.push(document);
        self.mem_usage += (leaf.capacity() - capacity) * size_of::<TantivyDocument>();
    }

    pub fn group_sizes(&self) -> Vec<usize> {
        self.root.group_sizes()
    }

    pub fn mem_usage(&self) -> usize {
        self.mem_usage
    }

    pub fn drain(self, writer: &mut SegmentWriter) -> crate::Result<()> {
        self.root.drain(&mut |document| {
            // Single-segment indexing has no deletes, so opstamps are unused.
            writer.add_document(AddOperation {
                opstamp: 0,
                document,
            })
        })
    }
}
