use super::{EndDb, EndDbSlicesProvider};
use super::compressed_slice::{CompressedSlice, CompressedSliceBuilder};
use crate::game::{SimpleGame, DecomposableGame};
use std::io;
use std::fmt;
use std::path::PathBuf;
pub use super::verifier::*;
use crate::dbs::{NimbersStorer, NimbersProvider};

/// Builds end database.
pub struct EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice>
    //where SlicesProvider: for<'si> EndDbSlicesProvider<'si, InSlicePosition=ISP, UncompressedSlice=US>,
          //SliceBuilder: CompressedSliceBuilder<ISP, US>
{
    pub enddb: EndDb<SlicesProvider, CompressedSlice>,
    pub builder: SliceBuilder,
    pub verifier: NimberChecker
}

impl<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice> EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, CompressedSlice>
    //where SlicesProvider: for<'si> EndDbSlicesProvider<'si, InSlicePosition=ISP, UncompressedSlice=US>,
    //      SliceBuilder: CompressedSliceBuilder<ISP, US>
{
    /// Finishes building and returns the end database.
    pub fn done(self) -> EndDb<SlicesProvider, CompressedSlice> {
        self.enddb
    }
}

pub trait EndDbBuilderForSimpleGame<GameType> where GameType: SimpleGame {   // BBMapEndDb methods for SimpleGames
    fn build_slice(&mut self, game: &GameType) -> bool;
    fn build_slice_cached<P: AsRef<std::path::Path>>(&mut self, game: &GameType, cache_dir: P) -> io::Result<bool>;

    /// Builds slices of the database upto the moment when all are built or the database size exceed optional threshold `target_size_bytes`.
    /// Slices are optionally cached in `cache_dir.0` directory and building is stopped on I/O errors if `cache_dir.1` is `true`.
    fn build<P: AsRef<std::path::Path>>(&mut self, game: &GameType, target_size_bytes: Option<usize>, cache_dir: Option<(P, bool)>);
}

pub trait EndDbBuilderForDecomposableGame<GameType> where GameType: DecomposableGame {   // BBMapEndDb methods for DecomposableGames
    fn build_slice(&mut self, game: &GameType) -> bool;
    fn build_slice_cached<P: AsRef<std::path::Path>>(&mut self, game: &GameType, cache_dir: P) -> io::Result<bool>;

    /// Builds slices of the database upto the moment when all are built or the database size exceed optional threshold `target_size_bytes`.
    /// Slices are optionally cached in `cache_dir.0` directory and building is stopped on I/O errors if `cache_dir.1` is `true`.
    fn build<P: AsRef<std::path::Path>>(&mut self, game: &GameType, target_size_bytes: Option<usize>, cache_dir: Option<(P, bool)>);
}

/// Creates cache directory and returns name of cache file.
fn cache_file_name<P: AsRef<std::path::Path>, G: fmt::Display>(cache_dir: P, game: &G, method_name: &str, slice_index: usize) -> io::Result<PathBuf> {
    let mut result = std::path::PathBuf::new();
    result.push(cache_dir);
    result.push(format!("{}-{}", game, method_name));
    std::fs::create_dir_all(&result)?;
    result.push(format!("{:08}", slice_index));
    result.set_extension("edb");
    Ok(result)
}

