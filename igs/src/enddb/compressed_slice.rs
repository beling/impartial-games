use std::io;
use std::collections::HashMap;
use std::fmt;
#[cfg(feature = "BP128")] use super::bp128delta::ClusterBP128;
use binout::{AsIs, Serializer};
pub use csf::{fp, ls};
use ph::fmph::{SeedSize, TwoToPowerBits, TwoToPowerBitsStatic, GroupSize};

use std::hash::BuildHasher;
use crate::enddb::SortedPositionNimberMap;
#[cfg(feature = "CMPH")] use crate::enddb::cmph::ClusterCMPH;
use crate::dbs::NimbersProvider;
use csf::coding::{BuildCoding, BuildMinimumRedundancy, Coding, SerializableCoding};
use std::borrow::Borrow;
use ph::{BuildDefaultSeededHasher, BuildSeededHasher, GetSize};

/// Slice of the end database, usually compressed/succinct.
pub trait CompressedSlice/*<InSliceGamePosition>: NimbersProvider<InSliceGamePosition>*/ {
    /// The size of self in bytes (including dynamically allocated memory).
    fn size_bytes(&self) -> usize;

    /// Writes given slice to the output.
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()>;
}

/// Constructs slices and reads/writes them from/to files.
pub trait CompressedSliceBuilder</*InSliceGamePosition,*/ UncompressedSlice>: fmt::Display {

    /// The type of slice supported by self.
    type CompressedSlice: CompressedSlice/*<InSliceGamePosition>*/;

    /// Construct slice with positions and nimbers included in src.
    fn construct(&self, src: UncompressedSlice) -> Self::CompressedSlice;

    /// Returns end database slice read from the input.
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized;
}

// Generate value which is (we hope) unique for hash_builder.
/*fn hasher_id<S: BuildHasher>(hash_builder: &S) -> u64 {
    let mut h = hash_builder.build_hasher();
    h.write_u64(0xf178015a3109cc1d);
    h.finish()
}*/

/// Generate value which is (we hope) unique for hash_builder.
fn hasher_id<S: BuildSeededHasher>(seeded_hash_builder: &S) -> u64 {
    seeded_hash_builder.hash_one(0xf178015a3109cc1du64, 1234)
}


// ------------------ FPCMap -----------------

impl<InSliceGamePosition, C, S> NimbersProvider<InSliceGamePosition> for fp::CMap::<C, S>
where InSliceGamePosition: std::hash::Hash, C: Coding<Value=u8>, S: BuildSeededHasher {
    #[inline(always)]
    fn get_nimber(&self, position: &InSliceGamePosition) -> Option<u8> {
        self.get(position).map(|v| *v.borrow())
    }
}

impl<C, S> CompressedSlice for fp::CMap::<C, S>
where C: SerializableCoding<Value=u8> + GetSize, S: BuildSeededHasher {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        GetSize::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()> {
        fp::CMap::<C, S>::write(&self, output, |o, v| AsIs::write(o, *v))
    }
}

//#[derive(Default)]
pub struct FPCMapBuilder<BC = BuildMinimumRedundancy, LSC = fp::OptimalLevelSize, CSB = fp::LoMemAcceptEquals, S = BuildDefaultSeededHasher>
where LSC: fp::LevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher {
    pub conf: fp::CMapConf<BC, LSC, CSB, S>
}

impl<BC, LSC, CSB, S> From<fp::CMapConf<BC, LSC, CSB, S>> for FPCMapBuilder<BC, LSC, CSB, S>
where LSC: fp::LevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher {
    fn from(conf: fp::CMapConf<BC, LSC, CSB, S>) -> Self {
        Self { conf }
    }
}

impl<BC, LSC, CSB, S> fmt::Display for FPCMapBuilder<BC, LSC, CSB, S>
where BC: BuildCoding<u8>, LSC: fmt::Display + fp::LevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FPCMap__{}__{}_levels__h{:X}", self.conf.coding.name(), self.conf.level_size_chooser, hasher_id(&self.conf.hash))
    }
}

