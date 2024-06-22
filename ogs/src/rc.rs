use crate::stats::NimberStats;
use crate::Game;
use crate::BitSet;
use crate::SolverEvent;

pub struct RCSplit {
    r: [u64; 1<<(16-6)],
    c: [u64; 1<<(16-6)],
    max_c: u16, // largest nimber in c
    r_positions: Vec<usize>,
}

impl Default for RCSplit {
    fn default() -> Self {
        let mut r = [0; 1<<(16-6)];
        r[0] = 1;   // adds 0 to r
        Self { r, c: [0; 1<<(16-6)], max_c: 0, r_positions: Default::default() }
    }
}

impl RCSplit {
    pub fn can_add_to_c(&self, nimber: u16) -> bool {
        for v in 1..=self.max_c {
            if self.c.get_nimber(v) && self.c.get_nimber(nimber ^ v) {
                return false;
            }
        }
        true
    }

    #[inline] pub fn add_to_c(&mut self, nimber: u16) {
        self.c.set_nimber(nimber);
        if nimber > self.max_c { self.max_c = nimber }
    }

    #[inline] pub fn add_to_r(&mut self, nimber: u16) {
        self.r.set_nimber(nimber);
    }

    #[inline] pub fn classify(&mut self, nimber: u16) {
        if self.can_add_to_c(nimber) {
            self.add_to_c(nimber)
        } else {
            self.add_to_r(nimber)
        }
    }

    pub fn clear(&mut self) {
        self.c.fill(0);
        self.r.fill(0);
        self.r[0] = 1;
        self.max_c = 0;
        self.r_positions.clear();
    }

    pub fn rebuild(&mut self, stats: &NimberStats, nimbers: &[u16]) {
        self.clear();
        for nimber in stats.nimbers_from_most_common() { self.classify(nimber); }
        for (position, nimber) in nimbers.iter().enumerate() {
            if self.r.get_nimber(*nimber) {
                self.r_positions.push(position);
            }
        }
    }
}

pub struct RCSolver<S> {
    game: Game,
    nimbers: Vec<u16>,
    nimber: NimberStats,
    split: RCSplit,
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