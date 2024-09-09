use crate::{Game, SolverEvent};

pub trait Solver: Iterator<Item=u16> {
    type Stats: SolverEvent;

    fn stats(&self) -> &Self::Stats;
    fn nimbers(&self) -> &[u16];
    fn game(&self) -> &Game;
    fn capacity(&self) -> usize;

    fn with_stats(game: Game, stats: Self::Stats) -> Self;
    fn with_capacity_stats(game: Game, capacity: usize, stats: Self::Stats) -> Self;

    #[inline] fn new(game: Game) -> Self where Self: Sized, Self::Stats: Default {
        Self::with_stats(game, Default::default())
    }
    
    #[inline] fn with_capacity(game: Game, capacity: usize) -> Self where Self: Sized, Self::Stats: Default {
        Self::with_capacity_stats(game, capacity, Default::default())
    }

    fn print_nimber_stat_to(&self, f: &mut dyn std::io::Write) -> std::io::Result<()>;

    fn print_nimber_stat(&self) -> std::io::Result<()> {
        self.print_nimber_stat_to(&mut std::io::stdout().lock())
    }

    /// Try to calculates (pre-period, period) of the game using the nimbers calculated so far.
    fn period(&self) -> Option<(usize, usize)> {
        self.game().period(self.nimbers())
    }
}