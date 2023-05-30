pub use super::Solver;
use crate::game::{Game, SimpleGame, DecomposableGame};
use crate::moves::{SimpleGameMoveSorter, DecomposableGameMoveSorter, ComponentsInfo};
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::nimber_set::{NimberSet, ExtendendNimberSet, WithLowest};
use crate::stats::{StatsCollector, ProgressReporter};
//use smallvec::{SmallVec};

/// Simple game solver that uses the method described in:
/// P. Beling, M, Rogalski, *On pruning search trees of impartial games*, Artificial Intelligence 283 (2020), doi: [10.1016/j.artint.2020.103262](https://doi.org/10.1016/j.artint.2020.103262).
pub trait BRSimpleGameSolver<G> where G: SimpleGame {

    /// Calculates the nimber of the given `position` if it is included in set `requested_nimbers`.
    /// Returns either the nimber of the `position` or `NOT_IN_SET` (possible only if the nimber is not in `requested_nimbers`).
    fn nimber_in_set(&mut self, position: G::Position, requested_nimbers: G::NimberSet) -> u8;

    /// Calculates nimber of `position` using the method developed by Beling.
    fn nimber_br(&mut self, position: G::Position) -> u8;

    fn nimber_of_initial_br(&mut self) -> u8;

    /// Calculates nimber of `position` using aspiration sets method developed by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_br_aspset_report_progress<PR: ProgressReporter>(&mut self, position: G::Position, position_is_winning: Option<bool>, progress_reporter: PR) -> u8;

    /// Calculates nimber of `position` using aspiration sets method developed by Beling.
    #[inline(always)] fn nimber_br_aspset(&mut self, position: G::Position) -> u8 {
        self.nimber_br_aspset_report_progress(position, None, ())
    }

    fn nimber_of_initial_br_aspset_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8;

    fn nimber_of_initial_br_aspset(&mut self) -> u8 { 
        self.nimber_of_initial_br_aspset_report_progress(())
    }
}

/// Decomposable game solver that uses the method described in:
/// P. Beling, M, Rogalski, *On pruning search trees of impartial games*, Artificial Intelligence 283 (2020), doi: [10.1016/j.artint.2020.103262](https://doi.org/10.1016/j.artint.2020.103262).
pub trait BRDecomposableGameSolver<G> where G: DecomposableGame {

    /// Calculates the nimber of the given `position` if it is included in set `requested_nimbers`.
    /// Returns either the nimber of the `position` or `NOT_IN_SET` (possible only if the nimber is not in `requested_nimbers`).
    fn nimber_in_set(&mut self, position: &G::Position, requested_nimbers: G::NimberSet) -> u8;

    fn nimber_of_component_br(&mut self, position: &G::Position) -> u8;

    fn nimber_of_initial_br(&mut self) -> u8;

    fn nimber_br(&mut self, position: &<G as DecomposableGame>::DecomposablePosition) -> u8;

    /// Calculates nimber of decomposed `position` using aspiration sets method developed by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_of_component_br_aspset_report_progress<PR: ProgressReporter>(&mut self, position: &G::Position, position_is_winning: Option<bool>, progress_reporter: PR) -> u8;

    #[inline(always)] fn nimber_of_component_br_aspset(&mut self, position: &G::Position) -> u8 {
        self.nimber_of_component_br_aspset_report_progress(position, None, ())
    }

    /// Calculates nimber of (possible decomposable) `position` using aspiration sets method developed by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_br_aspset_report_progress<PR: ProgressReporter + Clone>(&mut self, position: &<G as DecomposableGame>::DecomposablePosition, progress_reporter: PR) -> u8;

    #[inline(always)] fn nimber_br_aspset(&mut self, position: &<G as DecomposableGame>::DecomposablePosition) -> u8 {
        self.nimber_br_aspset_report_progress(position, ())
    }

    fn nimber_of_initial_br_aspset_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8;

    fn nimber_of_initial_br_aspset(&mut self) -> u8 { 
        self.nimber_of_initial_br_aspset_report_progress(())
    }
}

const NOT_IN_SET: u8 = 255;


/*struct BumpScopedResetMark<'b> {
    pub reset_mark: BumpResetMark,
    pub bump: &'b RefCell<bumpalo::Bump>
}

impl<'b> BumpScopedResetMark<'b> {
    pub fn new(bump: &'b RefCell<bumpalo::Bump>) -> Self {
        BumpScopedResetMark{ reset_mark: bump.borrow_mut().get_reset_mark(), bump }
    }
}

impl Drop for BumpScopedResetMark<'_> {
    fn drop(&mut self) {
        let mut b = self.bump.borrow_mut();
        let dump = b.get_reset_mark();
        b.reset_to_mark(std::mem::replace(&mut self.reset_mark, dump));
    }
}*/

