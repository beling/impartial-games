use clap::Args;

use crate::{solver::PruningMethod, tt::{TTConf, TTKind}, constdb::ConstDBConf};
use igs::{games::chomp, transposition_table::{NimbersProvider, ProtectedTT, NimbersStorer}, game::Game};

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
        let game = chomp::Chomp::new(self.cols, self.rows);
        if cdb.segments == 0 {
            self.run_with_cdb(&game, method, tt_conf, ())
        } else {
            todo!()
        }
    }

    fn run_with_cdb<CDB>(self, game: &chomp::Chomp, method: PruningMethod, tt_conf: TTConf, cdb: CDB) 
        where CDB: NimbersProvider<<chomp::Chomp as Game>::Position>,
    {
        match tt_conf.kind.unwrap_or_else(|| if game.board_size() > 40 { crate::tt::TTKind::Succinct } else { crate::tt::TTKind::HashMap }) {
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
    {
        if protect_tt {
            let min_fields_to_protect = game.board_size().saturating_sub(20);
            let tt = ProtectedTT::new(game, 
                format!("cram_{}x{}_TT.bin", self.cols, self.rows),
                |_, p| p.count_ones() as u8 >= min_fields_to_protect,
                tt);
            self.run_with_tt_cdb(game, method, tt, cdb);
        } else {
            self.run_with_tt_cdb(game, method, tt, cdb);
        }
    }

    fn run_with_tt_cdb<TT, CDB>(self, game: &Cram, method: PruningMethod, tt: TT, cdb: CDB) 
        where
         TT: NimbersProvider<<Cram as Game>::Position> + NimbersStorer<<Cram as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<Cram as Game>::Position>,
    {
        let mut solver = Solver::new(
            game,
            tt,
            cdb,
            //move_sorter
            SmallerComponentsFirst{},
            Without
        );
        print_nimber_of_decomposable(&mut solver, method);
        println!("TT size: {}", solver.transposition_table.len());  // TODO move to print_nimber_of_decomposable
        println!("{}", solver.stats);   // TODO move to print_nimber_of_decomposable
    }
}