#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! `igs` is a library for solving [impartial games](https://en.wikipedia.org/wiki/Impartial_game)
//! under the [normal play convention](https://en.wikipedia.org/wiki/Normal_play_convention),
//! i.e. for determining the [nimbers](https://en.wikipedia.org/wiki/Nimber) (Grundy numbers) of game positions.
//!
//! Main components:
//! - `game` defines the traits (`Game`, `SimpleGame`, `DecomposableGame`) that a game has to implement
//!   to be solvable, and `games` includes built-in support for [Cram](https://en.wikipedia.org/wiki/Cram_(game)),
//!   [Chomp](https://en.wikipedia.org/wiki/Chomp) (two models) and [Grundy's game](https://en.wikipedia.org/wiki/Grundy%27s_game);
//! - `solver` provides several nimber-calculation algorithms (by definition, Lemoine-Viennot's, Beling's);
//! - `transposition_table` provides hash tables for caching already calculated nimbers;
//! - `enddb` provides endgame databases for storing nimbers of positions close to the end of a game;
//! - `dbs`, `nimber_set`, `bit` and `moves` define auxiliary types used by the solver.

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