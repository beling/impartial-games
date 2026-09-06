#![macro_use]
//! Traits that a game has to implement to be solvable by the library's solvers,
//! and helper trait/method for serialization of game positions.

pub use crate::nimber_set::{NimberSet, ExtendedNimberSet};
use crate::dbs::NimbersProvider;
use crate::solver::{SolverForSimpleGame, SolverForDecomposableGame, StatsCollector};
use std::io;

/// All games have to implement this trait, plus
/// either [`SimpleGame`] (if game hasn't decomposable positions)
/// or [`DecomposableGame`] (if game has decomposable positions).
pub trait Game {

    /// In case of simple games, i.e. without decomposable positions: game position.
    /// In case of games with decomposable positions: a single component of decomposable position.
    type Position;

    /// Type used to store set of nimbers for this game.
    type NimberSet: NimberSet;

    /// Returns the number of moves available in the `position` given.
    fn moves_count(&self, position: &Self::Position) -> u16;

    /// Tries to provide theoretical solution for (i.e. the nimber of) the given `position`.
    /// One can use the [`TheoreticalSolutions`] provider returned by `self.theoretical_solutions()`
    /// as the `const_db` of any solver to utilize this method during search.
    ///
    /// The default implementation returns `None`.
    #[inline(always)]
    fn try_solve_theoretically(&self, _position: &Self::Position) -> Option<u8> {
        None
    }

    /// Returns nimber provider that delegates `get_nimber` to `self.try_solve_theoretically`.
    #[inline(always)]
    fn theoretical_solutions(&self) -> TheoreticalSolutions<'_, Self> {
        TheoreticalSolutions { game: &self }
    }

    /// Returns the initial game position.
    fn initial_position(&self) -> Self::Position;
    
    /// Returns the outcome of the initial position if it is known.
    /// `Some(true)` means that the position is winning (non-zero nimber),
    /// `Some(false)` means that the position is losing (zero nimber).
    #[inline(always)]
    fn is_initial_position_winning(&self) -> Option<bool> { None }
}

/// Trait implemented by all games without decomposable positions.
pub trait SimpleGame: Game {    // TODO separate life-time for solver?

    /// Iterator over the successors of (moves available in) a position.
    type Successors<'s>: Iterator<Item=Self::Position> + 's where Self: 's;

    /// Iterator over the successors of (moves available in) a position, which generates them in heuristic order.
    /// Usually the iterator generates as first the moves to smaller branches of a search tree.
    type HeuristicallyOrderedSuccessors<'s>: Iterator<Item=Self::Position> + 's where Self: 's;

    /// Returns iterator over the successors of (moves available in) the `position` given.
    fn successors(&self, position: &Self::Position) -> Self::Successors<'_>;

    /// Returns iterator over the successors of (moves available in) the `position` given,
    /// which generates them in heuristic order (usually the moves to smaller branches of a search tree first).
    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_>;

    /// Returns solver dedicated to game represented by `self` and collecting statistics in `stats`.
    fn solver_with_stats<'s, STATS: 's+StatsCollector>(&'s self, stats: STATS) -> Box<dyn SolverForSimpleGame<Game=Self, StatsCollector=STATS> + 's>;

    /// Returns solver dedicated to game represented by `self`.
    fn solver(&self) -> Box<dyn SolverForSimpleGame<Game=Self, StatsCollector=()> + '_> {
        self.solver_with_stats(())
    }
}

/// Trait implemented by all games with decomposable positions.
///
/// A (possibly) decomposable position can be split into independent components,
/// whose nimbers can be xored to obtain the nimber of the composed position
/// (as a consequence of the [Sprague-Grundy theorem](https://en.wikipedia.org/wiki/Sprague%E2%80%93Grundy_theorem)).
pub trait DecomposableGame: Game {

    /// Type of (possibly) decomposable position.
    type DecomposablePosition;

    /// Iterator over the successors of (moves available in) a position which is not decomposable.
    type Successors<'s>: Iterator<Item=Self::DecomposablePosition> where Self: 's;

    /// Iterator over the successors of (moves available in) a position which is not decomposable.
    /// Usually the iterator generates as first the moves to smaller branches of a search tree.
    type HeuristicallyOrderedSuccessors<'s>: Iterator<Item=Self::DecomposablePosition> where Self: 's;

    /// Iterator over components of a position that can be decomposable.
    type Components<'s>: Iterator<Item=Self::Position> where Self: 's;

    /// Returns iterator over the successors of (moves available in) the `position` given (which is not decomposable).
    fn successors(&self, position: &Self::Position) -> Self::Successors<'_>;

    /// Returns iterator over the successors of (moves available in) the `position` given (which is not decomposable).
    /// Usually the iterator generates as first the moves to smaller branches of a search tree.
    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_>;

    /// Returns the iterator over the components of the given `position` that can be decomposable.
    /// If the position is not decomposable, the iterator generates exactly one component.
    fn decompose(&self, position: &Self::DecomposablePosition) -> Self::Components<'_>;

    /// Returns solver dedicated to game represented by `self` and collecting statistics in `stats`.
    fn solver_with_stats<'s, STATS: 's+StatsCollector>(&'s self, stats: STATS) -> Box<dyn SolverForDecomposableGame<Game=Self, StatsCollector=STATS> + 's>;

    /// Returns solver dedicated to game represented by `self`.
    fn solver(&self) -> Box<dyn SolverForDecomposableGame<Game=Self, StatsCollector=()> + '_> {
        self.solver_with_stats(())
    }
}


/// Nimbers provider that delegates `get_nimber` to `Game::try_solve_theoretically` of the given game.
///
/// It is returned by `Game::theoretical_solutions`, and can be used as an `const_db` of any solver
/// to utilize theoretical solutions during search.
pub struct TheoreticalSolutions<'a, G: Game + ?Sized> {
    /// The game which positions are solved theoretically.
    pub game: &'a G
}

impl<G: Game> NimbersProvider<G::Position> for TheoreticalSolutions<'_, G> {
    #[inline(always)]
    fn get_nimber(&self, position: &G::Position) -> Option<u8> {
        self.game.try_solve_theoretically(position)
    }
}

/// Game whose position can be serialized and deserialized.
///
/// It is used (for example) by `transposition_table::ProtectedTT` to save/restore
/// the protected part of the transposition table.
pub trait SerializableGame: Game {
    /// Maximum number of bytes that `read_position`/`write_position` reads/writes.
    fn position_size_bytes(&self) -> usize;

    /// Writes the given position to the given output and returns error returned by output methods, if any.
    fn write_position(&self, output: &mut dyn io::Write, position: &Self::Position) -> io::Result<()>;

    /// Reads position from the given input and returns either the position read or error returned by output methods, if any.
    fn read_position(&self, input: &mut dyn io::Read) -> io::Result<Self::Position>;
}

/// Implements [`SerializableGame`] for the given `Game` whose position is represented
/// by a primitive (integer) type: positions are read/written as native-endian binary numbers.
macro_rules! impl_serializable_game_for {
    ($Game:ty) => {
        impl $crate::game::SerializableGame for $Game {
            fn position_size_bytes(&self) -> usize {
                ::std::mem::size_of::<Self::Position>()
            }

            fn write_position(&self, output: &mut dyn std::io::Write, position: &Self::Position) -> std::io::Result<()> {
                <::binout::AsIs as ::binout::Serializer::<Self::Position>>::write(output, *position)
            }

            fn read_position(&self, input: &mut dyn std::io::Read) -> std::io::Result<Self::Position> {
                <::binout::AsIs as ::binout::Serializer::<Self::Position>>::read(input)
            }
        }
    }
}