impl<G, TT, EDB, SORTER, STATS> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: Game,
    TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
    EDB: NimbersProvider<G::Position>,
    STATS: StatsCollector
{
    fn nimber_in_set_with_is_winning<F>(&mut self, is_winning: bool, nimber_in_set: F) -> u8 where F: Fn(&mut Self, G::Position, G::NimberSet)->u8 {
        if !is_winning { return 0; }
        let position = self.game.initial_position();
        let moves_count = self.game.moves_count(&position);
        if moves_count == 1 { return 1; }   // only one move with nimber 0
        let mut requested_nimbers = G::NimberSet::with_lowest(moves_count+1);
        requested_nimbers.remove(0);
        nimber_in_set(self, position, requested_nimbers)
    }
}

impl<G, TT, EDB, SORTER, STATS> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: SimpleGame,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: SimpleGameMoveSorter<G>,
          G::Position: Clone,
          STATS: StatsCollector
{
    /// caller have to call self.stats.pre() and optionally check const_db
    fn simple_nimber_in_set(&mut self, position: G::Position, requested_nimbers: G::NimberSet) -> u8 {
        // const_db is already checked by caller (ETC...)
        if let Some(v) = self.nimber_from_tt(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v;
        }
        self.stats.etc();
        let moves_count = self.game.moves_count(&position);
        //if moves_count == 0 { return 0; }
        let mut potential_nimbers = <<<G as Game>::NimberSet as NimberSet>::Extended as WithLowest>::with_lowest(moves_count + 1);
        /*if P.is_distinct_from(&R) {
            return NOT_IN_SET;
        }*/
        //let alloc_reset_mark = self.allocator.get_reset_mark();
        //let mut moves = bc::Vec::<G::Position>::with_capacity_in(moves_count as usize, &self.allocator);
        let mut moves = Vec::<G::Position>::with_capacity(moves_count as usize);
        //let mut moves = SmallVec::<[G::Position; 64]>::with_capacity(moves_count as usize);
        for m in self.game.successors_in_heuristic_ordered(&position) {  // ETC
            if potential_nimbers.is_distinct_from(&requested_nimbers) { // TODO sprawdzać rzadziej? (tylko w 1 przebiegu i po zmianie w P)
                self.stats.unknown();
                //drop(moves);
                //self.allocator.reset_to_mark(alloc_reset_mark);
                return NOT_IN_SET;
            }
            if let Some(v) = self.nimber_from_any_db(&m) {
                self.stats.db_skip(v);
                potential_nimbers.remove_nimber(v);
            } else {
                moves.push(m);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves);
        self.stats.recursive();
        let upto_largest_requested_nimber = requested_nimbers.upto_largest();
        let mut exact = true;
        for m in moves {
            if potential_nimbers.is_distinct_from(&requested_nimbers) {
                //self.allocator.reset_to_mark(alloc_reset_mark);
                self.stats.unknown();
                return NOT_IN_SET;
            }
            let potential_nimbers_without_largest = potential_nimbers.without_largest();
            self.stats.pre();
            let m_nimber = self.simple_nimber_in_set(m, potential_nimbers_without_largest.intersected_with(&upto_largest_requested_nimber));
            if m_nimber == NOT_IN_SET {
                potential_nimbers.remove_largest_hinted(&potential_nimbers_without_largest);
                exact = false;
            } else {
                potential_nimbers.remove_nimber_hinted(m_nimber, &potential_nimbers_without_largest);
            }
        }
        //self.allocator.reset_to_mark(alloc_reset_mark);
        if exact || !potential_nimbers.is_distinct_from(&upto_largest_requested_nimber) {
            let result = potential_nimbers.only_element();   // P includes only one element
            self.transposition_table.store_nimber(position, result);
            self.stats.exact(result);
            result
        } else {
            self.stats.unknown();
            NOT_IN_SET
        }
    }
}


impl<G, TT, EDB, SORTER, STATS> BRSimpleGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: SimpleGame,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: SimpleGameMoveSorter<G>,
          G::Position: Clone,
          STATS: StatsCollector
{
    fn nimber_in_set(&mut self, position: G::Position, requested_nimbers: G::NimberSet) -> u8 {
        self.stats.pre();
        if let Some(v) = self.nimber_from_const_db(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v;
        }
        self.simple_nimber_in_set(position, requested_nimbers)
    }

    #[inline(always)]
    fn nimber_br(&mut self, position: G::Position) -> u8 {
        let requested_nimbers = G::NimberSet::with_lowest(self.game.moves_count(&position)+1);
        self.nimber_in_set(position, requested_nimbers)
    }

    fn nimber_of_initial_br(&mut self) -> u8 {
        if let Some(is_winning) = self.game.is_initial_position_winning() {
            self.nimber_in_set_with_is_winning(is_winning, 
            |s, position, requested_nimbers|
                s.nimber_in_set(position, requested_nimbers))
        } else {
            self.nimber_br(self.game.initial_position())
        }
    }

    fn nimber_br_aspset_report_progress<PR: ProgressReporter>(&mut self, position: G::Position, position_is_winning: Option<bool>, mut progress_reporter: PR) -> u8 {
        if position_is_winning == Some(false) { return 0; }

        self.stats.pre();
        if let Some(v) = self.nimber_from_any_db(&position) {
            self.stats.db_cut(v);
            return v;
        }

        let (moves_count, mut nimbers_to_skip, mut moves) = self.etc_simple(&position);

        self.stats.recursive();
        progress_reporter.begin(moves_count);
        'results: for result in if position_is_winning == Some(true) {1} else {0} .. moves_count {
            progress_reporter.progress(result);
            debug_assert!(result < 256);
            let result = result as u8;
            if nimbers_to_skip.includes(result) { continue; }   // TODO use mex to iterate over nimbers_to_skip?
            let mut index = 0;
            while index < moves.len() {
                self.stats.pre();
                let m_nimber = self.simple_nimber_in_set(moves[index].clone(), G::NimberSet::singleton(result));
                if m_nimber != NOT_IN_SET {
                    SORTER::remove(&mut moves, index);
                    if m_nimber == result {
                        // as result is nimber of move, it is not nimber of position
                        continue 'results;
                    }
                    nimbers_to_skip.append(m_nimber);
                } else {
                    index += 1;
                }
            }
            // no move has nimber = result, so position has it
            self.transposition_table.store_nimber(position, result);
            self.stats.exact(result);
            progress_reporter.end();
            return result;
        }
        debug_assert!(moves_count < 256);
        let moves_count = moves_count as u8;
        self.transposition_table.store_nimber(position, moves_count);
        self.stats.exact(moves_count);
        moves_count // we have proved that are lower nimbers are not correct result
    }

    fn nimber_of_initial_br_aspset_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8 {
        self.nimber_br_aspset_report_progress(self.game.initial_position(), self.game.is_initial_position_winning(), progress_reporter)
    }
}



