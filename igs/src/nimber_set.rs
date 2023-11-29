use bitm::n_lowest_bits;
use crate::bit::ExtraBitMethods;

pub trait WithLowest {
    /// Construct the set which includes the `n` lowest nimbers, i.e. *{0, 1, ..., n-1}*.
    fn with_lowest(n: u16) -> Self;
}

pub trait NimberSet: Sized + WithLowest {

    type Extended: ExtendendNimberSet<Self>;

    /// Construct empty set of nimbers.
    fn empty() -> Self;

    fn singleton(only_element: u8) -> Self {
        let mut result = Self::empty();
        result.append(only_element);
        result
    }

    /// Append nimber to self.
    fn append(&mut self, nimber: u8);

    /// Append nimber from self.
    fn remove(&mut self, nimber: u8);

    /// Check if self includes nimber.
    fn includes(&self, nimber: u8) -> bool;

    /// Minimal nimber not included in the set.
    fn mex(&self) -> u8;

    /// Returns the intersection of self and other.
    fn intersected_with(&self, other: &Self) -> Self;

    /// Construct the set which includes all the nimbers upto `n`, i.e. *{0, 1, ..., n}*, where `n` is the largest element of self.
    fn upto_largest(&self) -> Self;

    /// Return (bit-)set consisted of all values from self, each xored with nimber.
    fn each_xored_with(&self, nimber: u8) -> Self;
}

pub trait ExtendendNimberSet<NimberSet>: WithLowest {

    /// Copy of self without the largest element.
    fn without_largest(&self) -> NimberSet;

    /**
     * Remove exactly one nimber from the set:
     * either a given nimber v (if it is in the set) or the largest nimber.
     */
    fn remove_nimber(&mut self, nimber: u8);

    /**
     * Remove exactly one nimber from the set:
     * either a given nimber v (if it is in the set) or the largest nimber.
     * (without_largest is passed for optimization)
    */
    fn remove_nimber_hinted(&mut self, nimber: u8, without_largest: &NimberSet);

    /// Remove the largest nimber from the set.
    /// (without_largest is passed for optimization)
    fn remove_largest_hinted(&mut self, without_largest: &NimberSet);

    /// Get the only (or any) element from the set.
    fn only_element(&self) -> u8;

    /// Check whether self is distinct from other.
    fn is_distinct_from(&self, other: &NimberSet) -> bool;
}

macro_rules! impl_nimber_sets_for_primitive {
    ($ext_type:ident extends $type:ty) => {
        impl WithLowest for $type {
            #[inline(always)] fn with_lowest(n: u16) -> Self {
                Self::with_lowest_bits_saturated(n as _)
            }
        }

        impl NimberSet for $type {
            type Extended = $ext_type;
            #[inline(always)] fn empty() -> Self { 0 }
            #[inline(always)] fn singleton(only_element: u8) -> Self { (1 as Self) << only_element }
            #[inline(always)] fn append(&mut self, nimber: u8) { *self |= (1 as Self) << nimber; }
            #[inline(always)] fn remove(&mut self, nimber: u8) { *self &= !((1 as Self) << nimber); }
            #[inline(always)] fn includes(&self, nimber: u8) -> bool { (*self & ((1 as Self) << nimber)) != 0 }
            #[inline(always)] fn mex(&self) -> u8 { (!*self).trailing_zeros() as u8 }
            #[inline(always)] fn intersected_with(&self, other: &Self) -> Self { *self & *other }
            //#[inline(always)] fn is_distinct_from(&self, other: &Self) -> bool { *self & *other == 0 }
            #[inline(always)] fn upto_largest(&self) -> Self { self.upto_leading_one() }
            //#[inline(always)] fn without_largest(&self) -> Self { *self & (*self>>1).upto_leading_one() }
            fn each_xored_with(&self, nimber: u8) -> Self {
                if nimber == 0 { return *self; }  // very common case
                let mut result = 0;
                let mut src = *self;
                while src != 0 {
                    let nimber_from_src = src.trailing_zeros() as u8;
                    result ^= (1 as Self) << (nimber_from_src ^ nimber);
                    src ^= (1 as Self) << nimber_from_src;
                }
                result
            }
        }

        pub struct $ext_type {
            /// subset of the lowest nimbers
            details: $type,

            /// number of nimbers larger than 8*sizeof(details)
            bigger_count: u16
        }

        impl WithLowest for $ext_type {
            #[inline(always)]
            fn with_lowest(n: u16) -> Self {
                if let Some(details) = <$type>::with_lowest_bits_checked(n as _) {
                    Self{ details, bigger_count: 0 }
                } else {
                    Self{ details: <$type>::MAX, bigger_count: n - std::mem::size_of::<$type>() as u16*8 }
                }


                /*if n >= std::mem::size_of::<$type>() as u16*8 {
                    Self{ details: <$type>::MAX, bigger_count: n - std::mem::size_of::<$type>() as u16*8 }
                } else {
                    Self{ details: <$type>::with_lowest_bits(n), bigger_count: 0 }
                }*/
            }
        }

        impl ExtendendNimberSet<$type> for $ext_type {

            #[inline(always)]
            fn without_largest(&self) -> $type {
                if self.bigger_count != 0 {
                    self.details
                } else {
                    self.details.without_leading_one()
                }
            }

            #[inline(always)]
            fn remove_nimber(&mut self, nimber: u8) {
                //assert_dbg!(v < size_of::<$type>()*8);
                let v_bit = (1 as $type) << nimber;
                if self.details & v_bit != 0 {
                    self.details ^= v_bit;
                } else {
                    if self.bigger_count != 0 {
                        self.bigger_count -= 1;
                    } else {
                        self.details = self.details.without_leading_one();
                    }
                }
            }

            #[inline(always)]
            fn remove_nimber_hinted(&mut self, nimber: u8, without_largest: &$type) {
                //assert_dbg!(v < size_of::<$type>()*8);
                let v_bit = (1 as $type) << nimber;
                if self.details & v_bit != 0 {
                    self.details ^= v_bit;
                } else {
                    self.remove_largest_hinted(without_largest);
                }
            }

            #[inline(always)]
            fn remove_largest_hinted(&mut self, without_largest: &$type) {
                if self.bigger_count != 0 {
                    self.bigger_count -= 1;
                } else {
                    self.details = *without_largest;
                }
            }

            #[inline(always)]
            fn only_element(&self) -> u8 {
                self.details.trailing_zeros() as _
            }

            #[inline(always)]
            fn is_distinct_from(&self, other: &$type) -> bool {
                self.details & other == 0
            }
        }
    }
}

