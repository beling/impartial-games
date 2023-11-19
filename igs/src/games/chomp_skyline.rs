use bitm::n_lowest_bits;

pub use crate::game::{Game, SimpleGame};
use crate::{solver::{StatsCollector, dedicated::BRSolver, Solver, SolverForSimpleGame}, bit::ExtraBitMethods};
use std::{fmt, iter::FusedIterator, collections::HashMap};


#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Chomp {
    cols: u8,
    rows: u8
}

impl Chomp {
    /// Construct Chomp game played on a board with given size (number of columns and rows).
    /// Note: `number_of_cols` should be greater or equal to `number_of_rows`, otherwise they are swapped.
    pub fn new(cols: u8, rows: u8) -> Self {
        assert!(cols > 0 && rows > 0 && cols + rows <= 64);
        if cols < rows {
            Self{ cols: rows, rows: cols }
        } else {
            Self{ cols, rows }
        }
    }

    fn normalized(position: u64) -> u64 {
        let mut mirrored = !(position<<1).reverse_bits();
        mirrored >>= mirrored.trailing_ones()+1;
        position.min(mirrored)
    }

    #[inline(always)]
    fn moves_count(mut p: u64) -> u16 {
        /*let mut moves = 0;  // total number of 1-0 pairs
        let zeros = !(p<<1);
        while p != 0 {
            moves += (p & zeros).count_ones() as u16; p >>= 1;
            moves += (p & zeros).count_ones() as u16; p >>= 1;
            moves += (p & zeros).count_ones() as u16; p >>= 1;
            moves += (p & zeros).count_ones() as u16; p >>= 1;
        }
        moves - 1*/

        /*let mut zeros = 1;  // total number of zeros (we have one, left-most 0 not included in representation)
        let mut moves = 0;  // total number of 1-0 pairs
        while p != 0 {
            let tz = p.trailing_zeros() as u16;
            p >>= tz + 1;   // is not correct if tz = 63
            //p >>= tz; p >>= 1;   // remove trailing zeros and the least significant 1
            zeros += tz;
            moves += zeros;
        }
        moves - 1*/

        let mut zeros = 1;  // total number of zeros (we have one, left-most 0 not included in representation)
        let mut moves = 0;  // total number of 1-0 pairs
        while p != 0 {
            let tz = p.trailing_zeros() as u16;
            p >>= tz;    // remove trailing zeros
            let to = p.trailing_ones() as u16;
            p >>= to;   // remove trailing ones
            zeros += tz;
            moves += to*zeros;
        }
        moves - 1

        /*let mut moves = p.count_ones() as u16;
        let zeros = !p;
        while p != 0 {
            let to = p.isolate_trailing_one();
            moves += (zeros & (to - 1)).count_ones() as u16;
            p ^= to;
        }
        moves - 1*/

        /*p <<= 1;
        const M1: u64 = (!0x01_01_01_01__01_01_01_01)>>1;
        const M2: u64 = (!0x03_03_03_03__03_03_03_03)>>2;
        const M3: u64 = (!0x07_07_07_07__07_07_07_07)>>3;
        const M4: u64 = (!0x0F_0F_0F_0F__0F_0F_0F_0F)>>4;
        const M5: u64 = (!0x1F_1F_1F_1F__1F_1F_1F_1F)>>5;
        const M6: u64 = (!0x3F_3F_3F_3F__3F_3F_3F_3F)>>6;
        const M7: u64 = (!0x7F_7F_7F_7F__7F_7F_7F_7F)>>7;
        let mut ones = p;
        let zeros = !ones;
        let mut moves = // here we count 1-0 pairs inside each byte
            ((ones>>1) & (zeros&M1)).count_ones() as u16 +
            ((ones>>2) & (zeros&M2)).count_ones() as u16 +
            ((ones>>3) & (zeros&M3)).count_ones() as u16 +
            ((ones>>4) & (zeros&M4)).count_ones() as u16 +
            ((ones>>5) & (zeros&M5)).count_ones() as u16 +
            ((ones>>6) & (zeros&M6)).count_ones() as u16 +
            ((ones>>7) & (zeros&M7)).count_ones() as u16;
        ones >>= 8;
        if ones != 0 {
            let mut total_zeros = (zeros & 0xFF).count_ones() as u16;
            loop {
                let new_ones = (ones & 0xFF).count_ones() as u16;
                moves += total_zeros * new_ones;
                ones >>= 8;
                if ones == 0 { break }
                total_zeros += 8 - new_ones;
            }
        }
        moves - 1*/
    }
}

impl fmt::Display for Chomp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "Chomp{}x{}_skyline_repr", self.number_of_cols, self.number_of_rows)
        write!(f, "Chomp_skyline_repr")
    }
}

// młodsze <-> starsze
// 01101011011

impl Game for Chomp {
    type Position = u64;
    type NimberSet = [u64; 4];
    //type NimberSet = u128;
    //type NimberSet = u64;

    #[inline(always)]
    fn moves_count(&self, p: &u64) -> u16 { // TODO try more efficient implementation with pre-calculating for each 1-byte
        Self::moves_count(*p)
    }

    fn initial_position(&self) -> Self::Position {
        n_lowest_bits(self.rows) << (self.cols-1)
    }

