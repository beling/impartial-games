use crate::Game;
use crate::BitSet;
use crate::SolverEvent;

pub struct NaiveSolver<S> {
    game: Game,
    nimbers: Vec<u16>,
    n: usize,
    pub stats: S
}

impl<S> NaiveSolver<S> {
    pub fn with_stats(game: Game, stats: S) -> Self {
        Self { n: 0, game, nimbers: Vec::new(), stats }
    }

    pub fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        Self { n: 0, game, nimbers: Vec::with_capacity(capacity), stats }
    }
}

impl NaiveSolver<()> {
    pub fn new(game: Game) -> Self {
        Self { n: 0, game, nimbers: Vec::new(), stats: () }
    }

    pub fn with_capacity(game: Game, capacity: usize) -> Self {
        Self { n: 0, game, nimbers: Vec::with_capacity(capacity), stats: () }
    }
}

impl<S: SolverEvent> Iterator for NaiveSolver<S> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut option_nimbers = [0u64; 1<<(16-6)]; // 2**16 bits
        if self.game.can_take_all(self.n) { option_nimbers.set_nimber(0) }
        for t in &self.game.taking {
            let t = *t as usize;
            if t >= self.n { break }
            option_nimbers.set_nimber(self.nimbers[self.n-t]);
            self.stats.take_option();
        }
        for b in &self.game.breaking {
            let b = *b as usize;
            if b >= self.n { break }
            let after_take = self.n - b;
            for i in 1 .. after_take/2 + 1 {
                option_nimbers.set_nimber(self.nimbers[i] ^ self.nimbers[after_take-i]);
                self.stats.break_option();
            }
        }
        let result = option_nimbers.mex();
        self.nimbers.push(result);
        self.n += 1;
        Some(result)
    }
}