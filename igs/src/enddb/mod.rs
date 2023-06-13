use crate::dbs::{NimbersProvider, NimbersStorer};
use crate::game::{DecomposableGame, Game, SimpleGame};

pub mod compressed_slice;
pub mod builder;
pub use self::compressed_slice::{CompressedSlice, CompressedSliceBuilder};
pub use self::builder::*;
use crate::enddb::compressed_slice::*;
use csf::fp;
use std::fmt::Display;

pub mod verifier;

#[cfg(feature = "BP128")] mod bp128delta;
#[cfg(feature = "BP128")] pub use bp128delta::ClusterBP128;

#[cfg(feature = "CMPH")] mod cmph;
#[cfg(feature = "CMPH")] pub use crate::enddb::cmph::ClusterCMPH;

pub mod sorted_position_nimber_map;
pub use sorted_position_nimber_map::SortedPositionNimberMap;
pub mod slices_provider;
pub use slices_provider::EndDbSlicesProvider;
use std::hash::Hash;
use csf::coding::{BuildCoding, Coding, minimum_redundancy, BuildMinimumRedundancy};
use ph::BuildSeededHasher;
use ph::fmph::{GroupSize, SeedSize, TwoToPowerBitsStatic};

/// End database - maps position near the end of the game to nimbers.
/// It is usually divided into slices.
pub struct EndDb<SlicesProvider, Slice>
    //where SlicesProvider: for<'si> EndDbSlicesProvider<'si>
{
    /// Vector of slices, each stores a fragment of database.
    pub slices: Vec<Slice>,

    /// Maps positions to slices.
    pub slice_provider: SlicesProvider
}

impl<SlicesProvider, SliceType> EndDb<SlicesProvider, SliceType>
    where SlicesProvider: EndDbSlicesProvider
{
    pub fn push_slice(&mut self, slice: SliceType) {
        self.slices.push(slice);
        self.slice_provider.slice_pushed(self.slices.len()-1);
    }
}

impl<SlicesProvider, SliceType, ISP> EndDb<SlicesProvider, SliceType>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
        SliceType: CompressedSlice
{
    /// Returns approximate, total (including heap memory) number of bytes occupied by self.
    pub fn size_bytes(&self) -> usize {
        self.slices.iter().map(|s| s.size_bytes()).sum::<usize>() + std::mem::size_of_val(self)
    }
}

impl<SlicesProvider, SliceType, G, ISP, CSM> EndDb<SlicesProvider, SliceType>
    where SlicesProvider: EndDbSlicesProvider<Game=G, InSlicePosition=ISP, UncompressedSlice=CSM>,
          SliceType: /*CompressedSlice +*/ NimbersProvider<ISP>,
          G: SimpleGame,
          //ISP: std::hash::Hash + Eq,
          CSM: NimbersStorer<ISP>+NimbersProvider<ISP>+Default,
{
    /// Calculates nimber of position (of given game) and adds it to the current_slice.
    /// Nimbers of successors (of the position) not included in neither current_slice nor previous slices,
    /// are also recursively calculated and added to current_slice.
    fn add_simple_game_nimber(&self, game: &G, position: G::Position, current_slice: &mut CSM) -> u8 {
        let mut nimbers = 0u64;
        for s in game.successors(&position) {
            nimbers |= 1u64 << self.get_simple_game_nimber(game, s, current_slice);
        }
        let result = (!nimbers).trailing_zeros() as u8;
        current_slice.store_nimber(self.slice_provider.strip(&position), result);
        result
    }

    /// Calculates nimber of position (of given game) from either current_slice or previous slices.
    /// If the position is not included there, its nimber (and possibly nimber of some its sub-positions)
    /// is calculated and added to current_slice.
    fn get_simple_game_nimber(&self, game: &G, position: G::Position, current_slice: &mut CSM) -> u8 {
        let striped = self.slice_provider.strip(&position);
        match self.slice_provider.position_to_slice(&position) {
            Some(slice_idx) if slice_idx < self.slices.len() =>
                if let Some(nimber) = self.slices[slice_idx].get_nimber(&striped) {
                    nimber
                } else {
                    self.add_simple_game_nimber(game, position, current_slice)
                },
            _ => if let Some(nimber) = current_slice.get_nimber(&striped) {
                nimber
            } else {
                self.add_simple_game_nimber(game, position, current_slice)
            }
        }
    }
}

