use crate::rcsplit::RCSplit;
use crate::stats::NimberStats;
use crate::Game;
use crate::BitSet;
use crate::Solver;
use crate::SolverEvent;

pub struct RCSolver<const DYNAMIC_REBUILD: bool = true, S = ()> {
    game: Game,
    nimbers: Vec<u16>,
    nimber_num: NimberStats,
    split: RCSplit,
    //nimbers_by_num: HashMap<u32, HashSet<u16>>,
    pub stats: S
}

impl<const DYNAMIC_REBUILD: bool, S: SolverEvent> Solver for RCSolver<DYNAMIC_REBUILD, S> {   
    type Stats = S;
    
    #[inline] fn stats(&self) -> &Self::Stats { &self.stats }
    #[inline] fn nimbers(&self) -> &[u16] { &self.nimbers }
    #[inline] fn game(&self) -> &Game { &self.game }
    #[inline] fn capacity(&self) -> usize { self.nimbers.capacity() }

    #[inline] fn with_stats(game: Game, stats: S) -> Self {
        Self { game, nimbers: Vec::new(), nimber_num: Default::default(), /*nimbers_by_num: Default::default(),*/ stats, split: Default::default() }
    }

    #[inline] fn with_capacity_stats(game: Game, capacity: usize, stats: S) -> Self {
        Self { game, nimbers: Vec::with_capacity(capacity), nimber_num: Default::default(), /*nimbers_by_num: Default::default(),*/ stats, split: Default::default() }
    }
    
    fn print_nimber_stat_to(&self, f: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(f, "{}", self.nimber_num)?;
        writeln!(f, "{}", self.split)
    }
}

impl<const DYNAMIC_REBUILD: bool, S: SolverEvent> Iterator for RCSolver<DYNAMIC_REBUILD, S> {
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
        if DYNAMIC_REBUILD {
            /*let result_occ = self.nimber_num.occurences[result as usize];
            if result_occ > 1 { self.nimbers_by_num.get_mut(&(result_occ-1)).unwrap().remove(&result); }
            self.nimbers_by_num.entry(result_occ).or_default().insert(result);*/
            if self.split.r.contain_nimber(result) {
                if n != 0 { self.split.r_positions.push(n); }
                if self.split.should_rebuild(result, &self.nimber_num) {
                    self.split.update(&self.nimber_num, &self.nimbers, &mut self.stats);
                }
            }
        } else {
            if n.is_power_of_two() {
                self.split.rebuild(&self.nimber_num, &self.nimbers, &mut self.stats);
            } else if self.split.r.contain_nimber(result) && n != 0 {
                self.split.r_positions.push(n);
            }
        }

        //self.split.rebuild(&self.nimber, &self.nimbers);
        Some(result)
    }
}