use crate::game::Game;
use super::Cram;
use crate::bit::{lowest_bit_of, ExtraBitMethods};
use std::iter::FusedIterator;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct CramSimpleMovesIterator {
    position: u64,
    horizontal_moves_left: u64,
    vertical_moves_left: u64,
    number_of_columns: u8
}

impl CramSimpleMovesIterator {
    pub fn new(cram: &Cram, position: <Cram as Game>::Position) -> Self {
        Self {
            position,
            horizontal_moves_left: cram.horizontal_moves(position),
            vertical_moves_left: cram.vertical_moves(position),
            number_of_columns: cram.number_of_cols
        }
    }
}

impl Iterator for CramSimpleMovesIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vertical_moves_left != 0 {
            let l = lowest_bit_of(self.vertical_moves_left);
            self.vertical_moves_left ^= l;
            Some(self.position ^ l ^ (l<<self.number_of_columns))
        } else if self.horizontal_moves_left != 0 {
            let l = lowest_bit_of(self.horizontal_moves_left);
            self.horizontal_moves_left ^= l;
            Some(self.position ^ l ^ (l<<1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.len();
        (r, Some(r))
    }
}

impl ExactSizeIterator for CramSimpleMovesIterator {
    #[inline(always)]
    fn len(&self) -> usize {
        (self.horizontal_moves_left.count_ones() + (self.vertical_moves_left).count_ones()) as usize
    }
}

impl FusedIterator for CramSimpleMovesIterator {}



pub struct Cram2ColumnsMovesIterator {
    position: u64,
    horizontal_moves_left: u64,
    horizontal_moves_mask: u64,
    vertical_moves_left: u64,
    vertical_moves_mask: u64,
    number_of_columns: u8
}

impl Cram2ColumnsMovesIterator {
    pub fn new(cram: &Cram, position: <Cram as Game>::Position) -> Self {
        Self {
            position,
            horizontal_moves_left: cram.horizontal_moves(position),
            horizontal_moves_mask: cram.center_row(position),
            vertical_moves_left: cram.vertical_moves(position),
            vertical_moves_mask: cram.center_column(position),
            number_of_columns: cram.number_of_cols
        }
    }
}

impl Iterator for Cram2ColumnsMovesIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vertical_moves_left != 0 {
            let mut current_moves = self.vertical_moves_left & self.vertical_moves_mask;
            while current_moves == 0 {
                self.vertical_moves_mask |= (self.vertical_moves_mask << 1) | (self.vertical_moves_mask >> 1);
                current_moves = self.vertical_moves_left & self.vertical_moves_mask;
            }
            let l = lowest_bit_of(current_moves);
            self.vertical_moves_left ^= l;
            Some(self.position ^ l ^ (l<<self.number_of_columns))
        } else if self.horizontal_moves_left != 0 {
            let mut current_moves = self.horizontal_moves_left & self.horizontal_moves_mask;
            while current_moves == 0 {
                self.horizontal_moves_mask |= (self.horizontal_moves_mask << self.number_of_columns) | (self.horizontal_moves_mask >> self.number_of_columns);
                current_moves = self.horizontal_moves_left & self.horizontal_moves_mask;
            }
            let l = lowest_bit_of(current_moves);
            self.horizontal_moves_left ^= l;
            Some(self.position ^ l ^ (l<<1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.len();
        (r, Some(r))
    }
}

impl ExactSizeIterator for Cram2ColumnsMovesIterator {
    #[inline(always)]
    fn len(&self) -> usize {
        (self.horizontal_moves_left.count_ones() + self.vertical_moves_left.count_ones()) as usize
    }
}

impl FusedIterator for Cram2ColumnsMovesIterator {}



pub struct CramCenterFirstMovesIterator {
    position: u64,
    horizontal_moves_left: u64,
    vertical_moves_left: u64,
    vertical_moves_mask: u64,
    expand_left: bool,
    number_of_columns: u8
}

impl CramCenterFirstMovesIterator {
    pub fn new(cram: &Cram, position: <Cram as Game>::Position) -> Self {
        Self {
            position,
            horizontal_moves_left: cram.horizontal_moves(position),
            vertical_moves_left: cram.vertical_moves(position),
            vertical_moves_mask: cram.center_column(position),
            expand_left: true,
            number_of_columns: cram.number_of_cols
        }
    }
}

impl Iterator for CramCenterFirstMovesIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vertical_moves_left != 0 {
            let mut current_moves = self.vertical_moves_left & self.vertical_moves_mask;
            while current_moves == 0 {
                //self.vertical_moves_mask |= (self.vertical_moves_mask << 1) | (self.vertical_moves_mask >> 1);
                if self.expand_left {
                    self.vertical_moves_mask |= self.vertical_moves_mask << 1;
                    self.expand_left = false;
                } else {
                    self.vertical_moves_mask |= self.vertical_moves_mask >> 1;
                    self.expand_left = true;
                }

                current_moves = self.vertical_moves_left & self.vertical_moves_mask
            }
            let l = if self.expand_left { lowest_bit_of(current_moves) } else { current_moves.isolate_leading_one() };
            self.vertical_moves_left ^= l;
            Some(self.position ^ l ^ (l<<self.number_of_columns))
        } else if self.horizontal_moves_left != 0 {
            let l = lowest_bit_of(self.horizontal_moves_left);
            self.horizontal_moves_left ^= l;
            Some(self.position ^ l ^ (l<<1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.len();
        (r, Some(r))
    }
}

impl ExactSizeIterator for CramCenterFirstMovesIterator {
    #[inline(always)]
    fn len(&self) -> usize {
        (self.horizontal_moves_left.count_ones() + (self.vertical_moves_left).count_ones()) as usize
    }
}

impl FusedIterator for CramCenterFirstMovesIterator {}



pub struct CramOptimalMovesIterator {
    values: BinaryHeap<(Reverse<u8>, u64)>
}

impl CramOptimalMovesIterator {
    fn add_dist_to_center_penalty(values: &mut [u8]) {
        let center = values.len() as u8 / 2;
        for i in 0..center { values[i as usize] += center-i; }
        for i in center+1..values.len() as u8 { values[i as usize] += i-center; }
    }

    pub fn new(cram: &Cram, position: <Cram as Game>::Position) -> Self {
        let mut columns = Vec::with_capacity(cram.number_of_cols as _);
        let mut p = position;
        while p != 0 {
            columns.push((p & cram.first_column_mask).count_ones() as u8);
            p &= !cram.first_column_mask;
            p >>= 1;
        }
        Self::add_dist_to_center_penalty(&mut columns);
        let mut rows = Vec::with_capacity(cram.number_of_rows as _);
        p = position;
        while p != 0 {
            rows.push((p & cram.first_row_mask).count_ones() as u8);
            p >>= cram.number_of_cols;
        }
        Self::add_dist_to_center_penalty(&mut rows);
        let mut v = cram.vertical_moves(position);
        let mut h = cram.horizontal_moves(position);
        let mut scored_moves = Vec::with_capacity((v.count_ones() + h.count_ones()) as usize);

        while v != 0 {
            let n = v.trailing_zeros() as u8;
            let b = 1 << n;
            v ^= b;
            let r = (n / cram.number_of_cols) as usize;
            let c = (n % cram.number_of_cols) as usize;
            let c = columns[c];
            scored_moves.push((Reverse(c+c+rows[r]+rows[r+1]),
                               position ^ b ^ (b<<cram.number_of_cols)));
        }

        while h != 0 {
            let n = h.trailing_zeros() as u8;
            let b = 1 << n;
            h ^= b;
            let r = (n / cram.number_of_cols) as usize;
            let c = (n % cram.number_of_cols) as usize;
            let r = rows[r];
            scored_moves.push((Reverse(columns[c]+columns[c+1]+r+r),
                               position ^ b ^ (b<<1)));
        }

        Self {
            values: scored_moves.into()
        }
    }
}

impl Iterator for CramOptimalMovesIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.len();
        (r, Some(r))
    }
}

impl ExactSizeIterator for CramOptimalMovesIterator {
    #[inline(always)]
    fn len(&self) -> usize {
        self.values.len()
    }
}

impl FusedIterator for CramOptimalMovesIterator {}