impl_nimber_sets_for_primitive!(ExtendU32NimberSet extends u32);
impl_nimber_sets_for_primitive!(ExtendU64NimberSet extends u64);
impl_nimber_sets_for_primitive!(ExtendU128NimberSet extends u128);

impl NimberSet for [u64; 4] {

    type Extended = [u64; 4];

    /*fn with_lowest(n: u8) -> Self {
        if n < 64 { [n_lowest_bits(n), 0, 0, 0] }
        else if n < 128 { [u64::MAX, n_lowest_bits(n-64), 0, 0] }
        else if n < 192 { [u64::MAX, u64::MAX, n_lowest_bits(n-128), 0] }
        else { [u64::MAX, u64::MAX, u64::MAX, n_lowest_bits(n-192)] }
    }*/

    #[inline(always)]
    fn empty() -> Self {
        [0, 0, 0, 0]
    }

    #[inline(always)]
    fn append(&mut self, nimber: u8) {
        self[(nimber / 64) as usize] |= 1u64 << (nimber % 64) as u64;
    }

    #[inline(always)]
    fn remove(&mut self, nimber: u8) {
        self[(nimber / 64) as usize] &= !(1u64 << (nimber % 64) as u64);
    }

    #[inline(always)]
    fn includes(&self, nimber: u8) -> bool {
        self[(nimber / 64) as usize] & (1u64 << (nimber % 64) as u64) != 0
    }

    fn mex(&self) -> u8 {
        if self[0] != u64::MAX { self[0].mex() }
        else if self[1] != u64::MAX { self[1].mex() + 64 }
        else if self[2] != u64::MAX { self[2].mex() + 128 }
        else { self[3].mex() + 192 }
    }

    #[inline(always)]
    fn intersected_with(&self, other: &Self) -> Self {
        [self[0] & other[0], self[1] & other[1], self[2] & other[2], self[3] & other[3]]
    }

    fn upto_largest(&self) -> Self {
        if self[3] != 0 { [u64::MAX, u64::MAX, u64::MAX, self[3].upto_leading_one()] }
        else if self[2] != 0 { [u64::MAX, u64::MAX, self[2].upto_leading_one(), 0] }
        else if self[1] != 0 { [u64::MAX, self[1].upto_leading_one(), 0, 0] }
        else { [self[0].upto_leading_one(), 0, 0, 0] }
    }

    fn each_xored_with(&self, nimber: u8) -> Self {
        if nimber == 0 { return *self; }  // very common case
        let mut result = Self::empty();
        let mut shift = 0;
        for segment in self {
            let mut src = *segment;
            while src != 0 {
                let nimber_from_src = src.trailing_zeros() as u8;
                result.append((nimber_from_src+shift) ^ nimber);
                src ^= 1u64 << nimber_from_src;
            }
            shift = shift.wrapping_add(64);
        }
        result
    }

}

