use std::fmt::Display;

pub trait SolverEvent {
    #[inline] fn take_option(&mut self) {}
    #[inline] fn break_option(&mut self) {}
    #[inline] fn rebuilding_rc(&mut self, _nimbers_len: usize) {}
}

impl SolverEvent for () {}

#[derive(Default, Clone, Copy)]
pub struct SolverIterations {
    pub taking: usize,
    pub breaking: usize,
    pub rebuilding_rc: usize,
    pub rebuilding_r_positions: usize
}

impl SolverEvent for SolverIterations {
    #[inline] fn take_option(&mut self) { self.taking += 1; }
    #[inline] fn break_option(&mut self) { self.breaking += 1; }
    #[inline] fn rebuilding_rc(&mut self, rebuilding_r_positions: usize) {
         self.rebuilding_rc += 1;
         self.rebuilding_r_positions += rebuilding_r_positions
    }
}

impl Display for SolverIterations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.taking != 0 { write!(f, "taking: {}  ", self.taking)?; }
        if self.breaking != 0 { write!(f, "breaking: {}  ", self.breaking)?; }
        write!(f, "total: {}", self.taking+self.breaking)?;
        if self.rebuilding_rc != 0 { write!(f, "  RC effort/rebuilds: {}/{}", self.rebuilding_r_positions, self.rebuilding_rc)?; }
        Ok(())
    }
}

pub struct NimberStats {
    pub occurences: [u32; 1<<16],
    pub max: u16
}

impl Default for NimberStats {
    fn default() -> Self { Self { occurences: [0; 1<<16], max: 0 } }
}

impl NimberStats {
    pub fn count(&mut self, nimber: u16) {
        self.occurences[nimber as usize] += 1;
        if nimber > self.max { self.max = nimber; }
    }

    /// Returns sorted vector of nimbers, from the most to the less commmon, skip chosen nimber
    pub fn nimbers_from_most_common(&self, to_skip: u16) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.max as usize);
        if to_skip == 0 {
            for nimber in 1..=self.max {
                if self.occurences[nimber as usize] != 0 {
                    result.push(nimber);
                }
            }
        } else {
            for nimber in 0..=self.max {
                if nimber != to_skip && self.occurences[nimber as usize] != 0 {
                    result.push(nimber);
                }
            }
        }
        // we use stable sort to lower nimber be the first in the case of tie
        result.sort_by(|a, b| self.occurences[*b as usize].cmp(&self.occurences[*a as usize]));
        result
    }
}

impl Display for NimberStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_plus() {  // pairs:
            for nimber in (0..=self.max).step_by(2) {
                let occ = (self.occurences[nimber as usize], self.occurences[nimber as usize + 1]);
                if occ.0 != 0 || occ.1 != 0 {
                    if nimber != 0 { write!(f, "\t")?; }
                    write!(f, "{:>2}:{}+{}", nimber>>1, occ.0, occ.1)?;
                }
            }
        } else {
            for nimber in 0..=self.max {
                let occ = self.occurences[nimber as usize];
                if occ != 0 {
                    if nimber != 0 { write!(f, "\t")?; }
                    write!(f, "{:>2}:{}", nimber, occ)?;
                }
            }
        }
        Ok(())
    }
}