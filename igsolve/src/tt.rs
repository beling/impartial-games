use clap::ValueEnum;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum TTKind {
    /// Disable Transposition Table
    None,
    /// Standard HashMap
    HashMap,
    /// Succinct Implementation
    Succinct
}

#[derive(clap::Args)]
pub struct TTConf {

    /// Implementation of transposition table to use. The default value depends on the game being solved
    #[arg(long="tt", value_enum)]
    pub kind: Option<TTKind>,

    /// Limit of TT size in GB
    #[arg(long="tt_size")]
    pub size: Option<usize>,

    /// Whether to save the most valuable part of the transposition table to disk (so that calculations can be resumed)
    #[arg(long="tt_protection", default_value_t = false)]
    pub protect: bool,
}

impl TTConf {
    pub fn size_log2(&self, default_size: usize) -> u8 {
        self.size.unwrap_or(default_size).checked_ilog2().unwrap_or(0) as u8
    }
}

/*let mut freq = [0u64; 9*5+1];
for (p, _) in solver.transposition_table {
    freq[p.count_ones() as usize] += 1;
}
let sum = freq.iter().sum::<u64>() as f64;
let mut s = 0;
for (n, v) in freq.iter().enumerate().rev() {
    s += v;
    println!("{:2} {} {} {:.2} {:.2}", n, v, s, (v*100) as f64 / sum, (s*100) as f64 / sum)
}*/