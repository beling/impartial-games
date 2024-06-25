use crate::Solver;
use crate::Game;
use crate::BitSet;
use crate::SolverEvent;

pub struct NaiveSolver<S = ()> {
    game: Game,
    nimbers: Vec<u16>,
    pub stats: S
}

impl<S: SolverEvent> Solver for NaiveSolver<S> {   
    type Stats = S;
    
    #[inline] fn stats(&self) -> &Self::Stats { &self.stats }
    #[inline] fn nimbers(&self) -> &[u16] { &self.nimbers }
    #[inline] fn game(&self) -> &Game { &self.game }
    #[inline] fn capacity(&self) -> usize { self.nimbers.capacity() }

    #[inline] fn with_stats(game: Game, stats: S) -> Self {
        Self { game, nimbers: Vec::new(), stats }
    }

    #[inline] fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        Self { game, nimbers: Vec::with_capacity(capacity), stats }
    }
}

impl<S: SolverEvent> Iterator for NaiveSolver<S> {
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
                option_nimbers.add_nimber(self.nimbers[i] ^ self.nimbers[after_take-i]);
                self.stats.break_option();
            }
        }
        let result = option_nimbers.mex();
        self.nimbers.push(result);
        Some(result)
    }
}