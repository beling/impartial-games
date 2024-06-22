use crate::Game;
use crate::BitSet;

pub struct NaiveSolver {
    game: Game,
    nimbers: Box<[u16]>,
    n: usize
}

impl NaiveSolver {
    pub fn new(game: Game, max_nimber: usize) -> Self {
        Self { n: 0, game, nimbers: vec![0; max_nimber+1].into_boxed_slice() }
    }
}

impl Iterator for NaiveSolver {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n > self.nimbers.len() { return None; }

        let mut option_nimbers = [0u64; 1<<(16-6)]; // 2**16 bits
        if self.game.can_take_all(self.n) { option_nimbers.set_nimber(0) }
        for t in &self.game.taking {
            let t = *t as usize;
            if t >= self.n { break }
            option_nimbers.set_nimber(self.nimbers[self.n-t]);
        }
        for b in &self.game.breaking {
            let b = *b as usize;
            if b >= self.n { break }
            let after_take = self.n - b;
            for i in 1 .. after_take/2 + 1 {
                option_nimbers.set_nimber(self.nimbers[i] ^ self.nimbers[after_take-i]);
            }
        }
        let result = option_nimbers.mex();
        self.nimbers[self.n] = result;
        self.n += 1;
        Some(result)
    }
}