impl<BC, LSC, InSliceGamePosition, CSB, S, HMS, Coding> CompressedSliceBuilder<HashMap<InSliceGamePosition, u8, HMS>> for FPCMapBuilder<BC, LSC, CSB, S>
where BC: BuildCoding<u8, Coding=Coding> + Clone,
      Coding: csf::coding::Coding<Value=u8> + csf::coding::SerializableCoding + GetSize,
        LSC: fp::LevelSizeChooser + fmt::Display + Clone,
      InSliceGamePosition: std::hash::Hash + Clone,
      CSB: fp::CollisionSolverBuilder + fp::IsLossless + Clone,
      S: BuildSeededHasher + Clone
{

    type CompressedSlice = fp::CMap::<Coding, S>;

    #[inline(always)]
    fn construct(&self, src: HashMap<InSliceGamePosition, u8, HMS>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_map_with_conf(&src, self.conf.clone(), &mut ())
        //Self::CompressedSlice::with_map_bpf_hash_lsc(&src, self.bits_per_fragment, BuildHasherDefault::<XxHash64>::default(), &self.level_size_chooser, &mut ())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.conf.hash.clone())
    }
}

impl<BC, LSC, InSliceGamePosition, CSB, S, Coding> CompressedSliceBuilder<SortedPositionNimberMap<InSliceGamePosition>>
for FPCMapBuilder<BC, LSC, CSB, S>
    where BC: BuildCoding<u8, Coding=Coding> + Clone,
      Coding: csf::coding::Coding<Value=u8> + csf::coding::SerializableCoding + GetSize,
      LSC: fp::LevelSizeChooser + fmt::Display + Clone,
      InSliceGamePosition: std::hash::Hash + Clone,
      CSB: fp::CollisionSolverBuilder + fp::IsLossless + Clone,
      S: BuildSeededHasher + Clone
{
    type CompressedSlice = fp::CMap::<Coding, S>;

    #[inline(always)]
    fn construct(&self, mut src: SortedPositionNimberMap<InSliceGamePosition>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_slices_with_conf(&mut src.positions, &src.nimbers, self.conf.clone(), &mut ())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.conf.hash.clone())
    }
}

// --------------- FPMap ----------------

impl<InSliceGamePosition: std::hash::Hash, S: BuildSeededHasher> NimbersProvider<InSliceGamePosition> for fp::Map<S> {
    #[inline(always)]
    fn get_nimber(&self, position: &InSliceGamePosition) -> Option<u8> {
        fp::Map::<S>::get(self, position).map(|nimber_u64| nimber_u64 as u8)
    }
}

impl<S: BuildSeededHasher> CompressedSlice for fp::Map<S> {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        GetSize::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()> {
        fp::Map::<S>::write(&self, output)
    }
}

//#[derive(Default)]
pub struct FPMapBuilder<LSC = fp::OptimalLevelSize, CSB = fp::LoMemAcceptEquals, S = BuildDefaultSeededHasher>
    where LSC: fp::SimpleLevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher
{
    pub conf: fp::MapConf<LSC, CSB, S>
}

impl<LSC, CSB, S> From<fp::MapConf<LSC, CSB, S>> for FPMapBuilder<LSC, CSB, S>
    where LSC: fp::SimpleLevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher {
    fn from(conf: fp::MapConf<LSC, CSB, S>) -> Self {
        Self { conf }
    }
}

impl<LSC: fmt::Display + fp::SimpleLevelSizeChooser, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher> fmt::Display for FPMapBuilder<LSC, CSB, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hid = hasher_id(&self.conf.hash);
        if self.conf.bits_per_value == 0 {
            write!(f, "FPMap__{}_levels__h{:X}", self.conf.level_size_chooser, hid)
        } else {
            write!(f, "FPMap{}__{}_levels__h{:X}", self.conf.bits_per_value, self.conf.level_size_chooser, hid)
        }
    }
}