impl<SlicesProvider, SliceType, G, ISP, CSM> EndDb<SlicesProvider, SliceType>
    where SlicesProvider: EndDbSlicesProvider<Game=G, InSlicePosition=ISP, UncompressedSlice=CSM>,
          SliceType: /*CompressedSlice +*/ NimbersProvider<ISP>,
          G: DecomposableGame,
          //ISP: std::hash::Hash + Eq,
          CSM: NimbersStorer<ISP>+NimbersProvider<ISP>+Default,
{
    /// Calculates nimber of the `position` (of the given `game`) and adds it to `current_slice`.
    /// Nimbers of successors (of the position) not included in neither current_slice nor previous slices,
    /// are also recursively calculated and added to the `current_slice`.
    fn add_decomposable_game_nimber(&self, game: &G, position: G::Position, current_slice: &mut CSM) -> u8 {
        let mut nimbers = 0u64;
        for s in game.successors(&position) {
            let mut s_nimber = 0;
            for s_component in game.decompose(&s) {
                s_nimber ^= self.get_decomposable_game_nimber(game, s_component, current_slice);
            }
            nimbers |= 1u64 << s_nimber;
        }
        let result = (!nimbers).trailing_zeros() as u8;
        current_slice.store_nimber(self.slice_provider.strip(&position), (!nimbers).trailing_zeros() as u8);
        result
    }

    /// Calculates nimber of the `position` (of the given `game`) from either `current_slice` or previous slices.
    /// If the `position` is not included there, its nimber (and possibly nimber of some its sub-positions)
    /// is calculated and added to current_slice.
    fn get_decomposable_game_nimber(&self, game: &G, position: G::Position, current_slice: &mut CSM) -> u8 {
        let striped = self.slice_provider.strip(&position);
        match self.slice_provider.position_to_slice(&position) {
            Some(slice_idx) if slice_idx < self.slices.len() =>
                if let Some(nimber) = self.slices[slice_idx].get_nimber(&striped) {
                    nimber
                } else {
                    self.add_decomposable_game_nimber(game, position, current_slice)
                },
            _ => if let Some(nimber) = current_slice.get_nimber(&striped) {
                nimber
            } else {
                self.add_decomposable_game_nimber(game, position, current_slice)
            }
        }
    }
}

impl<SlicesProvider, SliceType, G, ISP> NimbersProvider<G::Position> for EndDb<SlicesProvider, SliceType>
    where SlicesProvider: EndDbSlicesProvider<Game=G, InSlicePosition=ISP>,
        SliceType: NimbersProvider<ISP>,
        G: Game
{
    #[inline(always)] fn get_nimber(&self, position: &G::Position) -> Option<u8> {
        self.slice_provider.get_nimber(&self.slices, position)
    }
}

impl<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice> From<EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice>> for EndDb<SlicesProvider, CompressedSlice>
{
    fn from(builder: EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice>) -> Self {
        builder.enddb
    }
}

