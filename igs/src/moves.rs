//! Move sorting facilities: traits for sorting (and removing) moves in the order
//! from the easiest to the most difficult to solve, and [`DifficultEvaluator`]
//! which maps positions to such (ordered) difficulty measure.

use crate::game::{Game, SimpleGame, DecomposableGame};
use co_sort::Permutation;

/// Sorts moves of a simple game, used by the solver to prune a larger part of the search tree.
///
/// One can implement [`DifficultEvaluator`] instead of implementing `SimpleGameMoveSorter` directly.
pub trait SimpleGameMoveSorter<G> where G: SimpleGame {

    /// Sorts moves from the easiest to the most difficult.
    fn sort_moves(&self, game: &G, moves: &mut [<G as Game>::Position]);

    /// Removes `index`-th item from `moves`.
    /// Default implementation calls `moves.remove(index)`.
    /// However, if the order of moves does not need to be preserved (as when sort_moves does nothing), faster removal can be performed.
    fn remove(moves: &mut Vec<<G as Game>::Position>, index: usize) {
        moves.remove(index);
    }
}

/// Information about a decomposed position (move), i.e. about the components of the successor
/// of a (non-decomposable) position of a game with decomposable positions.
///
/// It describes a slice of the vector of components of many moves (which is passed to
/// [`DecomposableGameMoveSorter::sort_moves`] together with the vector of `ComponentsInfo`):
/// the components at indices `first..first+len` belong to the move, and `nimber`
/// is the xored nimber of the components whose nimbers were already known
/// (those components are not present in the slice).
#[derive(Copy, Clone)]
pub struct ComponentsInfo {
    /// Index of the first component (of decomposable position represented by `self`) in the vector of components.
    pub first: usize,

    /// Number of components (of decomposable position represented by `self`) in the vector of components.
    pub len: usize,

    /// Nimber of removed components of decomposable position represented by `self`,
    /// i.e. the xor of the nimbers of the components whose nimbers were already known
    /// (those components are not present in the vector of components).
    pub nimber: u8
}

impl ComponentsInfo {
    /// Constructs info for a move whose (not yet known) components start at index `first`.
    #[inline(always)]
    pub fn new(first: usize) -> Self {
        Self{ first, len: 0, nimber: 0 }
    }

    /// Returns the slice of `all` that contains the components of the move represented by `self`.
    #[inline(always)]
    pub fn as_slice<'a, T>(&self, all: &'a [T]) -> &'a [T] {
        &all[self.first..self.first+self.len]
    }

    /// Returns the mutable slice of `all` that contains the components of the move represented by `self`.
    #[inline(always)]
    pub fn as_slice_mut<'a, T>(&self, all: &'a mut [T]) -> &'a mut [T] {
        &mut all[self.first..self.first+self.len]
    }
}

/// Sorts moves of a decomposable game, used by the solver to prune larger part of the search tree.
pub trait DecomposableGameMoveSorter<G> where G: DecomposableGame {

    /// Sorts moves from the easiest to the most difficult.
    /// Additionally, moves the most difficult component of each decomposed move to the first index of its slice.
    fn sort_moves(&self, game: &G,
                  moves: &mut [ComponentsInfo],
                  move_components: &mut [<G as Game>::Position]
    );

    /// Removes `index`-th item from `moves`.
    /// Default implementation calls `moves.remove(index)`.
    /// However, if the order of moves does not need to be preserved (as when sort_moves does nothing), faster removal can be performed.
    #[inline(always)]
    fn remove(moves: &mut Vec<ComponentsInfo>, index: usize) {
        moves.remove(index);
    }
}

/// Evaluates how difficult (hard to solve) a game position is.
/// The larger the returned value, the more difficult the position is.
///
/// Every implementor of this trait is automatically a move sorter
/// for simple games ([`SimpleGameMoveSorter`]) and/or decomposable games
/// ([`DecomposableGameMoveSorter`]): it sorts moves in the order
/// of increasing values returned by `difficult_of`.
pub trait DifficultEvaluator {
    /// The game which positions are evaluated.
    type Game: Game;
    /// Type of the evaluation result; positions are sorted by the increasing value of this type.
    type PositionDifficult: Ord;
    /// Returns the difficulty of the position `to_evaluate`.
    fn difficult_of(&self, game: &Self::Game, to_evaluate: &<Self::Game as Game>::Position) -> Self::PositionDifficult;
}

impl<DE> SimpleGameMoveSorter<DE::Game> for DE
    where DE: DifficultEvaluator,
          DE::Game: SimpleGame,
