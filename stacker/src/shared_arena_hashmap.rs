use std::iter::{Cloned, Filter};
use std::mem;
use std::sync::Arc;

use super::{Addr, MemoryArena};
use crate::ByteDeduplicator;
use crate::fastcpy::fast_short_slice_copy;
use crate::memory_arena::store;

/// Returns the actual memory size in bytes
/// required to create a table with a given capacity.
/// required to create a table of size
pub fn compute_table_memory_size(capacity: usize) -> usize {
    capacity * mem::size_of::<KeyValue>()
}

#[cfg(not(feature = "compare_hash_only"))]
type HashType = u32;

#[cfg(feature = "compare_hash_only")]
type HashType = u64;

/// `KeyValue` is the item stored in the hash table.
/// The address points to a key followed by its value in an external memory arena.
/// With shared keys, a key index replaces the inline key bytes.
#[derive(Copy, Clone)]
struct KeyValue {
    key_value_addr: Addr,
    hash: HashType,
}

impl Default for KeyValue {
    fn default() -> Self {
        KeyValue {
            key_value_addr: Addr::null_pointer(),
            hash: 0,
        }
    }
}

impl KeyValue {
    #[inline]
    fn is_empty(&self) -> bool {
        self.key_value_addr.is_null()
    }
    #[inline]
    fn is_not_empty_ref(&self) -> bool {
        !self.key_value_addr.is_null()
    }
}

/// Customized `HashMap` with `&[u8]` keys
///
/// Its main particularity is that rather than storing its
/// keys in separate heap allocations, keys are stored in a memory arena
/// inline with the values by default. With a [`ByteDeduplicator`], strong references
/// retain shared key allocations outside the arena.
///
/// The quirky API has the benefit of avoiding
/// the computation of the hash of the key twice,
/// or copying the key as long as there is no insert.
///
/// SharedArenaHashMap is like ArenaHashMap but gets the memory arena
/// passed as an argument to the methods.
/// So one MemoryArena can be shared with multiple SharedArenaHashMap.
pub struct SharedArenaHashMap {
    table: Vec<KeyValue>,
    mask: usize,
    len: usize,
    shared_keys: Option<SharedKeys>,
}

// Owns the shared keys used by this map; the arena stores only their indices and Copy values.
struct SharedKeys {
    byte_deduplicator: Arc<ByteDeduplicator>,
    keys: Vec<Arc<Vec<u8>>>,
    // Charge each payload and its Arc allocation to every map that uses it.
    allocation_bytes: usize,
}

struct LinearProbing {
    pos: usize,
    mask: usize,
}

impl LinearProbing {
    #[inline]
    fn compute(hash: HashType, mask: usize) -> LinearProbing {
        LinearProbing {
            pos: hash as usize,
            mask,
        }
    }

    #[inline]
    fn next_probe(&mut self) -> usize {
        // Not saving the masked version removes a dependency.
        self.pos = self.pos.wrapping_add(1);
        self.pos & self.mask
    }
}

type IterNonEmpty<'a> = Filter<Cloned<std::slice::Iter<'a, KeyValue>>, fn(&KeyValue) -> bool>;

pub struct Iter<'a> {
    hashmap: &'a SharedArenaHashMap,
    memory_arena: &'a MemoryArena,
    inner: IterNonEmpty<'a>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a [u8], Addr);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(move |kv| {
            let (key, offset): (&'a [u8], Addr) = self
                .hashmap
                .get_key_value(kv.key_value_addr, self.memory_arena);
            (key, offset)
        })
    }
}

/// Returns the greatest power of two lower or equal to `n`.
/// Except if n == 0, in that case, return 1.
///
/// # Panics if n == 0
fn compute_previous_power_of_two(n: usize) -> usize {
    assert!(n > 0);
    let msb = (63u32 - (n as u64).leading_zeros()) as u8;
    1 << msb
}

impl Default for SharedArenaHashMap {
    fn default() -> Self {
        SharedArenaHashMap::with_capacity(4)
    }
}

impl SharedArenaHashMap {
    pub fn with_capacity(table_size: usize) -> SharedArenaHashMap {
        let table_size_power_of_2 = compute_previous_power_of_two(table_size);
        let table = vec![KeyValue::default(); table_size_power_of_2];

        SharedArenaHashMap {
            table,
            mask: table_size_power_of_2 - 1,
            len: 0,
            shared_keys: None,
        }
    }

