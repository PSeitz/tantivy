use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::sync::{Arc, Mutex, Weak};

const NUM_SHARDS: usize = 32;
const MIN_LOOKUPS_BETWEEN_CLEANUPS: usize = 64;

/// Deduplicates byte sequences without keeping unused sequences alive.
///
/// Equal bytes share an allocation while any caller owns a strong reference. The pool is
/// thread-safe, compares bytes even on hash collisions, and stores only weak references.
/// Expired metadata is periodically removed on lookup; call [`Self::remove_expired`] to
/// reclaim it while idle. Pool metadata (shards, hash tables, weak handles and expired Arc
/// control blocks) is separate from the conservative payload accounting of arena maps.
/// The byte buffer is freed when its last strong reference is dropped, even if a weak
/// reference remains in the pool.
pub struct ByteDeduplicator {
    hash_builder: RandomState,
    shards: [Mutex<Shard>; NUM_SHARDS],
}

// A shard groups weak byte allocations by hash behind one lock. Cleanup intervals grow
// with the amount of metadata scanned, so repeated full scans cost amortized linear work.
struct Shard {
    allocations_by_hash: HashMap<u64, Vec<Weak<Vec<u8>>>>,
    lookups_until_cleanup: usize,
}

impl Default for Shard {
    fn default() -> Self {
        Self {
            allocations_by_hash: HashMap::new(),
            lookups_until_cleanup: MIN_LOOKUPS_BETWEEN_CLEANUPS,
        }
    }
}

impl Shard {
    fn remove_expired(&mut self) {
        let mut num_live_allocations = 0;
        self.allocations_by_hash.retain(|_, allocations| {
            allocations.retain(|allocation| allocation.strong_count() != 0);
            allocations.shrink_to_fit();
            num_live_allocations += allocations.len();
            !allocations.is_empty()
        });
        self.allocations_by_hash.shrink_to_fit();
        self.lookups_until_cleanup = MIN_LOOKUPS_BETWEEN_CLEANUPS
            .max(num_live_allocations)
            .max(self.allocations_by_hash.capacity());
    }
}

impl Default for ByteDeduplicator {
    fn default() -> Self {
        Self {
            hash_builder: RandomState::new(),
            shards: std::array::from_fn(|_| Mutex::new(Shard::default())),
        }
    }
}

impl ByteDeduplicator {
    /// Creates an empty pool of weak byte allocations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an existing allocation for equal bytes, or allocates a copy.
    pub fn get_or_allocate(&self, bytes: &[u8]) -> Arc<Vec<u8>> {
        self.get_or_allocate_with_hash(bytes, self.hash_builder.hash_one(bytes))
    }

    fn get_or_allocate_with_hash(&self, bytes: &[u8], hash: u64) -> Arc<Vec<u8>> {
        let mut shard = self.shards[hash as usize % NUM_SHARDS].lock().unwrap();
        shard.lookups_until_cleanup -= 1;
        if shard.lookups_until_cleanup == 0 {
            shard.remove_expired();
        }
        let allocations = shard.allocations_by_hash.entry(hash).or_default();
        let mut matching_allocation = None;
        // Remove expired allocations at this hash on every lookup, so repeated allocation
        // and release of the same bytes cannot grow this list between shard cleanups.
        allocations.retain(|weak_allocation| {
            let Some(allocation) = weak_allocation.upgrade() else {
                return false;
            };
            if allocation.as_slice() == bytes {
                matching_allocation = Some(allocation);
            }
            true
        });
        if let Some(allocation) = matching_allocation {
            return allocation;
        }
        let allocation = Arc::new(bytes.to_vec());
        allocations.push(Arc::downgrade(&allocation));
        allocation
    }

