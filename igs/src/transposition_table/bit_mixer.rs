/// Slightly generalized (masked) version of the bit-mixing function found by <cite>[David Stafford][1]</cite>
/// (and used for example in <cite>[SplitMix][2]</cite>).
///
/// [1]: http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
/// [2]: https://dl.acm.org/doi/10.1145/2660193.2660195
#[inline]
pub fn stafford13(mut x: u64, mask: u64) -> u64 {
    x = ((x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9u64)) & mask;
    x = ((x ^ (x >> 27)).wrapping_mul(0x94d049bb133111ebu64)) & mask;
    x ^ (x >> 31)
}

/// Slightly generalized (masked) version of <cite>[moremur][1]</cite> function found by Pelle Evensen.
///
/// [1]: http://mostlymangling.blogspot.com/2019/12/stronger-better-morer-moremur-better.html
#[inline]
pub fn moremur(mut x: u64, mask: u64) -> u64 {    // probably the best
    x = ((x ^ (x >> 27)).wrapping_mul(0x3C79AC492BA7B653u64)) & mask;
    x = ((x ^ (x >> 33)).wrapping_mul(0x1C69B3F74AC4AE35u64)) & mask;
    x ^ (x >> 27)
}

/// Slightly generalized (masked) version of <cite>[mx3 revision 2][1]</cite> function found by Jon Maiga.
///
/// [1]: http://jonkagstrom.com/mx3/mx3_rev2.html
#[inline]
pub fn mx3(mut x: u64, mask: u64) -> u64 {
    x = ((x ^ (x >> 32)).wrapping_mul(0xbea225f9eb34556d)) & mask;
    x = ((x ^ (x >> 29)).wrapping_mul(0xbea225f9eb34556d)) & mask;
    x = ((x ^ (x >> 32)).wrapping_mul(0xbea225f9eb34556d)) & mask;
    x ^ (x >> 29)
}

/// Slightly generalized (masked) version of <cite>[xmxmx][1]</cite> function found by Jon Maiga.
///
/// [1]: http://jonkagstrom.com/tuning-murmur3/index.html
#[inline]
pub fn xmxmx(mut x: u64, mask: u64) -> u64 {
    x = ((x ^ (x >> 27)).wrapping_mul(0xe9846af9b1a615d)) & mask;
    x = ((x ^ (x >> 25)).wrapping_mul(0xe9846af9b1a615d)) & mask;
    x ^ (x >> 27)
}

/// Slightly generalized (masked) version of https://gist.github.com/degski/6e2069d6035ae04d5d6f64981c995ec2
#[inline]
pub fn degski(mut x: u64, mask: u64) -> u64 {
    x = ((x ^ (x >> 32)).wrapping_mul(0xD6E8FEB86659FD93)) & mask;
    x = ((x ^ (x >> 32)).wrapping_mul(0xD6E8FEB86659FD93)) & mask;
    x ^ (x >> 32)
}

/// Returns `x`.
#[inline]
pub fn without_mixing(x: u64, _mask: u64) -> u64 { x }