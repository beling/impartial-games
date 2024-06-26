use std::fmt::Display;

use clap::{Parser, ValueEnum};
use ogs::{Game, NaiveSolver, RC2Solver, RCSolver, Solver, SolverIterations};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Method {
    /// Naive
    Naive,
    /// RC
    RC,
    /// RC2
    RC2
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Method::Naive => write!(f, "naive"),
            Method::RC => write!(f, "RC"),
            Method::RC2 => write!(f, "RC2"),
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Conf {
    /// Game to solve
    game: Game,

    /// Method of calculating nimbers
    #[arg(short='m', value_enum, default_value_t=Method::Naive)]
    pub method: Method,

    /// The (last) position which nimber should be found
    #[arg(short='n', default_value_t = 10_000)]
    pub nimber: usize,

    /// Whether to print nimbers
    #[arg(short='p', long, default_value_t = false)]
    pub print_nimbers: bool,
}

/// Calculates checksum with fletcher 32 algorithm.
fn checksum(nimbers: &[u16]) -> u32 {
    let mut checksum = (0u16, 0u16);
    for n in nimbers {
        checksum.0 += checksum.0.wrapping_add(*n);
        checksum.1 += checksum.1.wrapping_add(checksum.0);
    }
    ((checksum.1 as u32) << 16) | checksum.0 as u32
}

impl Conf {
    fn run<S: Solver>(self) where S::Stats: Default+Display {
        let mut solver = S::with_capacity(self.game, self.nimber+1);
        if self.print_nimbers { print!("Nimbers: ") }
        for n in solver.by_ref().take(self.nimber+1) {
            if self.print_nimbers { print!(" {}", n) }
        }
        if self.print_nimbers { println!() }
        println!("Nimber of {}: {}, checksum: {}", self.nimber, solver.nimbers().last().unwrap(), checksum(solver.nimbers()));
        println!("{} iterations: {}", self.method, solver.stats());
        solver.print_nimber_stat().unwrap();
    }    
}

fn main() {
    let conf: Conf = Conf::parse();
    match conf.method {
        Method::Naive => conf.run::<NaiveSolver<SolverIterations>>(),
        Method::RC => conf.run::<RCSolver<SolverIterations>>(),
        Method::RC2 => conf.run::<RC2Solver<SolverIterations>>(),
    }
}