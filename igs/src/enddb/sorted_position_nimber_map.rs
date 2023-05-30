pub use crate::dbs::{NimbersProvider, NimbersStorer, HasLen};

/// Position -> Nimbers map stored in two vectors: positions (sorted) and nimbers.
/// Effective only when positions are added in ascendent order.
#[derive(Clone, Default)]
pub struct SortedPositionNimberMap<Position> {
    pub positions: Vec<Position>,
    pub nimbers: Vec<u8>
}

impl<Position> SortedPositionNimberMap<Position> {

    #[inline] pub fn new() -> Self {
        Self { positions: Vec::new(), nimbers: Vec::new() }
    }

    pub fn iter(&self) -> std::iter::Zip<std::slice::Iter<'_, Position>, std::slice::Iter<'_, u8>> {
        self.positions.iter().zip(self.nimbers.iter())
    }

    pub fn iter_mut(&mut self) -> std::iter::Zip<std::slice::IterMut<'_, Position>, std::slice::IterMut<'_, u8>> {
        self.positions.iter_mut().zip(self.nimbers.iter_mut())
    }

    pub fn values(&self) -> std::slice::Iter<'_, u8> {
        self.nimbers.iter()
    }
}

impl<'a, Position> IntoIterator for &'a SortedPositionNimberMap<Position> {
    type Item = (&'a Position, &'a u8);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, Position>, std::slice::Iter<'a, u8>>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, Position> IntoIterator for &'a mut SortedPositionNimberMap<Position> {
    type Item = (&'a mut Position, &'a mut u8);
    type IntoIter = std::iter::Zip<std::slice::IterMut<'a, Position>, std::slice::IterMut<'a, u8>>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<Position> IntoIterator for SortedPositionNimberMap<Position> {
    type Item = (Position, u8);
    type IntoIter = std::iter::Zip<std::vec::IntoIter<Position>, std::vec::IntoIter<u8>>;
    fn into_iter(self) -> Self::IntoIter { self.positions.into_iter().zip(self.nimbers.into_iter()) }
}

impl<Position: Ord> NimbersProvider<Position> for SortedPositionNimberMap<Position> {
    #[inline]
    fn get_nimber(&self, position: &Position) -> Option<u8> {
        Some(self.nimbers[self.positions.binary_search(position).ok()?])
    }
}

impl<Position: Ord> NimbersStorer<Position> for SortedPositionNimberMap<Position> {
    #[inline]
    fn store_nimber(&mut self, position: Position, nimber: u8) {
        if self.positions.last().map_or(true, |l| *l < position) {
            self.positions.push(position);
            self.nimbers.push(nimber);
        } else if let Err(index) = self.positions.binary_search(&position) {
            self.positions.insert(index, position);
            self.nimbers.insert(index, nimber);
        }
    }
}

impl<P> HasLen for SortedPositionNimberMap::<P> {
    #[inline(always)] fn len(&self) -> usize { self.positions.len() }
}