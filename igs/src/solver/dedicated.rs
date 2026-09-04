use crate::game::{Game, SimpleGame, DecomposableGame};
use super::*;
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::solver::lvb::{LVBSimpleGameSolver, LVBDecomposableGameSolver};
use crate::solver::br::{BRDecomposableGameSolver, BRSimpleGameSolver};

/// Solver dedicated to simple games, i.e. games without decomposable positions.
///
/// It is returned by `SimpleGame::solver`/`SimpleGame::solver_with_stats`.
pub trait SolverForSimpleGame {
    /// The type of the game that can be solved by `self`.
    type Game: ?Sized + SimpleGame;
    /// The type of the statistics collector used by `self`.
    type StatsCollector: StatsCollector;

    /// Calculates nimber of the given `position`.
    fn nimber(&mut self, position: <Self::Game as Game>::Position) -> u8;

    /// Returns reference to the game which can be solved by `self`.
    fn game(&self) -> &Self::Game;

    /// Returns reference to the statistics collector used by `self`.
    fn stats(&self) -> &Self::StatsCollector;
}

/// Solver dedicated to decomposable games.
///
/// It is returned by `DecomposableGame::solver`/`DecomposableGame::solver_with_stats`.
pub trait SolverForDecomposableGame {

    /// The type of the game that can be solved by `self`.
    type Game: ?Sized + DecomposableGame;
    /// The type of the statistics collector used by `self`.
    type StatsCollector: StatsCollector;

    /// Calculates nimber of the (non-decomposable) `position` (a component).
    fn nimber_of_component(&mut self, position: <Self::Game as Game>::Position) -> u8;

    /// Calculates nimber of the (possibly decomposable) `position`
    /// (as the xor of nimbers of all its components).
    fn nimber(&mut self, position: <Self::Game as DecomposableGame>::DecomposablePosition) -> u8; /*{
        let mut result = 0u8;
        for component in self.game().decompose(&position) {
            result ^= self.nimber_of_component(component);
        }
        result
    }*/

    /// Returns reference to the game which can be solved by `self`.
    fn game(&self) -> &Self::Game;

    /// Returns reference to the statistics collector used by `self`.
    fn stats(&self) -> &Self::StatsCollector;
}

/// Defines a dedicated solver named `$DedicatedSolverName` (documented with `$DedicatedSolverDoc`)
/// that wraps [`super::Solver`] and delegates simple-game solving to `$SimpleGetNimber`,
/// and decomposable-game solving to `$DecomposableGetNimberOfComponent`/`$DecomposableGetNimber`.
macro_rules! impl_dedicated_solver {
($DedicatedSolverName:ident<$G:ident, $SORTER:ident>,
 $DedicatedSolverDoc:expr,
 |$self:ident, $position:ident|
 $($s:path : $st:path),* {$SimpleGetNimber:expr}
 $($d:path : $dt:path),* {$DecomposableGetNimberOfComponent:expr}
 {$DecomposableGetNimber:expr}) => {

    #[doc = $DedicatedSolverDoc]
    pub struct $DedicatedSolverName<'a, $G, TT, EDB, $SORTER, STATS>
        where $G: Game,
              TT: NimbersProvider<$G::Position> + NimbersStorer<$G::Position>,
              EDB: NimbersProvider<$G::Position>,
              STATS: StatsCollector
    { /// The wrapped generic solver.
      pub solver: Solver<'a, G, TT, EDB, SORTER, STATS> }

    impl<$G, TT, EDB, $SORTER, STATS> SolverForSimpleGame for $DedicatedSolverName<'_, $G, TT, EDB, $SORTER, STATS>
        where $G: SimpleGame,
              TT: NimbersProvider<$G::Position> + NimbersStorer<$G::Position>,
              EDB: NimbersProvider<$G::Position>,
              STATS: StatsCollector,
              $($s : $st),*
    {
        type Game = $G;
        type StatsCollector = STATS;

        fn nimber(&mut $self, $position: $G::Position) -> u8 {
            $SimpleGetNimber
        }

        fn game(&self) -> &Self::Game {
            self.solver.game
        }

        fn stats(&self) -> &Self::StatsCollector {
            &self.solver.stats
        }
    }

    impl<$G, TT, EDB, $SORTER, STATS, DP> SolverForDecomposableGame for $DedicatedSolverName<'_, $G, TT, EDB, $SORTER, STATS>
        where $G: DecomposableGame<DecomposablePosition=DP>,
              TT: NimbersProvider<$G::Position> + NimbersStorer<$G::Position>,
              EDB: NimbersProvider<$G::Position>,
              STATS: StatsCollector,
              $($d : $dt),*
    {
        type Game = $G;
        type StatsCollector = STATS;

        fn nimber_of_component(&mut $self, $position: <Self::Game as Game>::Position) -> u8 {
            $DecomposableGetNimberOfComponent
        }

        fn nimber(&mut $self, $position: DP) -> u8 {
            $DecomposableGetNimber
        }

        fn game(&self) -> &Self::Game {

            self.solver.game
        }

        fn stats(&self) -> &Self::StatsCollector {
            &self.solver.stats
        }
    }
}
}

impl_dedicated_solver!(DefSolver<G, SORTER>,
    "Dedicated solver that calculates nimbers by their definition \
    (see `DefSimpleGameSolver`/`DefDecomposableGameSolver`).",
    |self, position|
    {self.solver.nimber_def(position)}
    {self.solver.nimber_of_component_def(position)}
    {self.solver.nimber_def(position)}
);

impl_dedicated_solver!(BRSolver<G, SORTER>,
    "Dedicated solver that calculates nimbers using Beling's method \
    (see `BRSimpleGameSolver`/`BRDecomposableGameSolver`).",
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_br(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_br(&position)}
    {self.solver.nimber_br(&position)}
);

impl_dedicated_solver!(BRAspSetSolver<G, SORTER>,
    "Dedicated solver that calculates nimbers using Beling's method with aspiration sets \
    (see `BRSimpleGameSolver`/`BRDecomposableGameSolver`, methods `nimber_br_aspset*`).",
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_br_aspset(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_br_aspset(&position)}
    {self.solver.nimber_br_aspset(&position)}
);

impl_dedicated_solver!(LVBSolver<G, SORTER>,
    "Dedicated solver that calculates nimbers using the improved (by Beling) \
    Lemoine-Viennot's method (see `LVBSimpleGameSolver`/`LVBDecomposableGameSolver`).",
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_lvb(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_lvb(position)}
    {self.solver.nimber_lvb(position)}
);