impl<LSC, InSliceGamePosition, CSB, S, HMS> CompressedSliceBuilder<HashMap<InSliceGamePosition, u8, HMS>>
for FPMapBuilder<LSC, CSB, S>
    where LSC: fp::SimpleLevelSizeChooser + fmt::Display + Clone,
          InSliceGamePosition: std::hash::Hash + Clone,
          CSB: fp::CollisionSolverBuilder + Clone,
          S: BuildSeededHasher + Clone
{

    type CompressedSlice = fp::Map::<S>;

    #[inline(always)]
    fn construct(&self, src: HashMap<InSliceGamePosition, u8, HMS>) -> Self::CompressedSlice {
        Self::CompressedSlice::with_map_conf(&src, self.conf.clone(), &mut ())
        //Self::CompressedSlice::with_map_bpf_hash_lsc(&src, self.bits_per_fragment, BuildHasherDefault::<XxHash64>::default(), &self.level_size_chooser, &mut ())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, self.conf.hash.clone())
    }
}

impl<LSC, InSliceGamePosition, CSB, S> CompressedSliceBuilder<SortedPositionNimberMap<InSliceGamePosition>>
for FPMapBuilder<LSC, CSB, S>
    where LSC: fp::SimpleLevelSizeChooser + fmt::Display + Clone,
          InSliceGamePosition: std::hash::Hash + Clone,
          CSB: fp::CollisionSolverBuilder + Clone,
          S: BuildSeededHasher + Clone
{
    type CompressedSlice = fp::Map::<S>;

    #[inline(always)]
    fn construct(&self, mut src: SortedPositionNimberMap<InSliceGamePosition>) -> Self::CompressedSlice {
        Self::CompressedSlice::with_slices_conf(&mut src.positions, &mut src.nimbers, self.conf.clone())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, self.conf.hash.clone())
    }
}


// ------------------ FPCMap2 -----------------

impl<InSliceGamePosition, GS, SS, C, S> NimbersProvider<InSliceGamePosition> for fp::GOCMap::<GS, SS, C, S>
where InSliceGamePosition: std::hash::Hash, C: Coding<Value=u8>, GS:GroupSize, SS: SeedSize, S: BuildSeededHasher {
    #[inline(always)]
    fn get_nimber(&self, position: &InSliceGamePosition) -> Option<u8> {
        self.get(position).map(|v| *v.borrow())
    }
}

impl<C, GS, SS, S> CompressedSlice for fp::GOCMap::<GS, SS, C, S>
where C: SerializableCoding<Value=u8> + GetSize, GS: GroupSize, SS: SeedSize, S: BuildSeededHasher {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        GetSize::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()> {
        fp::GOCMap::<GS, SS, C, S>::write(&self, output, |o, v| AsIs::write(o, *v))
    }
}

//#[derive(Default)]
pub struct FPCMap2Builder<GS: GroupSize = TwoToPowerBits, SS: SeedSize = TwoToPowerBitsStatic<2>, BC = BuildMinimumRedundancy, LSC = fp::OptimalLevelSize, S = BuildDefaultSeededHasher>
    where LSC: fp::LevelSizeChooser, S: BuildSeededHasher {
    pub conf: fp::GOCMapConf<GS, SS, BC, LSC, S>
}

impl<GS: GroupSize, SS: SeedSize, BC, LSC, S> From<fp::GOCMapConf<GS, SS, BC, LSC, S>> for FPCMap2Builder<GS, SS, BC, LSC, S>
where LSC: fp::LevelSizeChooser, S: BuildSeededHasher {
    fn from(conf: fp::GOCMapConf<GS, SS, BC, LSC, S>) -> Self {
        Self { conf }
    }
}

