use crate::game::Game;
use crate::dbs::NimbersProvider;
//use std::collections::HashMap;
//use crate::dbs::{NimbersProvider, NimbersStorer};

/// Provides a set of positions near the end of the game. The set is divided into slices.
pub trait EndDbSlicesProvider {

    /// Game for which the provider provides positions.
    type Game: Game;

    /// Iterator over positions included in the slice.
    type SliceIterator<'si>: Iterator<Item=<Self::Game as Game>::Position> + 'si where Self: 'si;

    /// Representation of the position, possibly striped to information which allows for distinguishing positions included in the same slice only.
    type InSlicePosition;

    /// Type used to build slice, which is farther (later) compressed.
    type UncompressedSlice;    //: NimbersProvider<Self::InSlicePosition> + NimbersStorer<Self::InSlicePosition> + Default;

    /// Returns either the index of slice that contains given `position` of `game`
    /// or `None` if the `position` is not in any slice.
    fn position_to_slice(&self, position: &<Self::Game as Game>::Position) -> Option<usize>;

    /// Returns representation of the `position`, possibly striped to information which allows for distinguishing positions included in the same slice only.
    /// The method is never called for positions for which `position_to_slice` return `None`.
    fn strip(&self, position: &<Self::Game as Game>::Position) -> Self::InSlicePosition;

    /// Returns the iterator over positions included in the slice with given index (`slice_index`)
    /// or `None` if the slice with given index (and all larger indices) does not exists.
    ///
    /// If `is_exhaustive()` returns `true`, the iterator can expose only the positions whose all options (moves) are included in slices
    /// with indices equal or less than `slice_index`. Otherwise, some positions might be skipped.
    fn slice_content<'si, 's: 'si, 'g: 'si>(&'s self, game: &'g Self::Game, slice_index: usize) -> Option<Self::SliceIterator<'si>>;

    /// Called after pushing back the slice (to the given, last index) which has just been built or read from file.
    fn slice_pushed(&mut self, _slice_index: usize) {}

    /// Indicates whether `slice_content` is exhaustive, i.e. before exposing any position, exposes all its successors.
    fn is_exhaustive(&self) -> bool { true }

    /// Returns filtered version of `self` that generates and accepts only the position that fulfil the given `predicate`.
    /// Result is exhaustive only if both `is_exhaustive` flag and `is_exhaustive()` method are `true`.
    fn filtered_ex<Predicate>(self, is_exhaustive: bool, predicate: Predicate) -> FilteredSliceProvider<Self, Predicate>
        where Self: Sized, Predicate: Fn(&<Self::Game as Game>::Position) -> bool
    {
        FilteredSliceProvider { slice_provider: self, predicate, is_exhaustive }
    }

    /// Returns filtered version of `self` that generates and accepts only the position that fulfil the given `predicate`.
    /// Result is not exhaustive.
    fn filtered<Predicate>(self, predicate: Predicate) -> FilteredSliceProvider<Self, Predicate>
        where Self: Sized, Predicate: Fn(&<Self::Game as Game>::Position) -> bool
    {
        FilteredSliceProvider { slice_provider: self, predicate, is_exhaustive: false }
    }

    /// Returns nimber obtained from proper element of `slices`.
    ///
    /// Default implementation uses `position_to_slice` and `strip` and should always work.
    /// However, some `EndDbSlicesProvider`es can reimplement this method for better performance.
    #[inline(always)] fn get_nimber<SliceType>(&self, slices: &[SliceType], position: &<Self::Game as Game>::Position) -> Option<u8>
        where SliceType: NimbersProvider<Self::InSlicePosition>,
    {
        slices.get(
            self.position_to_slice(position)?
        )?.get_nimber(&self.strip(&position))
    }
}

/// Filtered version of `slice_provider` that generates and accepts only the position that fulfil the given `predicate`.
pub struct FilteredSliceProvider<SliceProvider, Predicate> {
    pub slice_provider: SliceProvider,
    pub predicate: Predicate,
    pub is_exhaustive: bool
}

impl<SliceProvider, F, G> EndDbSlicesProvider for FilteredSliceProvider<SliceProvider, F>
where SliceProvider: EndDbSlicesProvider<Game=G>,
    F: Fn(&G::Position) -> bool,
    G: Game,
{
    type Game = G;
    type SliceIterator<'si> = std::iter::Filter<<SliceProvider as EndDbSlicesProvider>::SliceIterator<'si>, &'si F> where SliceProvider: 'si, F: 'si;// Filter<Iterator=SliceProvider::SliceIterator>;
    type InSlicePosition = SliceProvider::InSlicePosition;
    type UncompressedSlice = SliceProvider::UncompressedSlice;

    #[inline(always)]
    fn position_to_slice(&self, position: &G::Position) -> Option<usize> {
        if (self.predicate)(position) {
            self.slice_provider.position_to_slice(position)
        } else {
            None
        }
    }

    #[inline(always)]
    fn strip(&self, position: &<Self::Game as Game>::Position) -> Self::InSlicePosition {
        self.slice_provider.strip(position)
    }

    fn slice_content<'si, 's: 'si, 'g: 'si>(&'s self, game: &'g Self::Game, slice_index: usize) -> Option<Self::SliceIterator<'si>> {
        Some(self.slice_provider.slice_content(game, slice_index)?.filter(&self.predicate))
    }

    fn slice_pushed(&mut self, slice_index: usize) {
        self.slice_provider.slice_pushed(slice_index)
    }

    fn is_exhaustive(&self) -> bool {
        self.is_exhaustive && self.slice_provider.is_exhaustive()
    }

    #[inline(always)] fn get_nimber<SliceType>(&self, slices: &[SliceType], position: &<Self::Game as Game>::Position) -> Option<u8>
        where SliceType: NimbersProvider<Self::InSlicePosition>,
    {
        if (self.predicate)(position) {
            self.slice_provider.get_nimber(slices, position)
        } else {
            None
        }
    }
}
