use csf::utils::ceiling_div;

#[cfg(target_arch = "x86")]
use std::arch::x86::{__m128i, _mm_setzero_si128};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{__m128i, _mm_setzero_si128};

use std::os::raw::c_int;
use arrayvec::ArrayVec;

use csf::bits_to_store;
use superslice::*;
use std::mem::MaybeUninit;

use co_sort::*;
use std::collections::HashMap;

#[allow(improper_ctypes)]
#[link(name = "simdcomp")]
extern {
    fn simdmaxbitsd1(initvalue: u32, input: *const u32) -> u32;
    fn simdpackwithoutmaskd1(initvalue: u32, input: *const u32, output: *mut __m128i, bit: u32);
    fn simdpack_shortlength(input: *const u32, length: c_int, out: *mut __m128i, bit: u32) -> *mut __m128i;
    fn simdpackwithoutmask(input: *const u32, output: *mut __m128i, bit: u32);
    fn simdsearchd1(initOffset: *mut __m128i, input: *const __m128i, bit: u32, key: u32, presult: *mut u32) -> c_int;
    fn simdselectFOR(initvalue: u32, input: *const __m128i, bit: u32, slot: c_int) -> u32;
}

pub struct ClusterBP128 {
    block_second_positions: Box<[u32]>,
    blocks_metadata: Box<[u32]>,
    blocks_nimbers: Box<[u8]>,
    data: Box<[__m128i]>
}

impl ClusterBP128 {
    const NIMBERS_IN_BLOCK: usize = 128 * 2 + 2;   // 128*2 binary packed + 2 in meta-data

    fn metadata(data_index: u32, bits_per_entry: u8, max_nimber: u8) -> u32 {
        assert!(data_index < (1u32 << 23));
        assert!(bits_per_entry < (1u8 << 5));
        (data_index << 9) | ((bits_per_entry as u32) << 4) | (max_nimber as u32)
    }

    /// Returns: data_index, bits_per_entry, max_nimber
    fn get_metadata(&self, block_index: usize) -> (u32, u8, u8) {
        let rest = self.blocks_metadata[block_index];
        let max_nimber = (rest & 15) as u8;
        let bits_per_entry = ((rest >> 4) & ((1<<5)-1)) as u8;
        return (rest >> 9, bits_per_entry, max_nimber);
    }

    fn set_nimber_of_first(block_nimber: &mut u8, nimber_to_set: u8) {
        debug_assert!(nimber_to_set < 16);
        *block_nimber |= nimber_to_set;
    }

    fn get_nimber_of_first(block_nimber: u8) -> u8 { block_nimber & 15 }

    fn set_nimber_of_second(block_nimber: &mut u8, nimber_to_set: u8) {
        debug_assert!(nimber_to_set < 16);
        *block_nimber |= nimber_to_set << 4;
    }

    fn get_nimber_of_second(block_nimber: u8) -> u8 { block_nimber >> 4 }

    fn bits_to_store_2_nimbers(max_nimber: u8) -> u8 {
        return bits_to_store!(max_nimber*(max_nimber+2));
    }

    fn prefilter(positions: &[u32], nimbers: &[u8]) -> (Vec<u32>, Vec<u8>) {
        assert_eq!(positions.len(), nimbers.len());
        let mut pre_filtered_positions = Vec::with_capacity(positions.len());
        let mut pre_filtered_nimbers = Vec::with_capacity(nimbers.len());
        {
            let mut i = 0usize;
            while i+1 < positions.len() {
                if nimbers[i] != nimbers[i+1] {
                    pre_filtered_positions.push(positions[i]);
                    pre_filtered_nimbers.push(nimbers[i]);
                    i += 1;
                    pre_filtered_positions.push(positions[i]); // this can also append the last element
                    pre_filtered_nimbers.push(nimbers[i]);
                }
                i += 1;
            }
            if i < positions.len() {
                pre_filtered_positions.push(*positions.last().unwrap());
                pre_filtered_nimbers.push(*nimbers.last().unwrap());
            }
        }
        //CRAM_LOG("pre-filtering: " << uncompressed_whole.size() << " -> " << uncompressed.size() <<
        //" (" << div_rounded(uncompressed.size()*1000, uncompressed_whole.size()) << "‰)")
        (pre_filtered_positions, pre_filtered_nimbers)
    }

