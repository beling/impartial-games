//! Built-in support for a few impartial games:
//! [Cram](https://en.wikipedia.org/wiki/Cram_(game)), [Chomp](https://en.wikipedia.org/wiki/Chomp)
//! (two models) and [Grundy's game](https://en.wikipedia.org/wiki/Grundy%27s_game).
//!
//! Each submodule implements the [`Game`](crate::game::Game) trait (plus
//! [`SimpleGame`](crate::game::SimpleGame) and/or [`DecomposableGame`](crate::game::DecomposableGame))
//! for a game type, which makes the game solvable by the library solvers.

pub mod chomp;
pub mod chomp_skyline;
pub mod cram;
pub mod grundy_game;

pub use chomp::Chomp;
pub use chomp_skyline::Chomp as ChompSkyline;
pub use cram::Cram;
pub use grundy_game::GrundyGame;