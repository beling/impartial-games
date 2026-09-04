//! Verifiers of the slices of endgame databases: they can check the correctness
//! of the built (compressed) slices or report the statistics of their construction.
use std::time::{Duration, Instant};
use cpu_time::ProcessTime;
use crate::dbs::{HasLen, NimbersProvider};

use super::compressed_slice::CompressedSlice;

/// Verify if provider provides the same nimbers that are included in verification data (hash map)
pub trait Verifier<InSliceGamePosition, UncompressedSlice> {
    /// The data collected before the slice is compressed
    /// (and used afterwards to check the compressed slice).
    type VerificationData;

    /// Collects the data needed to verify (or describe) the slice
    /// that will be constructed from the given uncompressed map of nimbers.
    fn get_verification_data(&mut self, nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData;

    /// Verifies (or describes) the given compressed `provider`,
    /// using the data collected by `get_verification_data`.
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, data: Self::VerificationData, provider: &P);
}

impl<InSliceGamePosition, UncompressedSlice> Verifier<InSliceGamePosition, UncompressedSlice> for () {
    type VerificationData = ();
    fn get_verification_data(&mut self, _nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData { () }
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, _data: Self::VerificationData, _provider: &P) {}
}

/// Verifier that checks (by asserting) whether the compressed slice provides
/// the same nimbers as the uncompressed data.
#[derive(Default, Copy, Clone)]
pub struct CheckAll {}

impl<InSliceGamePosition, UncompressedSlice> Verifier<InSliceGamePosition, UncompressedSlice> for CheckAll
    where UncompressedSlice: Clone + IntoIterator<Item=(InSliceGamePosition, u8)>
{
    type VerificationData = UncompressedSlice;
    fn get_verification_data(&mut self, nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData {
        nimbers_of_positions.clone()
    }
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, data: Self::VerificationData, provider: &P) {
        for (p, n) in data {
            assert_eq!(Some(n), provider.get_nimber(&p));
        }
    }
}

/// Verifier that does not check the slices but prints the statistics of their construction:
/// the time and CPU time consumed and the compression ratio (in bits per element).
#[derive(Default, Copy, Clone)]
pub struct PrintStats {
    /// The total number of the (checked) elements.
    total_number_of_elements: usize,
    /// The total size (in bytes) of the (checked) slices.
    total_size: usize,
    /// The total (wall-clock) time spent on checking the slices.
    total_time: Duration,
    /// The total CPU time spent on checking the slices.
    total_cpu_time: Duration
}

/// Prints `label`, `size_bytes` (in bits) and `elements`, and their ratio.
fn print_bps(label: &str, size_bytes: usize, elements: usize) {
    let size_bits = size_bytes * 8;
    if elements != 0 {
        print!("{}: {}/{} = {:.3}", label, size_bits, elements, size_bits as f64 / elements as f64);
    } else {
        print!("{}: {}/{}", label, size_bits, elements);
    }
}

impl<InSliceGamePosition, UncompressedSlice: HasLen> Verifier<InSliceGamePosition, UncompressedSlice> for PrintStats {
    type VerificationData = (usize, Instant, ProcessTime);
    fn get_verification_data(&mut self, nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData {
        (nimbers_of_positions.len(), Instant::now(), ProcessTime::now())
    }
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, (number_of_elements, time, cpu_time): Self::VerificationData, provider: &P) {
        let cpu_time = cpu_time.elapsed();
        let time = time.elapsed();
        self.total_cpu_time += cpu_time;
        self.total_time += time;
        self.total_number_of_elements += number_of_elements;
        let slice_size = provider.size_bytes();
        self.total_size += slice_size;
        println!("Time:  slice {:.2?}  total: {:.2?}  CPU slice: {:.2?}  CPU total: {:.2?}", time, self.total_time, cpu_time, self.total_cpu_time);
        print!("Size [bits/element]:");
        print_bps("  slice", slice_size, number_of_elements);
        print_bps("  total", self.total_size, self.total_number_of_elements);
        println!();
    }
}