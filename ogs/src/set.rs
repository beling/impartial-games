/// Set of nimbers.
/// 
/// Implemented by `u64` slices.
pub trait BitSet {
    /// Returns the smallest nimber not included in the `self` set.
    fn mex(&self) -> u16;

    unsafe fn set_nimber_unchecked(&mut self, nimber: u16);
    fn add_nimber(&mut self, nimber: u16);
    fn contain_nimber(&self, nimber: u16) -> bool;

    unsafe fn set_bit_unchecked(&mut self, bit_nr: usize);
    fn set_bit(&mut self, bit_nr: usize);
    fn get_bit(&self, bit_nr: usize) -> bool;
    fn try_get_bit(&self, bit_nr: usize) -> Option<bool>;

    /// Returns the index of the most significant bit set.
    fn msb_index(&self) -> Option<u16>;
}

/// Implemented by `Vec<u64>`.
pub trait SetConstructor {
    /// Returns set to which nimbers from `0` to `max_nimber` can be inserted.
    fn with_max_nimber(max_nimber: u16) -> Self;
}

impl BitSet for [u64] {
    fn mex(&self) -> u16 { 
        let mut result = 0;
        for v in self {
            if *v == u64::MAX {
                result += 64;
            } else {
                return result + v.trailing_ones() as u16;
            }
        }
        result
    }

    #[inline] unsafe fn set_nimber_unchecked(&mut self, nimber: u16) {
        *self.get_unchecked_mut((nimber/64) as usize) = 1u64 << (nimber % 64) as u64;
    }
    
    #[inline] fn add_nimber(&mut self, nimber: u16) {
        self[(nimber/64) as usize] |= 1u64 << (nimber % 64) as u64;
    }
    
    #[inline] fn contain_nimber(&self, nimber: u16) -> bool {
        self[(nimber/64) as usize] & (1u64 << (nimber % 64)) != 0
    }

    unsafe fn set_bit_unchecked(&mut self, bit_nr: usize) {
        *self.get_unchecked_mut(bit_nr/64) = 1u64 << (bit_nr % 64) as u64;
    }

    fn set_bit(&mut self, bit_nr: usize) {
        self[bit_nr/64] |= 1u64 << (bit_nr % 64) as u64;
    }

    fn get_bit(&self, bit_nr: usize) -> bool {
        self[bit_nr/64] & (1u64 << (bit_nr % 64)) != 0
    }

    fn try_get_bit(&self, bit_nr: usize) -> Option<bool> {
        Some(self.get(bit_nr/64)? & (1u64 << (bit_nr % 64) as u64) != 0)
    }
    
    fn msb_index(&self) -> Option<u16> {
        for (i, v) in self.iter().copied().enumerate().rev() {
            if v != 0 { return Some(64*(i as u16+1) - v.leading_zeros() as u16-1); }
        }
        None
    }
}

impl SetConstructor for Vec<u64> {
    fn with_max_nimber(max_nimber: u16) -> Self {
        vec![0; max_nimber as usize / 64 + 1]
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_mex() {
        assert_eq!(vec![0u64].mex(), 0);
        assert_eq!(vec![0b1011u64].mex(), 2);
        assert_eq!(vec![u64::MAX].mex(), 64);
        assert_eq!(vec![u64::MAX, 0b1011u64].mex(), 66);
    }

    #[test]
    fn test_nimber_set() {
        let mut s: Vec<u64> = Vec::with_max_nimber(64);     // insert_nimber needs mut
        for i in 0..=64 { s.add_nimber(i); }
        assert_eq!(s.mex(), 65);
    }

    #[test]
    fn test_msb_index() {
        assert_eq!(vec![].msb_index(), None);
        assert_eq!(vec![0, 0, 0].msb_index(), None);
        assert_eq!(vec![1, 0, 0].msb_index(), Some(0));
        assert_eq!(vec![u64::MAX, 0, 0].msb_index(), Some(63));
        assert_eq!(vec![123u64, 1].msb_index(), Some(64));
        assert_eq!(vec![123u64, 2, 0].msb_index(), Some(65));
    }
}