    /// Creates a map whose keys share allocations through the given weak byte pool.
    pub fn with_byte_deduplicator(
        table_size: usize,
        byte_deduplicator: Arc<ByteDeduplicator>,
    ) -> SharedArenaHashMap {
        let mut map = Self::with_capacity(table_size);
        map.shared_keys = Some(SharedKeys {
            byte_deduplicator,
            keys: Vec::new(),
            allocation_bytes: 0,
        });
        map
    }

    #[inline]
    #[cfg(not(feature = "compare_hash_only"))]
    fn get_hash(&self, key: &[u8]) -> HashType {
        murmurhash32::murmurhash2(key)
    }

    #[inline]
    #[cfg(feature = "compare_hash_only")]
    fn get_hash(&self, key: &[u8]) -> HashType {
        /// Since we compare only the hash we need a high quality hash.
        use std::hash::Hasher;
        let mut hasher = ahash::AHasher::default();
        hasher.write(key);
        hasher.finish() as HashType
    }

    #[inline]
    fn probe(&self, hash: HashType) -> LinearProbing {
        LinearProbing::compute(hash, self.mask)
    }

    /// Table and key allocation usage, excluding the externally owned arena.
    /// Shared payloads, Vec headers, Arc counters and local handle capacity are charged
    /// to each map, so shared allocations may be counted more than once. The byte pool's
    /// separate shard/hash-table/weak-handle metadata is not included.
    #[inline]
    pub fn mem_usage(&self) -> usize {
        self.table.len() * mem::size_of::<KeyValue>()
            + self.shared_keys.as_ref().map_or(0, |keys| {
                keys.allocation_bytes + keys.keys.capacity() * mem::size_of::<Arc<Vec<u8>>>()
            })
    }

    #[inline]
    fn is_saturated(&self) -> bool {
        self.table.len() <= self.len * 2
    }

