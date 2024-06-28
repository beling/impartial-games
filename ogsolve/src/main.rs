use std::{fmt::Display, fs::File, io::Write};

use clap::{Parser, ValueEnum};
use ogs::{Game, NaiveSolver, RC2Solver, RCSolver, Solver, SolverIterations};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Method {
    /// Naive
    Naive,
    /// RC
    RC,
    /// RC2
    RC2,
    /// Predict the number of iterations of naive methods without calculating nimbers
    PredictNaive
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Method::Naive|Method::PredictNaive => write!(f, "naive"),
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
    pub position: usize,

    /// Print nimbers
    #[arg(short='p', long, default_value_t = false)]
    pub print_nimbers: bool,

    /// Save the benchmark results to a file with the given name or to ogsolve_benchmark.csv
    #[arg(short='b', long="benchmark", num_args=0..=1, default_missing_value="ogsolve_benchmark.csv", value_name="FILE_NAME")]
    pub save_benchmark: Option<String>,
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


/// Either opens or crates (and than put headers inside) and returns the file with given `file_name` (+`csv` extension).
fn csv_file(file_name: &str, header: &str) -> File {
    //let file_name = format!("{}.csv", file_name);
    let file_already_existed = std::path::Path::new(&file_name).exists();
    let mut file = std::fs::OpenOptions::new().append(true).create(true).open(&file_name).unwrap();
    if !file_already_existed { writeln!(file, "{}", header).unwrap(); }
    file
}

//const BENCHMARK_FILENAME: &'static str = "ogsolve_benchmark";
const BENCHMARK_HEADER: &'static str = "game, positions, method, checksum, take_iter, break_iter, rc_effort, rc_rebuilds";

impl Conf {
    fn predicted_naive_stats(&self) -> SolverIterations {
        SolverIterations{ taking: self.game.taking_iters(self.position), breaking: self.game.breaking_naive_iters(self.position), ..Default::default() }
    }

    fn run<S: Solver<Stats = SolverIterations>>(self) /*where S::Stats: Default+Display*/ {        
        let mut solver = S::with_capacity(self.game, self.position+1);
        if self.print_nimbers { print!("Nimbers: ") }
        for n in solver.by_ref().take(self.position+1) {
            if self.print_nimbers { print!(" {}", n) }
        }
        if self.print_nimbers { println!() }
        let checksum = checksum(solver.nimbers());
        println!("Nimber of {}: {}, checksum: {:X}", self.position, solver.nimbers().last().unwrap(), checksum);
        let stats = solver.stats();
        println!("{} iterations: {}", self.method, stats);
        solver.print_nimber_stat().unwrap();
        if let Some(filename) = self.save_benchmark {
            writeln!(csv_file(&filename, BENCHMARK_HEADER), "{}, {}, {}, {:X}, {}, {}, {}, {}",
                solver.game().to_string(), self.position, self.method, checksum,
                stats.taking, stats.breaking, stats.rebuilding_rc_nimbers_len, stats.rebuilding_rc).unwrap();
        }
    }    
}

fn main() {
    let conf: Conf = Conf::parse();
    let naive_iters = conf.predicted_naive_stats();
    println!("Predicted number of naive iterations: {}", naive_iters);
    match conf.method {
        Method::Naive => conf.run::<NaiveSolver<SolverIterations>>(),
        Method::RC => conf.run::<RCSolver<SolverIterations>>(),
        Method::RC2 => conf.run::<RC2Solver<SolverIterations>>(),
        Method::PredictNaive => {
            if let Some(filename) = conf.save_benchmark {
                writeln!(csv_file(&filename, BENCHMARK_HEADER), "{}, {}, {}, {}, {}, {}, {}, {}",
                    conf.game.to_string(), conf.position, conf.method, "",
                    naive_iters.taking, naive_iters.breaking, naive_iters.rebuilding_rc_nimbers_len, naive_iters.rebuilding_rc).unwrap();
            }
        },
    }
}