use crate::game::{Game, SimpleGame, DecomposableGame};
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::moves::{ComponentsInfo, SimpleGameMoveSorter, DecomposableGameMoveSorter};
use crate::nimber_set::NimberSet;

pub mod def;
pub use self::def::DefSimpleGameSolver as _;
pub use self::def::DefDecomposableGameSolver as _;

pub mod lvb;
pub use self::lvb::LVBSimpleGameSolver as _;
pub use self::lvb::LVBDecomposableGameSolver as _;

pub mod br;
pub use self::br::BRSimpleGameSolver as _;
pub use self::br::BRDecomposableGameSolver as _;

pub mod dedicated;
pub use self::dedicated::{SolverForSimpleGame, SolverForDecomposableGame};

pub mod stats;
pub use stats::StatsCollector;
use std::collections::HashMap;

mod outcome;

/// Solver that calculate nimbers of games.
///
/// It implements many methods:
/// - by definition,
/// - LV method (improved by Beling),
/// - Beling's method (described by Beling and Rogalski)
///
/// Empty type "()" can be given as transposition_table or const_db to calculate without these nimber bases.
/// Also a tuple of databases (const_db1, const_db2, ...) can be given as const_db to use multiple const databases.
pub struct Solver<'g, G, TT = HashMap::<<G as Game>::Position, u8>, EDB = (), SORTER = (), STATS = ()>
    where G: Game,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          STATS: StatsCollector
{
    /// Game to solve.
    pub game: &'g G,

    /// Transposition table used by the solver.
    pub transposition_table: TT,

    /// Const (usually end game) database used by the solver.
    pub const_db: EDB,

    /// Move sorter used by the solver.
    pub move_sorter: SORTER,

    /// Statistics collector.
    pub stats: STATS

    //pub allocator: Bump
}

impl<'a, G, TT, EDB, SORTER, STATS> Solver<'a, G, TT, EDB, SORTER, STATS>
    where G: Game,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          STATS: StatsCollector
{
    #[inline(always)]
    fn nimber_from_tt(&mut self, p: &G::Position) -> Option<u8> {
        self.stats.tt_read();
        self.transposition_table.get_nimber_and_self_organize(p)
    }

    #[inline(always)]
    fn nimber_from_const_db(&mut self, p: &G::Position) -> Option<u8> {
        self.stats.const_db_read();
        self.const_db.get_nimber_and_self_organize(p)
    }

    #[inline(always)]
    fn nimber_from_any_db(&mut self, p: &G::Position) -> Option<u8> {
        self.nimber_from_const_db(&p).or_else(|| self.nimber_from_tt(&p))
    }

    pub fn new(game: &'a G, transposition_table: TT, const_db: EDB, move_sorter: SORTER, stats: STATS) -> Self {
        Self { game, transposition_table, const_db, move_sorter, stats /*, allocator: Bump::with_capacity(16*1_024*1_024) Bump::new()*/ }
    }
}

impl<G, TT, EDB, SORTER, STATS> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: SimpleGame,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: SimpleGameMoveSorter<G>,
          STATS: StatsCollector
{
    #[inline(always)]
    fn etc_simple(&mut self, position: &<G as Game>::Position) -> (u16, G::NimberSet, Vec<<G as Game>::Position>) {
        self.stats.etc();
        let moves_count = self.game.moves_count(&position);
        let mut nimbers_to_skip = G::NimberSet::empty();
        let mut moves: Vec::<G::Position> = Vec::with_capacity(moves_count as usize);
        for m in self.game.successors_in_heuristic_ordered(&position) {
            if let Some(v) = self.nimber_from_any_db(&m) {
                self.stats.db_skip(v);
                nimbers_to_skip.append(v);
            } else {
                moves.push(m);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves);
        (moves_count, nimbers_to_skip, moves)
    }
}

impl<G, TT, EDB, SORTER, STATS, DP> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: DecomposableGameMoveSorter<G>,
          STATS: StatsCollector
{
    #[inline(always)]
    fn etc_decomposable(&mut self, position: &&<G as Game>::Position) -> (u16, G::NimberSet, Vec<<G as Game>::Position>, Vec<ComponentsInfo>) {
        self.stats.etc();
        let moves_count = self.game.moves_count(position);
        let mut nimbers_to_skip = G::NimberSet::empty();
        let mut move_components: Vec::<G::Position> = Vec::with_capacity(moves_count as usize * 2);
        let mut moves: Vec::<ComponentsInfo> = Vec::with_capacity(moves_count as usize);
        for composed_move in self.game.successors_in_heuristic_ordered(&position) {
            let info = self.decompose(&composed_move, &mut move_components);
            if info.len == 0 {  // nimber is known, for sure nimber of position != info.nimber
                nimbers_to_skip.append(info.nimber);
            } else {
                moves.push(info);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves, &mut move_components);
        (moves_count, nimbers_to_skip, move_components, moves)
    }
}


impl<G, TT, EDB, SORTER, STATS, DP> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          STATS: StatsCollector
{
    /// Decomposes position `composed_move`, and returns info about its components.
    /// Nimbers of components described by `const_db` or `transposition_table` are xored and stored in `info.nimber`.
    /// The rest of components are pushed to `move_components` and account in `info.len`.
    fn decompose(&mut self, composed_move: &DP, move_components: &mut Vec<<G as Game>::Position>) -> ComponentsInfo {
        let mut info = ComponentsInfo::new(move_components.len());
        for c in self.game.decompose(&composed_move) {
            if let Some(v) = self.nimber_from_any_db(&c) {
                self.stats.db_skip(v);
                info.nimber ^= v;
            } else {
                move_components.push(c);
                info.len += 1;
            }
        }
        info
    }

}