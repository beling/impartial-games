pub use super::Solver;
use crate::game::{Game, SimpleGame, DecomposableGame};
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::nimber_set::NimberSet;
use crate::solver::StatsCollector;

/// Solver that calculate nimbers using the mex function directly
/// and (if G is DecomposableGame) Sprague–Grundy theorem for decomposable positions.
pub trait DefSimpleGameSolver<G> where G: SimpleGame {
    fn nimber_def(&mut self, position: G::Position) -> u8;
    fn nimber_of_initial_def(&mut self) -> u8;
}

/// Solver that calculate nimbers using the mex function directly
/// and (if G is DecomposableGame) Sprague–Grundy theorem for decomposable positions.
pub trait DefDecomposableGameSolver<G> where G: DecomposableGame {

    fn nimber_of_component_def(&mut self, position: G::Position) -> u8;

    fn nimber_def(&mut self, position: <G as DecomposableGame>::DecomposablePosition) -> u8; /*{
        let mut result = 0u8;
        for component in self.game.decompose(&position) {
            result ^= self.nimber_of_component(component);
        }
        result
    }*/

    fn nimber_of_initial_def(&mut self) -> u8;
}
impl<G, TT, EDB, SORTER, STATS> DefSimpleGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: SimpleGame,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          STATS: StatsCollector
{
    fn nimber_def(&mut self, position: G::Position) -> u8 {
        self.stats.pre();
        if let Some(v) = self.nimber_from_any_db(&position) {
            self.stats.db_cut(v);
            return v;
        }
        self.stats.recursive();
        let mut nimbers = <G as Game>::NimberSet::empty();
        for m in self.game.successors(&position) {
            nimbers.append(self.nimber_def(m));
        }
        let result = nimbers.mex();
        self.transposition_table.store_nimber(position, result);
        self.stats.exact(result);
        result
    }

    fn nimber_of_initial_def(&mut self) -> u8 {
        let initial_position = self.game.initial_position();
        if let Some(is_winning) = self.game.is_initial_position_winning() {
            if !is_winning { return 0; }
            if self.game.moves_count(&initial_position) == 1 { return 1; }
        }
        self.nimber_def(initial_position)
    }
}

impl<G, TT, EDB, SORTER, STATS, DP> DefDecomposableGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          STATS: StatsCollector
{
    fn nimber_of_component_def(&mut self, position: G::Position) -> u8 {
        self.stats.pre();
        if let Some(v) = self.nimber_from_any_db(&position) {
            self.stats.db_cut(v);
            return v;
        }
        self.stats.recursive();
        let mut nimbers = <G as Game>::NimberSet::empty();
        for m in self.game.successors(&position) {
            nimbers.append(self.nimber_def(m));
        }
        let result = nimbers.mex();
        self.transposition_table.store_nimber(position, result);
        self.stats.exact(result);
        result
    }

    fn nimber_def(&mut self, position: DP) -> u8 {
        let mut result = 0u8;
        for component in self.game.decompose(&position) {
            result ^= self.nimber_of_component_def(component);
        }
        result
    }

    fn nimber_of_initial_def(&mut self) -> u8 {
        let initial_position = self.game.initial_position();
        if let Some(is_winning) = self.game.is_initial_position_winning() {
            if !is_winning { return 0; }
            if self.game.moves_count(&initial_position) == 1 { return 1; }
        }
        self.nimber_of_component_def(initial_position)
    }
}