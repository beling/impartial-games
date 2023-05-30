use std::collections::HashMap;
use std::fmt;

use arrayvec;
use bitm::n_lowest_bits;

pub use moves_iterator::{Cram2ColumnsMovesIterator, CramCenterFirstMovesIterator, CramOptimalMovesIterator};
pub use moves_sorter::{CramDifficultEvaluator, LessMovesFirst, SmallerComponentsFirst};

use crate::bit::{lowest_bit_of, repeat_bit_sequence};
pub use crate::game::{DecomposableGame, Game};
use crate::games::cram::moves_iterator::CramSimpleMovesIterator;
use crate::solver::{Solver, SolverForDecomposableGame, StatsCollector};
use crate::solver::dedicated::BRAspSetSolver;

pub mod moves_iterator;
pub mod moves_sorter;
pub mod slices_provider;

#[cfg(feature = "nauty-Traces-sys")] mod graph_canon;
#[cfg(feature = "nauty-Traces-sys")] pub use graph_canon::GraphCanonTT;

#[derive(Clone)]
pub struct Cram {
    rotated_row: Box<[u64]>,  // indexed by a single row
    first_column_mask: u64,
    last_column_mask: u64,
    first_row_mask: u64,
    outside_embedded_rect_mask: u64,    // bits that will be outside the board after rotated by 90 degree.
    number_of_cols: u8,
    number_of_rows: u8
}

impl fmt::Display for Cram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cram{}x{}", self.number_of_cols, self.number_of_rows)
    }
}

impl Game for Cram {
    type Position = u64;
    type NimberSet = u32;

    /// Return number of moves in p.
    #[inline]
    fn moves_count(&self, p: &u64) -> u16 {
        (self.horizontal_moves(*p).count_ones() + self.vertical_moves(*p).count_ones()) as u16
    }

    fn initial_position(&self) -> Self::Position {
        n_lowest_bits(self.board_size())
    }

    fn is_initial_position_winning(&self) -> Option<bool> {
        // even by even are losing, even by odd are winning
        if self.number_of_cols & 1 == 0 {   // even number of columns
            Some(self.number_of_rows & 1 != 0)  
        } else {    // odd number of columns
            (self.number_of_rows & 1 == 0).then_some(true)
        }
    }
}

impl_serializable_game_for!(Cram);

type DecomposedCramPosition = arrayvec::ArrayVec::<u64, 8>;

impl DecomposableGame for Cram {
    type DecomposablePosition = u64;
    type Successors<'c> = CramSimpleMovesIterator;
    type HeuristicallyOrderedSuccessors<'c> = Cram2ColumnsMovesIterator;
    //type HeuristicallyOrderedSuccessors = CramOptimalMovesIterator;
    type Components<'c> = <DecomposedCramPosition as IntoIterator>::IntoIter;

    #[inline(always)]
    fn successors(&self, position: &Self::Position) -> Self::Successors<'_> {
        Self::Successors::new(self, *position)
    }

    #[inline(always)]
    fn successors_in_heuristic_ordered(&self, position: &Self::Position) -> Self::HeuristicallyOrderedSuccessors<'_> {
        Self::HeuristicallyOrderedSuccessors::new(self, *position)
    }

    #[inline(always)]
    fn decompose(&self, position: &Self::DecomposablePosition) -> Self::Components<'_> {
        self.split(*position).into_iter()
    }

    fn solver_with_stats<'s, STATS: 's+StatsCollector>(&'s self, stats: STATS) -> Box<dyn SolverForDecomposableGame<Game=Self, StatsCollector=STATS> + 's> {
        // TODO better solver for bigger game (TT, EndDB)  Maybe LV?
        Box::new(BRAspSetSolver{
            solver: Solver::new(self, HashMap::new(), (), SmallerComponentsFirst{}, stats)
        })
    }
}

impl Cram {

