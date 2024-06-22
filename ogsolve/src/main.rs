use ogs::{Game, NaiveSolver, SolverIterations};

fn main() {
    let mut solver = NaiveSolver::with_capacity_stats(Game::from_ascii(b"4.007").unwrap(), 100, SolverIterations::default());
    for n in solver.by_ref().take(100) {
        print !("{} ", n)
    }
    println!();
    println!("Iterations: {}", solver.stats)
}