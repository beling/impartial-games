use super::Solver;
use super::outcome::OptionalOutcome;
use crate::game::{Game, SimpleGame, DecomposableGame};
use crate::moves::{SimpleGameMoveSorter, DecomposableGameMoveSorter, ComponentsInfo};
use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::nimber_set::NimberSet;
use crate::stats::{StatsCollector, ProgressReporter};

/// Simple game solver that uses the improved (by Beling) version of the method described in:
/// J. Lemoine, S. Viennot, *Nimbers are inevitable*, Theoretical Computer Science 462 (2012) 70–79, doi: [10.1016/j.tcs.2012.09.002](https://doi.org/10.1016/j.tcs.2012.09.002).
pub trait LVBSimpleGameSolver<G> where G: SimpleGame {
    /// Checks whether `position` has nimber `nim`.
    fn has_nimber(&mut self, position: &G::Position, nim: u8) -> bool;

    /// Calculates the nimber of `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_lvb_report_progress<OO: OptionalOutcome, PR: ProgressReporter>(&mut self, position: G::Position, position_outcome: OO, progress_reporter: PR) -> u8;

    /// Calculates the nimber of `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    #[inline(always)] fn nimber_lvb(&mut self, position: G::Position) -> u8 {
        self.nimber_lvb_report_progress(position, (), ())
    }

    /// Calculates the nimber of initial position of the game using the method developed by Lemoine-Viennot, and improved by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_of_initial_lvb_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8;

    /// Calculates the nimber of initial position of the game using the method developed by Lemoine-Viennot, and improved by Beling.
    fn nimber_of_initial_lvb(&mut self) -> u8 { 
        self.nimber_of_initial_lvb_report_progress(())
    }
}

/// Decomposable game solver that uses the improved (by Beling) version of the method described in:
/// J. Lemoine, S. Viennot, *Nimbers are inevitable*, Theoretical Computer Science 462 (2012) 70–79, doi: [10.1016/j.tcs.2012.09.002](https://doi.org/10.1016/j.tcs.2012.09.002).
pub trait LVBDecomposableGameSolver<G> where G: DecomposableGame {
    /// Checks whether `position` has nimber `nim`.
    fn has_nimber(&mut self, position: &G::Position, nim: u8) -> bool;

    /// Calculates the nimber of decomposed `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_of_component_lvb_report_progress<OO: OptionalOutcome, PR: ProgressReporter>(&mut self, position: G::Position, position_outcome: OO, progress_reporter: PR) -> u8;

    /// Calculates nimber of decomposed `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    #[inline(always)] fn nimber_of_component_lvb(&mut self, position: G::Position) -> u8 {
        self.nimber_of_component_lvb_report_progress(position, None, ())
    }

    /// Calculates nimber of (possibly decomposable) `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_lvb_report_progress<PR: ProgressReporter + Clone>(&mut self, position: <G as DecomposableGame>::DecomposablePosition, progress_reporter: PR) -> u8;

    /// Calculates nimber of (possibly decomposable) `position` using the method developed by Lemoine-Viennot, and improved by Beling.
    #[inline(always)] fn nimber_lvb(&mut self, position: <G as DecomposableGame>::DecomposablePosition) -> u8 {
        self.nimber_lvb_report_progress(position, ())
    }

    /// Calculates the nimber of initial position of the game using the method developed by Lemoine-Viennot, and improved by Beling.
    /// Reports search progress (nimber about to analyze) to `progress_reporter`.
    fn nimber_of_initial_lvb_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8;

    /// Calculates the nimber of initial position of the game using the method developed by Lemoine-Viennot, and improved by Beling.
    fn nimber_of_initial_lvb(&mut self) -> u8 { 
        self.nimber_of_initial_lvb_report_progress(())
    }
}


