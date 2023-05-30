#![doc = include_str!("../README.md")]

pub mod dbs;
pub mod nimber_set;
pub mod game;
pub mod moves;
pub mod solver;
pub mod transposition_table;
pub mod enddb;
pub mod bit;
pub mod games;

pub use solver::stats as stats;