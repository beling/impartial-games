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
    pub rebuilding_rc_nimbers_len: usize
}

impl SolverEvent for SolverIterations {
    #[inline] fn take_option(&mut self) { self.taking += 1; }
    #[inline] fn break_option(&mut self) { self.breaking += 1; }
    #[inline] fn rebuilding_rc(&mut self, nimbers_len: usize) {
         self.rebuilding_rc += 1;
         self.rebuilding_rc_nimbers_len += nimbers_len
    }
}

impl Display for SolverIterations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.taking != 0 { write!(f, "taking: {}, ", self.taking)?; }
        if self.breaking != 0 { write!(f, "breaking: {}, ", self.breaking)?; }
        write!(f, "total: {}", self.taking+self.breaking)?;
        if self.rebuilding_rc != 0 { write!(f, ", R/C rebuilds: {} x{:.1}", self.rebuilding_rc, self.rebuilding_rc_nimbers_len as f64 / self.rebuilding_rc as f64)?; }
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

    /*pub fn print_as_pairs(&self) {
        for nimber in 0..=self.max {
            let occ = self.occurences[nimber as usize];
            if occ != 0 {
                if nimber != 0 { print!(", ") }
                print!("({},{})x{}", nimber>>1, nimber&1, occ)
            }
        }
    }*/
}