//impl<G: SimpleGame, DE: DifficultEvaluator<G>> SimpleGameMoveSorter<G> for DE
{
    fn sort_moves(&self, game: &DE::Game, moves: &mut [<DE::Game as Game>::Position]) {
        //moves.sort_by_key(|m| { self.difficult_of(game, m) });
        // TODO sort_by_cached_key ? sort_unstable_by_key ?
        Permutation
        ::from(moves.iter().map(|m| { self.difficult_of(game, m) }).collect::<Vec<_>>().as_ref())
            .co_sort(&mut moves[..]);
    }
}

impl<DE> DecomposableGameMoveSorter<DE::Game> for DE
    where DE: DifficultEvaluator,
          DE::Game: DecomposableGame,
          DE::PositionDifficult: Default + std::ops::AddAssign + Clone
//impl<G: DecomposableGame, PD: Ord + Default + std::ops::AddAssign + Clone> DecomposableGameMoveSorter<G> for DifficultEvaluator<G, PositionDifficult=PD>
{
    /// Sorts moves by increasing sum of difficulties of their components
    /// (a move with a single component is sorted by the difficulty of that component).
    /// Additionally, within each multi-component move, the most difficult component is moved
    /// to the first index of the move's slice.
    fn sort_moves(&self, game: &DE::Game,
                  moves: &mut [ComponentsInfo],
                  move_components: &mut [<DE::Game as Game>::Position]
    ) {
        /*moves.sort_by_cached_key(|m| {
            match m.len {
                0 => { DE::PositionDifficult::default() }, // TODO niemożliwe, przynajmniej w LV
                1 => { // speed optimization, this is very common case that requires less work
                    self.difficult_of(game, &move_components[m.first])
                },
                _ => {  // 2 or more components
                    let mut difficult_max = self.difficult_of(game, &move_components[m.first]);
                    let mut total_difficult = difficult_max.clone();
                    for i in m.first+1..m.first+m.len {
                        let i_difficult = self.difficult_of(game, &move_components[i]);
                        total_difficult += i_difficult.clone();
                        if i_difficult > difficult_max {
                            move_components.swap(m.first, i); // most difficult goes to begin
                            difficult_max = i_difficult;
                        }
                    }
                    total_difficult
                }
            }
        });*/

        Permutation
        ::from(moves.iter().map(|m| {
            match m.len {
                0 => { DE::PositionDifficult::default() }, // TODO niemożliwe, przynajmniej w LV
                1 => { // speed optimization, this is very common case that requires less work
                    self.difficult_of(game, &move_components[m.first])
                },
                _ => {  // 2 or more components
                    let mut difficult_max = self.difficult_of(game, &move_components[m.first]);
                    let mut total_difficult = difficult_max.clone();
                    for i in m.first+1..m.first+m.len {
                        let i_difficult = self.difficult_of(game, &move_components[i]);
                        total_difficult += i_difficult.clone();
                        if i_difficult > difficult_max {
                            move_components.swap(m.first, i); // most difficult goes to begin
                            difficult_max = i_difficult;
                        }
                    }
                    total_difficult
                }
            }
        }).collect::<Vec<_>>().as_ref()).co_sort(&mut moves[..]);
    }
}

/// Move sorter that (initially) preserves the order in which moves were generated by the game methods.
/// It removes moves with `swap_remove`, which is O(1) but does not preserve order.
pub struct PreserveGeneratedOrder;

impl<G> SimpleGameMoveSorter<G> for PreserveGeneratedOrder where G: SimpleGame {
    #[inline(always)]
    fn sort_moves(&self, _game: &G, _moves: &mut [<G as Game>::Position]) {
        // do nothing
    }
}

impl<G> DecomposableGameMoveSorter<G> for PreserveGeneratedOrder where G: DecomposableGame {
    #[inline(always)]
    fn sort_moves(&self, _game: &G, _moves: &mut [ComponentsInfo], _move_components: &mut [<G as Game>::Position]) {
        // do nothing
    }
}

impl<G> SimpleGameMoveSorter<G> for () where G: SimpleGame {
    #[inline(always)]
    fn sort_moves(&self, _game: &G, _moves: &mut [<G as Game>::Position]) {
        // do nothing
    }

    #[inline(always)]
    fn remove(moves: &mut Vec<<G as Game>::Position>, index: usize) {
        moves.swap_remove(index);
    }
}

impl<G> DecomposableGameMoveSorter<G> for () where G: DecomposableGame {
    #[inline(always)]
    fn sort_moves(&self, _game: &G, _moves: &mut [ComponentsInfo], _move_components: &mut [<G as Game>::Position]) {
        // do nothing
    }

    #[inline(always)]
    fn remove(moves: &mut Vec<ComponentsInfo>, index: usize) {
        moves.swap_remove(index);
    }
}