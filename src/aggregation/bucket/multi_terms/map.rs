use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::hash::Hash;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::TantivyError;

/// Maps fixed-length array keys and tracks the highest key for eviction.
#[derive(Clone, Debug)]
struct ArrayHeapMap<K: Ord, V, const S: usize> {
    buckets: FxHashMap<[K; S], V>,
    heap: BinaryHeap<[K; S]>,
}

impl<K: Ord, V, const S: usize> Default for ArrayHeapMap<K, V, S> {
    fn default() -> Self {
        Self {
            buckets: FxHashMap::default(),
            heap: BinaryHeap::default(),
        }
    }
}

impl<K: Copy + Hash + Ord, V, const S: usize> ArrayHeapMap<K, V, S> {
    /// Returns an entry only when its key belongs in the first `size` keys.
    fn get_or_insert_with(
        &mut self,
        key: &[K],
        size: usize,
        create: impl FnOnce() -> V,
    ) -> Option<&mut V> {
        let key: &[K; S] = key.try_into().expect("Key length mismatch");
        if self.buckets.len() >= size {
            let highest = self.heap.peek().expect("page size is positive");
            if key > highest {
                return None;
            }
            if key < highest && !self.buckets.contains_key(key) {
                self.buckets.remove(&self.heap.pop().unwrap());
            }
        }
        Some(self.buckets.entry(*key).or_insert_with(|| {
            self.heap.push(*key);
            create()
        }))
    }

    fn memory_consumption(&self) -> u64 {
        let key_size = std::mem::size_of::<[K; S]>();
        let map_size = (key_size + std::mem::size_of::<V>()) * self.buckets.capacity();
        (map_size + key_size * self.heap.capacity()) as u64
    }
}

pub(super) const MAX_DYN_ARRAY_SIZE: usize = 16;

// Define the supported array lengths once, sharing them across construction and all operations.
macro_rules! define_array_heap_map {
    ($($variant:ident: $size:literal),+ $(,)?) => {
        /// Selects an array key length at construction, avoiding per-key heap allocations.
        #[derive(Clone, Debug)]
        pub(super) struct DynArrayHeapMap<K: Ord, V>(ArrayHeapMaps<K, V>);

        #[derive(Clone, Debug)]
        enum ArrayHeapMaps<K: Ord, V> {
            $($variant(ArrayHeapMap<K, V, $size>)),+
        }

        impl<K: Copy + Hash + Ord + 'static, V: 'static> DynArrayHeapMap<K, V> {
            pub(super) fn try_new(key_dimension: usize) -> crate::Result<Self> {
                match key_dimension {
                    $($size => Ok(Self(ArrayHeapMaps::$variant(ArrayHeapMap::default()))),)+
                    _ => Err(TantivyError::InvalidArgument(format!(
                        "array keys require between 1 and {MAX_DYN_ARRAY_SIZE} components, got {key_dimension}"
                    ))),
                }
            }

            #[cfg(test)]
            pub(super) fn size(&self) -> usize {
                match &self.0 { $(ArrayHeapMaps::$variant(map) => map.buckets.len()),+ }
            }

            /// Gets or inserts one of the first `size` keys, evicting the highest key if necessary.
            /// Panics for a mismatched key length or a zero page size.
            pub(super) fn get_or_insert_with(&mut self, key: &[K], size: usize, create: impl FnOnce() -> V) -> Option<&mut V> {
                match &mut self.0 { $(ArrayHeapMaps::$variant(map) => map.get_or_insert_with(key, size, create)),+ }
            }

            pub(super) fn memory_consumption(&self) -> u64 {
                match &self.0 { $(ArrayHeapMaps::$variant(map) => map.memory_consumption()),+ }
            }

            pub(super) fn into_iter(self) -> Box<dyn Iterator<Item = (SmallVec<[K; MAX_DYN_ARRAY_SIZE]>, V)>> {
                match self.0 {
                    $(ArrayHeapMaps::$variant(map) => Box::new(map.buckets.into_iter()
                        .map(|(key, value)| (SmallVec::from_slice(&key), value)))),+
                }
            }
        }
    };
}

define_array_heap_map! {
    Dim1: 1, Dim2: 2, Dim3: 3, Dim4: 4,
    Dim5: 5, Dim6: 6, Dim7: 7, Dim8: 8,
    Dim9: 9, Dim10: 10, Dim11: 11, Dim12: 12,
    Dim13: 13, Dim14: 14, Dim15: 15, Dim16: 16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dyn_array_heap_map() {
        assert!(DynArrayHeapMap::<u32, &str>::try_new(0).is_err());
        assert!(DynArrayHeapMap::<u32, &str>::try_new(MAX_DYN_ARRAY_SIZE + 1).is_err());
        for size in 1..=MAX_DYN_ARRAY_SIZE {
            let mut map = DynArrayHeapMap::<u32, &str>::try_new(size).unwrap();
            let key1 = vec![1u32; size];
            let key2 = vec![2u32; size];
            map.get_or_insert_with(&key1, 2, || "a");
            map.get_or_insert_with(&key2, 2, || "b");
            assert_eq!(map.size(), 2);
            assert!(map.memory_consumption() > 0);
            // Existing keys, including the highest key, retain their entries.
            *map.get_or_insert_with(&key1, 2, || panic!("existing key"))
                .unwrap() = "updated";
            assert_eq!(
                map.get_or_insert_with(&key2, 2, || panic!("existing key")),
                Some(&mut "b")
            );
            assert!(map
                .get_or_insert_with(&vec![3; size], 2, || panic!("rejected key"))
                .is_none());
            // A smaller key replaces the highest key, without changing the other entry.
            map.get_or_insert_with(&vec![0; size], 2, || "new");
            assert_eq!(map.size(), 2);
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            assert_eq!(entries[0].0.as_slice(), vec![0; size]);
            assert_eq!(entries[0].1, "new");
            assert_eq!(entries[1].0.as_slice(), key1);
            assert_eq!(entries[1].1, "updated");
        }
    }
}
