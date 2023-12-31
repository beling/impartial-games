use std::collections::HashMap;

use clap::Args;

use crate::{solver::{PruningMethod, Without, print_nimber_of_decomposable}, tt::{TTConf, TTKind}, constdb::ConstDBConf};
use igs::{games::GrundyGame, transposition_table::{NimbersProvider, NimbersStorer}, game::Game, solver::Solver};

#[derive(Args, Clone, Copy)]
pub struct Conf {
    /// initial position (stack height)
    #[arg(short='p', long)]
    position: u16,
}

impl Conf {
    pub fn run(self, method: Option<PruningMethod>, tt_conf: TTConf, cdb: ConstDBConf) {
        let method = method.unwrap_or(PruningMethod::Def);
        println!("---=== Grundy's game {} {:?} ===---", self.position, method);
        let game = GrundyGame(self.position);
        if cdb.segments == 0 {
            self.run_with_cdb(&game, method, tt_conf, ())
        } else {
            todo!("end-db is not yet supported for Grundy's game")
        }
    }

    fn run_with_cdb<CDB>(self, game: &GrundyGame, method: PruningMethod, tt_conf: TTConf, cdb: CDB) 
        where CDB: NimbersProvider<<GrundyGame as Game>::Position>,
    {   // TODO copied from Cram, should be fixed
        match tt_conf.kind.unwrap_or_else(|| crate::tt::TTKind::HashMap) {
            TTKind::None => self.run_with_prot_tt_cdb(game, method, (), tt_conf.protect, cdb),
            TTKind::HashMap => self.run_with_prot_tt_cdb(game, method, HashMap::new(), tt_conf.protect, cdb),
            TTKind::Succinct => todo!("Succinct TT is not yet supported for Grundy's game"),
        }
    }

    fn run_with_prot_tt_cdb<TT, CDB>(self, game: &GrundyGame, method: PruningMethod, tt: TT, protect_tt: bool, cdb: CDB) 
        where
         TT: NimbersProvider<<GrundyGame as Game>::Position> + NimbersStorer<<GrundyGame as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<GrundyGame as Game>::Position>,
    {
        if protect_tt {
            todo!("TT protection is not yet supported for Grundy's game")
        } else {
            self.run_with_tt_cdb(game, method, tt, cdb);
        }
    }

    fn run_with_tt_cdb<TT, CDB>(self, game: &GrundyGame, method: PruningMethod, tt: TT, cdb: CDB) 
        where
         TT: NimbersProvider<<GrundyGame as Game>::Position> + NimbersStorer<<GrundyGame as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<GrundyGame as Game>::Position>,
    {
        let mut solver = Solver::new(
            game,
            tt,
            cdb,
            //move_sorter
            (),
            Without
        );
        print_nimber_of_decomposable(&mut solver, method);
        println!("TT size: {}", solver.transposition_table.len());  // TODO move to print_nimber_of_decomposable
        println!("{}", solver.stats);   // TODO move to print_nimber_of_decomposable
    }
}