    /// Removes expired weak handles and releases unused hash-table capacity.
    ///
    /// Each shard is locked separately. Allocations dropped after their shard was visited
    /// may remain as expired metadata until a later cleanup.
    pub fn remove_expired(&self) {
        for shard in &self.shards {
            shard.lock().unwrap().remove_expired();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};

    use super::{ByteDeduplicator, MIN_LOOKUPS_BETWEEN_CLEANUPS};

    #[test]
    fn equal_bytes_and_hash_collisions() {
        let byte_deduplicator = ByteDeduplicator::new();
        let first = byte_deduplicator.get_or_allocate_with_hash(b"first", 0);
        let second = byte_deduplicator.get_or_allocate_with_hash(b"second", 0);
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(
            &first,
            &byte_deduplicator.get_or_allocate_with_hash(b"first", 0)
        ));
        assert_eq!(&*second, b"second");
        drop(first);
        byte_deduplicator.remove_expired();
        assert_eq!(
            byte_deduplicator.shards[0]
                .lock()
                .unwrap()
                .allocations_by_hash[&0]
                .len(),
            1
        );
        drop(second);
        byte_deduplicator.remove_expired();
        let shard = byte_deduplicator.shards[0].lock().unwrap();
        assert_eq!(shard.allocations_by_hash.capacity(), 0);
    }

    #[test]
    fn automatic_cleanup() {
        let byte_deduplicator = ByteDeduplicator::new();
        drop(byte_deduplicator.get_or_allocate_with_hash(b"expired", 0));
        for _ in 0..MIN_LOOKUPS_BETWEEN_CLEANUPS {
            drop(byte_deduplicator.get_or_allocate_with_hash(b"current", 32));
        }
        let shard = byte_deduplicator.shards[0].lock().unwrap();
        assert!(!shard.allocations_by_hash.contains_key(&0));
        assert!(shard.allocations_by_hash.contains_key(&32));
    }

    #[test]
    fn repeated_expiration_does_not_accumulate_at_one_hash() {
        let byte_deduplicator = ByteDeduplicator::new();
        byte_deduplicator.shards[0]
            .lock()
            .unwrap()
            .lookups_until_cleanup = 4096;
        for _ in 0..1000 {
            drop(byte_deduplicator.get_or_allocate_with_hash(b"repeated", 0));
        }
        let shard = byte_deduplicator.shards[0].lock().unwrap();
        assert_eq!(shard.allocations_by_hash[&0].len(), 1);
    }

    #[test]
    fn cleanup_interval_grows_with_live_metadata() {
        let byte_deduplicator = ByteDeduplicator::new();
        let allocations: Vec<_> = (0u64..4096)
            .map(|i| byte_deduplicator.get_or_allocate_with_hash(&i.to_le_bytes(), i * 32))
            .collect();
        byte_deduplicator.remove_expired();
        assert!(
            byte_deduplicator.shards[0]
                .lock()
                .unwrap()
                .lookups_until_cleanup
                >= allocations.len()
        );
    }

    #[test]
    fn concurrent_equal_bytes() {
        let byte_deduplicator = ByteDeduplicator::new();
        let barrier = Barrier::new(16);
        std::thread::scope(|scope| {
            let threads: Vec<_> = (0..16)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        byte_deduplicator.get_or_allocate(b"same bytes")
                    })
                })
                .collect();
            let allocations: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
            for allocation in &allocations {
                assert!(Arc::ptr_eq(&allocations[0], allocation));
            }
        });
    }

    #[test]
    fn arbitrary_bytes_and_expiration() {
        let byte_deduplicator = ByteDeduplicator::new();
        for bytes in [vec![], vec![0, 255, 0, 128], vec![42; 100_000]] {
            let allocation = byte_deduplicator.get_or_allocate(&bytes);
            assert_eq!(*allocation, bytes);
            assert!(Arc::ptr_eq(
                &allocation,
                &byte_deduplicator.get_or_allocate(&bytes)
            ));
            let weak = Arc::downgrade(&allocation);
            drop(allocation);
            assert!(weak.upgrade().is_none());
            assert_eq!(*byte_deduplicator.get_or_allocate(&bytes), bytes);
        }
        byte_deduplicator.remove_expired();
        assert!(
            byte_deduplicator
                .shards
                .iter()
                .all(|shard| { shard.lock().unwrap().allocations_by_hash.is_empty() })
        );
    }
}
