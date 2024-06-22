use ogs::{Game, NaiveSolver};

fn main() {
    for n in NaiveSolver::new(Game::from_ascii(b"4.007").unwrap(), 100) {
        print !("{} ", n)
    }
}