impl<G, TT, EDB, SORTER, STATS> LVBSimpleGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: SimpleGame,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: SimpleGameMoveSorter<G>,
          STATS: StatsCollector,
          G::Position: Clone    // to call has_nimber
{
    fn has_nimber(&mut self, position: &G::Position, nim: u8) -> bool {
        self.stats.pre();
        // has_nimber(position, nim) == false <=>
        // position + nim is winning <=>
        // losing is (<=> has_nimber is true for the pair):
        //  position + new_nim (for new_nim < nim) or
        //  move + nim (for move being successor of position)
        //
        //  position + new_nim is losing <=>
        //    nimber of position equals new_nim <=>
        //    new_nim is the smallest value for which all moves has nimber different than new_nim

        let moves_count = self.game.moves_count(position);
        if moves_count < nim as u16 {
            self.stats.unknown();
            return false;
        }
        //if let Some(v) = self.nimber_from_const_db(&position) { return v == nim; }   // already checked by ETC
        if let Some(v) = self.nimber_from_tt(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v == nim;
        }
        self.stats.etc();
        let mut nimbers_to_skip = G::NimberSet::empty();
        let mut moves: Vec::<G::Position> = Vec::with_capacity(moves_count as usize);
        for m in self.game.successors_in_heuristic_ordered(&position) {
            if let Some(v) = self.nimber_from_any_db(&m) {
                if v == nim {   // successor with nimber == nim, so position has nimber != nim
                    self.stats.db_cut(v);
                    return false;
                }
                // for sure nimber of position != v
                // both (m, nim) and (position, v) can be skipped
                self.stats.db_skip(v);
                nimbers_to_skip.append(v);
            } else {
                moves.push(m);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves);
        self.stats.recursive();
        for new_nim in 0..nim {
            if nimbers_to_skip.includes(new_nim) { continue; }   // TODO use mex to iterate over nimbers_to_skip?
            if let Some(index) = moves.iter().position(|m| self.has_nimber(&m, new_nim)) {
                SORTER::remove(&mut moves, index);  // moves.remove(index);
            } else {    // all moves have nimber != new_nim (and new_nim is the smallest value with this property)
                // so position has nimber new_nim != nim
                self.transposition_table.store_nimber(position.clone(), new_nim);
                self.stats.exact(new_nim);
                return false;   // TODO maybe should return new_nim instead of false?
            }
        }
        if moves_count > nim as u16 && moves.iter().any(|m| self.has_nimber(&m, nim)) {
            self.stats.unknown();
            return false;   // nimber of position > nim
        }
        self.transposition_table.store_nimber(position.clone(), nim);
        self.stats.exact(nim);
        true
    }

    fn nimber_lvb_report_progress<OO: OptionalOutcome, PR: ProgressReporter>(&mut self, position: G::Position, position_outcome: OO, mut progress_reporter: PR) -> u8 {
        if position_outcome.is_losing() { return 0; }

        self.stats.pre();
        if let Some(v) = self.nimber_from_any_db(&position) {
            self.stats.db_cut(v);
            return v;
        }

        let (moves_count, nimbers_to_skip, mut moves) = self.etc_simple(&position);

        self.stats.recursive();
        progress_reporter.begin(moves_count);
        for result in if position_outcome.is_winning(){1}else{0}..moves_count {
            progress_reporter.progress(result);
            debug_assert!(result < 256);
            let result = result as u8;
            if nimbers_to_skip.includes(result) { continue; }   // TODO use mex to iterate over nimbers_to_skip?
            if let Some(index) = moves.iter().position(|m| self.has_nimber(&m, result)) {
                SORTER::remove(&mut moves, index);  // moves.remove(index);
            } else {    // all moves have nimber != new_nim (and new_nim is the smallest value with this property)
                // so position has nimber new_nim != nim
                self.transposition_table.store_nimber(position, result);
                self.stats.exact(result);
                progress_reporter.end();
                return result;
            }
        }
        debug_assert!(moves_count < 256);
        let moves_count = moves_count as u8;
        self.transposition_table.store_nimber(position, moves_count);
        self.stats.exact(moves_count);
        progress_reporter.end();
        moves_count // we have proved that are lower nimbers are not correct result
    }

    fn nimber_of_initial_lvb_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8 {
        self.nimber_lvb_report_progress(self.game.initial_position(), self.game.is_initial_position_winning(), progress_reporter)
    }

}


impl<G, TT, EDB, SORTER, STATS, DP> Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: DecomposableGameMoveSorter<G>,
          STATS: StatsCollector,
          G::Position: Clone    // to call has_nimber
{
    /// Check if nimber of the decomposable position (described by m and move_components) equals nim.
    /// Reduce m.len to 1, changing m.nimber.
    #[inline(always)]
    fn decomposable_has_nimber(&mut self, m: &mut ComponentsInfo, move_components: &Vec<<G as Game>::Position>, nim: u8) -> bool {
        while m.len > 1 {
            self.stats.pre();
            m.nimber ^= self.nimber_of_component_inner(&move_components[m.first + m.len - 1], (), ());
            m.len -= 1;
        }
        self.has_nimber(&move_components[m.first], nim ^ m.nimber)
    }

    /// caller must call self.stats.pre(); before
    fn nimber_of_component_inner<OO: OptionalOutcome, PR: ProgressReporter>(&mut self, position: &G::Position, position_outcome: OO, mut progress_reporter: PR) -> u8 {
        //if let Some(v) = self.nimber_from_const_db(&position) { return v; }  // checked by caller (ETC)
        if let Some(v) = self.nimber_from_tt(&position) {
            self.stats.db_cut(v);
            return v;
        }

        let (moves_count, nimbers_to_skip, move_components, mut moves) = self.etc_decomposable(&position);

        self.stats.recursive();
        progress_reporter.begin(moves_count);
        for new_nim in if position_outcome.is_winning(){1}else{0} .. moves_count {
            progress_reporter.progress(new_nim);
            debug_assert!(new_nim < 256);
            let new_nim = new_nim as u8;
            if nimbers_to_skip.includes(new_nim) { continue; }
            if let Some(index) = moves.iter_mut().position(|m| self.decomposable_has_nimber(m, &move_components, new_nim)) {
                SORTER::remove(&mut moves, index);  // moves.remove(index);
            } else {    // all moves have nimber != new_nim (and new_nim is the smallest value with this property)
                // so position has nimber new_nim != nim
                self.transposition_table.store_nimber(position.clone(), new_nim);
                self.stats.exact(new_nim);
                progress_reporter.end();
                return new_nim;
            }
        }
        debug_assert!(moves_count < 256);
        let moves_count = moves_count as u8;
        self.transposition_table.store_nimber(position.clone(), moves_count);
        self.stats.exact(moves_count);
        progress_reporter.end();
        moves_count // we have proved that are lower nimbers are not correct result
    }
}

impl<G, TT, EDB, SORTER, STATS, DP> LVBDecomposableGameSolver<G> for Solver<'_, G, TT, EDB, SORTER, STATS>
    where G: DecomposableGame<DecomposablePosition=DP>,
          TT: NimbersProvider<G::Position> + NimbersStorer<G::Position>,
          EDB: NimbersProvider<G::Position>,
          SORTER: DecomposableGameMoveSorter<G>,
          STATS: StatsCollector,
          G::Position: Clone
{
    fn has_nimber(&mut self, position: &G::Position, nim: u8) -> bool {
        self.stats.pre();
        // has_nimber(position, nim) == false <=>
        // position + nim is winning <=>
        // losing is (<=> has_nimber is true for the pair):
        //  position + new_nim (for new_nim < nim) or
        //  move + nim (for move being successor of position)
        //
        //  position + new_nim is losing <=>
        //    nimber of position equals new_nim <=>
        //    new_nim is the smallest value for which all moves has nimber different than new_nim

        let moves_count = self.game.moves_count(position);
        if moves_count < nim as u16 {
            self.stats.unknown();
            return false;
        }
        //if let Some(v) = self.nimber_from_const_db(&position) { return v == nim; }   // already checked by ETC
        if let Some(v) = self.nimber_from_tt(&position) {   // this is checked by ETC but could changed
            self.stats.db_cut(v);
            return v == nim;
        }
        self.stats.etc();
        let mut nimbers_to_skip = G::NimberSet::empty();
        let mut move_components: Vec::<G::Position> = Vec::with_capacity(moves_count as usize * 2);
        let mut moves: Vec::<ComponentsInfo> = Vec::with_capacity(moves_count as usize);
        for composed_move in self.game.successors_in_heuristic_ordered(&position) {
            let info = self.decompose(&composed_move, &mut move_components);
            if info.len == 0 {  // nimber is known
                if info.nimber == nim {   // successor with nimber == nim, so position has nimber != nim
                    self.stats.unknown();
                    return false;
                }
                // for sure nimber of position != v
                // both (m, nim) and (position, v) can be skipped
                nimbers_to_skip.append(info.nimber);
            } else {
                moves.push(info);
            }
        }
        self.move_sorter.sort_moves(&self.game, &mut moves, &mut move_components);
        self.stats.recursive();
        for new_nim in 0..nim {
            if nimbers_to_skip.includes(new_nim) { continue; }
            if let Some(index) = moves.iter_mut().position(|m| {
                self.decomposable_has_nimber(m, &move_components, new_nim)
            }) {
                SORTER::remove(&mut moves, index);  // moves.remove(index);
            } else {    // all moves have nimber != new_nim (and new_nim is the smallest value with this property)
                // so position has nimber new_nim != nim
                self.transposition_table.store_nimber(position.clone(), new_nim);
                self.stats.exact(new_nim);
                return false;   // TODO maybe should return new_nim instead of false?
            }
        }
        if moves_count > nim as u16 && moves.iter_mut().any(|m| self.decomposable_has_nimber(m, &move_components, nim)) {
            self.stats.unknown();
            return false;   // nimber of position > nim
        }
        self.transposition_table.store_nimber(position.clone(), nim);
        self.stats.exact(nim);
        true
    }

    fn nimber_of_component_lvb_report_progress<OO: OptionalOutcome, PR: ProgressReporter>(&mut self, position: G::Position, position_outcome: OO, progress_reporter: PR) -> u8 {
        if position_outcome.is_losing() { return 0; }
        self.stats.pre();
        if let Some(v) = self.nimber_from_const_db(&position) {
            self.stats.db_cut(v);
            return v;
        }
        self.nimber_of_component_inner(&position, position_outcome, progress_reporter)
    }

    fn nimber_lvb_report_progress<PR: ProgressReporter + Clone>(&mut self, position: <G as DecomposableGame>::DecomposablePosition, progress_reporter: PR) -> u8 {
        /*self.game.decompose(&position)
            .map(|c| self.nimber_of_component_lvb(component))
            .fold(0, |s, x| s^x)*/
        let mut result = 0u8;
        for component in self.game.decompose(&position) {
            result ^= self.nimber_of_component_lvb_report_progress(component, None, progress_reporter.clone());
        }
        result
    }

    fn nimber_of_initial_lvb_report_progress<PR: ProgressReporter>(&mut self, progress_reporter: PR) -> u8 {
        self.nimber_of_component_lvb_report_progress(self.game.initial_position(), self.game.is_initial_position_winning(), progress_reporter)
    }
}