    /// Construct Cram game played on board with given size (number of columns and rows).
    /// Note that only boards with no more than 64 fields are supported.
    pub fn new(number_of_cols: u8, number_of_rows: u8) -> Cram {
        assert!(number_of_cols * number_of_rows <= 64);
        let embedded_rect_size = number_of_cols.min(number_of_rows);
        let rotated_row_size = 1usize << embedded_rect_size;
        let mut rotated_row = Vec::with_capacity(rotated_row_size);
        rotated_row.push(0);
        for row in 1..rotated_row_size {
            let bit_idx = row.trailing_zeros() as u8;
            let row_without_bit = row ^ (1usize<<bit_idx);
            //assert_eq!(row as u64 & outside_rect_mask, 0);
            rotated_row.push(rotated_row[row_without_bit] | (1usize << (bit_idx * number_of_cols)) as u64);
        }
        Cram {
            rotated_row: rotated_row.into_boxed_slice(),
            first_column_mask: repeat_bit_sequence(1u64, number_of_cols, number_of_rows),
            last_column_mask: repeat_bit_sequence(1u64 << (number_of_cols-1), number_of_cols, number_of_rows),
            first_row_mask: n_lowest_bits(number_of_cols),
            outside_embedded_rect_mask: !repeat_bit_sequence(n_lowest_bits(embedded_rect_size), number_of_cols, embedded_rect_size),
            number_of_cols,
            number_of_rows
        }
    }

    #[inline(always)] pub fn number_of_columns(&self) -> u8 { self.number_of_cols }
    #[inline(always)] pub fn number_of_rows(&self) -> u8 { self.number_of_rows }

    pub fn pos_from_str(&self, s: &str) -> u64 {
        let mut result = 0;
        let mut col = 0;
        let mut row = 0;
        let mut ignore_newline = false;
        for c in s.chars() {
            match c {
                ' '|'_'|'#' => {
                    if col >= self.number_of_cols || row >= self.number_of_rows {
                        panic!("The following position does not fit on a {}x{} board:\n{}", self.number_of_cols, self.number_of_rows, s);
                    }
                    if c == ' ' || c == '_' {
                        result |= 1 << (row * self.number_of_cols + col);
                    }
                    col += 1;
                    ignore_newline = false;
                },
                '\\'|'|' => {
                    row += 1; col = 0;
                    ignore_newline = true;
                },
                '\n' => {
                    if ignore_newline {
                        ignore_newline = false;
                    } else {
                        row += 1; col = 0;
                    }
                },
                other => { panic!("Unexpected character '{}'", other); }
            }
        }
        result
    }

    pub fn pos_to_custom_str(&self, mut pos: u64, empty: char, occupied: char, sep: char) -> String {
        if pos == 0 { return occupied.into() }
        let mut result = String::new();
        loop {
            for _ in 0..self.number_of_cols {
                result.push(if pos & 1 == 0 { occupied } else { empty });
                pos >>= 1;
            }
            if pos == 0 { break; }
            result.push(sep);
        }
        result
    }

    pub fn pos_to_multi_line_str(&self, pos: u64) -> String {
        let mut r = self.pos_to_custom_str(pos, '_', '#', '\n');
        r.push('\n');
        r
    }

    pub fn pos_to_one_line_str(&self, pos: u64) -> String {
        self.pos_to_custom_str(pos, '_', '#', '|')
    }

    /// Returns the number of empty cells in initial position.
    #[inline(always)] pub fn board_size(&self) -> u8 { self.number_of_rows * self.number_of_cols }

