use core::ptr;

pub use protected::ProtectedTT;

pub use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::dbs::HasLen;

pub mod bit_mixer;

mod protected;

const EMPTY_ENTRY: u32 = u32::MAX;

/// Clusters configuration.
pub struct ClusterConf {
    pub id_mask: u32,
    pub id_size: u8,
    pub capacity: u8,
    pub max_nimber: u8,
}

impl ClusterConf {

    pub fn new_log2(cluster_capacity_log2: u8, bits_per_nimber: u8) -> Self {
        let in_cluster_key_size = 32 - bits_per_nimber; // liczba bitów klucza przechowywanych we wpisach
        Self {
            id_mask: (1u32 << in_cluster_key_size).wrapping_sub(1),
            id_size: in_cluster_key_size,
            capacity: 1u8 << cluster_capacity_log2,
            max_nimber: (1u8 << bits_per_nimber).wrapping_sub(1),
        }
    }

    /// Returns id stored in given cluster's entry or for given key (casted to `u32`).
    #[inline(always)] fn id(&self, entry_or_key: u32) -> u32 {
        entry_or_key & self.id_mask
    }

    /// Returns nimber stored in given `entry`.
    #[inline(always)] fn nimber(&self, entry: u32) -> u8 {
        (entry >> self.id_size) as u8
    }

    /// Returns cluster's entry for given `key` and `nimber`.
    #[inline(always)] fn entry(&self, key: u64, nimber: u8) -> u32 {
        ((nimber as u32) << self.id_size) | (key as u32 & self.id_mask)
    }
}

/// Clusters update and lookup policy. The module `cluster_policy` includes many implementations.
pub trait ClusterPolicy {
    /// Conditionally stores `to_store` (which includes given nimber) in the given `cluster`.
    ///
    /// Default implementation inserts `to_store` at the beginning of `cluster`
    /// and shifts the content one position up, discarding the last entry.
    fn store_entry(&mut self, _cluster_conf: &ClusterConf, cluster: &mut [u32], to_store: u32, _nimber: u8) {
        unsafe {
            let p = cluster.as_mut_ptr();
            ptr::copy(p, p.offset(1), cluster.len() - 1);
            ptr::write(p, to_store);
        }
        /*for p in (1..cluster.len()).rev() {
            cluster[p] = cluster[p-1];
        }
        cluster[0] = to_store;*/
    }

    /// Lookups for the nimber for the given `id_to_find` entry id in the given `cluster`.
    #[inline(always)] fn get_nimber(&self, cluster_conf: &ClusterConf, cluster: &[u32], id_to_find: u32) -> Option<u8> {
        for e in cluster {
            if *e == EMPTY_ENTRY { return None; }
            if cluster_conf.id(*e) == id_to_find {
                return Some(cluster_conf.nimber(*e));
            }
        }
        None
    }

    /// Lookups for the nimber for the given `id_to_find` entry id in the given `cluster`.
    /// Optionally self-organize the `cluster`.
    ///
    /// Default implementation just calls `get_nimber`.
    #[inline(always)]
    fn get_nimber_and_self_organize(&self, cluster_conf: &ClusterConf, cluster: &mut [u32], id_to_find: u32) -> Option<u8> {
        self.get_nimber(cluster_conf, cluster, id_to_find)
    }
}

pub mod cluster_policy {
    use core::ptr;

    use super::{ClusterConf, ClusterPolicy, EMPTY_ENTRY};

    pub struct Fifo;
    impl ClusterPolicy for Fifo {}

    pub struct FifoLru;
    impl ClusterPolicy for FifoLru {
        #[inline(always)]
        fn get_nimber_and_self_organize(&self, cluster_conf: &ClusterConf, cluster: &mut [u32], id_to_find: u32) -> Option<u8> {
            for i in 0..cluster.len() {
                let e = cluster[i];
                if e == EMPTY_ENTRY { return None; }
                if cluster_conf.id(e) == id_to_find {
                    if i != 0 {
                        cluster[i] = cluster[i-1];
                        cluster[i-1] = e;
                    }
                    return Some(cluster_conf.nimber(e));
                }
            }
            None
        }
    }

