use std::collections::HashMap;

use clap::Args;

use crate::{solver::{PruningMethod, Without, print_nimber_of_simple}, tt::{TTConf, TTKind}, constdb::ConstDBConf};
use igs::{games::chomp::{self, FewerBarsFirst}, transposition_table::{NimbersProvider, ProtectedTT, NimbersStorer, TTSuccinct64, bit_mixer::stafford13, cluster_policy::Fifo}, game::Game, solver::Solver};

fn saturating_combinations(n: u64, mut k: u64) -> u64 {
    if k > n { return 0; }
    k = k.min(n-k);
    if k == 0 { return 1; }
    if k == 1 { return n; }
    if k == 2 { return if n & 1 == 0 { n/2*(n-1) } else { (n-1)/2*n } }
    if k > 33 { return u64::MAX; }
    let mut t = [1u64; 34];
    for n in 2..=n as usize {
        let k_max = (k as usize).min(n-1);
        t[k_max] = match t[k_max].checked_add(t[k_max-1]) {
            Some(r) => r,
            None => return u64::MAX
        };
        for k in (1..k_max).rev() {
            t[k] = t[k] + t[k-1];
        }
    }
    t[k as usize]
}

/// Returns approximate number of positions in Chomp of given size.
pub fn aproximate_position_num(rows: u8, cols: u8) -> u64 {
    saturating_combinations(rows as u64 + cols as u64, rows as u64)
}

#[derive(Args, Clone, Copy)]
pub struct ChompConf {
    /// Number of rows
    #[arg(short='r', long)]
    rows: u8,

    /// Number of columns
    #[arg(short='c', long)]
    cols: u8,
}

impl ChompConf {
    pub fn run(self, method: Option<PruningMethod>, tt_conf: TTConf, cdb: ConstDBConf) {
        let method = method.unwrap_or(PruningMethod::Def);
        println!("---=== Chomp {}x{} {:?} ===---", self.cols, self.rows, method);
        let game = chomp::Chomp::new(self.cols, self.rows);
        if cdb.segments == 0 {
            self.run_with_cdb(&game, method, tt_conf, ())
        } else {
            todo!()
        }
    }

    fn run_with_cdb<CDB>(self, game: &chomp::Chomp, method: PruningMethod, tt_conf: TTConf, cdb: CDB) 
        where CDB: NimbersProvider<<chomp::Chomp as Game>::Position>,
    {   // TODO copied from Cram, should be fixed
        match tt_conf.kind.unwrap_or_else(|| if aproximate_position_num(self.cols, self.rows) > (1<<28) { crate::tt::TTKind::Succinct } else { crate::tt::TTKind::HashMap }) {
            TTKind::None => self.run_with_prot_tt_cdb(game, method, (), tt_conf.protect, cdb),
            TTKind::HashMap => self.run_with_prot_tt_cdb(game, method, HashMap::new(), tt_conf.protect, cdb),
            TTKind::Succinct => self.run_with_prot_tt_cdb(
                game, method,
                 TTSuccinct64::new(tt_conf.size_log2(8) + (32-4) /*GB*/, 2, 4, stafford13, Fifo),
                  tt_conf.protect, cdb),
        }
    }

    fn run_with_prot_tt_cdb<TT, CDB>(self, game: &chomp::Chomp, method: PruningMethod, tt: TT, protect_tt: bool, cdb: CDB) 
        where
         TT: NimbersProvider<<chomp::Chomp as Game>::Position> + NimbersStorer<<chomp::Chomp as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<chomp::Chomp as Game>::Position>,
    {   // TODO copied from Cram, should be fixed
        if protect_tt {
            let min_fields_to_protect = game.board_size().saturating_sub(20);
            let tt = ProtectedTT::new(game, 
                format!("chomp_{}x{}_TT.bin", self.cols, self.rows),
                |_, p| p.count_ones() as u16 >= min_fields_to_protect,
                tt);
            self.run_with_tt_cdb(game, method, tt, cdb);
        } else {
            self.run_with_tt_cdb(game, method, tt, cdb);
        }
    }

    fn run_with_tt_cdb<TT, CDB>(self, game: &chomp::Chomp, method: PruningMethod, tt: TT, cdb: CDB) 
        where
         TT: NimbersProvider<<chomp::Chomp as Game>::Position> + NimbersStorer<<chomp::Chomp as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<chomp::Chomp as Game>::Position>,
    {
        let mut solver = Solver::new(
            game,
            tt,
            cdb,
            //move_sorter
            FewerBarsFirst{}, // TODO configurable
            Without
        );
        print_nimber_of_simple(&mut solver, method);
        println!("TT size: {}", solver.transposition_table.len());  // TODO move to print_nimber_of_simple
        println!("{}", solver.stats);   // TODO move to print_nimber_of_simple
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_combinations() {
        assert_eq!(saturating_combinations(3, 0), 1);
        assert_eq!(saturating_combinations(3, 1), 3);
        assert_eq!(saturating_combinations(3, 2), 3);
        assert_eq!(saturating_combinations(3, 3), 1);
        assert_eq!(saturating_combinations(3, 4), 0);

        assert_eq!(saturating_combinations(6, 2), 15);
        assert_eq!(saturating_combinations(6, 3), 20);

        assert_eq!(saturating_combinations(100, 33), u64::MAX);
        assert_eq!(saturating_combinations(100, 40), u64::MAX);
    }
}