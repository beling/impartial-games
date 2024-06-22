use ogs::{Game, NaiveSolver};

fn main() {
    for n in NaiveSolver::with_capacity(Game::from_ascii(b"4.007").unwrap(), 100).take(100) {
        print !("{} ", n)
    }
}