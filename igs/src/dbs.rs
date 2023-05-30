use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[cfg(feature = "lru")] use lru::LruCache;

/// Provide nimbers.
pub trait NimbersProvider<GamePosition> {
    /// Returns nimber of the given `position` or `None` if `self` does not know the nimber.
    fn get_nimber(&self, position: &GamePosition) -> Option<u8>;

    /// Returns nimber of the given `position` or `None` if `self` does not know the nimber.
    ///
    /// This method usually just calls `get_nimber` (which is the default implementation),
    /// but self-organization structures, like LRU-caches, change `self` as well.
    #[inline(always)] fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.get_nimber(position)
    }
}

/// Store nimbers.
pub trait NimbersStorer<GamePosition>: NimbersProvider<GamePosition> {
    /// Saves `nimber` of the given `position`.
    fn store_nimber(&mut self, position: GamePosition, nimber: u8);
}

impl<GamePosition: Eq + Hash> NimbersProvider<GamePosition> for HashMap<GamePosition, u8> {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.get(position).map(|v| *v)
    }
}

impl<GamePosition: Eq + Hash> NimbersStorer<GamePosition> for HashMap<GamePosition, u8> {
    #[inline(always)]
    fn store_nimber(&mut self, position: GamePosition, nimber: u8) {
        self.insert(position, nimber);
    }
}

impl<GamePosition: Ord> NimbersProvider<GamePosition> for BTreeMap<GamePosition, u8> {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.get(position).map(|v| *v)
    }
}

impl<GamePosition: Ord> NimbersStorer<GamePosition> for BTreeMap<GamePosition, u8> {
    #[inline(always)]
    fn store_nimber(&mut self, position: GamePosition, nimber: u8) {
        self.insert(position, nimber);
    }
}

#[cfg(feature = "lru")] impl<GamePosition: Eq + Hash> NimbersProvider<GamePosition> for LruCache<GamePosition, u8> {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.peek(position).map(|v| *v)
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.get(position).map(|v| *v)
    }
}

#[cfg(feature = "lru")] impl<GamePosition: Eq + Hash> NimbersStorer<GamePosition> for LruCache<GamePosition, u8> {
    #[inline(always)]
    fn store_nimber(&mut self, position: GamePosition, nimber: u8) {
        self.insert(position, nimber);
    }
}

impl<GamePosition> NimbersProvider<GamePosition> for () {
    #[inline(always)]
    fn get_nimber(&self, _position: &GamePosition) -> Option<u8> {
        None
    }
}

impl<GamePosition, DB1: NimbersProvider<GamePosition>> NimbersProvider<GamePosition> for (DB1,) {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber(position)
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber_and_self_organize(position)
    }
}

// Thanks to this two providers given as a tuple can constitute end_db.
impl<GamePosition, DB1: NimbersProvider<GamePosition>, DB2: NimbersProvider<GamePosition>> NimbersProvider<GamePosition>
for (DB1, DB2) {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber(position)
            .or_else(|| {self.1.get_nimber(position)})
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber_and_self_organize(position)
            .or_else(|| {self.1.get_nimber_and_self_organize(position)})
    }
}

// Thanks to this three providers given as a tuple can constitute end_db.
impl<GamePosition, DB1: NimbersProvider<GamePosition>, DB2: NimbersProvider<GamePosition>, DB3: NimbersProvider<GamePosition>> NimbersProvider<GamePosition>
for (DB1, DB2, DB3) {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber(position)
            .or_else(|| {self.1.get_nimber(position)})
            .or_else(|| {self.2.get_nimber(position)})
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber_and_self_organize(position)
            .or_else(|| {self.1.get_nimber_and_self_organize(position)})
            .or_else(|| {self.2.get_nimber_and_self_organize(position)})
    }
}

// Thanks to this four providers given as a tuple can constitute end_db.
impl<GamePosition, DB1: NimbersProvider<GamePosition>, DB2: NimbersProvider<GamePosition>, DB3: NimbersProvider<GamePosition>, DB4: NimbersProvider<GamePosition>> NimbersProvider<GamePosition>
for (DB1, DB2, DB3, DB4) {
    #[inline(always)]
    fn get_nimber(&self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber(position)
            .or_else(|| {self.1.get_nimber(position)})
            .or_else(|| {self.2.get_nimber(position)})
            .or_else(|| {self.3.get_nimber(position)})
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &GamePosition) -> Option<u8> {
        self.0.get_nimber_and_self_organize(position)
            .or_else(|| {self.1.get_nimber_and_self_organize(position)})
            .or_else(|| {self.2.get_nimber_and_self_organize(position)})
            .or_else(|| {self.3.get_nimber_and_self_organize(position)})
    }
}

impl<GamePosition> NimbersStorer<GamePosition> for () {
    #[inline(always)]
    fn store_nimber(&mut self, _position: GamePosition, _nimber: u8) {
        // Do nothing
    }
}

pub trait HasLen {
    fn len(&self) -> usize;
}

impl<K, V, S> HasLen for HashMap::<K, V, S> {
    #[inline(always)] fn len(&self) -> usize { HashMap::<K, V, S>::len(self) }
}

impl<K, V> HasLen for BTreeMap<K, V> {
    #[inline(always)] fn len(&self) -> usize { BTreeMap::<K, V>::len(self) }
}

impl HasLen for () {
    #[inline(always)] fn len(&self) -> usize { 0 }
}


