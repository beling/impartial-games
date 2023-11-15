pub use crate::game::{Game, SimpleGame};
use std::fmt;


#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Chomp {

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
    fn moves_count(&self, p: &u64) -> u16 {
        let mut p = *p;
        let mut zeros = 0;  // total number of zeros
        // TODO let mut zeros = 1; if the least significant 1 is not included in p
        let mut moves = 0;  // total number of 1-0 pairs
        while p != 0 {
            let tz = p.trailing_zeros() as u16;
            p >>= tz + 1;   // remove trailing zeros and the least significant 1
            zeros += tz;
            moves += zeros;
        }
        moves - 1
    }

    /*#[inline(always)]
    fn moves_count(&self, p: &u64) -> u16 {
        const M1: u64 = (!0x01_01_01_01__01_01_01_01)>>1;
        const M2: u64 = (!0x03_03_03_03__03_03_03_03)>>2;
        const M3: u64 = (!0x07_07_07_07__07_07_07_07)>>3;
        const M4: u64 = (!0x0F_0F_0F_0F__0F_0F_0F_0F)>>4;
        const M5: u64 = (!0x1F_1F_1F_1F__1F_1F_1F_1F)>>5;
        const M6: u64 = (!0x3F_3F_3F_3F__3F_3F_3F_3F)>>6;
        const M7: u64 = (!0x7F_7F_7F_7F__7F_7F_7F_7F)>>7;

        let mut ones = *p;
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

        moves - 1
    }*/

    /*fn try_solve_theoretically(&self, position: &Self::Position) -> Option<u8> {
        if self.row(*position, 1) <= 1 {
            Some((self.rows_count(*position) - 1) ^ (self.row(*position, 0) - 1))
        } else {
            None
        }
    }*/
}



pub struct ChompMovesIterator<'a> {
    chomp: &'a Chomp,
    position: u64,

}

impl Iterator for ChompMovesIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}