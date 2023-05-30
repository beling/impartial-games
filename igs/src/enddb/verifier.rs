use std::time::{Duration, Instant};
use cpu_time::ProcessTime;
use crate::dbs::{HasLen, NimbersProvider};

use super::compressed_slice::CompressedSlice;

/// Verify if provider provides the same nimbers that are included in verification data (hash map)
pub trait Verifier<InSliceGamePosition, UncompressedSlice> {
    type VerificationData;
    fn get_verification_data(&mut self, nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData;
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, data: Self::VerificationData, provider: &P);
}

impl<InSliceGamePosition, UncompressedSlice> Verifier<InSliceGamePosition, UncompressedSlice> for () {
    type VerificationData = ();
    fn get_verification_data(&mut self, _nimbers_of_positions: &UncompressedSlice) -> Self::VerificationData { () }
    fn check<P: NimbersProvider<InSliceGamePosition> + CompressedSlice>(&mut self, _data: Self::VerificationData, _provider: &P) {}
}

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

#[derive(Default, Copy, Clone)]
pub struct PrintStats {
    total_number_of_elements: usize,
    total_size: usize,
    total_time: Duration,
    total_cpu_time: Duration
}

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