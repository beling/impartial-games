#![doc = include_str!("../README.md")]

use clap::{Parser, Subcommand};
mod solver;
use solver::PruningMethod;

mod tt;
use tt::TTConf;

mod constdb;
use constdb::ConstDBConf;

mod cram;
mod chomp;
mod chomp_skyline;
mod grundy_game;


//#[allow(non_camel_case_types)]
//#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[derive(Subcommand)]
pub enum GameConf {
    /// solve Cram
    Cram(cram::Conf),
    /// solve Chomp
    Chomp(chomp::Conf),
    /// solve Chomp using alternative, "skyline" model
    ChompSL(chomp_skyline::Conf),
    /// solve Grundy's game
    Grundy(grundy_game::Conf),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Impartial games solver.
pub struct Conf {
    /// Game to solve
    #[command(subcommand)]
    pub game: GameConf,

    #[command(flatten)]
    pub tt: TTConf,

    #[command(flatten)]
    pub cdb: ConstDBConf,

    /// Pruning method. The default value depends on the game being solved
    #[arg(short='m', long, value_enum)]
    pub method: Option<PruningMethod>
}

fn main() {
    let conf: Conf = Conf::parse();
    match conf.game {
        GameConf::Cram(cram_conf) => cram_conf.run(conf.method, conf.tt, conf.cdb),
        GameConf::Chomp(chomp_conf) => chomp_conf.run(conf.method, conf.tt, conf.cdb),
        GameConf::ChompSL(chomp_conf) => chomp_conf.run(conf.method, conf.tt, conf.cdb),
        GameConf::Grundy(grundy_conf) => grundy_conf.run(conf.method, conf.tt, conf.cdb),
    }
}
