//! Grundy's game - an impartial game in which the starting configuration is a single heap
//! of objects, and the two players take turns splitting a single heap into two heaps
//! of different sizes.
//!
//! See: <https://en.wikipedia.org/wiki/Grundy%27s_game>
use std::{iter::FusedIterator, collections::HashMap};

use crate::{game::{Game, DecomposableGame}, solver::{dedicated::DefSolver, Solver}};

/// Grundy's game with associated initial position.
/// 
/// Rules of the game:
/// The starting configuration is a single heap of objects, and the two players
/// take turn splitting a single heap into two heaps of different sizes.
/// See: <https://en.wikipedia.org/wiki/Grundy%27s_game>
///
/// A heap of `v + 2` objects is represented by the number `v` (in particular, the initial
/// position of `GrundyGame(n)` is the heap of `n` objects represented as `n - 2`).
/// Heaps of 1 and 2 objects, which are equivalent to the empty game (their nimbers are 0),
/// may be omitted from the representations of moves and decomposable positions.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GrundyGame(pub u16);

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

    fn solver_with_stats<'s, STATS: 's+crate::solver::StatsCollector>(&'s self, stats: STATS) -> Box<dyn crate::solver::SolverForDecomposableGame<Game=Self, StatsCollector=STATS> + 's>
    {
        Box::new(DefSolver{solver: Solver::new(self, HashMap::new(), (), (), stats)})
    }
    
}

/// Iterator over the moves (successors) of a Grundy's game position (a single heap).
///
/// Each move splits the heap into two heaps of different sizes and is represented as
/// a pair `[a, b]` of the values of the resulting heaps (see [`GrundyGame`]),
/// sorted so that `a <= b`. Heaps of 1 and 2 objects, which are equivalent to the empty game
/// (their nimbers are 0), may be omitted; a move with a single resulting heap is represented
/// as `[a, u16::MAX]`.
pub struct GrundyGameMovesIterator([u16; 2]);

impl GrundyGameMovesIterator {
    /// Constructs the iterator over the moves (successors) of the given `position` (a single heap).
    pub fn new(position: u16) -> Self {
        Self([0, position])
    }
}

impl Iterator for GrundyGameMovesIterator {
    type Item = [u16; 2];

    #[inline] fn next(&mut self) -> Option<Self::Item> {
        (self.0[0] < self.0[1]).then(|| {
            self.0[1] -= 1;
            let mut result = self.0;
            self.0[0] += 1;
            if result[0] <= 1 { return [result[1], u16::MAX]; }
            result[0] -= 1;
            result
        })
    }

    #[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for GrundyGameMovesIterator {
    #[inline] fn len(&self) -> usize {
        ((self.0[1] + 1 - self.0[0]) / 2) as usize
    }
}

impl FusedIterator for GrundyGameMovesIterator {}

/// Iterator over the components of a decomposable Grundy's game position.
///
/// The position `[a, b]` consists of the heaps `a` and `b` (see [`GrundyGame`]);
/// the second heap is absent if `b` equals `u16::MAX`.
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

    fn test_zero_game(g: GrundyGame) {
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 0);
        assert_eq!(g.moves_count(&inital_pos), 0);
        assert_eq!(g.successors(&inital_pos).next(), None);
    }

    #[test]
    fn grundy0() {
        test_zero_game(GrundyGame(0));
        test_zero_game(GrundyGame(1));
        test_zero_game(GrundyGame(2));
    }

    #[test]
    fn grundy3() {
        let g = GrundyGame(3);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 1);
        assert_eq!(g.moves_count(&inital_pos), 1);
        let mut s = g.successors(&inital_pos);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [0]);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy4() {
        let g = GrundyGame(4);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 2);
        assert_eq!(g.moves_count(&inital_pos), 1);
        let mut s = g.successors(&inital_pos);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1]);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy5() {
        let g = GrundyGame(5);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 3);
        assert_eq!(g.moves_count(&inital_pos), 2);
        let mut s = g.successors(&inital_pos);
        assert_eq!(s.len(), 2);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [2]);
        assert_eq!(s.len(), 1);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1]);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy7() {
        let g = GrundyGame(7);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 5);
        assert_eq!(g.moves_count(&inital_pos), 3);
        let mut s = g.successors(&inital_pos);
        assert_eq!(s.len(), 3);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [4]);
        assert_eq!(s.len(), 2);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [3]);
        assert_eq!(s.len(), 1);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy8() {
        let g = GrundyGame(8);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 6);
        assert_eq!(g.moves_count(&inital_pos), 3);
        let mut s = g.successors(&inital_pos);
        assert_eq!(s.len(), 3);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [5]);
        assert_eq!(s.len(), 2);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [4]);
        assert_eq!(s.len(), 1);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1, 3]);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn grundy9() {
        let g = GrundyGame(9);
        let inital_pos = g.initial_position();
        assert_eq!(inital_pos, 7);
        assert_eq!(g.moves_count(&inital_pos), 4);
        let mut s = g.successors(&inital_pos);
        assert_eq!(s.len(), 4);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [6]);
        assert_eq!(s.len(), 3);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [5]);
        assert_eq!(s.len(), 2);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [1, 4]);
        assert_eq!(s.len(), 1);
        assert_eq!(g.decompose(&s.next().unwrap()).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), None);
    }
}