impl<GS: GroupSize, SS: SeedSize, BC, LSC, S> fmt::Display for FPCMap2Builder<GS, SS, BC, LSC, S>
where BC: BuildCoding<u8>, LSC: fmt::Display + fp::LevelSizeChooser, S: BuildSeededHasher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FPGOMap__{}__{}_levels__gr_seed_{}b_size_{}b__h{:X}",
                self.conf.coding.name(),
                self.conf.level_size_chooser,
                Into::<u8>::into(self.conf.bits_per_seed), 
                Into::<u8>::into(self.conf.bits_per_group),
               hasher_id(&self.conf.hash_builder))
    }
}

impl<BC, LSC, InSliceGamePosition, S, HMS, Coding> CompressedSliceBuilder<HashMap<InSliceGamePosition, u8, HMS>>
for FPCMap2Builder<TwoToPowerBits, TwoToPowerBitsStatic<2>, BC, LSC, S>
    where BC: BuildCoding<u8, Coding=Coding> + Clone,
          Coding: csf::coding::Coding<Value=u8> + csf::coding::SerializableCoding + GetSize,
        LSC: fp::LevelSizeChooser + fmt::Display + Clone,
          InSliceGamePosition: std::hash::Hash + Clone,
          S: BuildSeededHasher + Clone
{

    type CompressedSlice = fp::GOCMap::<TwoToPowerBits, TwoToPowerBitsStatic<2>, Coding, S>;

    #[inline(always)]
    fn construct(&self, src: HashMap<InSliceGamePosition, u8, HMS>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_map_with_conf(&src, self.conf.clone(), &mut ())
        //Self::CompressedSlice::with_map_bpf_hash_lsc(&src, self.bits_per_fragment, BuildHasherDefault::<XxHash64>::default(), &self.level_size_chooser, &mut ())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.conf.hash_builder.clone())
    }
}

impl<BC, LSC, InSliceGamePosition, S, Coding> CompressedSliceBuilder<SortedPositionNimberMap<InSliceGamePosition>>
for FPCMap2Builder<TwoToPowerBits, TwoToPowerBitsStatic<2>, BC, LSC, S>
    where BC: BuildCoding<u8, Coding=Coding> + Clone,
          Coding: csf::coding::Coding<Value=u8> + csf::coding::SerializableCoding + GetSize,
          LSC: fp::LevelSizeChooser + fmt::Display + Clone,
          InSliceGamePosition: std::hash::Hash + Clone,
          S: BuildSeededHasher + Clone
{
    type CompressedSlice = fp::GOCMap::<TwoToPowerBits, TwoToPowerBitsStatic<2>, Coding, S>;

    #[inline(always)]
    fn construct(&self, mut src: SortedPositionNimberMap<InSliceGamePosition>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_slices_with_conf(&mut src.positions, &src.nimbers, self.conf.clone(), &mut ())
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.conf.hash_builder.clone())
    }
}




// --------------- LSMap ----------------

impl<InSliceGamePosition: std::hash::Hash, S: BuildSeededHasher> NimbersProvider<InSliceGamePosition> for ls::Map<S> {
    #[inline(always)]
    fn get_nimber(&self, position: &InSliceGamePosition) -> Option<u8> {
        Some(ls::Map::<S>::get(self, position) as u8)
    }
}

impl<S: BuildSeededHasher> CompressedSlice for ls::Map<S> {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        GetSize::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()> {
        ls::Map::<S>::write(&self, output)
    }
}

#[derive(Default, Copy, Clone)]
pub struct LSMapBuilder<BH: BuildSeededHasher = BuildDefaultSeededHasher> {
    pub hash: BH
}

impl<BH: BuildSeededHasher> fmt::Display for LSMapBuilder<BH> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LSMap_h{:X}", hasher_id(&self.hash))
    }
}

impl<InSliceGamePosition, BH, HMS> CompressedSliceBuilder<HashMap<InSliceGamePosition, u8, HMS>> for LSMapBuilder<BH>
    where InSliceGamePosition: std::hash::Hash, BH: BuildSeededHasher + Clone, HMS: BuildHasher
{
    type CompressedSlice = ls::Map<BH>;

    #[inline(always)]
    fn construct(&self, src: HashMap<InSliceGamePosition, u8, HMS>) -> Self::CompressedSlice {
        Self::CompressedSlice::try_from_hashmap(src, ls::MapConf::hash(self.hash.clone())).unwrap()
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, self.hash.clone())
    }
}

