use std::fmt::Display;

pub trait SolverEvent {
    #[inline] fn take_option(&mut self) {}
    #[inline] fn break_option(&mut self) {}
}

impl SolverEvent for () {}

#[derive(Default, Clone, Copy)]
pub struct SolverIterations {
    pub taking: usize,
    pub breaking: usize,
}

impl SolverEvent for SolverIterations {
    #[inline] fn take_option(&mut self) { self.taking += 1; }
    #[inline] fn break_option(&mut self) { self.breaking += 1; }
}

impl Display for SolverIterations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "taking: {}, breaking: {}, total: {}", self.taking, self.breaking, self.taking+self.breaking)
    }
}

pub struct NimberStats {
    pub occurences: [usize; 1<<16],
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

    /// Returns sorted vector of nimbers, from the most to the less commmon
    pub fn nimbers_from_most_common(&self) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.max as usize);
        for nimber in 1..=self.max {
            if self.occurences[nimber as usize] != 0 {
                result.push(nimber);
            }
        }
        // we use stable sort to lower nimber be the first in the case of tie
        result.sort_by(|a, b| self.occurences[*b as usize].cmp(&self.occurences[*a as usize]));
        result
    }
}