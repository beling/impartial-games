use crate::stats::NimberStats;
use crate::Game;
use crate::BitSet;
use crate::SolverEvent;

struct Split {
    r: [u64; 1<<(16-6)],
    c: [u64; 1<<(16-6)],
    r_positions: Vec<usize>,
}

impl Default for Split {
    fn default() -> Self {
        let mut r = [0; 1<<(16-6)];
        r[0] = 1;   // adds 0 to r
        Self { r, c: [0; 1<<(16-6)], r_positions: Default::default() }
    }
}

pub struct RCSolver<S> {
    game: Game,
    nimbers: Vec<u16>,
    nimber: NimberStats,
    split: Split,
    pub stats: S
}

impl<S> RCSolver<S> {
    pub fn with_stats(game: Game, stats: S) -> Self {
        Self { game, nimbers: Vec::new(), nimber: Default::default(), stats, split: Default::default() }
    }

    pub fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        Self { game, nimbers: Vec::with_capacity(capacity), nimber: Default::default(), stats, split: Default::default() }
    }
}

impl RCSolver<()> {
    pub fn new(game: Game) -> Self {
        Self { game, nimbers: Vec::new(), nimber: Default::default(), stats: (), split: Default::default() }
    }

    pub fn with_capacity(game: Game, capacity: usize) -> Self {
        Self { game, nimbers: Vec::with_capacity(capacity), nimber: Default::default(), stats: (), split: Default::default() }
    }
}

impl<S: SolverEvent> Iterator for RCSolver<S> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut option_nimbers = [0u64; 1<<(16-6)]; // 2**16 bits
        let n = self.nimbers.len();
        self.game.consider_taking(&self.nimbers, &mut option_nimbers, &mut self.stats);
        for b in &self.game.breaking {
            let b = *b as usize;
            if b >= n { break }
            let after_take = n - b;
            for i in 1 .. after_take/2 + 1 {
                option_nimbers.set_nimber(self.nimbers[i] ^ self.nimbers[after_take-i]);
                self.stats.break_option();
            }
        }
        let result = option_nimbers.mex();
        self.nimber.count(result);
        self.nimbers.push(result);
        Some(result)
    }
}