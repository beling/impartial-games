pub use crate::game::{Game, SimpleGame};
use crate::bit::repeat_bit_sequence;
use std::mem::MaybeUninit;
use std::mem;
use crate::moves::DifficultEvaluator;
use crate::solver::dedicated::DefSolver;
use crate::solver::{SolverForSimpleGame, Solver, StatsCollector};
use std::collections::HashMap;
use std::fmt;
use bitm::n_lowest_bits;
use csf::bits_to_store;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Chomp {
    /// `ones[i]` consists of `i` ones, at positions: `0 * bits_per_row`, `1 * bits_per_row`, ..., `(i-1)*bits_per_row`
    ones: [u64; 17],

    /// `0..01..1` mask with `bits_per_row` lowest bits set
    first_row_mask: u64,

    /// `1..10..01..10..` mask that indicates rows with even indices (0, 2, ...)
    even_rows_mask: u64,

    number_of_cols: u8,
    number_of_rows: u8,

    /// Position is represented by (bit-)array of fields, each is stored at `bits_per_row` bits.
    bits_per_row: u8
}

impl fmt::Display for Chomp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chomp{}x{}_absolute_repr", self.number_of_cols, self.number_of_rows)
    }
}

impl Game for Chomp {
    type Position = u64;
    type NimberSet = [u64; 4];
    //type NimberSet = u128;
    //type NimberSet = u64;

    #[inline(always)]
    fn moves_count(&self, p: &u64) -> u16 {
        (self.squares_count(*p) - 1) as u16
    }

    fn try_solve_theoretically(&self, position: &Self::Position) -> Option<u8> {
        if self.row(*position, 1) <= 1 {
            Some((self.rows_count(*position) - 1) ^ (self.row(*position, 0) - 1))
        } else {
            None
        }
    }

    fn initial_position(&self) -> Self::Position {
        repeat_bit_sequence(self.number_of_cols as u64, self.bits_per_row, self.number_of_rows)
    }

    #[inline(always)]
    fn is_initial_position_winning(&self) -> Option<bool> { Some(self.number_of_cols > 1 || self.number_of_rows > 1) }
}

impl_serializable_game_for!(Chomp);

impl Chomp {
    /// Construct Chomp game played on a board with given size (number of columns and rows).
    /// Note: `number_of_cols` should be greater or equal to `number_of_rows`, otherwise they are swapped.
    pub fn new(mut number_of_cols: u8, mut number_of_rows: u8) -> Chomp {
        if number_of_rows > number_of_cols { std::mem::swap(&mut number_of_cols, &mut number_of_rows); }
        let bits_per_row = bits_to_store!(number_of_cols);
        assert!(number_of_rows as u16 * bits_per_row as u16 <= 64);
        let first_row_mask = n_lowest_bits(bits_per_row);
        Chomp {
            ones: {
                let mut data: [MaybeUninit<u64>; 17] = unsafe { MaybeUninit::uninit().assume_init() };
                data[0] = MaybeUninit::new(0);
                data[1] = MaybeUninit::new(1);
                //let rows_to_fill = number_of_cols.min(16);
                for i in 1..16 {
                    let prev = unsafe { data[i].as_ptr().read() };
                    data[i+1] = MaybeUninit::new(prev | (prev << bits_per_row));
                }
                unsafe { mem::transmute::<_, [u64; 17]>(data) }
            },
            first_row_mask,
            even_rows_mask: repeat_bit_sequence(first_row_mask, 2*bits_per_row, (number_of_rows+1)/2),
            number_of_cols,
            number_of_rows,
            bits_per_row
        }
    }

    /// Construct position from array of rows lengths.
    ///
    /// Input requirements (the method panics if they are not met):
    /// - rows must be in non-increasing order, not empty, and has length <= number_of_rows;
    /// - rows[0] is the length of the first row, includes the poisoned square, and must be in range [1, number_of_cols].
    pub unsafe fn pos_without_normalization(&self, rows: &[u8]) -> u64 {
        assert!(rows.len() <= self.number_of_rows as usize);
        assert!(!rows.is_empty());
        assert!(1 <= rows[0] && rows[0] <= self.number_of_cols);
        let mut result = rows[0] as u64;
        for i in 1..rows.len() {
            assert!(rows[i] <= rows[i-1]);
            result |= (rows[i] as u64) << ((i as u8) * self.bits_per_row);
        }
        result
    }

