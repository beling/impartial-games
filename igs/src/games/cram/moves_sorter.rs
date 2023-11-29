pub use crate::game::Game;
use crate::moves::DifficultEvaluator;
use crate::enddb::EndDb;
use super::Cram;

/// Implements `DifficultEvaluator` for `Cram`.
/// This evaluator requires and works well with `EndDb`.
/// It also should be constructed from `EndDb`, after construction of all slices of end db.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CramDifficultEvaluator {
    /// Bits of the position representation that lies outside the mask increase difficult.
    /// Correct value can be calculated from Cram end database.
    outside_mask: u64
}

impl DifficultEvaluator for CramDifficultEvaluator {
    type Game = Cram;
    type PositionDifficult = u16;

    fn difficult_of(&self, _game: &Cram, to_evaluate: &<Cram as Game>::Position) -> Self::PositionDifficult {
        let v = ((to_evaluate.count_ones() << 3) + (to_evaluate & self.outside_mask).count_ones()) as u16;
        v * v
    }
}

impl<SliceType> From<&EndDb<&Cram, SliceType>> for CramDifficultEvaluator {
    fn from(enddb: &EndDb<&Cram, SliceType>) -> Self {
        let size = (enddb.slices.len() * (1<<32)) as u64;
        Self { outside_mask:
        if size == 0 { !0 }
        // !((size.next_power_of_two()).wrapping_sub(1) //??
        else if size.is_power_of_two() { !(size.wrapping_sub(1)) }
        else { !((size.next_power_of_two()>>1).wrapping_sub(1)) }
        }
    }
}

/// Move sorter and difficult evaluator which evaluate positions with less components and more empty fields as harder to solve.
pub struct SmallerComponentsFirst;

impl DifficultEvaluator for SmallerComponentsFirst {
    type Game = Cram;
    type PositionDifficult = u16;

    fn difficult_of(&self, _game: &Cram, to_evaluate: &<Cram as Game>::Position) -> Self::PositionDifficult {
        let v = to_evaluate.count_ones() as u16;
        v * v
    }
}

/// Move sorter and difficult evaluator which evaluate positions with less components and more moves as harder to solve.
pub struct LessMovesFirst;

impl DifficultEvaluator for LessMovesFirst {
    type Game = Cram;
    type PositionDifficult = u16;

    fn difficult_of(&self, game: &Cram, to_evaluate: &<Cram as Game>::Position) -> Self::PositionDifficult {
        let v = game.moves_count(to_evaluate) as u16;
        v * v
    }
}