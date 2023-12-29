use std::iter::FusedIterator;

use crate::game::{Game, DecomposableGame};

/// Grundy's game with associated initial position.
/// 
/// Rules of the game:
/// The starting configuration is a single heap of objects, and the two players
/// take turn splitting a single heap into two heaps of different sizes.
/// See: <https://en.wikipedia.org/wiki/Grundy%27s_game>
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GrundyGame(u16);

impl Game for GrundyGame {
    type Position = u16;
    type NimberSet = [u64; 4];

    #[inline] fn moves_count(&self, position: &Self::Position) -> u16 {
        (position+1) / 2
    }

    #[inline] fn initial_position(&self) -> Self::Position {
        self.0.saturating_sub(2)
    }
}

impl DecomposableGame for GrundyGame {
    type DecomposablePosition = [u16; 2];

    type Successors<'s> = GrundyGameMovesIterator where Self: 's;

    type HeuristicallyOrderedSuccessors<'s> = GrundyGameMovesIterator where Self: 's;

    type Components<'s> = GrundyGameComponentsIterator where Self: 's;

    fn successors(&self, position: &Self::Position) -> Self::Successors<'_> {
        Self::Successors::new(*position)
    }

    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_> {
        Self::HeuristicallyOrderedSuccessors::new(*position)
    }

    fn decompose(&self, position: &Self::DecomposablePosition) -> Self::Components<'_> {
        GrundyGameComponentsIterator(*position)
    }

    fn solver_with_stats<'s, STATS: 's+crate::solver::StatsCollector>(&'s self, stats: STATS) -> Box<dyn crate::solver::SolverForDecomposableGame<Game=Self, StatsCollector=STATS> + 's> {
        todo!()
    }
}

pub struct GrundyGameMovesIterator([u16; 2]);

impl GrundyGameMovesIterator {
    pub fn new(position: u16) -> Self {
        Self([0, position.saturating_sub(1)])
    }
}

impl Iterator for GrundyGameMovesIterator {
    type Item = [u16; 2];

    #[inline] fn next(&mut self) -> Option<Self::Item> {
        (self.0[0] <= self.0[1]).then(|| {
            let mut result = self.0;
            self.0[1] -= 1;
            self.0[0] += 1;
            if result[0] <= 1 { return [result[1], u16::MAX]; }
            result[0] -= 1;
            result
        })
    }
}

impl FusedIterator for GrundyGameMovesIterator {}

pub struct GrundyGameComponentsIterator([u16; 2]);

impl Iterator for GrundyGameComponentsIterator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        (self.0[0] != u16::MAX).then(|| {
            let result = self.0[0];
            self.0[0] = self.0[1];
            self.0[1] = u16::MAX;
            result
        })
    }

    #[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for GrundyGameComponentsIterator {
    #[inline] fn len(&self) -> usize {
        (self.0[0] != u16::MAX) as usize + (self.0[1] != u16::MAX) as usize
    }
}

impl FusedIterator for GrundyGameComponentsIterator {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_zero_game(g0: GrundyGame) {
        let inital_pos = g0.initial_position();
        assert_eq!(inital_pos, 0);
        assert_eq!(g0.moves_count(&inital_pos), 0);
        assert_eq!(g0.successors(&inital_pos).next(), None);
    }

    #[test]
    fn grundy0() {
        test_zero_game(GrundyGame(0));
        test_zero_game(GrundyGame(1));
        test_zero_game(GrundyGame(2));
    }

    #[test]
    fn grundy3() {
        let g3 = GrundyGame(3);
        let inital_pos = g3.initial_position();
        assert_eq!(inital_pos, 1);
        assert_eq!(g3.moves_count(&inital_pos), 1);
        let mut s = g3.successors(&inital_pos);
        assert_eq!(g3.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [0]);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy5() {
        let g5 = GrundyGame(5);
        let inital_pos = g5.initial_position();
        assert_eq!(inital_pos, 3);
        assert_eq!(g5.moves_count(&inital_pos), 2);
        let mut s = g5.successors(&inital_pos);
        assert_eq!(g5.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [2]);
        assert_eq!(g5.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1]);
        assert_eq!(s.next(), None);
    }
}