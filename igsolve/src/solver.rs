use std::{fmt::{Display, Formatter}, time::Instant};

use clap::{ValueEnum};
use igs::{stats::{StatsCollector, PrintProgress}, game::DecomposableGame, solver::{def::DefDecomposableGameSolver, lvb::LVBDecomposableGameSolver, br::BRDecomposableGameSolver}};

#[derive(ValueEnum, Clone, Debug)]
pub enum PruningMethod {
    Def,
    Lvb,
    Br,
    BrAspSet
}

pub struct Without;
impl StatsCollector for Without {}
impl Display for Without {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result { Ok(()) }
}

pub fn print_nimber_of_decomposable<'a, G, S>(solver: &mut S, method: PruningMethod)
where G: DecomposableGame,
      S: DefDecomposableGameSolver<G> + LVBDecomposableGameSolver<G> + BRDecomposableGameSolver<G>
{
    let now = Instant::now();
    let nimber = match method {
        PruningMethod::Def => solver.nimber_of_initial_def(),
        PruningMethod::Lvb => solver.nimber_of_initial_lvb_report_progress(PrintProgress),
        PruningMethod::Br => solver.nimber_of_initial_br(),
        PruningMethod::BrAspSet => solver.nimber_of_initial_br_aspset_report_progress(PrintProgress),
    };
    let calc_time = now.elapsed();
    println!("Nimber of initial position: {nimber}");
    println!("Calculation time: {calc_time:?}");
}