pub trait ExtraBitMethods {
    /// Returns 0..01..1 mask with `n` ones, i.e. the largest `n`-bit number.
    fn with_lowest_bits(n: u8) -> Self;

    /// Returns 0..01..1 mask with `n` ones, or `Self::MAX` if `n` is too large.
    fn with_lowest_bits_saturated(n: u32) -> Self;

    /// Returns Some with 0..01..1 mask with n ones, or `None` if n is too large.
    fn with_lowest_bits_checked(n: u32) -> Option<Self> where Self: Sized;

    /// Returns the lowest bit of `self`.
    fn isolate_trailing_one(self) -> Self;

    /// Returns the highest bit of `self`.
    fn isolate_leading_one(self) -> Self;

    /// Returns the smallest 0..01..1 mask witch includes all ones of `self`.
    fn upto_leading_one(self) -> Self;

    /// Returns the copy of `self` without leading one.
    fn without_leading_one(self) -> Self;

    /// Clear leading one in `self`.
    fn clear_leading_one(&mut self);
}

macro_rules! impl_some_extra_bits_methods {
    () => {
        #[inline(always)] fn with_lowest_bits(n: u8) -> Self {
            ((1 as Self) << n).wrapping_sub(1)
        }

        #[inline(always)] fn with_lowest_bits_saturated(n: u32) -> Self {
            if let Some(shifted) = (1 as Self).checked_shl(n) {
                shifted.wrapping_sub(1)
            } else {
                Self::MAX
            }
        }

        #[inline(always)] fn with_lowest_bits_checked(n: u32) -> Option<Self> {
            (1 as Self).checked_shl(n).map(|v|v.wrapping_sub(1))
        }

        #[inline(always)] fn isolate_trailing_one(self) -> Self {
            self & self.wrapping_neg()
        }

        #[inline(always)] fn isolate_leading_one(self) -> Self {
            self & !(self>>1).upto_leading_one()
        }

        #[inline(always)] fn without_leading_one(self) -> Self {
            self & (self>>1).upto_leading_one()
        }

        #[inline(always)] fn clear_leading_one(&mut self) {
            *self &= (*self>>1).upto_leading_one()
        }
    }
}

impl ExtraBitMethods for u32 {
    impl_some_extra_bits_methods!();

    fn upto_leading_one(mut self) -> Self {
        self |= self >> 1;
        self |= self >> 2;
        self |= self >> 4;
        self |= self >> 8;
        self | (self >> 16)
    }
}

impl ExtraBitMethods for u64 {
    impl_some_extra_bits_methods!();

    fn upto_leading_one(mut self) -> Self {
        self |= self >> 1;
        self |= self >> 2;
        self |= self >> 4;
        self |= self >> 8;
        self |= self >> 16;
        self | (self >> 32)
    }
}

impl ExtraBitMethods for u128 {
    impl_some_extra_bits_methods!();

    fn upto_leading_one(mut self) -> Self {
        self |= self >> 1;
        self |= self >> 2;
        self |= self >> 4;
        self |= self >> 8;
        self |= self >> 16;
        self |= self >> 32;
        self | (self >> 64)
    }
}

/// Returns the word with the isolated least significant bit set in `x`.
#[inline(always)]
pub const fn lowest_bit_of(x: u64) -> u64 { x & x.wrapping_neg() }

pub const fn repeat_bit_sequence(mut sequence: u64, sequence_width: u8, mut how_many_times: u8) -> u64 {
    // TODO lepszy algorytm jak potęgowanie
    while how_many_times > 1 {
        sequence = (sequence << sequence_width) | sequence;
        how_many_times -= 1;
    }
    sequence
}

#[cfg(test)]
mod tests {
    use bitm::n_lowest_bits;
    use super::*;

    #[test]
    fn bit() {
        assert_eq!(n_lowest_bits(0), 0);
        assert_eq!(n_lowest_bits(3), 0b111);

        assert_eq!(lowest_bit_of(0b10110), 0b10);
        assert_eq!(lowest_bit_of(0b10011), 1);
    }

    #[test]
    fn extra_bit_methods_of_u64() {
        assert_eq!(u64::with_lowest_bits(0), 0);
        assert_eq!(u64::with_lowest_bits(3), 0b111);

        assert_eq!(0b10110u64.isolate_trailing_one(), 0b10);
        assert_eq!(0b10011u64.isolate_trailing_one(), 1);

        assert_eq!(0u64.upto_leading_one(), 0);
        assert_eq!(1u64.upto_leading_one(), 1);
        assert_eq!(0b10110u64.upto_leading_one(), 0b11111);
        assert_eq!(u64::MAX.upto_leading_one(), u64::MAX);

        assert_eq!(1u64.without_leading_one(), 0);
        assert_eq!(0b10110u64.without_leading_one(), 0b00110);
        assert_eq!(u64::MAX.without_leading_one(), u64::MAX>>1);
    }
}