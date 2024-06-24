mod set;
mod game;
mod stats;

// solvers:
mod naive;
mod rc;
mod rc2;

pub use set::BitSet;
pub use game::Game;
pub use stats::{SolverEvent, SolverIterations};

pub use naive::NaiveSolver;
pub use rc::RCSolver;