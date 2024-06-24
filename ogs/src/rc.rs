use crate::stats::NimberStats;
use crate::Game;
use crate::BitSet;
use crate::SolverEvent;

pub(crate) struct RCSplit {
    pub(crate) r: [u64; 1<<(16-6)],
    pub(crate) c: [u64; 1<<(16-6)],
    pub(crate) max_c: u16, // largest nimber in c
    pub(crate) r_positions: Vec<usize>,
}

impl Default for RCSplit {
    fn default() -> Self {
        let mut r = [0; 1<<(16-6)];
        r[0] = 1;   // adds 0 to r
        Self { r, c: [0; 1<<(16-6)], max_c: 0, r_positions: Default::default() }
    }
}

impl RCSplit {
    #[inline] pub fn can_add_to_c(&self, nimber: u16) -> bool {
        for v in 1..=self.max_c {
            if self.c.contain_nimber(v) && self.c.contain_nimber(nimber ^ v) {
                return false;
            }
        }
        true
    }

    #[inline] pub fn add_to_c(&mut self, nimber: u16) {
        self.c.add_nimber(nimber);
        if nimber > self.max_c { self.max_c = nimber }
    }

    #[inline] pub fn add_to_r(&mut self, nimber: u16) {
        self.r.add_nimber(nimber);
    }

    #[inline] pub fn add_to(&mut self, nimber: u16, to_c: bool) -> bool {
        if to_c {
            self.add_to_c(nimber)
        } else {
            self.add_to_r(nimber)
        }
        to_c
    }

    #[inline] pub fn classify(&mut self, nimber: u16) -> bool {
        self.add_to(nimber, self.can_add_to_c(nimber))
    }

    #[inline] pub fn classify_d(&mut self, nimber: u16, d: u16) -> bool {
        self.add_to(nimber, self.can_add_to_c(nimber ^ d))
    }

    pub fn in_c(&mut self, nimber: u16) -> bool {
        if self.c.contain_nimber(nimber) { return true; }
        if self.r.contain_nimber(nimber) { return false; }
        self.classify(nimber)
    }

    /// Never adds nimber to c.
    pub fn in_r(&mut self, nimber: u16, d: u16) -> bool {
        if self.c.contain_nimber(nimber) { return false; }
        if self.r.contain_nimber(nimber) { return true; }
        if self.can_add_to_c(nimber ^ d) {
            false
        } else {
            self.r.add_nimber(nimber);
            true
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
        for nimber in stats.nimbers_from_most_common() {
            self.classify(nimber);
        }
        for position in 1..nimbers.len() {
            if self.r.contain_nimber(nimbers[position]) {
                self.r_positions.push(position);
            }
        }
    }

    pub fn rebuild_d(&mut self, stats: &NimberStats, nimbers: &[u16], d: u16) {
        self.clear();
        for nimber in stats.nimbers_from_most_common() {
            self.classify_d(nimber, d);
        }
        for position in 1..nimbers.len() {
            if self.r.contain_nimber((nimbers[position]<<1) | (position as u16 & 1)) {
                self.r_positions.push(position);
            }
        }
    }

    pub fn should_rebuild(&self, recent_nimber: u16, stats: &NimberStats) -> bool {
        let r_occ = stats.occurences[recent_nimber as usize];
        for c in 1..=stats.max {
            let c_occ = stats.occurences[c as usize];
            if c_occ == 0 || !self.c.contain_nimber(c) { continue; }
            let c_grater = c > recent_nimber;
            if (c_grater && r_occ == c_occ) || (!c_grater && r_occ == c_occ+1) {
                return true;
            }
        }
        false
    }
}

pub struct RCSolver<S> {
    game: Game,
    nimbers: Vec<u16>,
    nimber_num: NimberStats,
    split: RCSplit,
    pub stats: S
}

impl<S> RCSolver<S> {
    pub fn with_stats(game: Game, stats: S) -> Self {
        Self { game, nimbers: Vec::new(), nimber_num: Default::default(), stats, split: Default::default() }
    }

    pub fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        Self { game, nimbers: Vec::with_capacity(capacity), nimber_num: Default::default(), stats, split: Default::default() }
    }
}

impl RCSolver<()> {
    pub fn new(game: Game) -> Self {
        Self::with_stats(game, ())
    }

    pub fn with_capacity(game: Game, capacity: usize) -> Self {
        Self::with_capacity(game, capacity)
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
            if b+1 >= n { break }
            let after_take = n - b;
            for i in &self.split.r_positions {
                if *i >= after_take { break; }
                option_nimbers.add_nimber(self.nimbers[*i] ^ self.nimbers[after_take-i]);
                self.stats.break_option();
            }
        }
        let mut result = option_nimbers.mex();
        if !self.split.in_c(result) {
            'outer: for b in &self.game.breaking {
                let b = *b as usize;
                if b+1 >= n { break }
                let after_take = n - b;
                for i in 1 .. after_take/2 + 1 {
                    let option_nimber = self.nimbers[i] ^ self.nimbers[after_take-i];
                    option_nimbers.add_nimber(option_nimber);
                    self.stats.break_option();
                    if result == option_nimber {
                        result = option_nimbers.mex();
                        if self.split.in_c(result) {
                            break 'outer;
                        }
                    }
                }
            }
        }
        self.nimber_num.count(result);
        self.nimbers.push(result);
        if self.split.r.contain_nimber(result) {
            if n != 0 { self.split.r_positions.push(n); }
            if self.split.should_rebuild(result, &self.nimber_num) {
                self.split.rebuild(&self.nimber_num, &self.nimbers);
                self.stats.rebuilding_rc();
            }
        }
        //self.split.rebuild(&self.nimber, &self.nimbers);
        Some(result)
    }
}