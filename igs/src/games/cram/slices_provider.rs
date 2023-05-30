use crate::enddb::{EndDbSlicesProvider, SortedPositionNimberMap};
use std::iter::FusedIterator;
use bitm::n_lowest_bits;
use super::Cram;
use crate::dbs::NimbersProvider;
use crate::game::Game;

pub struct SliceIterator<'a> {
    cram: &'a Cram,
    current_position: u64,
    end: u64
}

impl SliceIterator<'_> {
    pub fn new(cram: &Cram, slice_index: usize, max_end: u64) -> SliceIterator {
        let slice_index = slice_index as u64;
        SliceIterator { cram, current_position: slice_index << 32, end: max_end.min((slice_index+1)<<32) }
    }
}

impl Iterator for SliceIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_position != self.end {
            let p = self.current_position;
            self.current_position += 1;
            if self.cram.is_normalized_component(p) { return Some(p); }
        }
        None
    }
}

impl FusedIterator for SliceIterator<'_> {}

impl EndDbSlicesProvider for &Cram {
    type Game = Cram;
    type SliceIterator<'si> = SliceIterator<'si> where Self: 'si;
    type InSlicePosition = u32;
    type UncompressedSlice = SortedPositionNimberMap<u32>;
    //type SliceIterator = impl Iterator<u64>;

    #[inline(always)] fn position_to_slice(&self, position: &u64) -> Option<usize> {
        Some((position >> 32) as usize)
    }

    #[inline(always)] fn strip(&self, position: &u64) -> Self::InSlicePosition {
        (*position & (u32::MAX as u64)) as u32
    }

    fn slice_content<'si, 's: 'si, 'g: 'si>(&'s self, game: &'g Self::Game, slice_index: usize) -> Option<Self::SliceIterator<'si>> {
        let max_last_position = self.initial_position();
        (slice_index as u64 <= max_last_position >> 32).then(|| SliceIterator::new(&game, slice_index, max_last_position+1))
    }

    /*fn slice_content(&self, game: &Cram, mut slice_index: usize) -> Self::SliceIterator {
        let s = slice_index as u64;
        (s<<32..(s+1)<<32).map(|p| game.is_normalized_component(p))
    }*/
}

/// Slice Provider for Cram that expose position only up to the given number.
pub struct LimitedSliceProvider {
    /// Last position exposed.
    pub last_pos: u64
}

impl EndDbSlicesProvider for LimitedSliceProvider {
    type Game = Cram;
    type SliceIterator<'si> = SliceIterator<'si>;
    type InSlicePosition = u32;
    type UncompressedSlice = SortedPositionNimberMap<u32>;

    #[inline(always)] fn position_to_slice(&self, position: &u64) -> Option<usize> {
        (*position <= self.last_pos).then(|| (position >> 32) as usize)
    }

    #[inline(always)] fn strip(&self, position: &u64) -> Self::InSlicePosition {
        (*position & (u32::MAX as u64)) as u32
    }

    fn slice_content<'si, 's: 'si, 'g: 'si>(&'s self, game: &'g Self::Game, slice_index: usize) -> Option<Self::SliceIterator<'si>> {
        ((slice_index as u64) << 32 <= self.last_pos).then(
            || Self::SliceIterator::new(game, slice_index, self.last_pos+1)
        )
    }
}

pub struct LimitedColumnsSliceIterator<'a> {
    cram: &'a Cram,
    current_position_compressed: u64,
    end_position_compressed: u64,
    compressed_row_mask: u64,
    number_of_columns_in_end_db: u8
}

/// Returns a modified copy of `src` that has changed number of columns from `src_nr_of_cols` to `dst_nr_of_cols`.
/// `src_row_mask` shows fields to read from each row of `src` and usually is equal to `(1<<src_nr_of_cols)-1`
#[inline]
fn change_number_of_columns(src: u64, src_row_mask: u64, src_nr_of_cols: u8, dst_nr_of_cols: u8) -> u64 {
    let mut result = src & src_row_mask;
    let mut src_to_transfer = src >> src_nr_of_cols;
    let mut result_shift = dst_nr_of_cols;
    while src_to_transfer != 0 {
        result |= (src_to_transfer & src_row_mask) << result_shift;
        result_shift += dst_nr_of_cols;
        src_to_transfer >>= src_nr_of_cols;
    }
    result
}

impl LimitedColumnsSliceIterator<'_> {
    pub fn new(cram: &Cram, slice_index: usize, max_end: u64, number_of_columns_in_end_db: u8) -> LimitedColumnsSliceIterator {
        let slice_index = slice_index as u64;
        LimitedColumnsSliceIterator { cram,
            current_position_compressed: slice_index << 32,
            end_position_compressed: max_end.min((slice_index+1)<<32),
            compressed_row_mask: (1u64 << number_of_columns_in_end_db) - 1,
            number_of_columns_in_end_db
        }
    }

    #[inline(always)]
    pub fn current_position(&self) -> u64 {
        change_number_of_columns(
            self.current_position_compressed,
            self.compressed_row_mask,
            self.number_of_columns_in_end_db,
            self.cram.number_of_cols
        )
    }
}