    /// Construct position (in canonical form) from array of rows lengths.
    ///
    /// Input requirements (the method panics if they are not met):
    /// - rows must be in non-increasing order, not empty, and has length <= number_of_rows;
    /// - rows[0] is the length of the first row, includes the poisoned square, and must be in range [1, number_of_cols].
    #[inline(always)]
    pub fn pos(&self, rows: &[u8]) -> u64 {
        self.normalized(unsafe { self.pos_without_normalization(rows) })
    }

    /// Convert position to array of rows lengths.
    pub fn pos_to_arr(&self, position: u64) -> Box<[u8]> {
        (0..self.rows_count(position)).map(|r| { self.row(position, r) }).collect()
    }

    /// Returns canonical (normalized) representation of the position.
    pub fn normalized(&self, position: u64) -> u64 {
        //println!("{:?}", self.pos_to_arr(position));
        let row = self.first_row(position);
        if row > self.rows_count(position) { return position; }
        let mut transposed = self.ones[row as usize];
        let mut rest = position >> self.bits_per_row;
        while rest != 0 {
            transposed += self.ones[self.first_row(rest) as usize];
            self.erase_first_row(&mut rest);
        }
        position.min(transposed)
    }

    #[inline(always)]
    pub fn is_normalized(&self, position: u64) -> bool {
        position == self.normalized(position)
    }

    /// Returns the number of chocolate squares that constitute given board, including the poisoned one.
    fn squares_count(&self, board: u64) -> u8 {
        // TODO jeśli bits_per_row jest małe, to można też zliczać za pomocą count_ones() osobno liczbę 1 w rzędzie 1, 2, 4, ...
        let mut result = (board & self.even_rows_mask) + ((board>>self.bits_per_row) & self.even_rows_mask);  // up to 2 rows supported
        result += result >> (self.bits_per_row << 1); // up to 4 rows supported
        result += result >> (self.bits_per_row << 2); // up to 8 rows supported
        result += result >> (self.bits_per_row << 3); // up to 16 rows supported
        (result & (self.first_row_mask | (self.first_row_mask<<self.bits_per_row))) as u8
    }

    /// Returns the number of non-empty rows of given board.
    #[inline(always)]
    fn rows_count(&self, board: u64) -> u8 {
        // position of the leading one divided by bits_per_row (rounded up)
        (63 - (board.leading_zeros() as u8) + self.bits_per_row) / self.bits_per_row
    }

    /// Returns length of position's row with given index.
    #[inline(always)]
    fn row(&self, position: u64, index: u8) -> u8 {
        ((position >> (index*self.bits_per_row)) & self.first_row_mask) as u8
    }

    #[inline(always)]
    fn first_row(&self, position: u64) -> u8 {
        (position & self.first_row_mask) as u8
    }

    #[inline(always)]
    fn erase_first_row(&self, position: &mut u64) {
        *position >>= self.bits_per_row;
    }

    /// Increment position number to next number that represent valid position.
    /// Neither before nor after call position do not have to be normalized,
    /// but both must be valid.
    fn inc_position_number(&self, position: &mut u64) {
        let mut row_shift = 0u8;
        let mut row_len = *position & self.first_row_mask;
        while row_len as u8 == self.number_of_cols {
            row_shift += self.bits_per_row;
            row_len = (*position >> row_shift) & self.first_row_mask;
        }
        *position += 1u64 << row_shift;
        if row_shift != 0 {
            *position &= !n_lowest_bits(row_shift);
            row_len += 1;
            loop {
                row_shift -= self.bits_per_row;
                *position |= row_len << row_shift;
                if row_shift == 0 { break; }
            }
        }
    }

    /// Check if the given position is valid (but not necessary normalized).
    pub fn is_valid(&self, mut position: u64) -> bool {
        let mut row = self.first_row(position);
        if row == 0 || row > self.number_of_cols { return false; }
        self.erase_first_row(&mut position);
        while position != 0 {
            let next_row = self.first_row(position);
            if row < next_row { return false; }
            row = next_row;
            self.erase_first_row(&mut position);
        }
        true
    }

}