impl<InSliceGamePosition, BH> CompressedSliceBuilder<SortedPositionNimberMap<InSliceGamePosition>> for LSMapBuilder<BH>
    where InSliceGamePosition: std::hash::Hash, BH: BuildSeededHasher + Clone
{
    type CompressedSlice = ls::Map<BH>;

    #[inline(always)]
    fn construct(&self, src: SortedPositionNimberMap<InSliceGamePosition>) -> Self::CompressedSlice {
        Self::CompressedSlice::try_with_conf_kv(&src.positions, &src.nimbers, ls::MapConf::hash(self.hash.clone())).unwrap()
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, self.hash.clone())
    }
}


// --------------- LSCMap ----------------

impl<InSliceGamePosition, C, S> NimbersProvider<InSliceGamePosition> for ls::CMap<C, S>
where InSliceGamePosition: std::hash::Hash,
      C: Coding<Value=u8>,
      S: BuildSeededHasher
{
    #[inline(always)]
    fn get_nimber(&self, position: &InSliceGamePosition) -> Option<u8>
    {
        self.get(position).map(|v| *v.borrow())
    }
}

impl<C, S> CompressedSlice for ls::CMap<C, S>
where //InSliceGamePosition: std::hash::Hash,
    C: SerializableCoding<Value=u8> + GetSize,
      S: BuildSeededHasher
{
    #[inline(always)]
    fn size_bytes(&self) -> usize {
        <ls::CMap::<C, S> as GetSize>::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, output: &mut dyn io::Write) -> io::Result<()> {
        ls::CMap::<C, S>::write(&self, output, |o, v| AsIs::write(o, *v))
    }
}

#[derive(Copy, Clone)]
pub struct LSCMapBuilder<BC, BH: BuildSeededHasher = BuildDefaultSeededHasher> {
    pub hash: BH,
    pub coding: BC
}

impl<BC: BuildCoding<u8>, BH: BuildSeededHasher> fmt::Display for LSCMapBuilder<BC, BH> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LSCMap_{}_h{:X}", self.coding.name(), hasher_id(&self.hash))
    }
}

impl<InSliceGamePosition, BC, BH, HMS, Coding> CompressedSliceBuilder<HashMap<InSliceGamePosition, u8, HMS>> for LSCMapBuilder<BC, BH>
    where InSliceGamePosition: std::hash::Hash,
        BC: BuildCoding<u8, Coding=Coding>,
    Coding: csf::coding::Coding<Value=u8> + csf::coding::SerializableCoding + GetSize,
        BH: BuildSeededHasher + Clone
{
    type CompressedSlice = ls::CMap<Coding, BH>;

    #[inline(always)]
    fn construct(&self, src: HashMap<InSliceGamePosition, u8, HMS>) -> Self::CompressedSlice {
        Self::CompressedSlice::try_from_map_with_builder_conf(&src, &self.coding, ls::MapConf::hash(self.hash.clone()), 0).unwrap()
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.hash.clone())
    }
}

impl<InSliceGamePosition, BC, BH, Coding> CompressedSliceBuilder<SortedPositionNimberMap<InSliceGamePosition>> for LSCMapBuilder<BC, BH>
    where InSliceGamePosition: std::hash::Hash,
          BC: BuildCoding<u8, Coding=Coding>,
          BH: BuildSeededHasher + Clone,
        Coding: csf::coding::SerializableCoding<Value=u8> + GetSize
{
    type CompressedSlice = ls::CMap<Coding, BH>;

    #[inline(always)]
    fn construct(&self, src: SortedPositionNimberMap<InSliceGamePosition>) -> Self::CompressedSlice {
        Self::CompressedSlice::try_from_kv_with_builder_conf(&src.positions, &src.nimbers,
                                                         &self.coding,
                                                         ls::MapConf::hash(self.hash.clone()),
                                                     0).unwrap()
    }

    #[inline(always)]
    fn read(&self, input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        Self::CompressedSlice::read_with_hasher(input, |i| AsIs::read(i), self.hash.clone())
    }
}


