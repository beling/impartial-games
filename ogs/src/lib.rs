mod set;
mod game;
mod stats;

mod naive;
mod rc;

pub use set::BitSet;
pub use game::Game;
pub use stats::{SolverEvent, SolverIterations};

pub use naive::NaiveSolver;
pub use rc::RCSolver;