pub struct ChompMovesIterator<'a> {
    chomp: &'a Chomp,
    /// Position for which we generate moves.
    position: u64,
    current_move: u64,
    current_transposed_move: u64,
    transposed_over_highest: u64,
    ones_for_current_col: u64,
    transposed_below: [MaybeUninit<u64>; 17],
    current_row_shift: u8,
    highest_row_shift: u8,  // shows row with the highest index and value >= current_col
    current_col: u8,
    rows_count: u8
}

impl ChompMovesIterator<'_> {
    fn new(chomp: &Chomp, position: u64) -> ChompMovesIterator {
        let rows_count = chomp.rows_count(position);
        let current_row_shift = rows_count * chomp.bits_per_row;
        let mut result = ChompMovesIterator {
            chomp,
            position,
            current_move: position,
            current_transposed_move: 0,
            transposed_over_highest: 0,
            ones_for_current_col: 0,
            transposed_below: unsafe { MaybeUninit::uninit().assume_init() },
            current_row_shift,
            highest_row_shift: current_row_shift - chomp.bits_per_row,
            current_col: 0,
            rows_count
        };
        result.transposed_below[0] = MaybeUninit::new(0);
        if chomp.first_row(position) <= rows_count {
            for i in 0..rows_count {
                result.transposed_below[(i+1) as usize] = MaybeUninit::new(
                    unsafe { result.transposed_below[i as usize].as_ptr().read() } +
                        chomp.ones[chomp.row(position, i) as usize]
                )
            }
        }
        result
    }

    /*fn dbg_check_returned(&self, returned: u64) {
        let should_be = self.chomp.normalized(self.current_move);
        if should_be != returned {
            println!("{:?} != {:?} (current move = {:?})",
                     self.chomp.pos_to_arr(returned),
                     self.chomp.pos_to_arr(should_be),
                     self.chomp.pos_to_arr(self.current_move));
        }
    }*/
}

impl Iterator for ChompMovesIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row_shift == 0 || // row nr 0 or
            (self.current_row_shift == self.chomp.bits_per_row && self.current_col == 0)  // col nr. 0 and row nr. 1
        {   // go to the next column:
            self.current_col += 1;
            loop {
                let pos_row = ((self.position >> self.highest_row_shift) & self.chomp.first_row_mask) as u8;
                if pos_row > self.current_col { break; }    // length of this row must be decreased to current_col
                if self.highest_row_shift == 0 { return None; } // no more rows = no more moves
                self.highest_row_shift -= self.chomp.bits_per_row;
                if pos_row <= 16 { self.transposed_over_highest += self.chomp.ones[pos_row as usize]; }
            }
            self.current_row_shift = self.highest_row_shift;
            self.current_move = self.position;
            self.current_transposed_move = self.transposed_over_highest;
            if self.current_col <= 16 { self.ones_for_current_col = self.chomp.ones[self.current_col as usize]; }
        } else {
            self.current_row_shift -= self.chomp.bits_per_row;
        }
        self.current_transposed_move += self.ones_for_current_col;
        self.current_move &= !(self.chomp.first_row_mask << self.current_row_shift);
        self.current_move |= (self.current_col as u64) << self.current_row_shift;
        if self.chomp.first_row(self.current_move) <= self.rows_count {
            let transposed = self.current_transposed_move +
                unsafe { self.transposed_below[(self.current_row_shift / self.chomp.bits_per_row) as usize].as_ptr().read() };
            if transposed < self.current_move {
                //self.dbg_check_returned(transposed);
                return Some(transposed)
            }
        }
        //self.dbg_check_returned(self.current_move);
        Some(self.current_move)
    }

    //fn size_hint(&self) -> (usize, Option<usize>) {
    //}
}

impl SimpleGame for Chomp {
    type Successors<'s> = ChompMovesIterator<'s>;
    type HeuristicallyOrderedSuccessors<'s> = ChompMovesIterator<'s>;

    #[inline(always)]
    fn successors(&self, position: &Self::Position) -> Self::Successors<'_> {
        ChompMovesIterator::new(self, *position)
    }

    #[inline(always)]
    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_> {
        ChompMovesIterator::new(self, *position)
    }

    fn solver_with_stats<'s, STATS: 's+StatsCollector>(&'s self, stats: STATS) -> Box<dyn SolverForSimpleGame<Game=Self, StatsCollector=STATS> + 's> {
        Box::new(DefSolver{
            solver: Solver::new(self, HashMap::new(), (), FewerBarsFirst{}, stats)
        })
    }
}