    pub struct Lru;
    impl ClusterPolicy for Lru {
        #[inline(always)]
        fn get_nimber_and_self_organize(&self, cluster_conf: &ClusterConf, cluster: &mut [u32], id_to_find: u32) -> Option<u8> {
            for i in 0..cluster.len() {
                let e = cluster[i];
                if e == EMPTY_ENTRY { return None; }
                if cluster_conf.id(e) == id_to_find {
                    if i != 0 { unsafe {
                            let p = cluster.as_mut_ptr();
                            ptr::copy(p, p.offset(1), i);
                            ptr::write(p, e);
                    } }
                    return Some(cluster_conf.nimber(e));
                }
            }
            None
        }
    }

    pub struct LowestNimbers;

    impl ClusterPolicy for LowestNimbers {
        #[inline(always)] fn store_entry(&mut self, cluster_conf: &ClusterConf, cluster: &mut [u32], to_store: u32, nimber: u8) {
            nimbers_store_entry(cluster_conf, cluster, to_store, |stored| nimber <= stored)
        }
    }

    pub struct LargestNimbers;

    impl ClusterPolicy for LargestNimbers {
        #[inline(always)] fn store_entry(&mut self, cluster_conf: &ClusterConf, cluster: &mut [u32], to_store: u32, nimber: u8) {
            nimbers_store_entry(cluster_conf, cluster, to_store, |stored| nimber >= stored)
        }
    }

    /// `store_nimber` implementation for `LowestNimbers` and `GreatestNimbers`.
    /// `should_be_stored_before` should return `true` only if the nimber added is preferred over the nimber in the argument.
    fn nimbers_store_entry<Compare: Fn(u8) -> bool>(cluster_conf: &ClusterConf, cluster: &mut [u32], to_store: u32, should_be_stored_before: Compare) {
        for i in 0..cluster.len() {
            let e = cluster[i];
            if e == EMPTY_ENTRY {
                cluster[i] = to_store;
                return;
            }
            if should_be_stored_before(cluster_conf.nimber(e)) {
                unsafe {
                    //cluster.copy_within(i.., i+1);
                    //cluster[i] =
                    let p = cluster.as_mut_ptr().offset(i as _);
                    ptr::copy(p, p.offset(1), cluster.len()-1-i);
                    ptr::write(p, to_store);
                }
                return;
            }
        }
    }

    #[derive(Default, Copy, Clone)]
    pub struct BalancedRandom { index: u32 }

    impl ClusterPolicy for BalancedRandom {
        fn store_entry(&mut self, _cluster_conf: &ClusterConf, cluster: &mut [u32], to_store: u32, _nimber: u8) {
            let mut i = cluster.len() - 1;
            if cluster[i] == EMPTY_ENTRY {  // we have empty entries
                while i != 0 {
                    i -= 1;
                    if cluster[i] != EMPTY_ENTRY {
                        cluster[i+1] = to_store;
                        return;
                    }
                }
                cluster[0] = to_store;  // all entries are empty here
            } else {    // full cluster, overwriting:
                cluster[self.index as usize] = to_store;
                self.index = (self.index.wrapping_add(1)) % cluster.len() as u32;
            }
        }
    }
}

/// Succinct transposition table for 64-bit positions.
///
/// It uses only 32-bit to encode position and nimber, but has some limitations.
/// It stores only information about the position whose nimber is less than or equal to `max_nimber`.
/// Position are identified by a fragment stored in the entry and the index of the entry - strictly speaking
/// index of the cluster assigned to the position. So, when the cluster is full,
/// more positions assigned to the cluster cannot be stored, even if the whole table is not full
/// (then LIFO strategy is used).
pub struct TTSuccinct64<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy = cluster_policy::Fifo> {
    data: Box<[u32]>,
    /// Clusters configuration.
    cluster_conf: ClusterConf,
    /// maximal position number which can be stored (larger are ignored)
    key_mask: u64,
    /// mix_bits(position, mask) is bijection that returns position with mixed bits shown by 0..01..1 mask (key_mask)
    mix_bits: BitMixer,
    /// Used to update clusters or search in clusters.
    cluster_policy: Policy
}

