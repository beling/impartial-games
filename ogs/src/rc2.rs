use crate::SolverEvent;
use crate::{rc::RCSplit, Game, BreakingMoveIterator};
use crate::stats::NimberStats;
use crate::BitSet;

pub struct RC2Solver<S> {
    game: Game,
    breaking: [Vec<u8>; 2], // breaking moves splitted to even and odd
    nimbers: Vec<u16>,
    nimber: NimberStats,
    split: [RCSplit; 2],
    pub stats: S
}


impl<S> RC2Solver<S> {
    fn split_breaking_moves(game: &Game) -> [Vec<u8>; 2] {
        let mut result = [Vec::<u8>::new(), Vec::<u8>::new()];
        for (i, m) in game.breaking.iter().enumerate() {
            result[i & 1].push(*m);
        }
        result
    }

    pub fn with_stats(game: Game, stats: S) -> Self {
        let breaking = Self::split_breaking_moves(&game);
        Self { game, breaking, nimbers: Vec::new(), nimber: Default::default(), stats, split: Default::default() }
    }

    pub fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        let breaking = Self::split_breaking_moves(&game);
        Self { game, breaking, nimbers: Vec::with_capacity(capacity), nimber: Default::default(), stats, split: Default::default() }
    }
}

impl RC2Solver<()> {
    pub fn new(game: Game) -> Self {
        Self::with_stats(game, ())
    }

    pub fn with_capacity(game: Game, capacity: usize) -> Self {
        Self::with_capacity(game, capacity)
    }
}

impl<S: SolverEvent> Iterator for RC2Solver<S> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut option_nimbers = [0u64; 1<<(16-6)]; // 2**16 bits
        let n = self.nimbers.len();
        self.game.consider_taking(&self.nimbers, &mut option_nimbers, &mut self.stats);
        for d in [0, 1] {
            for b in &self.breaking[d] {
                let b = *b as usize;
                if b+1 >= n { break }
                let after_take = n - b;
                for i in &self.split[d].r_positions {
                    if *i >= after_take { break; }
                    option_nimbers.add_nimber(self.nimbers[*i] ^ self.nimbers[after_take-i]);
                    self.stats.break_option();
                }
            }
        }
        let nd = n as u16 & 1;
        let mut result = (option_nimbers.mex() << 1) | nd;
        let mut must_move = [self.split[0].in_r(result), self.split[1].in_r(result)];
        let mut moves = [
            BreakingMoveIterator::for_slice(n, &self.breaking[0]).fuse(),
            BreakingMoveIterator::for_slice(n, &self.breaking[1]).fuse()
        ];
        while must_move[0] || must_move[1] {
            for d in [0, 1] {
                while must_move[d] {
                    if let Some((a, b)) = moves[d].next() {
                        let option_nimber = self.nimbers[a] ^ self.nimbers[b];
                        option_nimbers.add_nimber(option_nimber);
                        if result>>1 == option_nimber {
                            result = (option_nimbers.mex() << 1) | nd;
                            must_move = [self.split[0].in_r(result), self.split[1].in_r(result)];
                        }
                        self.stats.break_option();
                    } else { must_move[d] = false; }
                }
            }
        }
        self.nimber.count(result);
        self.nimbers.push(result>>1);
        if self.split.r.contain_nimber(result) {
            if n != 0 { self.split.r_positions.push(n); }
            if self.split.should_rebuild(result, &self.nimber) {
                self.split.rebuild(&self.nimber, &self.nimbers);
                self.stats.rebuilding_rc();
            }
        }
        //self.split.rebuild(&self.nimber, &self.nimbers);
        Some(result>>1)
    }
}