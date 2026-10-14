use std::io;
use std::sync::Arc;

use sstable::SSTable;
use stacker::{ByteDeduplicator, MemoryArena, SharedArenaHashMap};

pub(crate) struct TermIdMapping {
    unordered_to_ord: Vec<OrderedId>,
}

impl TermIdMapping {
    pub fn to_ord(&self, unordered: UnorderedId) -> OrderedId {
        self.unordered_to_ord[unordered.0 as usize]
    }
}

/// When we add values, we cannot know their ordered id yet.
/// For this reason, we temporarily assign them a `UnorderedId`
/// that will be mapped to an `OrderedId` upon serialization.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct UnorderedId(pub u32);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct OrderedId(pub u32);

/// `DictionaryBuilder` for dictionary encoding.
///
/// It stores the different terms encounterred and assigns them a temporary value
/// we call unordered id.
///
/// Upon serialization, we will sort the ids and hence build a `UnorderedId -> Term ordinal`
/// mapping.
#[derive(Default)]
pub(crate) struct DictionaryBuilder {
    dict: SharedArenaHashMap,
}

impl DictionaryBuilder {
    /// Shares term bytes with other owners while assigning dictionary-local ids.
    pub fn with_byte_deduplicator(byte_deduplicator: Arc<ByteDeduplicator>) -> Self {
        Self {
            dict: SharedArenaHashMap::with_byte_deduplicator(4, byte_deduplicator),
        }
    }

    /// Get or allocate an unordered id.
    /// (This ID is simply an auto-incremented id.)
    pub fn get_or_allocate_id(&mut self, term: &[u8], arena: &mut MemoryArena) -> UnorderedId {
        let next_id = self.dict.len() as u32;
        let unordered_id = self
            .dict
            .mutate_or_create(term, arena, |unordered_id: Option<u32>| {
                if let Some(unordered_id) = unordered_id {
                    unordered_id
                } else {
                    next_id
                }
            });
        UnorderedId(unordered_id)
    }

    fn build_sorted_terms<'a>(&'a self, arena: &'a MemoryArena) -> Vec<(&'a [u8], UnorderedId)> {
        let mut terms: Vec<(&[u8], UnorderedId)> = self
            .dict
            .iter(arena)
            .map(|(k, v)| (k, arena.read(v)))
            .collect();
        terms.sort_unstable_by_key(|(key, _)| *key);
        terms
    }

    /// Serialize the dictionary into an fst, and returns the
    /// `UnorderedId -> TermOrdinal` map.
    pub fn serialize<'a, W: io::Write + 'a>(
        &self,
        arena: &MemoryArena,
        wrt: &mut W,
    ) -> io::Result<TermIdMapping> {
        let terms = self.build_sorted_terms(arena);
        // TODO Remove the allocation.
        let mut unordered_to_ord: Vec<OrderedId> = vec![OrderedId(0u32); terms.len()];
        let mut sstable_builder = sstable::VoidSSTable::writer(wrt);
        for (ord, (key, unordered_id)) in terms.into_iter().enumerate() {
            let ordered_id = OrderedId(ord as u32);
            sstable_builder.insert(key, &())?;
            unordered_to_ord[unordered_id.0 as usize] = ordered_id;
        }
        sstable_builder.finish()?;
        Ok(TermIdMapping { unordered_to_ord })
    }

    /// Build the `UnorderedId -> OrderedId` mapping in memory without serializing.
    pub fn build_term_id_mapping(&self, arena: &MemoryArena) -> TermIdMapping {
        let terms = self.build_sorted_terms(arena);
        let mut unordered_to_ord: Vec<OrderedId> = vec![OrderedId(0u32); terms.len()];
        for (ord, (_key, unordered_id)) in terms.into_iter().enumerate() {
            unordered_to_ord[unordered_id.0 as usize] = OrderedId(ord as u32);
        }
        TermIdMapping { unordered_to_ord }
    }

    /// Counts shared term bytes for this owner, excluding shared pool metadata.
    /// Summing owners can therefore count the same bytes more than once.
    pub(crate) fn mem_usage(&self) -> usize {
        self.dict.mem_usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_builder_with_byte_deduplicator() {
        let byte_deduplicator = Arc::new(ByteDeduplicator::default());
        let mut first = DictionaryBuilder::with_byte_deduplicator(byte_deduplicator.clone());
        let mut second = DictionaryBuilder::with_byte_deduplicator(byte_deduplicator);
        let mut first_arena = MemoryArena::default();
        let mut second_arena = MemoryArena::default();
        first.get_or_allocate_id(b"zebra", &mut first_arena);
        let first_shared = first.get_or_allocate_id(b"shared", &mut first_arena);
        let second_shared = second.get_or_allocate_id(b"shared", &mut second_arena);
        second.get_or_allocate_id(b"apple", &mut second_arena);
        assert_eq!(first_shared, UnorderedId(1));
        assert_eq!(second_shared, UnorderedId(0));
        assert_eq!(
            first.get_or_allocate_id(b"shared", &mut first_arena),
            first_shared
        );
        let first_terms = first.build_sorted_terms(&first_arena);
        let second_terms = second.build_sorted_terms(&second_arena);
        assert!(std::ptr::eq(first_terms[0].0, second_terms[1].0));

        let mut first_bytes = Vec::new();
        let first_mapping = first.serialize(&first_arena, &mut first_bytes).unwrap();
        assert_eq!(first_mapping.to_ord(first_shared), OrderedId(0));
        drop(first);
        drop(first_arena);
        let mut second_bytes = Vec::new();
        let second_mapping = second.serialize(&second_arena, &mut second_bytes).unwrap();
        assert_eq!(second_mapping.to_ord(second_shared), OrderedId(1));

        for (terms, expected) in [
            ([b"zebra".as_slice(), b"shared".as_slice()], first_bytes),
            ([b"shared".as_slice(), b"apple".as_slice()], second_bytes),
        ] {
            let mut dictionary = DictionaryBuilder::default();
            let mut arena = MemoryArena::default();
            for term in terms {
                dictionary.get_or_allocate_id(term, &mut arena);
            }
            let mut bytes = Vec::new();
            dictionary.serialize(&arena, &mut bytes).unwrap();
            assert_eq!(bytes, expected);
        }
    }

    #[test]
    fn test_dictionary_builder() {
        let mut arena = MemoryArena::default();
        let mut dictionary_builder = DictionaryBuilder::default();
        let hello_uid = dictionary_builder.get_or_allocate_id(b"hello", &mut arena);
        let happy_uid = dictionary_builder.get_or_allocate_id(b"happy", &mut arena);
        let tax_uid = dictionary_builder.get_or_allocate_id(b"tax", &mut arena);
        let mut buffer = Vec::new();
        let id_mapping = dictionary_builder.serialize(&arena, &mut buffer).unwrap();
        assert_eq!(id_mapping.to_ord(hello_uid), OrderedId(1));
        assert_eq!(id_mapping.to_ord(happy_uid), OrderedId(0));
        assert_eq!(id_mapping.to_ord(tax_uid), OrderedId(2));
    }
}