macro_rules! impl_bbenddbfor_trait_methods {
    ($GameType:path) => {
        fn build_slice_cached<P: AsRef<std::path::Path>>(&mut self, game: &$GameType, cache_dir: P) -> io::Result<bool> {
            let filename = cache_file_name(cache_dir, game, &self.builder.to_string(), self.enddb.slices.len());
            if let Ok(ref f) = filename {
                if self.read_slice_from_file(f).is_ok() { return Ok(true); }
            }
            if self.build_slice(game) {
                self.write_slice_to_file(filename?, self.enddb.slices.len()-1)?;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        fn build<P: AsRef<std::path::Path>>(&mut self, game: &$GameType, target_size_bytes: Option<usize>, cache_dir: Option<(P, bool)>) {
            let mut sizes: Option<(usize, usize)> = if let Some(target) = target_size_bytes {
                let current = self.enddb.size_bytes();
                if current >= target { return }
                Some((current, target))
            } else { None };
            loop {
                if let Some((ref d, stop_on_write_err)) = cache_dir {
                    let res = self.build_slice_cached(game, d);
                    if let Ok(false) = res { return }   // no more slices?
                    if stop_on_write_err { res.unwrap(); }
                } else {
                    if !self.build_slice(game) { return }
                }
                if let Some((ref mut current, target)) = sizes {
                    *current += self.enddb.slices.last().unwrap().size_bytes();
                    if *current >= target { return }
                }
            }
        }
    };
}

impl<SlicesProvider, SliceBuilder, NimberChecker, G, ISP, US> EndDbBuilderForSimpleGame<G> for EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, SliceBuilder::CompressedSlice>
    where G: SimpleGame + fmt::Display,
          SlicesProvider: EndDbSlicesProvider<Game=G, InSlicePosition=ISP, UncompressedSlice=US>,
          SliceBuilder: CompressedSliceBuilder<US>,
          NimberChecker: Verifier<ISP, US>,
          //ISP: std::hash::Hash + Eq + Clone,
          US: NimbersProvider<ISP> + NimbersStorer<ISP> + Default,
        SliceBuilder::CompressedSlice: NimbersProvider<ISP>
{
    fn build_slice(&mut self, game: &G) -> bool {
        let mut current_slice = US::default();
        if let Some(slice_content) = self.enddb.slice_provider.slice_content(game, self.enddb.slices.len()) {
            for p in slice_content {
                self.enddb.add_simple_game_nimber(game, p, &mut current_slice);
            }
        } else {
            return false;
        }
        let verification_data = self.verifier.get_verification_data(&current_slice);
        self.enddb.push_slice(self.builder.construct(current_slice));
        self.verifier.check(verification_data, self.enddb.slices.last().unwrap());
        true
    }

    impl_bbenddbfor_trait_methods!(G);
}

impl<SlicesProvider, SliceBuilder, NimberChecker, G, ISP, US> EndDbBuilderForDecomposableGame<G> for EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, SliceBuilder::CompressedSlice>
    where G: DecomposableGame + fmt::Display,
          SlicesProvider: EndDbSlicesProvider<Game=G, InSlicePosition=ISP, UncompressedSlice=US>,
          SliceBuilder: CompressedSliceBuilder<US>,
          NimberChecker: Verifier<ISP, US>,
          //ISP: std::hash::Hash + Eq + Clone,
          US: NimbersProvider<ISP> + NimbersStorer<ISP> + Default,
          SliceBuilder::CompressedSlice: NimbersProvider<ISP>
{
    fn build_slice(&mut self, game: &G) -> bool {
        let mut current_slice = US::default();
        if let Some(slice_content) = self.enddb.slice_provider.slice_content(game, self.enddb.slices.len()) {
            for p in slice_content {
                self.enddb.add_decomposable_game_nimber(game, p, &mut current_slice);
            }
        } else {
            return false;
        }
        let verification_data = self.verifier.get_verification_data(&current_slice);
        self.enddb.push_slice(self.builder.construct(current_slice));
        self.verifier.check(verification_data, self.enddb.slices.last().unwrap());
        true
    }

    impl_bbenddbfor_trait_methods!(G);
}

impl<SlicesProvider, SliceBuilder, NimberChecker, ISP, US> EndDbBuilder<SlicesProvider, SliceBuilder, NimberChecker, SliceBuilder::CompressedSlice>
    where SlicesProvider: EndDbSlicesProvider<InSlicePosition=ISP, UncompressedSlice=US>,
          SliceBuilder: CompressedSliceBuilder<US>
{
    /// Writes `slice_index`-th slice to the given `output`.
    pub fn write_slice(&self, output: &mut dyn io::Write, slice_index: usize) -> io::Result<()> {
        self.enddb.slices[slice_index].write(output)
    }

    /// Writes `slice_index`-th slice to the file with given name (`filename`).
    pub fn write_slice_to_file<P: AsRef<std::path::Path>>(&self, filename: P, slice_index: usize) -> io::Result<()> {
        self.write_slice(&mut std::fs::File::create(filename)?, slice_index)
    }

    /// Reads a slice from the given `input` and adds it to the end of `self.slices`.
    pub fn read_slice(&mut self, input: &mut dyn io::Read) -> io::Result<()> {
        self.enddb.push_slice(self.builder.read(input)?);
        Ok(())
    }

    /// Reads a slice from file with given name (`filename`) and adds it to the end of `self.slices`.
    pub fn read_slice_from_file<P: AsRef<std::path::Path>>(&mut self, filename: P) -> io::Result<()> {
        self.read_slice(&mut std::fs::File::open(filename)?)
    }
}
