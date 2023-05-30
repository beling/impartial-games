use crate::game::{Game, SimpleGame, DecomposableGame};
use co_sort::Permutation;

/// One can implement DifficultEvaluator instead of SimpleGameMoveSorter directly.
pub trait SimpleGameMoveSorter<G> where G: SimpleGame {

    /// sort moves from the easiest to the most difficult
    fn sort_moves(&self, game: &G, moves: &mut [<G as Game>::Position]);

    /// Remove index-th item from moves.
    /// Default implementation calls moves.remove(index).
    /// However, if order of moves do not need to be preserved (as sort_moves do nothing), faster removing can be performed.
    fn remove(moves: &mut Vec<<G as Game>::Position>, index: usize) {
        moves.remove(index);
    }
}

#[derive(Copy, Clone)]
pub struct ComponentsInfo {
    /// index of the first component (of decomposable position represented by self) in the vector of components
    pub first: usize,

    /// number of components (of decomposable position represented by self) in the vector of components
    pub len: usize,

    /// nimber of removed components of decomposable position represented by self
    pub nimber: u8
}

impl ComponentsInfo {
    #[inline(always)]
    pub fn new(first: usize) -> Self {
        Self{ first, len: 0, nimber: 0 }
    }

    #[inline(always)]
    pub fn as_slice<'a, T>(&self, all: &'a [T]) -> &'a [T] {
        &all[self.first..self.first+self.len]
    }

    #[inline(always)]
    pub fn as_slice_mut<'a, T>(&self, all: &'a mut [T]) -> &'a mut [T] {
        &mut all[self.first..self.first+self.len]
    }
}

pub trait DecomposableGameMoveSorter<G> where G: DecomposableGame {

    /// Sort moves from the easiest to the most difficult.
    /// Additionally, move the most difficult component of decomposed move to the first position.
    fn sort_moves(&self, game: &G,
                  moves: &mut [ComponentsInfo],
                  move_components: &mut [<G as Game>::Position]
    );

    /// Remove index-th item from moves.
    /// Default implementation calls moves.remove(index).
    /// However, if order of moves do not need to be preserved (as sort_moves do nothing), faster removing can be performed.
    #[inline(always)]
    fn remove(moves: &mut Vec<ComponentsInfo>, index: usize) {
        moves.remove(index);
    }
}

pub trait DifficultEvaluator {
    type Game: Game;
    type PositionDifficult: Ord;
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

/// Move sorter that preserve order generated by game methods.
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