use crate::game::{Game, SimpleGame, DecomposableGame};
use super::*;
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::solver::lvb::{LVBSimpleGameSolver, LVBDecomposableGameSolver};
use crate::solver::br::{BRDecomposableGameSolver, BRSimpleGameSolver};

pub trait SolverForSimpleGame {
    type Game: ?Sized + SimpleGame;
    type StatsCollector: StatsCollector;

    fn nimber(&mut self, position: <Self::Game as Game>::Position) -> u8;

    fn game(&self) -> &Self::Game;

    fn stats(&self) -> &Self::StatsCollector;
}

pub trait SolverForDecomposableGame {

    type Game: ?Sized + DecomposableGame;
    type StatsCollector: StatsCollector;

    fn nimber_of_component(&mut self, position: <Self::Game as Game>::Position) -> u8;

    fn nimber(&mut self, position: <Self::Game as DecomposableGame>::DecomposablePosition) -> u8; /*{
        let mut result = 0u8;
        for component in self.game().decompose(&position) {
            result ^= self.nimber_of_component(component);
        }
        result
    }*/

    fn game(&self) -> &Self::Game;

    fn stats(&self) -> &Self::StatsCollector;
}

macro_rules! impl_dedicated_solver {
($DedicatedSolverName:ident<$G:ident, $SORTER:ident>,
 |$self:ident, $position:ident|
 $($s:path : $st:path),* {$SimpleGetNimber:expr}
 $($d:path : $dt:path),* {$DecomposableGetNimberOfComponent:expr}
 {$DecomposableGetNimber:expr}) => {

    pub struct $DedicatedSolverName<'a, $G, TT, EDB, $SORTER, STATS>
        where $G: Game,
              TT: NimbersProvider<$G::Position> + NimbersStorer<$G::Position>,
              EDB: NimbersProvider<$G::Position>,
              STATS: StatsCollector
    { pub solver: Solver<'a, G, TT, EDB, SORTER, STATS> }

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
    |self, position|
    {self.solver.nimber_def(position)}
    {self.solver.nimber_of_component_def(position)}
    {self.solver.nimber_def(position)}
);

impl_dedicated_solver!(BRSolver<G, SORTER>,
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_br(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_br(&position)}
    {self.solver.nimber_br(&position)}
);

impl_dedicated_solver!(BRAspSetSolver<G, SORTER>,
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_br_aspset(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_br_aspset(&position)}
    {self.solver.nimber_br_aspset(&position)}
);

impl_dedicated_solver!(LVBSolver<G, SORTER>,
    |self, position|
    SORTER: SimpleGameMoveSorter<G>, G::Position: Clone {self.solver.nimber_lvb(position)}
    SORTER: DecomposableGameMoveSorter<G>, G::Position: Clone
    {self.solver.nimber_of_component_lvb(position)}
    {self.solver.nimber_lvb(position)}
);