impl Iterator for LimitedColumnsSliceIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_position_compressed != self.end_position_compressed {
            let p = self.current_position();
            self.current_position_compressed += 1;
            if self.cram.is_normalized_component(p) { return Some(p); }
        };
        None
    }
}

impl FusedIterator for LimitedColumnsSliceIterator<'_> {}

// Slice Provider for Cram that expose only position whose all empty cells are included in the given number of first columns.
pub struct LimitedColumnsSliceProvider {
    //content_mask: u64,
    end_position: u64,
    outside_max_rectangle: u64,
    cram_first_row_mask: u64,
    number_of_columns_in_end_db: u8,
    cram_number_of_columns: u8
}

impl LimitedColumnsSliceProvider {
    pub fn new(cram: &Cram, number_of_columns_in_end_db: u8) -> Self {
        Self {
            end_position: 0,
            outside_max_rectangle: !cram.rectangle(number_of_columns_in_end_db, cram.number_of_rows),
            cram_first_row_mask: cram.first_row_mask,
            number_of_columns_in_end_db,
            cram_number_of_columns: cram.number_of_cols
        }
    }

    pub fn with_limited_rows(cram: &Cram, number_of_columns_in_end_db: u8, number_of_full_rows: u8, number_of_columns_in_last_row: u8) -> Self {
        let cells = cram.rectangle(number_of_columns_in_end_db, number_of_full_rows) |
            (n_lowest_bits(number_of_columns_in_last_row) << (cram.number_of_cols * number_of_full_rows));
        Self {
            end_position: 0,
            outside_max_rectangle: !cells,
            cram_first_row_mask: cram.first_row_mask,
            number_of_columns_in_end_db,
            cram_number_of_columns: cram.number_of_cols
        }
    }

    #[inline(always)] fn contains(&self, position: u64) -> bool {
        (position & self.outside_max_rectangle) == 0 && position < self.end_position
    }

    #[inline(always)] fn compressed(&self, position: u64) -> u64 {
        change_number_of_columns(
            position,
            self.cram_first_row_mask,
            self.cram_number_of_columns,
            self.number_of_columns_in_end_db
        )
    }

    #[inline(always)]
    pub fn uncompressed(&self, compressed_position: u64) -> u64 {
        change_number_of_columns(
            compressed_position,
            (1u64 << self.number_of_columns_in_end_db) - 1,
            self.number_of_columns_in_end_db,
            self.cram_number_of_columns
        )
    }

    pub fn max_compressed_end(&self) -> u64 {
        1u64 << (!self.outside_max_rectangle).count_ones()
    }
}

impl EndDbSlicesProvider for LimitedColumnsSliceProvider {
    type Game = Cram;
    type SliceIterator<'si> = LimitedColumnsSliceIterator<'si>;
    type InSlicePosition = u32;
    type UncompressedSlice = SortedPositionNimberMap<u32>;

    #[inline(always)] fn position_to_slice(&self, position: &u64) -> Option<usize> {
        self.contains(*position).then(|| (self.compressed(*position) >> 32) as usize)
    }

    #[inline(always)] fn strip(&self, position: &u64) -> Self::InSlicePosition {
        (self.compressed(*position) & (u32::MAX as u64)) as u32
    }

    fn slice_content<'si, 's: 'si, 'g: 'si>(&'s self, game: &'g Self::Game, slice_index: usize) -> Option<Self::SliceIterator<'si>> {
        let max_compressed_end = self.max_compressed_end();
        (slice_index as u64 <= (max_compressed_end-1) >> 32).then(
            || Self::SliceIterator::new(game, slice_index, max_compressed_end, self.number_of_columns_in_end_db)
        )
    }

    fn slice_pushed(&mut self, slice_index: usize) {
        //self.content_mask |= ((1<<self.number_of_columns_in_end_db)-1) << (slice_index * game.number_of_cols);
        self.end_position = self.uncompressed(self.max_compressed_end().min((slice_index as u64+1)<<32));
    }

    #[inline(always)] fn get_nimber<SliceType>(&self, slices: &[SliceType], position: &<Self::Game as Game>::Position) -> Option<u8>
        where SliceType: NimbersProvider<Self::InSlicePosition>,
    {
        if self.contains(*position) {
            let compressed = self.compressed(*position);
            slices.get((compressed >> 32) as usize)?.get_nimber(&((compressed & (u32::MAX as u64)) as u32))
        } else {
            None
        }
    }
}