    #[inline]
    fn get_key_value<'a>(&'a self, addr: Addr, memory_arena: &'a MemoryArena) -> (&'a [u8], Addr) {
        let data = memory_arena.slice_from(addr);
        if let Some(keys) = &self.shared_keys {
            let index = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
            return (keys.keys[index].as_slice(), addr.offset(4));
        }
        let key_bytes_len_bytes = unsafe { data.get_unchecked(..2) };
        let key_bytes_len = u16::from_le_bytes(key_bytes_len_bytes.try_into().unwrap());
        let key_bytes: &[u8] = unsafe { data.get_unchecked(2..2 + key_bytes_len as usize) };
        (key_bytes, addr.offset(2 + key_bytes_len as u32))
    }

    #[inline]
    #[cfg(not(feature = "compare_hash_only"))]
    fn get_value_addr_if_key_match(
        &self,
        target_key: &[u8],
        addr: Addr,
        memory_arena: &MemoryArena,
    ) -> Option<Addr> {
        use crate::fastcmp::fast_short_slice_compare;

        let (stored_key, value_addr) = self.get_key_value(addr, memory_arena);
        if fast_short_slice_compare(stored_key, target_key) {
            Some(value_addr)
        } else {
            None
        }
    }
    #[inline]
    #[cfg(feature = "compare_hash_only")]
    fn get_value_addr_if_key_match(
        &self,
        _target_key: &[u8],
        addr: Addr,
        memory_arena: &MemoryArena,
    ) -> Option<Addr> {
        if self.shared_keys.is_some() {
            return Some(addr.offset(4));
        }
        // For the compare_hash_only feature, it would make sense to store the keys at a different
        // memory location. Here they will just pollute the cache.
        let data = memory_arena.slice_from(addr);
        let key_bytes_len_bytes = &data[..2];
        let key_bytes_len = u16::from_le_bytes(key_bytes_len_bytes.try_into().unwrap());
        let value_addr = addr.offset(2 + key_bytes_len as u32);

        Some(value_addr)
    }

    #[inline]
    fn set_bucket(&mut self, hash: HashType, key_value_addr: Addr, bucket: usize) {
        self.len += 1;

        self.table[bucket] = KeyValue {
            key_value_addr,
            hash,
        };
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn iter<'a>(&'a self, memory_arena: &'a MemoryArena) -> Iter<'a> {
        Iter {
            inner: self
                .table
                .iter()
                .cloned()
                .filter(KeyValue::is_not_empty_ref),
            hashmap: self,
            memory_arena,
        }
    }

    fn resize(&mut self) {
        let new_len = (self.table.len() * 2).max(1 << 3);
        let mask = new_len - 1;
        self.mask = mask;
        let new_table = vec![KeyValue::default(); new_len];
        let old_table = mem::replace(&mut self.table, new_table);
        for key_value in old_table.into_iter().filter(KeyValue::is_not_empty_ref) {
            let mut probe = LinearProbing::compute(key_value.hash, mask);
            loop {
                let bucket = probe.next_probe();
                if self.table[bucket].is_empty() {
                    self.table[bucket] = key_value;
                    break;
                }
            }
        }
    }

    /// Get a value associated to a key.
    #[inline]
    pub fn get<V>(&self, key: &[u8], memory_arena: &MemoryArena) -> Option<V>
    where V: Copy + 'static {
        let hash = self.get_hash(key);
        let mut probe = self.probe(hash);
        loop {
            let bucket = probe.next_probe();
            let kv: KeyValue = self.table[bucket];
            if kv.is_empty() {
                return None;
            } else if kv.hash == hash
                && let Some(val_addr) =
                    self.get_value_addr_if_key_match(key, kv.key_value_addr, memory_arena)
            {
                let v = memory_arena.read(val_addr);
                return Some(v);
            }
        }
    }

    /// `update` create a new entry for a given key if it does not exist
    /// or updates the existing entry.
    ///
    /// The actual logic for this update is define in the `updater`
    /// argument.
    ///
    /// If the key is not present, `updater` will receive `None` and
    /// will be in charge of returning a default value.
    /// If the key already as an associated value, then it will be passed
    /// `Some(previous_value)`.
    ///
    /// The key will be truncated to u16::MAX bytes.
    #[inline]
    pub fn mutate_or_create<V>(
        &mut self,
        key: &[u8],
        memory_arena: &mut MemoryArena,
        mut updater: impl FnMut(Option<V>) -> V,
    ) -> V
    where
        V: Copy + 'static,
    {
        if self.is_saturated() {
            self.resize();
        }
        // Limit the key size to u16::MAX in both storage modes.
        let key = &key[..std::cmp::min(key.len(), u16::MAX as usize)];
        let hash = self.get_hash(key);
        let mut probe = self.probe(hash);
        let mut bucket = probe.next_probe();
        let mut kv: KeyValue = self.table[bucket];
        loop {
            if kv.is_empty() {
                // The key does not exist yet.
                let val = updater(None);
                let key_addr = if let Some(keys) = &mut self.shared_keys {
                    let index = u32::try_from(keys.keys.len()).expect("too many deduplicated keys");
                    let allocation = keys.byte_deduplicator.get_or_allocate(key);
                    keys.allocation_bytes += allocation.capacity()
                        + mem::size_of::<Vec<u8>>()
                        + 2 * mem::size_of::<usize>();
                    keys.keys.push(allocation);
                    let num_bytes = mem::size_of::<u32>() + mem::size_of::<V>();
                    let key_addr = memory_arena.allocate_space(num_bytes);
                    let data = memory_arena.slice_mut(key_addr, num_bytes);
                    data[..4].copy_from_slice(&index.to_le_bytes());
                    store(&mut data[4..], val);
                    key_addr
                } else {
                    let num_bytes = mem::size_of::<u16>() + key.len() + mem::size_of::<V>();
                    let key_addr = memory_arena.allocate_space(num_bytes);
                    let data = memory_arena.slice_mut(key_addr, num_bytes);
                    let key_len_bytes: [u8; 2] = (key.len() as u16).to_le_bytes();
                    data[..2].copy_from_slice(&key_len_bytes);
                    let stop = 2 + key.len();
                    fast_short_slice_copy(key, &mut data[2..stop]);
                    store(&mut data[stop..], val);
                    key_addr
                };

                self.set_bucket(hash, key_addr, bucket);
                return val;
            }
            if kv.hash == hash
                && let Some(val_addr) =
                    self.get_value_addr_if_key_match(key, kv.key_value_addr, memory_arena)
            {
                let v = memory_arena.read(val_addr);
                let new_v = updater(Some(v));
                memory_arena.write_at(val_addr, new_v);
                return new_v;
            }
            // This allows fetching the next bucket before the loop jmp
            bucket = probe.next_probe();
            kv = self.table[bucket];
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::sync::Arc;

    use super::{SharedArenaHashMap, compute_previous_power_of_two};
    use crate::{ByteDeduplicator, MemoryArena};

    #[test]
    fn deduplicated_keys_share_allocations_but_not_values() {
        let pool = Arc::new(ByteDeduplicator::new());
        let mut first = SharedArenaHashMap::with_byte_deduplicator(4, pool.clone());
        let mut second = SharedArenaHashMap::with_byte_deduplicator(4, pool.clone());
        let mut first_arena = MemoryArena::default();
        let mut second_arena = MemoryArena::default();
        first.mutate_or_create(b"same", &mut first_arena, |_: Option<u64>| 10);
        second.mutate_or_create(b"same", &mut second_arena, |_: Option<u64>| 20);
        first.mutate_or_create(b"same", &mut first_arena, |v: Option<u64>| v.unwrap() + 1);
        let first_keys = &first.shared_keys.as_ref().unwrap().keys;
        let second_keys = &second.shared_keys.as_ref().unwrap().keys;
        assert_eq!(first_keys.len(), 1);
        assert!(Arc::ptr_eq(&first_keys[0], &second_keys[0]));
        assert!(Arc::ptr_eq(&first_keys[0], &pool.get_or_allocate(b"same")));
        assert_eq!(first.get::<u64>(b"same", &first_arena), Some(11));
        assert_eq!(second.get::<u64>(b"same", &second_arena), Some(20));
        let weak = Arc::downgrade(&first_keys[0]);
        drop(first);
        assert!(weak.upgrade().is_some());
        drop(second);
        assert!(weak.upgrade().is_none());
        pool.remove_expired();
    }

    #[test]
    fn deduplicated_keys_resize_and_preserve_arbitrary_bytes() {
        let pool = Arc::new(ByteDeduplicator::new());
        let mut map = SharedArenaHashMap::with_byte_deduplicator(1, pool.clone());
        let mut arena = MemoryArena::default();
        let mut keys = vec![vec![], vec![0, 255, 128, 0], vec![42; u16::MAX as usize]];
        let mut long_key = keys[2].clone();
        long_key[u16::MAX as usize - 1] = 43;
        keys.push(long_key);
        keys.extend((0u32..10_000).map(|i| i.to_le_bytes().to_vec()));
        let allocations: Vec<_> = keys.iter().map(|k| pool.get_or_allocate(k)).collect();
        for (i, key) in keys.iter().enumerate() {
            map.mutate_or_create(key, &mut arena, |v: Option<u64>| {
                assert_eq!(v, None);
                i as u64
            });
        }
        assert_eq!(map.len(), keys.len());
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(map.get::<u64>(key, &arena), Some(i as u64));
            assert!(Arc::ptr_eq(
                &map.shared_keys.as_ref().unwrap().keys[i],
                &allocations[i]
            ));
        }
        for (key, addr) in map.iter(&arena) {
            assert_eq!(key, keys[arena.read::<u64>(addr) as usize]);
        }
        assert_eq!(map.get::<u64>(b"missing", &arena), None);
    }

    #[test]
    fn deduplicated_keys_keep_the_length_limit() {
        let byte_deduplicator = Arc::new(ByteDeduplicator::new());
        let mut map = SharedArenaHashMap::with_byte_deduplicator(4, byte_deduplicator.clone());
        let mut arena = MemoryArena::default();
        let mut key = vec![42; u16::MAX as usize];
        map.mutate_or_create(&key, &mut arena, |_: Option<u64>| 1);
        key.push(43);
        map.mutate_or_create(&key, &mut arena, |value: Option<u64>| {
            assert_eq!(value, Some(1));
            2
        });
        let (stored_key, addr) = map.iter(&arena).next().unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(stored_key, &key[..u16::MAX as usize]);
        assert_eq!(arena.read::<u64>(addr), 2);
        assert_eq!(map.get::<u64>(stored_key, &arena), Some(2));
    }

    #[test]
    fn deduplicated_key_memory_accounting() {
        let pool = Arc::new(ByteDeduplicator::new());
        let mut map = SharedArenaHashMap::with_byte_deduplicator(4, pool);
        let mut arena = MemoryArena::default();
        let table_bytes = map.mem_usage();
        map.mutate_or_create(b"payload", &mut arena, |_: Option<u32>| 1);
        let keys = map.shared_keys.as_ref().unwrap();
        let expected = table_bytes
            + keys.keys.capacity() * std::mem::size_of::<Arc<Vec<u8>>>()
            + keys.keys[0].capacity()
            + std::mem::size_of::<Vec<u8>>()
            + 2 * std::mem::size_of::<usize>();
        assert_eq!(map.mem_usage(), expected);
        map.mutate_or_create(b"payload", &mut arena, |_: Option<u32>| 2);
        assert_eq!(map.mem_usage(), expected);
    }

    #[test]
    fn test_hash_map() {
        let mut memory_arena = MemoryArena::default();
        let mut hash_map: SharedArenaHashMap = SharedArenaHashMap::default();
        hash_map.mutate_or_create(b"abc", &mut memory_arena, |opt_val: Option<u32>| {
            assert_eq!(opt_val, None);
            3u32
        });
        hash_map.mutate_or_create(b"abcd", &mut memory_arena, |opt_val: Option<u32>| {
            assert_eq!(opt_val, None);
            4u32
        });
        hash_map.mutate_or_create(b"abc", &mut memory_arena, |opt_val: Option<u32>| {
            assert_eq!(opt_val, Some(3u32));
            5u32
        });
        let mut vanilla_hash_map = HashMap::new();
        let iter_values = hash_map.iter(&memory_arena);
        for (key, addr) in iter_values {
            let val: u32 = memory_arena.read(addr);
            vanilla_hash_map.insert(key.to_owned(), val);
        }
        assert_eq!(vanilla_hash_map.len(), 2);
    }

    #[test]
    fn test_long_key_truncation() {
        // Keys longer than u16::MAX are truncated.
        let mut memory_arena = MemoryArena::default();
        let mut hash_map: SharedArenaHashMap = SharedArenaHashMap::default();
        let key1 = (0..u16::MAX as usize).map(|i| i as u8).collect::<Vec<_>>();
        hash_map.mutate_or_create(&key1, &mut memory_arena, |opt_val: Option<u32>| {
            assert_eq!(opt_val, None);
            4u32
        });
        // Due to truncation, this key is the same as key1
        let key2 = (0..u16::MAX as usize + 1)
            .map(|i| i as u8)
            .collect::<Vec<_>>();
        hash_map.mutate_or_create(&key2, &mut memory_arena, |opt_val: Option<u32>| {
            assert_eq!(opt_val, Some(4));
            3u32
        });
        let mut vanilla_hash_map = HashMap::new();
        let iter_values = hash_map.iter(&memory_arena);
        for (key, addr) in iter_values {
            let val: u32 = memory_arena.read(addr);
            vanilla_hash_map.insert(key.to_owned(), val);
            assert_eq!(key.len(), key1[..].len());
            assert_eq!(key, &key1[..])
        }
        assert_eq!(vanilla_hash_map.len(), 1); // Both map to the same key
    }

    #[test]
    fn test_empty_hashmap() {
        let memory_arena = MemoryArena::default();
        let hash_map: SharedArenaHashMap = SharedArenaHashMap::default();
        assert_eq!(hash_map.get::<u32>(b"abc", &memory_arena), None);
    }

    #[test]
    fn test_compute_previous_power_of_two() {
        assert_eq!(compute_previous_power_of_two(8), 8);
        assert_eq!(compute_previous_power_of_two(9), 8);
        assert_eq!(compute_previous_power_of_two(7), 4);
        assert_eq!(compute_previous_power_of_two(u64::MAX as usize), 1 << 63);
    }

    #[test]
    fn test_many_terms() {
        let mut memory_arena = MemoryArena::default();
        let mut terms: Vec<String> = (0..20_000).map(|val| val.to_string()).collect();
        let mut hash_map: SharedArenaHashMap = SharedArenaHashMap::default();
        for term in terms.iter() {
            hash_map.mutate_or_create(
                term.as_bytes(),
                &mut memory_arena,
                |_opt_val: Option<u32>| 5u32,
            );
        }
        let mut terms_back: Vec<String> = hash_map
            .iter(&memory_arena)
            .map(|(bytes, _)| String::from_utf8(bytes.to_vec()).unwrap())
            .collect();
        terms_back.sort();
        terms.sort();

        for pos in 0..terms.len() {
            assert_eq!(terms[pos], terms_back[pos]);
        }
    }
}
