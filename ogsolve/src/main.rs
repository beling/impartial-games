use std::{fmt::Display, fs::File, io::Write, time::Instant};

use clap::{Parser, ValueEnum, ArgAction};
use ogs::{Game, NaiveSolver, RC2Solver, RCSolver, Solver, SolverIterations};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Method {
    /// Naive
    Naive,
    /// RC
    RC,
    /// RC with static moments of rebuilding the R/C split
    RCS,
    /// RC2
    RC2,
    /// RC2 with static moments of rebuilding the R/C split
    RC2S,
    /// Predict the number of iterations of naive methods without calculating nimbers
    PredictNaive
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Method::Naive|Method::PredictNaive => write!(f, "naive"),
            Method::RC => write!(f, "rc"),
            Method::RCS => write!(f, "rcs"),
            Method::RC2 => write!(f, "rc2"),
            Method::RC2S => write!(f, "rc2s"),
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Conf {
    /// Game to solve
    game: Game,

    /// Method(s) of calculating nimbers
    #[arg(short='m', ignore_case = true, default_value="naive", action=ArgAction::Append, value_delimiter=',')]
    pub method: Vec<Method>,

    /// The (last) position which nimber should be found
    #[arg(short='n', default_value_t = 10_000)]
    pub position: usize,

    /// Print nimbers
    #[arg(short='p', long, default_value_t = false)]
    pub print_nimbers: bool,

    /// Print nimber and solver statistics
    #[arg(short='s', long="stats", default_value_t = false)]
    pub print_stats: bool,

    /// Save the benchmark results to a file with the given name (ogsolve_benchmark.csv by default)
    #[arg(short='b', long="benchmark", num_args=0..=1, default_missing_value="ogsolve_benchmark.csv", value_name="FILE_NAME")]
    pub benchmark_filename: Option<String>,
}

/// Calculates checksum with fletcher 32 algorithm.
fn checksum(nimbers: &[u16]) -> u32 {
    let mut checksum = (0u16, 0u16);
    for n in nimbers {
        checksum.0 = checksum.0.wrapping_add(*n);
        checksum.1 = checksum.1.wrapping_add(checksum.0);
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
const BENCHMARK_HEADER: &'static str = "game, positions, method, checksum, period, preperiod, take_iter, break_iter, rc_effort, rc_rebuilds, time_micros, zeros_count";

impl Conf {
    fn predicted_naive_stats(&self) -> SolverIterations {
        SolverIterations{ taking: self.game.taking_iters(self.position), breaking: self.game.breaking_naive_iters(self.position), ..Default::default() }
    }

    fn run<S: Solver<Stats = SolverIterations>>(&self, method: Method) /*where S::Stats: Default+Display*/ {
        println!("Solving {} with {}:", self.game.to_string(), method);
        let mut solver = S::with_capacity(self.game.clone(), self.position+1);
        if self.print_nimbers { print!(" nimbers:") }
        let start_moment = Instant::now();
        let mut zeros = 0;
        for n in solver.by_ref().take(self.position+1) {
            if self.print_nimbers { print!(" {}", n) }
            if n == 0 { zeros += 1 }
        }
        let time = start_moment.elapsed();
        if self.print_nimbers { println!() }
        let period = solver.period();
        if let Some((preperiod, period)) = period {
            println!(" found period of length {period} and pre-period {preperiod}")
        }
        let checksum = checksum(solver.nimbers());
        println!(" nimber of {}: {}  losing positions: {:.2}%  checksum: {:X}", self.position, solver.nimbers().last().unwrap(), 100.0 * zeros as f64 / solver.nimbers().len() as f64, checksum);
        let stats = solver.stats();
        println!(" iterations:  {stats}");
        println!(" calculation time: {time:#.2?}");
        if self.print_stats { solver.print_nimber_stat().unwrap(); }
        if let Some(ref filename) = self.benchmark_filename {
            let (p, pp) = if let Some((preperiod, period)) = period {
                (period.to_string(), preperiod.to_string())
            } else {
                ("".to_owned(), "".to_owned())
            };
            writeln!(csv_file(&filename, BENCHMARK_HEADER), "{}, {}, {}, {:X}, {}, {}, {}, {}, {}, {}, {}, {}",
                solver.game().to_string(), self.position, method, checksum, p, pp,
                stats.taking, stats.breaking, stats.rebuilding_r_positions, stats.rebuilding_rc, time.as_micros(), zeros).unwrap();
        }
    }    
}

fn main() {
    let conf: Conf = Conf::parse();
    let naive_iters = conf.predicted_naive_stats();
    println!("Predicted number of naive iterations to solve {}: {}", conf.game.to_string(), naive_iters);
    for method in conf.method.iter().copied() {
        match method {
            Method::Naive => conf.run::<NaiveSolver<SolverIterations>>(method),
            Method::RC => conf.run::<RCSolver<true, SolverIterations>>(method),
            Method::RCS => conf.run::<RCSolver<false, SolverIterations>>(method),
            Method::RC2 => conf.run::<RC2Solver<true, SolverIterations>>(method),
            Method::RC2S => conf.run::<RC2Solver<false, SolverIterations>>(method),
            Method::PredictNaive => {
                if let Some(ref filename) = conf.benchmark_filename {
                    writeln!(csv_file(&filename, BENCHMARK_HEADER), "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                        conf.game.to_string(), conf.position, method, "", "", "",
                        naive_iters.taking, naive_iters.breaking, naive_iters.rebuilding_r_positions, naive_iters.rebuilding_rc, "").unwrap();
                }
            },
        }
    }
}