    fn from_prefiltered(positions: &[u32], nimbers: &[u8]) -> Self {
        let blocks_count = ceiling_div(nimbers.len(), Self::NIMBERS_IN_BLOCK);
        let mut block_second_positions = vec![0u32; blocks_count].into_boxed_slice();
        let mut blocks_metadata = vec![0u32; blocks_count].into_boxed_slice();
        let mut blocks_nimbers = vec![0u8; blocks_count].into_boxed_slice();
        let mut data: Option<Box<[__m128i]>> = None;

        for &store_phase in [false, true].iter() { // if store_phase == false, the only goal is to calculate data_size
            let mut data_size = 0usize;
            for block_i in 0..blocks_count {
                let last_block = block_i + 1 == blocks_count;
                let idx_beg = block_i * Self::NIMBERS_IN_BLOCK;
                let (idx_end, size) = if last_block {
                    (nimbers.len(), nimbers.len() - idx_beg)
                } else {
                    (idx_beg + Self::NIMBERS_IN_BLOCK, Self::NIMBERS_IN_BLOCK)
                };
                //assert(uncomp_size > 0);
                if store_phase { Self::set_nimber_of_first(&mut blocks_nimbers[block_i], nimbers[idx_beg]); }
                if size == 1 { // we have exactly one position in this block:
                    assert!(last_block); // which is possible only in the last block
                    if store_phase {
                        blocks_metadata[block_i] = Self::metadata(data_size as _, 0, 0); // needed for calculating data_size only
                        block_second_positions[block_i] = positions[idx_beg].checked_add(1).unwrap();
                    }
                    break;  // we do not need the seond nimber in the block, as the only operation is reading the first one
                }
                if store_phase {
                    block_second_positions[block_i] = positions[idx_beg + 1];
                    Self::set_nimber_of_second(&mut blocks_nimbers[block_i], nimbers[idx_beg + 1]);
                }
                if size == 2 { // we have exactly two position in this block:
                    debug_assert!(last_block); // which is possible only in the last block
                    if store_phase { blocks_metadata[block_i] = Self::metadata(data_size as _, 0, 0); } // needed for calculating data_size only
                    break;  // we do not need any data, as the only operation is reading first and second nimber of this block
                }
                let max_nimber = *nimbers[idx_beg + 2..idx_end].iter().max().unwrap();
                //let to_compress = Vec::<u32>::with_capacity(128);
                //let nimbers_to_compress = [0u32; 128]; // i-th entry stores a pair of nimbers, for to_compress[i] and prev. pos.
                let mut to_compress = ArrayVec::<[u32; 128]>::new();
                let mut nimbers_to_compress = ArrayVec::<[u32; 128]>::new();
                //let to_compress_size = 0; // where to store next value and nimbers in to_compress
                let mut uncompress_i = 2usize;   // index of next value of uncomp_beg to be compressed
                let mut recent_value_compressed = 0u32;  // can also be uninitialized
                /*auto
                add_to_compress = [&](Cram::Position position, Nimber nimber_of_prev, Nimber nimber_of_position) {
                    recent_value_compressed = position - uncomp_beg[1].position();
                    to_compress[to_compress_size] = ensure_compressible(recent_value_compressed);
                    nimbers_to_compress[to_compress_size] = nimber_of_position * (max_nimber + 1) + nimber_of_prev;
                    to_compress_size + +;
                };*/
                while uncompress_i + 1 < size {    // we add pairs of uncompress_i, uncompress_i+1
                    let p = idx_beg + uncompress_i + 1;
                    //add_to_compress(p.position(), uncomp_beg[uncompress_i].nimber(), p.nimber()):
                    recent_value_compressed = positions[p] - positions[idx_beg + 1];
                    to_compress.push(recent_value_compressed);
                    nimbers_to_compress.push(nimbers[p] as u32 * (max_nimber + 1) as u32  + nimbers[p-1] as u32);
                    uncompress_i += 2;
                }
                if uncompress_i < size {   // uncompress vector has odd size, possible only in the last block
                    //add_to_compress(uncompressed.back().position(), n, n);
                    recent_value_compressed = positions.last().unwrap() - positions[idx_beg + 1];
                    to_compress.push(recent_value_compressed);
                    let n = *nimbers.last().unwrap() as u32;
                    nimbers_to_compress.push(n * (max_nimber + 1) as u32 + n);
                    //uncompress_i += 2;
                }
                let to_compress_real_size = to_compress.len();
                while to_compress.len() < 128 {  // fill the rest of buffer with some garbages with delta=1, possible only in the last block
                    recent_value_compressed = recent_value_compressed.checked_add(1).unwrap();
                    to_compress.push(recent_value_compressed);
                    nimbers_to_compress.push(0);
                }
                let positions_bits = unsafe { simdmaxbitsd1(0, to_compress.as_ptr()) };
                //dbg!(&to_compress); dbg!(positions_bits); dbg!(&positions[idx_beg..idx_end]);
                let nimbers_bits = Self::bits_to_store_2_nimbers(max_nimber);
                if store_phase {
                    blocks_metadata[block_i] = Self::metadata(data_size as _, positions_bits as _, max_nimber);
                    unsafe { simdpackwithoutmaskd1(0, to_compress.as_mut_ptr(), data.as_mut().unwrap().as_mut_ptr().offset(data_size as _), positions_bits) };
                    let nimbers_begin = data.as_mut().unwrap().as_mut_ptr().wrapping_offset(data_size as isize + positions_bits as isize);
                    if to_compress.len() != to_compress_real_size {
                        assert!(last_block); // possible in the last block
                        let nimbers_end = unsafe {
                            simdpack_shortlength(nimbers_to_compress.as_ptr(), to_compress_real_size as _, nimbers_begin, nimbers_bits as _)
                        };
                        let compressed = unsafe { nimbers_end.offset_from(nimbers_begin) };
                        if compressed != ceiling_div(nimbers_bits as usize * to_compress_real_size, 128) as _ {
                            panic!("Error while saving {} nimbers pair, each on {} bits: {} != {}",
                                   to_compress_real_size, nimbers_bits, compressed, ceiling_div(nimbers_bits as usize * to_compress_real_size, 128));
                        }
                    } else {
                        unsafe { simdpackwithoutmask(nimbers_to_compress.as_ptr(), nimbers_begin, nimbers_bits as _); }
                    }
                }
                data_size += positions_bits as usize + if last_block {
                    ceiling_div(nimbers_bits as usize * to_compress_real_size, 128)
                } else {
                    nimbers_bits as usize
                };
            }
            /*if store_phase {
                // finish, return lenght of data array
                /*if (data_size != this->data_size())
                CRAM_PANIC("Wrong data size: " << data_size << " != " << this->data_size());*/
                // print some statistics:
                const std
                ::size_t
                meta_size = blocks_metadata_size_bytes();
                const std
                ::size_t
                total_size = data_size * 16 + meta_size;
                CRAM_LOG(data_size * 16 << "+" << meta_size << " = " << total_size
                    << " bytes  " << div_rounded(meta_size * 1000, total_size) << "‰ meta-data  "
                    << frac(total_size * 8, uncompressed_whole.size()) << " bits/el");

                return data_size;
            } else*/
            //data.reserve_exact(data_size);
            if !store_phase {
                data = Some(vec![unsafe { _mm_setzero_si128() }; data_size].into_boxed_slice());
            }
        }
        Self {
            block_second_positions,
            blocks_metadata,
            blocks_nimbers,
            data: data.unwrap()
        }
    }

    #[inline]
    pub fn from_sorted(positions: &[u32], nimbers: &[u8]) -> Self {
        let (positions, nimbers) = Self::prefilter(positions, nimbers);
        Self::from_prefiltered(&positions, &nimbers)
    }

    #[inline]
    pub fn from_unsorted(positions: &mut [u32], nimbers: &mut [u8]) -> Self {
        co_sort!(positions, nimbers);
        Self::from_sorted(positions, nimbers)
    }

    pub fn get(&self, p: u32) -> u8 {
        //let mut second_p_in_block = self.block_second_positions.partition_point(|e| *e <= p); // OK but unstable
        let mut block_index = self.block_second_positions.upper_bound(&p);
        if block_index == 0 { // value lower than the second value in the cluster
            return Self::get_nimber_of_first(self.blocks_nimbers[0]);
        }
        block_index -= 1;    // now p >= second_p_in_block
        let key = p - self.block_second_positions[block_index];

        if key == 0 {   // we ask exactly for the second value in the block?
            return Self::get_nimber_of_second(self.blocks_nimbers[block_index]);
        }

        let (data_index, bits_per_entry, mut max_nimber) = self.get_metadata(block_index);

        let mut found_key = MaybeUninit::<u32>::uninit();
        let mut offset = unsafe{_mm_setzero_si128()};
        let data_block_begin = self.data.as_ptr().wrapping_offset(data_index as isize);
        let in_block_index = unsafe{simdsearchd1(&mut offset, data_block_begin, bits_per_entry as u32, key, found_key.as_mut_ptr())};
        if in_block_index == 128 {  // we ask for the first value in the next block?
            return Self::get_nimber_of_first(self.blocks_nimbers[block_index + 1]);
        }
        let nimbers_pair = unsafe {simdselectFOR(0,
                                                 data_block_begin.wrapping_offset(bits_per_entry as isize),
                                                 Self::bits_to_store_2_nimbers(max_nimber) as u32,
                                                 in_block_index)} as u8;
        // here key <= found_key
        max_nimber += 1;
        if unsafe {found_key.assume_init()} == key {
            nimbers_pair / max_nimber // key nimber is at higher part
        } else {
            nimbers_pair % max_nimber
        }
    }

    pub fn size_bytes(&self) -> usize {
        self.block_second_positions.len() * std::mem::size_of::<u32>() +
            self.blocks_metadata.len() * std::mem::size_of::<u32>() +
            self.blocks_nimbers.len() * std::mem::size_of::<u8>() +
            self.data.len() * std::mem::size_of::<__m128i>() +
            std::mem::size_of_val(self)
    }
}