    /// Check if `position` can be rotated by 90 degrees.
    #[inline(always)] fn can_rotate90(&self, position: u64) -> bool {  // always true if initial board is a square
        position & self.outside_embedded_rect_mask == 0
    }

    /// Returns (lowest bits of) horizontal moves in the position `p`.
    #[inline(always)] fn horizontal_moves(&self, p: u64) -> u64 {
        p & ((p & !self.first_column_mask) >> 1) //& !self.forbidden_horizontal_moves(p)
    }

    /// Returns (lowest bits of) vertical moves in the position `p`.
    #[inline(always)] fn vertical_moves(&self, p: u64) -> u64 {
        p & (p >> self.number_of_cols)
    }

    /// Remove empty topmost rows from `p`.
    #[inline(always)] fn align_up(&self, p: &mut u64) {
        while (*p & self.first_row_mask) == 0 { *p >>= self.number_of_cols; }
    }

    /// Remove empty leftmost columns from `p`.
    #[inline(always)] fn align_left(&self, p: &mut u64) {
        while (*p & self.first_column_mask) == 0 { *p >>= 1; }
    }

    /// Returns a copy of `p` rotated 90 degree right.
    /// Can be called only if `self.can_rotate90(p)` is `true`, panics otherwise.
    fn rotated90right(&self, mut p: u64) -> u64 {
        let mut result = self.rotated_row[(p & self.first_row_mask) as usize];
        p >>= self.number_of_cols;
        while p != 0 {
            result <<= 1;
            result |= self.rotated_row[(p & self.first_row_mask) as usize];
            p >>= self.number_of_cols;
        }
        result
    }

    /// Returns vertically flipped copy of `p`.
    fn flipped_vertically(&self, mut p: u64) -> u64 {
        let mut result = p & self.first_row_mask;   // copy first row to result
        p >>= self.number_of_cols;                      // remove first row from p
        while p != 0 {  // while p is not empty
            result <<= self.number_of_cols;     // copy first row from p
            result |= p & self.first_row_mask;  // to result
            p >>= self.number_of_cols;          // remove first row from p
        }
        result
    }

    /// Returns horizontally flipped copy of `p`.
    fn flipped_horizontally(&self, mut p: u64) -> u64 {
        let mut result = p & self.first_column_mask;
        p &= !self.first_column_mask;
        p >>= 1;
        while p != 0 {
            result <<= 1;
            result |= p & self.first_column_mask;
            p &= !self.first_column_mask;
            p >>= 1;
        }
        result
    }

    /// Returns normalized (canonical) form of position `p`.
    fn normalized(&self, mut p: u64) -> u64 {
        self.align_up(&mut p);
        self.align_left(&mut p);
        let mut f = self.flipped_vertically(p);
        let mut res = p.min(f);
        if self.can_rotate90(p) {
            for _ in 0..3 {
                p = self.rotated90right(p);
                if p < res { res = p; }
                f = self.flipped_vertically(p);
                if f < res { res = f; }
            }
        } else {
            p = self.flipped_horizontally(p);
            if p < res { res = p; }
            f = self.flipped_vertically(p);
            if f < res { res = f; }
        }
        res
    }

    #[inline(always)] fn shifted_up(&self, p: u64) -> u64 { p >> self.number_of_cols }
    #[inline(always)] fn shifted_left(&self, p: u64) -> u64  { (p & !self.first_column_mask) >> 1 }
    #[inline(always)] fn shifted_right(&self, p: u64) -> u64  { (p & !self.last_column_mask) << 1 }
    #[inline(always)] fn shifted_down(&self, p: u64) -> u64  { p << self.number_of_cols }

    /// Returns all tufts, i.e. cells of `p` that have exactly one neighbor.
    fn tufts(&self, p: u64) -> u64 {
        let b = self.shifted_up(p); // with bottom neighbor (&p is at the end)
        let t = self.shifted_down(p);  // with top neighbor (&p is at the end)
        let r = self.shifted_left(p);   // with right neighbor (&p is at the end)
        let l = self.shifted_right(p);  // with left neighbor (&p is at the end)
        /*let odd_neighbors = b ^ t ^ r ^ l;
        let many_neighbors = (l | r) & (t | b); // all with >2 neighbors and some with 2 neighbors
        p & odd_neighbors & !many_neighbors*/
        //p & ((t | b | (l & r)) ^ (l | r | (t & b))) // another found with program
        p & (r ^ (b | (t ^ l))) & (b ^ r ^ (t | l)) // found with program and gives the shortest assembly
        //(t  ^  b  ^  l  ^  r)  &  ((b  |  l)  ^  (t  |  r)) & p // another found with program and gives the shortest assembly
    }

    /// Returns a copy of the position `p` with reduced stars and removed bridges.
    fn reduced_stars(&self, p: u64) -> u64 {
        let tufts = self.tufts(p);

        //let mut result = p;
        //let mut seen_neighbors = 0;

        //  stars already found, now bottom neighbors of tufts
        let mut stars = p & self.shifted_down(tufts);
        //let bottom_neighbors = only_bottom << self.number_of_cols;
        //result &= !((seen_neighbors & bottom_neighbors) >> self.number_of_cols);
        //seen_neighbors |= bottom_neighbors;

        let right_neighbors = p & self.shifted_right(tufts); // right neighbors of tufts
        //result &= !((seen_neighbors & right_neighbors) >> 1);
        let mut result = p ^ ((stars & right_neighbors) >> 1);
        stars |= right_neighbors;

        let top_neighbors = p & self.shifted_up(tufts); // top neighbors of tufts
        result &= !((stars & top_neighbors) << self.number_of_cols);
        stars |= top_neighbors;

        let left_neighbors = p & self.shifted_left(tufts); // left neighbors of tufts
        result &= !((stars & left_neighbors) << 1);
        stars |= left_neighbors;

        let mut bridges = p ^ (tufts | stars);
        bridges &= !(self.shifted_up(bridges) | self.shifted_right(bridges) |
            self.shifted_left(bridges) | self.shifted_down(bridges));

        result ^ bridges
    }

    /// Checks if the position `p` has reduced stars and removed bridges.
    fn has_reduced_stars(&self, p: u64) -> bool {
        let with_single_neighbor = self.tufts(p);
        let mut seen_neighbors = p & self.shifted_down(with_single_neighbor);   // seen neighbors of cells that has only 1, bottom neighbor

        let right_neighbors = p & self.shifted_right(with_single_neighbor); // neighbors of cells that has only 1, right neighbor
        if (seen_neighbors & right_neighbors) != 0 { return false; }
        seen_neighbors |= right_neighbors;

        let top_neighbors = p & self.shifted_up(with_single_neighbor);
        if (seen_neighbors & top_neighbors) != 0 { return false; }
        seen_neighbors |= top_neighbors;

        let left_neighbors = p & self.shifted_left(with_single_neighbor);
        if (seen_neighbors & left_neighbors) != 0 { return false; }
        seen_neighbors |= left_neighbors;

        let mut bridges = p ^ (with_single_neighbor | seen_neighbors);
        bridges &= !(self.shifted_up(bridges) | self.shifted_right(bridges) |
            self.shifted_left(bridges) | self.shifted_down(bridges));
        bridges == 0
    }


    /*fn reduced_stars(&self, p: u64) -> u64 {
        let b = p & self.shifted_up(p);     // with bottom neighbor
        let t = p & self.shifted_down(p);   // with top neighbor
        let r = p & self.shifted_left(p);   // with right neighbor
        let l = p & self.shifted_right(p);  // with left neighbor
        let tb = t|b;
        let lr = l|r;

        //let mut result = p;
        //let mut seen_neighbors = 0;

        let only_bottom = b & !(lr|t);        // with only 1, bottom neighbor
        let mut seen_neighbors = only_bottom << self.number_of_cols;
        //let bottom_neighbors = only_bottom << self.number_of_cols;
        //result &= !((seen_neighbors & bottom_neighbors) >> self.number_of_cols);
        //seen_neighbors |= bottom_neighbors;

        let only_right = r & !(l|tb);       // with only 1, right neighbor
        let right_neighbors = only_right << 1;
        //result &= !((seen_neighbors & right_neighbors) >> 1);
        let mut result = p ^ ((seen_neighbors & right_neighbors) >> 1);
        seen_neighbors |= right_neighbors;

        let only_top = t & !(lr|b);        // with only 1, top neighbor
        let top_neighbors = only_top >> self.number_of_cols;
        result &= !((seen_neighbors & top_neighbors) << self.number_of_cols);
        seen_neighbors |= top_neighbors;

        let only_left = l & !(r|tb);        // with only 1, left neighbor
        let left_neighbors = only_left >> 1;
        result &= !((seen_neighbors & left_neighbors) << 1);
        seen_neighbors |= left_neighbors;

        let with_single_neighbor = only_bottom | only_top | only_right | only_left;
        let mut bridges = p ^ (with_single_neighbor | seen_neighbors);
        bridges &= !(self.shifted_up(bridges) | self.shifted_right(bridges) |
            self.shifted_left(bridges) | self.shifted_down(bridges));

        result ^ bridges
    }*/

    /// Calculates the fields in which it is not worth making horizontal moves
    /// (by putting the left part of the domino),
    /// because they are equivalent to other moves available.
    ///
    /// Returned set of fields contains `self.last_column_mask` plus possibly some more.
    fn forbidden_horizontal_moves(&self, p: u64) -> u64 {
        let sl = self.shifted_left(p);
        let sr = self.shifted_right(p);
        let occupied_l = p & sl & !sr;  // with occupied left neighbour, but empty right
        let occupied_r = p & sr & !sl;  // with occupied right neighbour, but empty left
        let sd = self.shifted_down(p);
        let su = self.shifted_up(p);
        //let occupied_t = su & p & !sd;  // with occupied top neighbour, but empty bottom
        //let occupied_b = sd & p & !su;  // with occupied bottom neighbour, but empty top
        // NOTE: we do not need &p as later it is occupied_t/b are & with occupied_l/r that are already &p
        let occupied_t = su & !sd;  // with occupied top neighbour, but empty bottom
        let occupied_b = sd & !su;  // with occupied bottom neighbour, but empty top

        let d_r = ((occupied_l & occupied_t) << self.number_of_cols) & ((occupied_r & occupied_b) >> 1);    // already shifted to left column
        let d_l = ((occupied_r & occupied_t) << (self.number_of_cols-1)) & occupied_l & occupied_b;
        let forbidden_bottom = d_l | d_r;
        self.last_column_mask | forbidden_bottom | self.shifted_up(forbidden_bottom)

        //let mut d_r = ((occupied_l & occupied_t) << (self.number_of_cols+1)) & occupied_r & occupied_b;
        //d_r |= self.shifted_up(d_r);
        //let mut d_l = ((occupied_r & occupied_t) << (self.number_of_cols-1)) & occupied_l & occupied_b;
        //d_l |= self.shifted_up(d_l);

        // self.first_column_mask | d_r | (d_l << 1), // right parts of the moves
        //self.last_column_mask | d_l | (d_r >> 1)
    }

    #[inline(always)]
    fn push_or_remove(result: &mut DecomposedCramPosition, component: u64) {
        if let Some(in_vec_index) = result.iter().position(|x| *x == component) {
            // two same components has same values and their xor is 0, we can skip both:
            result.swap_remove(in_vec_index);
        } else {
            result.push(component);
        }
    }

    /// Adds to `result` all connected components of empty fields in `p` that is star-reduced.
    fn split_star_reduced_to(&self, result: &mut DecomposedCramPosition, p: u64) {
        let forbidden_horizontal_moves = self.forbidden_horizontal_moves(p);
        let components_need_splitting = forbidden_horizontal_moves != self.last_column_mask;
        let can_look_for_r = !forbidden_horizontal_moves;
        let can_look_for_l = can_look_for_r << 1;
        //let (can_look_for_l, can_look_for_r) = self.can_look_for_left_right(p);
        let mut rest = p;
        while rest != 0 {
            let mut prev = lowest_bit_of(rest);
            let mut component = (prev |
                ((prev & can_look_for_l) >> 1) |
                ((prev & can_look_for_r) << 1) |
                self.shifted_down(prev) |
                self.shifted_up(prev)) & rest;
            if prev == component { // if single, isolated empty cell then ignore
                rest ^= component;
                continue;
            }
            loop {
                prev = component;
                component |= (
                    ((component & can_look_for_l) >> 1) |
                        ((component & can_look_for_r) << 1) |
                        self.shifted_down(component) |
                        self.shifted_up(component)) & rest;
                if prev == component { break; }
            }
            rest ^= component;
            if components_need_splitting && component != p {    // if component == p then 2x2 pattern does not cause splitting
                let star_reduced_component = self.reduced_stars(component);
                if star_reduced_component == component { // no further star reducing:
                    Self::push_or_remove(result, self.normalized(component));
                } else {
                    self.split_star_reduced_to(result, star_reduced_component);
                }
            } else {
                Self::push_or_remove(result, self.normalized(component));
            }
        }
    }

    /// Finds all connected components of empty fields in `p`.
    /// Skips single field components and normalize the rest.
    /// Skips pairs of duplicated components.
    /// Returns sequence of normalized, unique components of `p`.
    #[inline(always)]
    fn split(&self, p: u64) -> DecomposedCramPosition {
        let mut result = DecomposedCramPosition::new();
        self.split_star_reduced_to(&mut result, self.reduced_stars(p));
        result
    }

    /// Checks if star-reduced position `p` consists of exactly one component.
    fn is_connected(&self, p: u64) -> bool {
        let can_look_for_r = !self.forbidden_horizontal_moves(p);
        let can_look_for_l = can_look_for_r << 1;
        let mut component = lowest_bit_of(p);
        loop {
            let prev = component;
            component |= (
                ((component & can_look_for_l) >> 1) |
                ((component & can_look_for_r) << 1) |
                self.shifted_down(component) |
                self.shifted_up(component)) & p;
            if component == prev { return p == component; }
        }
    }

    /// Check if `p` constitutes a single, normalized component.
    pub fn is_normalized_component(&self, p: u64) -> bool {
        if (p & self.first_row_mask) == 0 || (p & self.first_column_mask) == 0 ||
            self.flipped_vertically(p) < p || !self.is_connected(p) || !self.has_reduced_stars(p)
        {
            return false;
        }
        if self.can_rotate90(p) {
            let mut r = p;
            for _ in 0..3 {
                r = self.rotated90right(r);
                if r < p || self.flipped_vertically(r) < p { return false; }
            }
        } else {
            let fh = self.flipped_horizontally(p);
            if fh < p || self.flipped_vertically(fh) < p { return false; }
        }
        return true;
    }

    pub fn center_column(&self, mut position: u64) -> u64 {
        let without_2_first_cols = !(self.first_column_mask | (self.first_column_mask << 1));
        let mut center_col = self.first_column_mask;
        position &= without_2_first_cols;
        position >>= 2;
        while position != 0 {
            position &= without_2_first_cols;
            position >>= 2;
            center_col <<= 1;
        }
        center_col
    }

    pub fn center_2columns(&self, mut position: u64) -> u64 {
        let mut center_col = self.first_column_mask | (self.first_column_mask << 1);
        let without_2_first_cols = !center_col;
        position &= without_2_first_cols;
        position >>= 2;
        while position != 0 {
            position &= without_2_first_cols;
            position >>= 2;
            center_col <<= 1;
        }
        center_col
    }

    pub fn center_row(&self, mut position: u64) -> u64 {
        let mut center_row = self.first_row_mask;
        let dbl_number_of_cols = 2*self.number_of_cols;
        position >>= dbl_number_of_cols;
        while position != 0 {
            position >>= dbl_number_of_cols;
            center_row <<= self.number_of_cols;
        }
        center_row
    }

    /// Returns the biggest rectangular subset of the board fields which is embedded in the board and includes (0,0) field.
    pub fn embedded_rectangle(&self) -> u64 {
        !self.outside_embedded_rect_mask
    }

    /// Returns the rectangular subset of the board fields which has given size and includes (0,0) field.
    pub fn rectangle(&self, cols: u8, rows: u8) -> u64 {
        repeat_bit_sequence(n_lowest_bits(cols), self.number_of_cols, rows)
    }

    /*pub fn columns_mask(&self, number_of_columns: u8) -> u64 {
        let mut result = 0;
        for _ in 0..number_of_columns {
            result <<= 1;
            result |= self.first_column_mask;
        }
        result
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tester { pub cram: Cram }

    impl Tester {
        pub fn new(number_of_cols: u8, number_of_rows: u8) -> Self {
            Self { cram: Cram::new(number_of_cols, number_of_rows) }
        }

        pub fn pos(&self, s: &str) -> u64 { self.cram.pos_from_str(s) }

        pub fn str(&self, pos: u64) -> String { self.cram.pos_to_multi_line_str(pos) }

        pub fn one_line_str(&self, pos: u64) -> String { self.cram.pos_to_one_line_str(pos) }

        pub fn check<F: FnOnce(&Cram, u64)->u64>(&self, (f, op_name): (F, &str), input: &str, expected_output: &str) -> (u64, u64, u64) {
            let input = self.pos(input);
            let expected = self.pos(expected_output);
            let got = f(&self.cram, input);
            assert_eq!(got, expected, "Position:\n{}after {} is:\n{}but should be:\n{}",
                       self.str(input),
                       op_name,
                       self.str(got),
                       self.str(expected)
            );
            (input, got, expected)
        }

        pub fn check_reduce_stars(&self, input: &str, expected: &str) {
            let (i, g, e) = self.check(
                (|c, i| c.reduced_stars(i), "reducing stars"),
                input, expected
            );
            assert!(self.cram.has_reduced_stars(g), "{}has_reduced_stars returned false, but true expected", self.str(g));    // result should has star reduced
            if g == e && i != g {   // got is fine, input is different, so input has unreduced stars
                assert!(!self.cram.has_reduced_stars(i), "{}has_reduced_stars returned true, but false expected", self.str(i))
            }
        }

        pub fn check_multi<F: FnOnce(&Cram, u64)->DecomposedCramPosition>(&self, (f, op_name): (F, &str), input: &str, expected: &[&str]) {
            let input = self.pos(input);
            let mut got = f(&self.cram, input).to_vec();
            //assert_eq!(got.len(), expected.len(), "expected {} components, but got {}", expected.len(), got.len());
            got.sort();
            let mut expected: Vec<_> = expected.iter().map(|s| self.pos(s)).collect();
            expected.sort();
            assert_eq!(got, expected, "Position:\n{}\nafter {} is:\n{:?}\nbut should be:\n{:?}",
                       self.str(input), op_name,
                       got.iter().map(|p| self.one_line_str(*p)).collect::<Vec<_>>(),
                       expected.iter().map(|p| self.one_line_str(*p)).collect::<Vec<_>>()
            );
        }

        /*pub fn check_split(&self, input: &str, expected: &[&str]) {
            let input = self.pos(input);
            let mut got = self.cram.split(input).to_vec();
            //assert_eq!(got.len(), expected.len(), "expected {} components, but got {}", expected.len(), got.len());
            got.sort();
            let mut expected: Vec<_> = expected.iter().map(|s| self.pos(s)).collect();
            expected.sort();
            assert_eq!(got, expected, "Position:\n{}\nafter splitting is:\n{:?}\nbut should be:\n{:?}",
                self.str(input),
                got.iter().map(|p| self.one_line_str(*p)).collect::<Vec<_>>(),
                expected.iter().map(|p| self.one_line_str(*p)).collect::<Vec<_>>()
            );
        }*/
    }

    #[test]
    fn shifts() {
        let t = Tester::new(4, 3);
        let up = (|c: &Cram, i| c.shifted_up(i), "shifting up");
        t.check(up, "__", "");
        t.check(up, "_|_#_", "_#_");
        let down = (|c: &Cram, i| c.shifted_down(i), "shifting down");
        t.check(down, "__", "##|__");
        t.check(down, "_#_|___", "###|_#_|___");
        let left = (|c: &Cram, i| c.shifted_left(i), "shifting left");
        t.check(left, "__", "_");
        t.check(left, "_#_|___", "#_|__");
        let right = (|c: &Cram, i| c.shifted_right(i), "shifting right");
        t.check(right, "__", "#__");
        t.check(right, "_#_|___", "#_#_|#___");
    }

    #[test]
    fn reduce_stars() {
        let t = Tester::new(6, 5);
        t.check_reduce_stars("#_#|___|#_#", "#_#|#_#");
        t.check_reduce_stars("__", "__");
        t.check_reduce_stars("_|_", "_|_");
        t.check_reduce_stars("___", "__");
        t.check_reduce_stars("___|###|___", "__|##|__");
        t.check_reduce_stars("____", "____");
        t.check_reduce_stars("____|####|____", "____|####|____");
        t.check_reduce_stars("#_#|___", "#_#|#_#");
        t.check_reduce_stars("___|_#_", "_#_|_#_");   // bridge destructing
        t.check_reduce_stars("_#_|___", "_#_|_#_");   // bridge destructing
        t.check_reduce_stars("___|_#_|___", "___|_#_|___");
        t.check_reduce_stars("____|_##_", "____|_##_");
        t.check_reduce_stars("#_##|____|_##_", "#_##|__#_|_##_");   // bridge destructing
        t.check_reduce_stars("______|_####_|_####_", "______|_####_|_####_");
        t.check_reduce_stars("##____|_##_##|____##", "##____|_##_##|____##");
    }

    #[test]
    fn split() {
        let t = Tester::new(7, 5);
        let split = (|c: &Cram, p| c.split(p), "splitting");
        t.check_multi(split, "__|__", &[]);
        t.check_multi(split, "__||_", &["__"]);
        t.check_multi(split, "_____", &[]);
        t.check_multi(split, "______", &["______"]);
        t.check_multi(split, "#_____#|___#___|__###__", &[]);
        t.check_multi(split, "_____|__#__", &[]);
        t.check_multi(split, "______|_####_|_####_", &["______|_####_|_####_"]);
        t.check_multi(split, "##____|_##_##|____##", &["##____|_##_##|____##"]);
    }
}