    /*fn try_solve_theoretically(&self, position: &Self::Position) -> Option<u8> {
        if self.row(*position, 1) <= 1 {
            Some((self.rows_count(*position) - 1) ^ (self.row(*position, 0) - 1))
        } else {
            None
        }
    }*/
}

impl SimpleGame for Chomp {
    type Successors<'s> = ChompMovesIterator/*<'s>*/;
    type HeuristicallyOrderedSuccessors<'s> = ChompMovesIterator/*<'s>*/;

    fn successors(&self, position: &Self::Position) -> Self::Successors<'_> {
        ChompMovesIterator::new(*position)
    }

    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_> {
        ChompMovesIterator::new(*position)
    }

    fn solver_with_stats<'s, STATS: 's+StatsCollector>(&'s self, stats: STATS) -> Box<dyn SolverForSimpleGame<Game=Self, StatsCollector=STATS> + 's> {
        Box::new(BRSolver{
            solver: Solver::new(self, HashMap::new(), (), () /* TODO */, stats)
        })
    }
}

impl_serializable_game_for!(Chomp);


pub struct ChompMovesIterator/*<'a>*/ {
    //chomp: &'a Chomp,
    position: u64,
    one_idx: u8,  // index of the current 1
    zero_idx: i8,  // index of the current 0
    number_of_ones: u8,    // number of ones in range [0, one_idx]
    ones_mask: u64,    // 0..01..1 with the number of ones in range [zero_idx, one_idx]
    zeros: u64, // 0s to process with the current 1
    result_template: u64
}

impl ChompMovesIterator {
    pub fn new(position: u64) -> Self {
        let number_of_ones = position.count_ones() as u8;
        let one_idx = position.ilog2() as u8;
        Self {
            position,
            one_idx,
            zero_idx: -1,
            number_of_ones,
            ones_mask: n_lowest_bits(number_of_ones),
            zeros: !position & n_lowest_bits(one_idx),
            result_template: 0, 
        }
    }
}

impl Iterator for ChompMovesIterator/*<'_>*/ {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.zeros == 0 {
            // goto next 1:
            let remaining_ones = self.position & n_lowest_bits(self.one_idx);
            if remaining_ones == 0 { return None; }
            self.one_idx = remaining_ones.ilog2() as u8;
            self.result_template = self.position ^ remaining_ones;
            self.zero_idx = -1;
            self.number_of_ones -= 1;
            self.ones_mask = n_lowest_bits(self.number_of_ones);
            self.zeros = !self.position & n_lowest_bits(self.one_idx);
            Some(Chomp::normalized(self.result_template >> self.number_of_ones))
        } else {
            // goto next 0:
            let zero_idx = self.zeros.trailing_zeros() as i8;
            self.zeros ^= 1 << zero_idx;
            self.ones_mask >>= zero_idx - self.zero_idx - 1;
            self.zero_idx = zero_idx;
            self.result_template |= self.position & n_lowest_bits(zero_idx as u8);
            Some(Chomp::normalized(self.result_template | (self.ones_mask << zero_idx)))
        }
    }
}

impl FusedIterator for ChompMovesIterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        assert_eq!(Chomp::normalized(0b1), 0b1);    // X -> X
        assert_eq!(Chomp::normalized(0b11), 0b10);  // #/X -> #X
        assert_eq!(Chomp::normalized(0b10), 0b10);  // #X -> #X
        assert_eq!(Chomp::normalized(0b1101), 0b1010);
        assert_eq!(Chomp::normalized(0b1010), 0b1010);
        assert_eq!(Chomp::normalized(0b1110), 0b1100);
        assert_eq!(Chomp::normalized(0b1100), 0b1100);
    }

    #[test]
    fn moves_iterator_for_1010() {
        //   ##     #   ##           #
        //  ##X ->  X   #X   ##X   ##X
        // 1010    11  110   100  1001
        assert_eq!(Chomp::moves_count(0b1010), 4);
        let mut iter = ChompMovesIterator::new(0b1010);
        assert_eq!(iter.next(), Some(Chomp::normalized(0b11)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b110)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b100)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b1001)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn moves_iterator_for_1100() {
        //  ###     #   ##           #    ##
        //  ##X ->  X   #X   ##X   ##X   ##X
        // 1100    11  110   100  1001  1010
        assert_eq!(Chomp::moves_count(0b1100), 5);
        let mut iter = ChompMovesIterator::new(0b1100);
        assert_eq!(iter.next(), Some(Chomp::normalized(0b11)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b110)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b100)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b1001)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b1010)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn moves_iterator_for_1111() {
        //   #
        //   #             #
        //   #  ->     #   #
        //   X     X   X   X
        // 1111    1  11  111
        assert_eq!(Chomp::moves_count(0b1111), 3);
        let mut iter = ChompMovesIterator::new(0b1111);
        assert_eq!(iter.next(), Some(Chomp::normalized(0b1)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b11)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b111)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn moves_iterator_for_100() {
        //  ##X  ->  X  #X
        //  100      1  10
        assert_eq!(Chomp::moves_count(0b100), 2);
        let mut iter = ChompMovesIterator::new(0b100);
        assert_eq!(iter.next(), Some(Chomp::normalized(0b1)));
        assert_eq!(iter.next(), Some(Chomp::normalized(0b10)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn moves_iterator_for_end_position() {
        assert_eq!(Chomp::moves_count(0b1), 0);
        let mut iter = ChompMovesIterator::new(0b1);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}