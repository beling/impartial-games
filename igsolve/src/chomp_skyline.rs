use std::collections::HashMap;

use clap::Args;

use crate::{solver::{PruningMethod, Without, print_nimber_of_simple}, tt::{TTConf, TTKind}, constdb::ConstDBConf, chomp::aproximate_position_num};
use igs::{games::chomp_skyline, transposition_table::{NimbersProvider, ProtectedTT, NimbersStorer, TTSuccinct64, bit_mixer::stafford13, cluster_policy::Fifo}, game::Game, solver::Solver};

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
        let method = method.unwrap_or(PruningMethod::Br);
        println!("---=== Chomp {}x{} {:?} ===---", self.cols, self.rows, method);
        let game = chomp_skyline::Chomp::new(self.cols, self.rows);
        if cdb.segments == 0 {
            self.run_with_cdb(&game, method, tt_conf, ())
        } else {
            todo!()
        }
    }

    fn run_with_cdb<CDB>(self, game: &chomp_skyline::Chomp, method: PruningMethod, tt_conf: TTConf, cdb: CDB) 
        where CDB: NimbersProvider<<chomp_skyline::Chomp as Game>::Position>,
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

    fn run_with_prot_tt_cdb<TT, CDB>(self, game: &chomp_skyline::Chomp, method: PruningMethod, tt: TT, protect_tt: bool, cdb: CDB) 
        where
         TT: NimbersProvider<<chomp_skyline::Chomp as Game>::Position> + NimbersStorer<<chomp_skyline::Chomp as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<chomp_skyline::Chomp as Game>::Position>,
    {   // TODO copied from Cram, should be fixed
        if protect_tt {
            let min_fields_to_protect = (self.cols+self.rows).saturating_sub(20);
            let tt = ProtectedTT::new(game, 
                format!("chomp_{}x{}_TT.bin", self.cols, self.rows),
                |_, p| p.count_ones() as u8 >= min_fields_to_protect,
                tt);
            self.run_with_tt_cdb(game, method, tt, cdb);
        } else {
            self.run_with_tt_cdb(game, method, tt, cdb);
        }
    }

    fn run_with_tt_cdb<TT, CDB>(self, game: &chomp_skyline::Chomp, method: PruningMethod, tt: TT, cdb: CDB) 
        where
         TT: NimbersProvider<<chomp_skyline::Chomp as Game>::Position> + NimbersStorer<<chomp_skyline::Chomp as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<chomp_skyline::Chomp as Game>::Position>,
    {
        let mut solver = Solver::new(
            game,
            tt,
            cdb,
            //move_sorter
            (), // TODO replace with FewerBarsFirst,
            Without
        );
        print_nimber_of_simple(&mut solver, method);
        println!("TT size: {}", solver.transposition_table.len());  // TODO move to print_nimber_of_simple
        println!("{}", solver.stats);   // TODO move to print_nimber_of_simple
    }
}