impl<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy> TTSuccinct64<BitMixer, Policy> {

    /// Construct TTSuccinct64 which stores `2` to power of `capacity_log2` entries.
    /// Each entry has `4` bytes, and the result consumes `2` to power `capacity_log2+2` bytes.
    /// Each entry encodes nimber (using `bits_per_nimber` bits) and `32-bits_per_nimber` bits of position id.
    /// Entries are grouped in clusters, and each cluster stores `2` to power `cluster_capacity_log2` entries.
    pub fn new(capacity_log2: u8, cluster_capacity_log2: u8, bits_per_nimber: u8, bit_mixer: BitMixer, cluster_policy: Policy) -> Self {
        assert!(capacity_log2 >= cluster_capacity_log2);
        assert!(bits_per_nimber <= 8);
        let cluster_conf = ClusterConf::new_log2(cluster_capacity_log2, bits_per_nimber);
        let clusters_num_log2 = capacity_log2 - cluster_capacity_log2;  // log2 z liczby klastrów
        let bits_per_key = clusters_num_log2 + cluster_conf.id_size;    // całkowita liczba bitów użyta z klucza
        assert!(bits_per_key <= 64);
        Self {
            data: vec![EMPTY_ENTRY; 1usize<< capacity_log2].into_boxed_slice(),
            key_mask: (1u64 << bits_per_key).wrapping_sub(1),
            cluster_conf,
            mix_bits: bit_mixer,
            cluster_policy
        }
    }

    /// Returns the capacity of the table (total number of entries).
    pub fn capacity(&self) -> usize { self.data.len() }

    /// Returns first index of the cluster for the given `key`.
    #[inline(always)] fn cluster_begin(&self, key: u64) -> usize {
        ((key >> self.cluster_conf.id_size) as usize) * (self.cluster_conf.capacity as usize)
    }

    /// Returns the cluster for the given `key`.
    #[inline(always)] fn cluster(&self, key: u64) -> &[u32] {
        let cl_beg = self.cluster_begin(key);
        &self.data[cl_beg..cl_beg+(self.cluster_conf.capacity as usize)]
    }

    fn pos_id_and_cluster(&self, position: u64) -> (u32, &[u32]) {
        let key = (self.mix_bits)(position, self.key_mask);
        (self.cluster_conf.id(key as u32), &self.cluster(key))
    }
}

impl<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy> HasLen for TTSuccinct64<BitMixer, Policy> {
    /// Returns the number of elements in the table (number of occupied entries).
    fn len(&self) -> usize {
        self.data.iter().filter(|e| **e != EMPTY_ENTRY).count()
    }
}

impl<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy> NimbersProvider<u64> for TTSuccinct64<BitMixer, Policy> {
    fn get_nimber(&self, position: &u64) -> Option<u8> {
        if *position > self.key_mask { return None; }
        let (id_to_find, cluster) = self.pos_id_and_cluster(*position);
        self.cluster_policy.get_nimber(&self.cluster_conf, cluster, id_to_find)
    }

    fn get_nimber_and_self_organize(&mut self, position: &u64) -> Option<u8> {
        if *position > self.key_mask { return None; }
        let key = (self.mix_bits)(*position, self.key_mask);
        let id_to_find = self.cluster_conf.id(key as u32);
        let cl_beg = self.cluster_begin(key);
        self.cluster_policy.get_nimber_and_self_organize(
            &self.cluster_conf,
            &mut self.data[cl_beg..cl_beg+(self.cluster_conf.capacity as usize)],
            id_to_find)
    }
}

