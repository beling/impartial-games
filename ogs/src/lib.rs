mod set;
mod game;
mod solver;
mod stats;

// solvers:
mod naive;
mod rcsplit;
mod rc;
mod rc2;

pub use set::BitSet;
pub use game::{Game, BreakingMoveIterator};
pub use stats::{SolverEvent, SolverIterations};

pub use solver::Solver;
pub use naive::NaiveSolver;
pub use rc::RCSolver;
pub use rc2::RC2Solver;