impl<G, TT, EDB, SORTER, STATS, DP> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: DecomposableGameMoveSorter<G>,
          STATS: StatsCollector,
          G::Position: Clone
{
    fn decomposable_nimber_in_set(&mut self, position: &G::Position, requested_nimbers: G::NimberSet) -> u8 {
        // const_db is already checked by ETC
        if let Some(v) = self.nimber_from_tt(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v;
        }
        self.stats.etc();
        let moves_count = self.game.moves_count(&position);
        let mut potential_nimbers = <<<G as Game>::NimberSet as NimberSet>::Extended as WithLowest>::with_lowest(moves_count + 1);
        let mut move_components: Vec::<G::Position> = Vec::with_capacity(moves_count as usize * 2);
        let mut moves: Vec::<ComponentsInfo> = Vec::with_capacity(moves_count as usize);
        for composed_move in self.game.successors_in_heuristic_ordered(&position) {
            if potential_nimbers.is_distinct_from(&requested_nimbers) {
                self.stats.unknown();
                return NOT_IN_SET;
            }
            let info = self.decompose(&composed_move, &mut move_components);
            if info.len == 0 {  // nimber is known
                potential_nimbers.remove_nimber(info.nimber);
            } else {
                moves.push(info);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves, &mut move_components);
        self.stats.recursive();
        let upto_largest_requested_nimber = requested_nimbers.upto_largest();
        let mut exact = true;
        for mut m in moves {
            if potential_nimbers.is_distinct_from(&requested_nimbers) {
                self.stats.unknown();
                return NOT_IN_SET;
            }
            let potential_nimbers_without_largest = potential_nimbers.without_largest();

            while m.len > 1 {
                // TODO inne algorytmy liczenia nimbera składowych? Przekazać obiekt strategii jako ZST parametr funkcji
                self.stats.pre();
                let component = &move_components[m.first + m.len - 1];
                let requested_nimbers = G::NimberSet::with_lowest(self.game.moves_count(&component) + 1);
                m.nimber ^= self.decomposable_nimber_in_set(component, requested_nimbers);
                m.len -= 1;
            }
            self.stats.pre();
            let first_comp_nimber = self.decomposable_nimber_in_set(&move_components[m.first], potential_nimbers_without_largest.intersected_with(&upto_largest_requested_nimber).each_xored_with(m.nimber));
            if first_comp_nimber == NOT_IN_SET {
                potential_nimbers.remove_largest_hinted(&potential_nimbers_without_largest);
                exact = false;
            } else {
                potential_nimbers.remove_nimber_hinted(first_comp_nimber ^ m.nimber, &potential_nimbers_without_largest);
            }
        }
        if exact || !potential_nimbers.is_distinct_from(&upto_largest_requested_nimber) {
            let result = potential_nimbers.only_element();   // P includes only one element
            self.transposition_table.store_nimber(position.clone(), result);
            self.stats.exact(result);
            result
        } else {
            self.stats.unknown();
            NOT_IN_SET
        }
    }
}


impl<G, TT, EDB, SORTER, STATS, DP> BRDecomposableGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: DecomposableGameMoveSorter<G>,
          STATS: StatsCollector,
          G::Position: Clone
{

    fn nimber_in_set(&mut self, position: &G::Position, requested_nimbers: G::NimberSet) -> u8 {
        self.stats.pre();
        if let Some(v) = self.nimber_from_const_db(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v;
        }
        self.decomposable_nimber_in_set(position, requested_nimbers)
    }

    #[inline(always)]
    fn nimber_of_component_br(&mut self, position: &G::Position) -> u8 {
        let requested_nimbers = G::NimberSet::with_lowest(self.game.moves_count(&position)+1);
        self.nimber_in_set(&position, requested_nimbers)
    }

    fn nimber_of_initial_br(&mut self) -> u8 {
        if let Some(is_winning) = self.game.is_initial_position_winning() {
            self.nimber_in_set_with_is_winning(is_winning, 
            |s, position, requested_nimbers|
                s.nimber_in_set(&position, requested_nimbers))
        } else {
            self.nimber_of_component_br(&self.game.initial_position())
        }
    }

    fn nimber_br(&mut self, position: &<G as DecomposableGame>::DecomposablePosition) -> u8 {
        let mut result = 0u8;
        for component in self.game.decompose(position) {
            result ^= self.nimber_of_component_br(&component);
        }
        result
    }

    fn nimber_of_component_br_aspset_report_progress<PR: ProgressReporter>(&mut self, position: &G::Position, position_is_winning: Option<bool>, mut progress_reporter: PR) -> u8 {
        if position_is_winning == Some(false) { return 0; }

        self.stats.pre();
        if let Some(v) = self.nimber_from_any_db(&position) {
            self.stats.db_cut(v);
            return v;
        }

        let (moves_count, mut nimbers_to_skip, move_components, mut moves) = self.etc_decomposable(&position);

        self.stats.recursive();
        progress_reporter.begin(moves_count);
        'results: for result in if position_is_winning == Some(true) {1} else {0}..moves_count {
            progress_reporter.progress(result);
            debug_assert!(result < 256);
            let result = result as u8;
            if nimbers_to_skip.includes(result) { continue; }   // TODO use mex to iterate over nimbers_to_skip?
            let mut index = 0;
            while index < moves.len() {
                let m = &mut moves[index];
                while m.len > 1 {
                    // TODO inne algorytmy liczenia nimbera składowych? Przekazać obiekt strategii jako ZST parametr funkcji
                    self.stats.pre();
                    let component = &move_components[m.first + m.len - 1];
                    let requested_nimbers = G::NimberSet::with_lowest(self.game.moves_count(&component)+1);
                    m.nimber ^= self.decomposable_nimber_in_set(component, requested_nimbers);
                    m.len -= 1;
                }
                self.stats.pre();
                let mut m_nimber = self.decomposable_nimber_in_set(&move_components[m.first], G::NimberSet::singleton(result ^ m.nimber));
                if m_nimber != NOT_IN_SET {
                    m_nimber ^= m.nimber;
                    SORTER::remove(&mut moves, index);
                    if m_nimber == result {
                        // as the result is the nimber of move, it is not the nimber of position
                        continue 'results;
                    }
                    nimbers_to_skip.append(m_nimber);
                } else {
                    index += 1;
                }
            }
            // no move has nimber = result, so position has it
            self.transposition_table.store_nimber(position.clone(), result);
            self.stats.exact(result);
            progress_reporter.end();
            return result;
        }
        debug_assert!(moves_count < 256);
        let moves_count = moves_count as u8;
        self.transposition_table.store_nimber(position.clone(), moves_count);
        self.stats.exact(moves_count);
        moves_count // we have proved that are lower nimbers are not correct result
    }

    fn nimber_br_aspset_report_progress<PR: ProgressReporter + Clone>(&mut self, position: &<G as DecomposableGame>::DecomposablePosition, progress_reporter: PR) -> u8 {
        let mut result = 0u8;
        for component in self.game.decompose(position) {
            result ^= self.nimber_of_component_br_aspset_report_progress(&component, None, progress_reporter.clone());
        }
        result
    }

    fn nimber_of_initial_br_aspset_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8 {
        self.nimber_of_component_br_aspset_report_progress(&self.game.initial_position(), self.game.is_initial_position_winning(), progress_reporter)
    }
}