impl<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy> NimbersStorer<u64> for TTSuccinct64<BitMixer, Policy> {
    fn store_nimber(&mut self, position: u64, nimber: u8) {
        if position > self.key_mask || nimber > self.cluster_conf.max_nimber { return; }
        let key = (self.mix_bits)(position, self.key_mask);
        let to_store = self.cluster_conf.entry(key, nimber);
        if to_store == EMPTY_ENTRY { return; }
        let cl_beg = self.cluster_begin(key);
        self.cluster_policy.store_entry(
            &self.cluster_conf,
            &mut self.data[cl_beg..cl_beg+(self.cluster_conf.capacity as usize)],
            to_store, nimber)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_succinct64() {
        let mut tt = TTSuccinct64::new(4, 2, 2, bit_mixer::stafford13, cluster_policy::Fifo);
        tt.store_nimber(1, 0);
        tt.store_nimber(3, 1);
        tt.store_nimber(4, 2);
        tt.store_nimber(5, 4);  // nimber too large, should be ignored
        assert_eq!(tt.get_nimber(&1), Some(0));
        assert_eq!(tt.get_nimber(&3), Some(1));
        assert_eq!(tt.get_nimber(&4), Some(2));
        assert_eq!(tt.get_nimber(&5), None);
        assert_eq!(tt.capacity(), 16);
        assert_eq!(tt.len(), 3);
    }

    /// Constructs the cluster with entries nimbers: 0, 1, 2, .., 15; 4 times each
    fn construct_cluster(policy: &mut dyn ClusterPolicy) -> (Vec::<u32>, ClusterConf) {
        let mut cluster = vec![EMPTY_ENTRY; 8];
        let cluster_conf = ClusterConf::new_log2(2, 4);
        for id in 1..65 {
            let nimber = (id%16) as u8;
            policy.store_entry(&cluster_conf, &mut cluster, cluster_conf.entry(id, nimber), nimber);
            assert_eq!(cluster.iter().filter(|e| **e!=EMPTY_ENTRY).count(), id.min(cluster.len() as _) as _);
        }
        (cluster, cluster_conf)
    }

    /// Test policy and expect to latest added entries should be included in the cluster.
    fn test_policy_latest(policy: &mut dyn ClusterPolicy) {
        let (mut cluster, cluster_conf) = construct_cluster(policy);
        for id in 1..65 {
            let nimber = (id%16) as u8;
            let result = policy.get_nimber_and_self_organize(&cluster_conf, &mut cluster, id);
            if id >= 65-8 {
                assert_eq!(result, Some(nimber));
            } else {
                assert!(result.is_none());
            }
        }
    }

    #[test]
    fn policy_fifo() {
        test_policy_latest(&mut cluster_policy::Fifo);
    }

    #[test]
    fn policy_balanced_random() {
        test_policy_latest(&mut cluster_policy::BalancedRandom::default());
    }

    #[test]
    fn policy_lowest_nimbers() {
        let mut policy = cluster_policy::LowestNimbers;
        let (mut cluster, cluster_conf) = construct_cluster(&mut policy);
        for id in 1..65 {
            let nimber = (id%16) as u8;
            let result = policy.get_nimber_and_self_organize(&cluster_conf, &mut cluster, id);
            if nimber <= 1 {
                assert_eq!(result, Some(nimber));
            } else {
                assert!(result.is_none());
            }
        }
    }

    #[test]
    fn policy_largest_nimbers() {
        let mut policy = cluster_policy::LargestNimbers;
        let (mut cluster, cluster_conf) = construct_cluster(&mut policy);
        for id in 1..65 {
            let nimber = (id%16) as u8;
            let result = policy.get_nimber_and_self_organize(&cluster_conf, &mut cluster, id);
            if nimber >= 14 {
                assert_eq!(result, Some(nimber));
            } else {
                assert!(result.is_none());
            }
        }
    }
}