fn iter_to_vecs<'a, 'b, M: Iterator<Item=(&'a u32, &'b u8)>>(map: M, len: usize) -> (Vec<u32>, Vec<u8>) {
    let mut positions = Vec::with_capacity(len);
    let mut nimbers = Vec::with_capacity(len);
    for (p, n) in map {
        positions.push(*p);
        nimbers.push(*n);
    };
    (positions, nimbers)
}

#[inline]
fn map_to_vecs<S>(map: &HashMap<u32, u8, S>) -> (Vec<u32>, Vec<u8>) {
    iter_to_vecs(map.iter(), map.len())
}

impl<S> From<&HashMap<u32, u8, S>> for ClusterBP128 {
    fn from(map: &HashMap<u32, u8, S>) -> Self {
        let (mut positions, mut nimbers) = map_to_vecs(map);
        Self::from_unsorted(&mut positions, &mut nimbers)
    }
}

impl<S> From<HashMap<u32, u8, S>> for ClusterBP128 {
    fn from(map: HashMap<u32, u8, S>) -> Self {
        let (mut positions, mut nimbers) = map_to_vecs(&map);
        drop(map);
        Self::from_unsorted(&mut positions, &mut nimbers)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use maplit::hashmap;

    #[test]
    fn with_hashmap1() {
        let map = ClusterBP128::from(hashmap!(1=>2u8));
        assert_eq!(map.get(1), 2);
    }

    #[test]
    fn with_hashmap2() {
        let map = ClusterBP128::from(hashmap!(1=>2u8, 3=>6u8));
        assert_eq!(map.get(1), 2);
        assert_eq!(map.get(3), 6);
    }

    #[test]
    fn with_hashmap3() {
        let map = ClusterBP128::from(hashmap!(123=>1u8, 34=>2u8, 5=>1u8));
        assert_eq!(map.get(123), 1);
        assert_eq!(map.get(34), 2);
        assert_eq!(map.get(5), 1);
    }

    #[test]
    fn with_hashmap130() {
        let mut m = HashMap::new();
        for i in 0u8..130u8 { m.insert(2*(i as u32), i/10); }
        let map = ClusterBP128::from(m);
        for i in 0u8..130u8 { assert_eq!(map.get(2*(i as u32)), i/10); }
    }
}