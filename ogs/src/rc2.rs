use crate::{Solver, SolverEvent};
use crate::{rc::RCSplit, Game, BreakingMoveIterator};
use crate::stats::NimberStats;
use crate::BitSet;

pub struct RC2Solver<S = ()> {
    game: Game,
    breaking: [Vec<u8>; 2], // breaking moves splitted to even and odd
    nimbers: Vec<u16>,
    nimber_num: NimberStats,
    split: [RCSplit; 2],
    pub stats: S
}

impl<S: SolverEvent> Solver for RC2Solver<S> {   
    type Stats = S;
    
    #[inline] fn stats(&self) -> &Self::Stats { &self.stats }
    #[inline] fn nimbers(&self) -> &[u16] { &self.nimbers }
    #[inline] fn game(&self) -> &Game { &self.game }
    #[inline] fn capacity(&self) -> usize { self.nimbers.capacity() }

    #[inline] fn with_stats(game: Game, stats: S) -> Self {
        let breaking = Self::split_breaking_moves(&game);
        Self { game, breaking, nimbers: Vec::new(), nimber_num: Default::default(), stats, split: [RCSplit::new(0), RCSplit::new(1)] }
    }

    #[inline] fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        let breaking = Self::split_breaking_moves(&game);
        Self { game, breaking, nimbers: Vec::with_capacity(capacity), nimber_num: Default::default(), stats, split: [RCSplit::new(0), RCSplit::new(1)] }
    }
}

impl<S> RC2Solver<S> {
    fn split_breaking_moves(game: &Game) -> [Vec<u8>; 2] {
        let mut result = [Vec::<u8>::new(), Vec::<u8>::new()];
        for (i, m) in game.breaking.iter().enumerate() {
            result[i & 1].push(*m);
        }
        result
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
        let mut to_check = [self.split[0].in_r(result, 0), self.split[1].in_r(result, 1)];
        let mut moves = [
            BreakingMoveIterator::for_slice(n, &self.breaking[0]).fuse(),
            BreakingMoveIterator::for_slice(n, &self.breaking[1]).fuse()
        ];
        while to_check[0] || to_check[1] {
            for d in [0, 1] {
                while to_check[d] {
                    if let Some((a, b)) = moves[d].next() {
                        let option_nimber = self.nimbers[a] ^ self.nimbers[b];
                        option_nimbers.add_nimber(option_nimber);
                        if (result>>1) == option_nimber {
                            result = (option_nimbers.mex() << 1) | nd;
                            to_check = [self.split[0].in_r(result, 0), self.split[1].in_r(result, 1)];
                        }
                        self.stats.break_option();
                    } else { to_check[d] = false; }
                }
            }
        }
        self.nimber_num.count(result);
        self.nimbers.push(result>>1);
        for d in [0, 1] {
            if self.split[d].r.contain_nimber(result) {
                if n != 0 { self.split[d].r_positions.push(n); }
                if self.split[d].should_rebuild_d(result, &self.nimber_num) {
                    self.split[d].rebuild_d(&self.nimber_num, &self.nimbers, d as u16);
                    self.stats.rebuilding_rc();
                }
                self.split[d].rebuild_d(&self.nimber_num, &self.nimbers, d as u16);
            } else {
                self.split[d].add_to_c(result);
            }
        }
        /*self.nimber_num.print_as_pairs(); println!();
        self.split[0].print_as_pairs(); println!();
        self.split[1].print_as_pairs();*/
        //self.split.rebuild(&self.nimber, &self.nimbers);
        Some(result>>1)
    }
}