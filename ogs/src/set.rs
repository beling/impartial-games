/// Set of nimbers.
/// 
/// Implemented by `u64` slices.
pub trait BitSet {
    /// Returns the smallest nimber not included in the `self` set.
    fn mex(&self) -> u16;

    unsafe fn set_nimber_unchecked(&mut self, nimber: u16);
    fn set_nimber(&mut self, nimber: u16);
    fn get_nimber(&self, nimber: u16) -> bool;
    fn try_get_nimber(&self, bit_nr: u16) -> Option<bool>;

    unsafe fn set_bit_unchecked(&mut self, bit_nr: usize);
    fn set_bit(&mut self, bit_nr: usize);
    fn get_bit(&self, bit_nr: usize) -> bool;
    fn try_get_bit(&self, bit_nr: usize) -> Option<bool>;
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
    
    #[inline] fn set_nimber(&mut self, nimber: u16) {
        self[(nimber/64) as usize] |= 1u64 << (nimber % 64) as u64;
    }
    
    #[inline] fn get_nimber(&self, nimber: u16) -> bool {
        self[(nimber/64) as usize] & (1u64 << (nimber % 64)) != 0
    }

    #[inline(always)] fn try_get_nimber(&self, nimber: u16) -> Option<bool> {
        Some(self.get((nimber/64) as usize)? & (1u64 << (nimber % 64) as u64) != 0)
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
        for i in 0..=64 { s.set_nimber(i); }
        assert_eq!(s.mex(), 65);
    }
}