impl<SlicesProvider: EndDbSlicesProvider, C, S> EndDb<SlicesProvider, fp::CMap::<C, S>>
{
    pub fn with_fpcmap(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

impl<SlicesProvider, GS: GroupSize, SS: SeedSize, C, S> EndDb<SlicesProvider, fp::GOCMap::<C, GS, SS, S>>
    where SlicesProvider: EndDbSlicesProvider
{
    pub fn with_fpcmap2(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

impl<SlicesProvider, S> EndDb<SlicesProvider, fp::Map::<S>>
    where SlicesProvider: EndDbSlicesProvider
{
    pub fn with_fpmap(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

impl<SlicesProvider, S> EndDb<SlicesProvider, ls::Map<S>>
    where SlicesProvider: EndDbSlicesProvider
{
    pub fn with_lsmap(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

impl<SlicesProvider, C, S> EndDb<SlicesProvider, ls::CMap::<C, S>>
    where SlicesProvider: EndDbSlicesProvider,
          //C: for <'d> Coding<'d, Value=u8>
{
    pub fn with_lscmap(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

#[cfg(feature = "BP128")]
impl<SlicesProvider> EndDb<SlicesProvider, ClusterBP128>
    where SlicesProvider: for<'si> EndDbSlicesProvider<'si, InSlicePosition=u32>
{
    pub fn with_bp128(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

#[cfg(feature = "CMPH")]
impl<SlicesProvider> EndDb<SlicesProvider, ClusterCMPH>
    where SlicesProvider: for<'si> EndDbSlicesProvider<'si, InSlicePosition=u32>
{
    pub fn with_chd(slice_provider: SlicesProvider) -> Self {
        Self { slices: Vec::new(), slice_provider }
    }
}

impl<C, SlicesProvider, ISP, S> EndDb<SlicesProvider, fp::CMap::<C, S>>
    where C: Coding<Value=u8>,
          SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          ISP: std::hash::Hash + Clone,
          S: BuildSeededHasher
{
    pub fn build_with_fpcmap_conf_verifier<BC, LSC, CSB, Checker>(
        slice_provider: SlicesProvider,
        fpcconf: fp::CMapConf<BC, LSC, CSB, S>,
        verifier: Checker
    ) -> EndDbBuilder<SlicesProvider, FPCMapBuilder<BC, LSC, CSB, S>, Checker, fp::CMap::<C, S>>
        where BC: BuildCoding<u8, Coding=C>,
              LSC: fp::LevelSizeChooser+Display+Clone,
              CSB: fp::CollisionSolverBuilder
    {
        EndDbBuilder::<SlicesProvider, FPCMapBuilder<BC, LSC, CSB, S>, Checker, fp::CMap::<C, S>> {
            enddb: Self::with_fpcmap(slice_provider),
            builder: fpcconf.into(),
            verifier
        }
    }

    #[inline]
    pub fn build_with_fpcmap_conf<BC, LSC, CSB>(slice_provider: SlicesProvider, fpcconf: fp::CMapConf<BC, LSC, CSB, S>)
        -> EndDbBuilder<SlicesProvider, FPCMapBuilder<BC, LSC, CSB, S>, (), fp::CMap::<C, S>>
        where BC: BuildCoding<u8, Coding=C>,
              LSC: fp::LevelSizeChooser+Display+Clone,
              CSB: fp::CollisionSolverBuilder
    {
        Self::build_with_fpcmap_conf_verifier(slice_provider, fpcconf, ())
    }
}

impl<SlicesProvider, ISP: Hash> EndDb<SlicesProvider, fp::CMap::<minimum_redundancy::Coding<u8>>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
    ISP: std::hash::Hash + Clone
{
    #[inline]
    pub fn build_with_fpcmap_verifier<Checker>(
        slice_provider: SlicesProvider,
        verifier: Checker) -> EndDbBuilder<SlicesProvider, FPCMapBuilder<BuildMinimumRedundancy>, Checker, fp::CMap::<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_fpcmap_conf_verifier(slice_provider, fp::CMapConf::default(), verifier)
    }

    #[inline]
    pub fn build_with_fpcmap(slice_provider: SlicesProvider) -> EndDbBuilder<SlicesProvider, FPCMapBuilder<BuildMinimumRedundancy>, (), fp::CMap::<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_fpcmap_conf_verifier(slice_provider, fp::CMapConf::default(), ())
    }
}

impl<GS, SS, C, SlicesProvider, ISP, S> EndDb<SlicesProvider, fp::GOCMap::<C, GS, SS, S>>
    where C: Coding<Value=u8>,
          SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          ISP: std::hash::Hash + Clone,
          S: BuildSeededHasher,
          GS: GroupSize, SS: SeedSize
{
    pub fn build_with_fpcmap2_conf_verifier<BC, LSC, Checker>(
        slice_provider: SlicesProvider,
        fpcconf: fp::GOCMapConf<BC, LSC, GS, SS, S>,
        verifier: Checker
    ) -> EndDbBuilder<SlicesProvider, FPCMap2Builder<GS, SS, BC, LSC, S>, Checker, fp::GOCMap::<C, GS, SS, S>>
        where BC: BuildCoding<u8, Coding=C>, LSC: fp::LevelSizeChooser+Display+Clone
    {
        EndDbBuilder::<SlicesProvider, FPCMap2Builder<GS, SS, BC, LSC, S>, Checker, fp::GOCMap::<C, GS, SS, S>> {
            enddb: Self::with_fpcmap2(slice_provider),
            builder: fpcconf.into(),
            verifier
        }
    }

    #[inline]
    pub fn build_with_fpcmap2_conf<BC, LSC>(slice_provider: SlicesProvider, fpcconf: fp::GOCMapConf<BC, LSC, GS, SS, S>)
                                              -> EndDbBuilder<SlicesProvider, FPCMap2Builder<GS, SS, BC, LSC, S>, (), fp::GOCMap::<C, GS, SS, S>>
        where BC: BuildCoding<u8, Coding=C>, LSC: fp::LevelSizeChooser+Display+Clone
    {
        Self::build_with_fpcmap2_conf_verifier(slice_provider, fpcconf, ())
    }
}

impl<SlicesProvider, ISP: Hash> EndDb<SlicesProvider, fp::GOCMap::<minimum_redundancy::Coding<u8>, TwoToPowerBitsStatic::<4>, TwoToPowerBitsStatic<2>>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          ISP: std::hash::Hash + Clone
{
    #[inline]
    pub fn build_with_fpcmap2_verifier<Checker>(
        slice_provider: SlicesProvider,
        verifier: Checker) -> EndDbBuilder<SlicesProvider, FPCMap2Builder<TwoToPowerBitsStatic::<4>, TwoToPowerBitsStatic<2>, BuildMinimumRedundancy>, Checker, fp::GOCMap::<minimum_redundancy::Coding<u8>, TwoToPowerBitsStatic<4>, TwoToPowerBitsStatic<2>>>
    {
        Self::build_with_fpcmap2_conf_verifier(slice_provider, fp::GOCMapConf::default(), verifier)
    }

    #[inline]
    pub fn build_with_fpcmap2(slice_provider: SlicesProvider) -> EndDbBuilder<SlicesProvider, FPCMap2Builder, (), fp::GOCMap::<minimum_redundancy::Coding<u8>, TwoToPowerBitsStatic::<4>, TwoToPowerBitsStatic<2>>>
    {
        Self::build_with_fpcmap2_conf_verifier(slice_provider, fp::GOCMapConf::default(), ())
    }
}

impl<SlicesProvider, ISP, S> EndDb<SlicesProvider, fp::Map::<S>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          ISP: std::hash::Hash + Clone,
          S: BuildSeededHasher
{
    pub fn build_with_fpmap_conf_verifier<LSC, CSB, Checker>(
        slice_provider: SlicesProvider,
        fpconf: fp::MapConf<LSC, CSB, S>,
        verifier: Checker
    ) -> EndDbBuilder<SlicesProvider, FPMapBuilder<LSC, CSB, S>, Checker, fp::Map::<S>>
        where LSC: fp::SimpleLevelSizeChooser+Display+Clone,
              CSB: fp::CollisionSolverBuilder
    {
        EndDbBuilder::<SlicesProvider, FPMapBuilder<LSC, CSB, S>, Checker, fp::Map::<S>> {
            enddb: Self::with_fpmap(slice_provider),
            builder: fpconf.into(),
            verifier
        }
    }

    #[inline]
    pub fn build_with_fpmap_conf<LSC, CSB>(slice_provider: SlicesProvider, fpconf: fp::MapConf<LSC, CSB, S>)
                                                   -> EndDbBuilder<SlicesProvider, FPMapBuilder<LSC, CSB, S>, (), fp::Map::<S>>
        where LSC: fp::SimpleLevelSizeChooser+Display+Clone,
              CSB: fp::CollisionSolverBuilder
    {
        Self::build_with_fpmap_conf_verifier(slice_provider, fpconf, ())
    }
}

impl<SlicesProvider, ISP: Hash> EndDb<SlicesProvider, fp::Map>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          ISP: std::hash::Hash + Clone
{
    #[inline]
    pub fn build_with_fpmap_verifier<Checker>(
        slice_provider: SlicesProvider,
        verifier: Checker) -> EndDbBuilder<SlicesProvider, FPMapBuilder, Checker, fp::Map>
    {
        Self::build_with_fpmap_conf_verifier(slice_provider, fp::MapConf::default(), verifier)
    }

    #[inline]
    pub fn build_with_fpmap(slice_provider: SlicesProvider) -> EndDbBuilder<SlicesProvider, FPMapBuilder, (), fp::Map> {
        Self::build_with_fpmap_conf_verifier(slice_provider, fp::MapConf::default(), ())
    }
}


impl<SlicesProvider, ISP> EndDb<SlicesProvider, ls::Map>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          //ISP: std::hash::Hash + Clone
{
    pub fn build_with_lsmap_verifier<Checker>(slice_provider: SlicesProvider, verifier: Checker) -> EndDbBuilder<SlicesProvider, LSMapBuilder, Checker, ls::Map> {
        EndDbBuilder::<SlicesProvider, LSMapBuilder, Checker, ls::Map> {
            enddb: Self::with_lsmap(slice_provider),
            builder: LSMapBuilder::default(),
            verifier
        }
    }

    #[inline]
    pub fn build_with_lsmap(slice_provider: SlicesProvider) -> EndDbBuilder<SlicesProvider, LSMapBuilder, (), ls::Map> {
        Self::build_with_lsmap_verifier(slice_provider, ())
    }
}

impl<SlicesProvider, ISP, S> EndDb<SlicesProvider, ls::Map<S>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
        S: BuildSeededHasher
//ISP: std::hash::Hash + Clone
{
    pub fn build_with_lsmap_hash_verifier<Checker>(slice_provider: SlicesProvider, hash: S, verifier: Checker) -> EndDbBuilder<SlicesProvider, LSMapBuilder<S>, Checker, ls::Map<S>> {
        EndDbBuilder::<SlicesProvider, LSMapBuilder<S>, Checker, ls::Map<S>> {
            enddb: Self::with_lsmap(slice_provider),
            builder: LSMapBuilder{hash},
            verifier
        }
    }

    #[inline]
    pub fn build_with_lsmap_hash(slice_provider: SlicesProvider, hash_builder: S) -> EndDbBuilder<SlicesProvider, LSMapBuilder<S>, (), ls::Map<S>> {
        Self::build_with_lsmap_hash_verifier(slice_provider, hash_builder, ())
    }
}

impl<SlicesProvider, ISP, C> EndDb<SlicesProvider, ls::CMap::<C>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>, C: Coding<Value=u8>
{
    pub fn build_with_lscmap_coding_verifier<BC, Checker>(slice_provider: SlicesProvider, coding: BC, verifier: Checker)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BC>, Checker, ls::CMap::<C>>
    {
        EndDbBuilder::<SlicesProvider, LSCMapBuilder<BC>, Checker, ls::CMap::<C>> {
            enddb: Self::with_lscmap(slice_provider),
            builder: LSCMapBuilder{hash: Default::default(), coding},
            verifier
        }
    }

    #[inline]
    pub fn build_with_lscmap_coding<BC>(slice_provider: SlicesProvider, coding: BC)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BC>, (), ls::CMap::<C>>
    {
        Self::build_with_lscmap_coding_verifier(slice_provider, coding, ())
    }
}

impl<SlicesProvider, ISP> EndDb<SlicesProvider, ls::CMap::<minimum_redundancy::Coding<u8>>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          //ISP: std::hash::Hash + Clone
{
    pub fn build_with_lscmap_bpf_verifier<Checker>(slice_provider: SlicesProvider, bits_per_fragment: u8, verifier: Checker)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy>, Checker, ls::CMap<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_lscmap_coding_verifier(slice_provider, BuildMinimumRedundancy{ bits_per_fragment }, verifier)
    }

    #[inline]
    pub fn build_with_lscmap_verifier<Checker>(slice_provider: SlicesProvider, verifier: Checker)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy>, Checker, ls::CMap<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_lscmap_bpf_verifier(slice_provider, 0, verifier)
    }

    #[inline]
    pub fn build_with_lscmap_bpf(slice_provider: SlicesProvider, bits_per_fragment: u8)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy>, (), ls::CMap<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_lscmap_bpf_verifier(slice_provider, bits_per_fragment, ())
    }

    #[inline]
    pub fn build_with_bdzhmap(slice_provider: SlicesProvider)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy>, (), ls::CMap<minimum_redundancy::Coding<u8>>>
    {
        Self::build_with_lscmap_bpf(slice_provider, 0)
    }
}

impl<SlicesProvider, ISP, C, S> EndDb<SlicesProvider, ls::CMap::<C, S>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>,
          C: Coding<Value=u8>, S: BuildSeededHasher
{
    pub fn build_with_lscmap_coding_hash_verifier<BC, Checker>(slice_provider: SlicesProvider, coding: BC, hash: S, verifier: Checker)
                                                         -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BC, S>, Checker, ls::CMap::<C, S>>
    {
        EndDbBuilder::<SlicesProvider, LSCMapBuilder<BC, S>, Checker, ls::CMap::<C, S>> {
            enddb: Self::with_lscmap(slice_provider),
            builder: LSCMapBuilder{hash, coding},
            verifier
        }
    }

    #[inline]
    pub fn build_with_lscmap_coding_hash<BC>(slice_provider: SlicesProvider, coding: BC, hash: S)
                                       -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BC, S>, (), ls::CMap::<C, S>>
    {
        Self::build_with_lscmap_coding_hash_verifier(slice_provider, coding, hash, ())
    }
}

impl<SlicesProvider, ISP, S> EndDb<SlicesProvider, ls::CMap::<minimum_redundancy::Coding<u8>, S>>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP>, S: BuildSeededHasher
{
    #[inline] pub fn build_with_lscmap_hash_bpf_verifier<Checker>(slice_provider: SlicesProvider, hash: S, bits_per_fragment: u8, verifier: Checker)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy, S>, Checker, ls::CMap::<minimum_redundancy::Coding<u8>, S>>
    {
        Self::build_with_lscmap_coding_hash_verifier(slice_provider, BuildMinimumRedundancy{ bits_per_fragment }, hash, verifier)
    }

    #[inline] pub fn build_with_lscmap_hash_verifier<Checker>(slice_provider: SlicesProvider, hash: S, verifier: Checker)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy, S>, Checker, ls::CMap::<minimum_redundancy::Coding<u8>, S>>
    {
        Self::build_with_lscmap_hash_bpf_verifier(slice_provider, hash,0, verifier)
    }

    #[inline] pub fn build_with_lscmap_hash_bpf(slice_provider: SlicesProvider, hash: S, bits_per_fragment: u8)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy, S>, (), ls::CMap::<minimum_redundancy::Coding<u8>, S>>
    {
        Self::build_with_lscmap_hash_bpf_verifier(slice_provider, hash, bits_per_fragment, ())
    }

    #[inline] pub fn build_with_lscmap_hash(slice_provider: SlicesProvider, hash: S)
        -> EndDbBuilder<SlicesProvider, LSCMapBuilder<BuildMinimumRedundancy, S>, (), ls::CMap::<minimum_redundancy::Coding<u8>, S>>
    {
        Self::build_with_lscmap_hash_bpf(slice_provider, hash, 1)
    }
}

#[cfg(feature = "BP128")]
impl<SlicesProvider> EndDb<SlicesProvider, ClusterBP128>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=u32>
{
    pub fn build_with_bp128_verifier<Checker>(slice_provider: SlicesProvider, verifier: Checker) -> EndDbBuilder<SlicesProvider, BP128Buider, Checker, ClusterBP128> {
        EndDbBuilder::<SlicesProvider, BP128Buider, Checker, ClusterBP128> {
            enddb: Self::with_bp128(slice_provider),
            builder: BP128Buider::default(),
            verifier
        }
    }

    #[inline]
    pub fn build_with_bp128(slice_provider: SlicesProvider) -> EndDbBuilder<SlicesProvider, BP128Buider, (), ClusterBP128> {
        Self::build_with_bp128_verifier(slice_provider, ())
    }
}

#[cfg(feature = "CMPH")]
impl<SlicesProvider> EndDb<SlicesProvider, ClusterCMPH>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=u32>
{
    pub fn build_with_chd_verifier<Checker>(slice_provider: SlicesProvider, lambda: u8, verifier: Checker) -> EndDbBuilder<SlicesProvider, CMPHBuider, Checker, ClusterCMPH> {
        EndDbBuilder::<SlicesProvider, CMPHBuider, Checker, ClusterCMPH> {
            enddb: Self::with_chd(slice_provider),
            builder: CMPHBuider{lambda},
            verifier
        }
    }

    #[inline]
    pub fn build_with_chd(slice_provider: SlicesProvider, lambda: u8) -> EndDbBuilder<SlicesProvider, CMPHBuider, (), ClusterCMPH> {
        Self::build_with_chd_verifier(slice_provider, lambda, ())
    }
}