impl WithLowest for [u64; 4] {
    #[inline(always)]
    fn with_lowest(n: u16) -> Self {
        if n < 64 { [n_lowest_bits(n as _), 0, 0, 0] }
        else if n < 128 { [u64::MAX, n_lowest_bits(n as u8 - 64), 0, 0] }
        else if n < 192 { [u64::MAX, u64::MAX, n_lowest_bits(n as u8 - 128), 0] }
        else if n < 256 { [u64::MAX, u64::MAX, u64::MAX, n_lowest_bits(n as u8 - 192)] }
        else { [u64::MAX, u64::MAX, u64::MAX, u64::MAX] }
    }
}

impl ExtendendNimberSet<[u64; 4]> for [u64; 4] {

    fn without_largest(&self) -> Self {
        if self[3] != 0 { [self[0], self[1], self[2], self[3].without_leading_one()] }
        else if self[2] != 0 { [self[0], self[1], self[2].without_leading_one(), 0] }
        else if self[1] != 0 { [self[0], self[1].without_leading_one(), 0, 0] }
        else { [self[0].without_leading_one(), 0, 0, 0] }
    }

    fn remove_nimber(&mut self, nimber: u8) {
        let index = (nimber / 64) as usize;
        let mask = 1u64 << (nimber % 64);
        if self[index] & mask != 0 {
            self[index] ^= mask;
        } else {    // remove largest:
            if self[3] != 0 { self[3].clear_leading_one(); }
            else if self[2] != 0 { self[2].clear_leading_one(); }
            else if self[1] != 0 { self[1].clear_leading_one(); }
            else { self[0].clear_leading_one(); }
        }
    }

    fn remove_nimber_hinted(&mut self, nimber: u8, without_largest: &[u64; 4]) {
        let index = (nimber / 64) as usize;
        let mask = 1u64 << (nimber % 64);
        if self[index] & mask != 0 {
            self[index] ^= mask;
        } else {
            self.remove_largest_hinted(without_largest);
        }
    }

    #[inline(always)]
    fn remove_largest_hinted(&mut self, without_largest: &[u64; 4]) {
        *self = *without_largest;
    }

    fn only_element(&self) -> u8 {
        if self[0] != 0 { self[0].trailing_zeros() as u8 }
        else if self[1] != 0 { self[1].trailing_zeros() as u8 + 64 }
        else if self[2] != 0 { self[2].trailing_zeros() as u8 + 128 }
        else { self[3].trailing_zeros() as u8 + 192 }
    }

    #[inline(always)]
    fn is_distinct_from(&self, other: &Self) -> bool {
        (self[0] & other[0]) == 0 && (self[1] & other[1]) == 0 &&
            (self[2] & other[2]) == 0 && (self[3] & other[3]) == 0
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32() {
        assert_eq!(u32::empty(), 0);
        assert_eq!(u32::with_lowest_bits(0), 0);
        assert_eq!(u32::with_lowest_bits(3), 0b111);
        assert_eq!(u32::with_lowest_bits_saturated(0), 0);
        assert_eq!(u32::with_lowest_bits_saturated(3), 0b111);
        assert_eq!(u32::with_lowest_bits_saturated(32), u32::MAX);
        assert_eq!(u32::with_lowest_bits_saturated(33), u32::MAX);
        //assert_eq!(u32::with_lowest(32), u32::MAX);
        assert_eq!(0b1101u32.each_xored_with(0), 0b1101);
        assert_eq!(1u32.each_xored_with(1), 0b10);
        assert_eq!(0b11001u32.each_xored_with(1), 0b100110); //0^1=1, 3^1=2, 4^1=5
    }

    #[test]
    fn u64x4() {
        type T = [u64; 4];
        assert_eq!(T::empty(), [0, 0, 0, 0]);
        assert_eq!(T::with_lowest(0), [0, 0, 0, 0]);
        assert_eq!(T::with_lowest(3), [0b111, 0, 0, 0]);
        assert_eq!(T::with_lowest(64), [u64::MAX, 0, 0, 0]);
        let t65 = T::with_lowest(65);
        assert_eq!(t65, [u64::MAX, 0b1, 0, 0]);
        assert_eq!(t65.without_largest(), [u64::MAX, 0, 0, 0]);
        assert_eq!(t65.without_largest().without_largest(), [u64::MAX>>1, 0, 0, 0]);
        assert_eq!(T::with_lowest(64+3), [u64::MAX, 0b111, 0, 0]);
        assert_eq!(T::with_lowest(128), [u64::MAX, u64::MAX, 0, 0]);
        assert_eq!(T::with_lowest(129), [u64::MAX, u64::MAX, 1, 0]);
        assert_eq!(T::with_lowest(128+64), [u64::MAX, u64::MAX, u64::MAX, 0]);
        assert_eq!(T::with_lowest(128+64+1), [u64::MAX, u64::MAX, u64::MAX, 1]);
        assert_eq!(T::with_lowest(256), [u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
        assert_eq!(T::with_lowest(257), [u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
        //assert_eq!(u32::with_lowest(32), u32::MAX);
    }
}