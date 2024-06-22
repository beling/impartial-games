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