// --------------- BP128 ----------------

#[cfg(feature = "BP128")]
impl NimbersProvider<u32> for ClusterBP128 {
    #[inline(always)]
    fn get_nimber(&self, position: &u32) -> Option<u8> {
        Some(ClusterBP128::get(self, *position))
    }
}

#[cfg(feature = "BP128")]
impl CompressedSlice<u32> for ClusterBP128 {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        ClusterBP128::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, _output: &mut dyn io::Write) -> io::Result<()> {
        unimplemented!()
    }
}

#[cfg(feature = "BP128")]
#[derive(Default, Copy, Clone)]
pub struct BP128Buider {}

#[cfg(feature = "BP128")]
impl fmt::Display for BP128Buider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BP128delta")
    }
}

#[cfg(feature = "BP128")]
impl CompressedSliceBuilder<u32, HashMap<u32, u8>> for BP128Buider {
    type CompressedSlice = ClusterBP128;

    #[inline(always)]
    fn construct(&self, src: HashMap<u32, u8>) -> Self::CompressedSlice {
        src.into()
    }

    #[inline(always)]
    fn read(&self, _input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        unimplemented!()
    }
}

#[cfg(feature = "BP128")]
impl CompressedSliceBuilder<u32, SortedPositionNimberMap<u32>> for BP128Buider {
    type CompressedSlice = ClusterBP128;

    #[inline(always)]
    fn construct(&self, src: SortedPositionNimberMap<u32>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_sorted(&src.positions, &src.nimbers)
    }

    #[inline(always)]
    fn read(&self, _input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        unimplemented!()
    }
}


// --------------- CMPH ----------------

#[cfg(feature = "CMPH")]
impl NimbersProvider<u32> for ClusterCMPH {
    #[inline(always)]
    fn get_nimber(&self, position: &u32) -> Option<u8> {
        Some(ClusterCMPH::get(self, *position) as u8)
    }
}

#[cfg(feature = "CMPH")]
impl CompressedSlice<u32> for ClusterCMPH {

    #[inline(always)]
    fn size_bytes(&self) -> usize {
        ClusterCMPH::size_bytes(self)
    }

    #[inline(always)]
    fn write(&self, _output: &mut dyn io::Write) -> io::Result<()> {
        unimplemented!()
    }
}

#[cfg(feature = "CMPH")]
#[derive(Copy, Clone)]
pub struct CMPHBuider {
    /// an average number of keys per bucket; it can be tuned to obtain different trade-offs between generation time and representation size
    pub lambda: u8
}

#[cfg(feature = "CMPH")]
impl fmt::Display for CMPHBuider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CHD{}", self.lambda)
    }
}

/*impl CompressedSliceBuilder<u32, HashMap<u32, u8>> for CMPHBuider {
    type CompressedSlice = ClusterCMPH;

    #[inline(always)]
    fn construct(&self, src: HashMap<u32, u8>) -> Self::CompressedSlice {
        src.into()
    }
}*/

#[cfg(feature = "CMPH")]
impl CompressedSliceBuilder<u32, SortedPositionNimberMap<u32>> for CMPHBuider {
    type CompressedSlice = ClusterCMPH;

    #[inline(always)]
    fn construct(&self, src: SortedPositionNimberMap<u32>) -> Self::CompressedSlice {
        Self::CompressedSlice::from_kv_lambda(&src.positions, &src.nimbers, self.lambda)
    }

    #[inline(always)]
    fn read(&self, _input: &mut dyn io::Read) -> io::Result<Self::CompressedSlice> where Self::CompressedSlice: std::marker::Sized {
        unimplemented!()
    }
}