/*impl<'c> DecomposableGame<'c> for Chomp {
    type DecomposablePosition = u64;
    type Successors = ChompMovesIterator;
    type Components = arrayvec::IntoIter<[u64; 1]>;

    fn successors<'b: 's>(&'b self, position: &Self::Position) -> Self::Successors {
        ChompMovesIterator::new(&self, *position)
    }

    fn decompose<'b: 'c>(&'b self, position: &Self::DecomposablePosition) -> Self::Components {
        //self.split(*position).into_iter()
        arrayvec::ArrayVec::from([*position]).into_iter()
    }
}*/

pub struct SliceIterator<'a> {
    chomp: &'a Chomp,
    /// Always in canonical form.
    current_position: u64,
    /// Do not have to be normalized.
    last_position: u64
}

impl SliceIterator<'_> {
    /*pub fn new(chomp: &Chomp, slice_index: usize) -> SliceIterator {
        // TODO
        let slice_index = slice_index as u64;
        let max_in_slice = slice_index << 32;
        SliceIterator { chomp, current_position: slice_index << 32, last_position: (slice_index+1)<<32 }
    }*/
}

impl Iterator for SliceIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_position != self.last_position {
            self.chomp.inc_position_number(&mut self.current_position);
            if self.chomp.is_normalized(self.current_position) {
                return Some(self.current_position);
            }
        }
        None
    }
}

//impl FusedIterator for SliceIterator<'_> {}

pub struct FewerBarsFirst;

impl DifficultEvaluator for FewerBarsFirst {
    type Game = Chomp;
    type PositionDifficult = u8;

    fn difficult_of(&self, game: &Chomp, to_evaluate: &<Chomp as Game>::Position) -> Self::PositionDifficult {
        game.squares_count(*to_evaluate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chomp1x1() {
        let chomp = Chomp::new(1, 1);
        assert_eq!(chomp.ones[0], 0);
        assert_eq!(chomp.ones[1], 1);
        assert_eq!(chomp.number_of_cols, 1);
        assert_eq!(chomp.number_of_rows, 1);
        assert_eq!(chomp.bits_per_row, 1);
        assert_eq!(chomp.first_row_mask, 1);
        assert_eq!(chomp.even_rows_mask, 1);
        let init_pos = chomp.initial_position();
        assert_eq!(init_pos, 1);
        assert_eq!(chomp.rows_count(init_pos), 1);
        assert_eq!(chomp.squares_count(init_pos), 1);
        assert_eq!(chomp.moves_count(&init_pos), 0);
        assert_eq!(chomp.successors(&init_pos).collect::<Vec<_>>(), Vec::<u64>::new());
    }

    fn pos_vec(chomp: &Chomp, positions: &[&[u8]]) -> Vec<u64> {
        positions.iter().map(|a| {chomp.pos(*a)}).collect()
    }

    #[test]
    fn chomp3x2() {
        let chomp = Chomp::new(3, 2);
        assert_eq!(chomp, Chomp::new(2, 3));
        assert_eq!(chomp.ones[0], 0);
        assert_eq!(chomp.ones[1], 1);
        assert_eq!(chomp.ones[2], 0b101);
        assert_eq!(chomp.number_of_cols, 3);
        assert_eq!(chomp.number_of_rows, 2);
        assert_eq!(chomp.bits_per_row, 2);
        assert_eq!(chomp.first_row_mask, 3);
        assert_eq!(chomp.even_rows_mask, 3);
        let init_pos = chomp.initial_position();
        assert_eq!(init_pos, 0b11_11);
        assert_eq!(chomp.rows_count(init_pos), 2);
        assert_eq!(chomp.squares_count(init_pos), 6);
        assert_eq!(chomp.moves_count(&init_pos), 5);
        assert_eq!(chomp.successors(&init_pos).collect::<Vec<_>>(),
                   pos_vec(&chomp, &[&[3], &[3,1], &[2], &[3,2], &[2,2]]));
    }
}