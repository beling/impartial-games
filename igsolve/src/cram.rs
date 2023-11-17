use std::collections::HashMap;
use clap::Args;
use igs::{games::cram::{Cram, slices_provider::LimitedColumnsSliceProvider, SmallerComponentsFirst}, enddb::{EndDb, PrintStats, EndDbBuilderForDecomposableGame}, transposition_table::{NimbersProvider, TTSuccinct64, ProtectedTT, bit_mixer::stafford13, cluster_policy::Fifo, NimbersStorer}, game::Game, solver::Solver};
use crate::{solver::{PruningMethod, print_nimber_of_decomposable, Without}, tt::{TTConf, TTKind}, constdb::ConstDBConf};

#[derive(Args, Clone, Copy)]
pub struct CramConf {
    /// Number of rows
    #[arg(short='r', long)]
    rows: u8,

    /// Number of columns
    #[arg(short='c', long)]
    cols: u8,

    /// Maximum number of columns of positions included in the end database or 0 for no limit
    #[arg(long, default_value_t=6)]
    edb_cols: u8
}

impl CramConf {
    pub fn run(self, method: Option<PruningMethod>, tt_conf: TTConf, cdb: ConstDBConf) {
        let method = method.unwrap_or(PruningMethod::BrAspSet);
        println!("---=== Cram {}x{} {:?} ===---", self.cols, self.rows, method);
        let game = Cram::new(self.cols, self.rows);
        if cdb.segments == 0 {
            self.run_with_cdb(&game, method, tt_conf, ())
        } else if self.edb_cols == 0 || self.edb_cols >= self.cols {    // no columns limit?
            let mut enddb = EndDb::build_with_lsmap_verifier(
                &game,
                PrintStats::default());
            for _ in 0..cdb.segments { enddb.build_slice_cached(&game, "igsolve_enddb").unwrap(); }
            self.run_with_cdb(&game, method, tt_conf, enddb.done())
        } else {
            let mut enddb = EndDb::build_with_lsmap_verifier(
                LimitedColumnsSliceProvider::new(&game, self.edb_cols),
                PrintStats::default());
            for _ in 0..cdb.segments { enddb.build_slice_cached(&game, format!("igsolve_enddb_{}cols", self.edb_cols)).unwrap(); }   
            // TODO edb_cols should be deeper in path
            self.run_with_cdb(&game, method, tt_conf, enddb.done())
        }
    }

    fn run_with_cdb<CDB>(self, game: &Cram, method: PruningMethod, tt_conf: TTConf, cdb: CDB) 
        where CDB: NimbersProvider<<Cram as Game>::Position>,
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

    fn run_with_prot_tt_cdb<TT, CDB>(self, game: &Cram, method: PruningMethod, tt: TT, protect_tt: bool, cdb: CDB) 
        where
         TT: NimbersProvider<<Cram as Game>::Position> + NimbersStorer<<Cram as Game>::Position> + igs::dbs::HasLen,
         CDB: NimbersProvider<<Cram as Game>::Position>,
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

/*fn solve_cram<'a, TT, EDB, SORTER, S>(solver: &mut Solver<'a, Cram, TT, EDB, SORTER, S>, method: PruningMethod)
    where TT: NimbersProvider<<Cram as Game>::Position> + NimbersStorer<<Cram as Game>::Position> + igs::dbs::HasLen,
          EDB: NimbersProvider<<Cram as Game>::Position>,
          SORTER: DecomposableGameMoveSorter<Cram>,
          S: StatsCollector + Display
{
    println!("---=== Cram {}x{} {:?} ===---", solver.game.number_of_columns(), solver.game.number_of_rows(), method);
    print_nimber_of_decomposable(solver, solver.game.init_pos(), method);
    println!("TT size: {}", solver.transposition_table.len());
    println!("{}", solver.stats);
}*/

/*#[allow(dead_code)]
fn benchmark_tt<BitMixer: Fn(u64, u64) -> u64, Policy: ClusterPolicy>(tt_name: &str, game: &Cram, bit_mixer: BitMixer, cluster_policy: Policy) {
    println!("{tt_name}");
    let mut solver = Solver::new(
        game,
        //HashMap::new(), // 11113419 entries; mx3 11113417;
        //TTSuccinct64::new(4 /*16*/ + 28 /*GB*/, 2, 3),
        TTSuccinct64::new(23, 2, 4, bit_mixer, cluster_policy),
        (), //enddb,
        SmallerComponentsFirst{},
        EventStats::default()
    );
    solve_cram(&mut solver, PruningMethod::BrAspSet);
}*/

/*fn solve_cram9x7() {
    let game = Cram::new(9,7);
    let mut enddb = EndDb::build_with_bdzmap_verifier(
        LimitedColumnsSliceProvider::new(&game, 6),
        PrintStats::default());
    for _ in 0..16 { enddb.build_slice_cached(&game, "enddb_6cols").unwrap(); }
    let tt = TTSuccinct64::new(5 /*32*/ + 28 /*GB*/, 2, 4, stafford13, Fifo);
    let tt = ProtectedTT::new(&game, "protectedTT.bin",
                              |_, p| {p.count_ones() >= 9*7-20},
                              tt);
    let mut solver = Solver::new(
        &game,
        tt,
        enddb.done(),
        //move_sorter
        SmallerComponentsFirst{},
        Without
    );
    solve_cram(&mut solver, PruningMethod::BrAspSet);
}*/

/*fn test_solver() {
    let cram = Cram::new(6,5);
    let mut solver = Solver::new(
        &cram,
        HashMap::new(),
        (),
        SmallerComponentsFirst{},
        EventStats::default()
    );
    solve_cram(&mut solver, PruningMethod::BrAspSet);

    let mut enddb = EndDb::build_with_bdzmap_verifier(
        &cram,
        PrintStats::default());
    enddb.build(&cram, None, Some(("whole", true)));
    check_results(&cram, solver.transposition_table, &enddb.done(), true);
}*/

// enddb: 1-32  2-33  4-34  8-35  16-36  32-37  64-38  128-39  256-40  512-41  1024-42
// 7, 14, 21, 28, 35, 42

/*fn check_results<I: IntoIterator<Item=(u64, u8)>, P: NimbersProvider<u64>>(cram: &Cram, to_check: I, provider: &P, report_missing: bool) {
    for (p, v) in to_check {
        if let Some(expected) = provider.get_nimber(&p) {
            if expected != v {
                println!("Expected value {} but got {} for position:\n{}", expected, v, cram.pos_to_multi_line_str(p));
            }
        } else if report_missing {
            println!("No expected value for position (got {}):\n{}", v, cram.pos_to_multi_line_str(p));
        }
    }
}*/