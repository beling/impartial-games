use ogs::{Game, NaiveSolver, RC2Solver, RCSolver, SolverIterations};

fn main() {
    let game = Game::from_ascii(b"4.007").unwrap();
    //let game = Game::from_ascii(b"0.161").unwrap();
    let iters = 100;
    println!("naive:");
    let mut solver = NaiveSolver::with_capacity_stats(game.clone(), iters, SolverIterations::default());
    for n in solver.by_ref().take(iters) {
        print !("{} ", n)
    }
    println!();
    println!("Iterations: {}", solver.stats);
    println!();
    println!("RC:");
    let mut solver = RCSolver::with_capacity_stats(game.clone(), iters, SolverIterations::default());
    for n in solver.by_ref().take(iters) {
        print !("{} ", n)
    }
    println!();
    println!("Iterations: {}", solver.stats);
    println!();
    println!("Iterations: {}", solver.stats);
    println!();
    println!("RC2:");
    let mut solver = RC2Solver::with_capacity_stats(game, iters, SolverIterations::default());
    for n in solver.by_ref().take(iters) {
        print !("{} ", n)
    }
    println!();
    println!("Iterations: {}", solver.stats)
}