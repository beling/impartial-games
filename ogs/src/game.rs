use crate::{BitSet, SolverEvent};

use std::{iter::FusedIterator, str::FromStr};

#[derive(Default, Clone)]
pub struct Game {
    pub taking_all: [u64; 4],   // set
    pub taking: Vec<u8>,
    pub breaking: Vec<u8>,
    //breaking_even: Vec<u8>,
    //breaking_odd: Vec<u8>
}

impl Game {
    #[inline] fn parse_octal(c: u8) -> Option<u8> {
        (b'0' <= c && c <= b'8').then(|| c - b'0')
    }

    pub fn from_ascii(mut s: &[u8]) -> Option<Game> {
        let mut result = Self::default();
        if s.starts_with(b"4.") || s.starts_with(b"4,") || s.starts_with(b"4d") {
            result.breaking.push(0);
            s = &s[2..];
        } else if s.starts_with(b"0.") || s.starts_with(b"0,") || s.starts_with(b"0d") {
            s = &s[2..];
        } else if s.starts_with(b".") || s.starts_with(b",") || s.starts_with(b"d") {
            s = &s[1..];
        }
        let mut position = 0u8;
        for c in s {
            position = position.checked_add(1)?;
            let oct = Self::parse_octal(*c)?;
            if oct & 1 != 0 { result.taking_all.add_nimber(position as u16) }
            if oct & 2 != 0 { result.taking.push(position) }
            if oct & 4 != 0 { result.breaking.push(position) }
        }
        Some(result)
    }

    #[inline] pub fn can_take_all(&self, n: usize) -> bool {
        self.taking_all.try_get_bit(n).unwrap_or(false)
    }

    pub(crate) fn consider_taking<S: SolverEvent>(&self, nimbers: &[u16], option_nimbers: &mut [u64; 1<<(16-6)], stats: &mut S) {
        let n = nimbers.len();
        if self.can_take_all(n) { option_nimbers.add_nimber(0) }
        for t in &self.taking {
            let t = *t as usize;
            if t >= n { break }
            option_nimbers.add_nimber(nimbers[n-t]);
            stats.take_option();
        }
    }

    #[inline] pub fn breaking_moves(&self, n: usize) -> BreakingMoveIterator<std::iter::Copied<std::slice::Iter<'_, u8>>> {
        //BreakingMoveIterator::for_iter(n, self.breaking.iter().copied())
        BreakingMoveIterator::for_slice(n, self.breaking.as_slice())
    }

    /// Returns rules as a sequence of octal numbers.
    pub fn rules(&self) -> [u8; 256] {
        let mut result = [0; 256];
        for a in 0..256 {
            if self.taking_all.get_bit(a) {
                result[a] |= 1;
            }
        }
        for t in &self.taking { result[*t as usize] |= 2; }
        for b in &self.breaking { result[*b as usize] |= 4; }
        result
    }

    /// Returns rules as an ascii string, using given decimal separator (for example `b'.'`).
    pub fn to_ascii(&self, separator: u8) -> Vec<u8> {
        let rules = self.rules();
        let mut number_of_rules = 0;
        for (i, r) in rules.iter().enumerate() {
            if *r != 0 { number_of_rules = i; }
        }
        number_of_rules += 1;
        let mut result = Vec::with_capacity(number_of_rules);
        result.push(rules[0] + b'0');
        result.push(separator);
        for r in 1..number_of_rules {
            result.push(rules[r] + b'0');
        }
        result
    }

    /// Returns the total number of taking iterations needed (by any of the methods: naive, RC or RC2)
    /// to calculate the nimbers of all positions up to and including the one given.
    pub fn taking_iters(&self, position: usize) -> usize {
        self.taking.iter().map(|t| position.saturating_sub(*t as usize)).sum()
    }

    /// Returns the total number of breaking iterations needed by the naive method
    /// to calculate the nimbers of all positions up to and including the one specified.
    pub fn breaking_naive_iters(&self, position: usize) -> usize {
        self.breaking.iter().map(|b| {
            let b = *b as usize;
            if position < b + 2 { 0 } else
            if position & 1 != b & 1 { let k = (position - b - 1) / 2; k*k+k } // difference is odd
            else { let hd = (position - b) / 2; let k = hd-1; k*k+k + hd }  // difference is even
        }).sum()
    }

    /// Returns the total number of iterations needed by the naive method
    /// to calculate the nimbers of all positions up to and including the one specified.
    pub fn naive_iters(&self, position: usize) -> usize {
        self.taking_iters(position) + self.breaking_naive_iters(position)
    }

    /// Try to calculates (pre-period, period) of the game using the nimbers of its few first positions.
    pub fn period(&self, nimbers: &[u16]) -> Option<(usize, usize)> {
        // uses Theorem 3.73 from https://dspace.cvut.cz/bitstream/handle/10467/82669/F8-DP-2019-Lomic-Simon-thesis.pdf
        // which is based on Guy and Smiths THE G-VALUES OF VARIOUS GAMES 1956
        let mut max_to_take = 
            (self.taking_all.msb_index().unwrap_or(0) as u8)
            .max(*self.taking.last().unwrap_or(&0));

        let log2mult = if let Some(b) = self.breaking.last() {   // has breaking moves?
            max_to_take = max_to_take.max(*b);
            1   // we need *2 (or <<1) for octal games
        } else { 0 };   // and *1 (or <<0) otherwise

        let len = nimbers.len();
        for period in 1 .. (len >> log2mult) {
            let mut preperiod = len - period;
            while preperiod > 0 && nimbers[preperiod-1] == nimbers[preperiod-1+period] {
                preperiod -= 1;
            }
            //if len >= (2*preperiod + 2*period + max_to_take - 1).max(max_to_take+2) {
            if len > ((preperiod + period) << log2mult) + max_to_take as usize {
                return Some((preperiod, period));
            }
        }
        None
    }
}

impl FromStr for Game {
    type Err = &'static str;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii(s.as_bytes()).ok_or("game description must be in format [X.]OOO... where X is 0 or 4 and (up to 255) Os are octal digits")
    }
}

impl ToString for Game {
    fn to_string(&self) -> String {
        unsafe{ String::from_utf8_unchecked(self.to_ascii(b'.')) }
    }
}

/*impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}*/

pub struct BreakingMoveIterator<I> {
    current: I,
    n: usize,
    after_take: usize,
    last_i: usize,
    i: usize
}

impl<I> BreakingMoveIterator<I> {
    #[inline] pub fn for_iter(n: usize, to_take_iterator: I) -> Self {
        Self { current: to_take_iterator, n, after_take: 0, last_i: 0, i: 0 }
    }
}

impl<'a, IU: Into<usize> + Copy> BreakingMoveIterator<std::iter::Copied<std::slice::Iter<'a, IU>>> {
    #[inline] pub fn for_slice(n: usize, slice: &'a [IU]) -> Self {
        Self::for_iter(n, slice.iter().copied())
    }
}

impl<IU: Into<usize>, I: Iterator<Item = IU>> Iterator for BreakingMoveIterator<I> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.last_i {
            let b = self.current.next()?.into();
            if b+1 >= self.n { return None; } // n-b >= 2  <=>  !(2 > n-b)  <=>  !(b+2 > n)  <=>  !(b+1 >= n)
            self.after_take = self.n - b;   // >= 2
            self.last_i = self.after_take / 2;  // >= 1
            self.i = 1;
        } else {
            self.i += 1;
        }
        Some((self.i, self.after_take - self.i))
    }
}

impl<IU: Into<usize>, I: Iterator<Item = IU> + FusedIterator> FusedIterator for BreakingMoveIterator<I>{}


#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_4d() {
        let g = Game::from_str("4.").unwrap();
        assert_eq!(g.breaking_moves(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(g.breaking_moves(1).collect::<Vec<_>>(), vec![]);
        assert_eq!(g.breaking_moves(2).collect::<Vec<_>>(), vec![(1, 1)]);
        assert_eq!(g.breaking_moves(3).collect::<Vec<_>>(), vec![(1, 2)]);
        assert_eq!(g.breaking_moves(4).collect::<Vec<_>>(), vec![(1, 3), (2, 2)]);
        assert_eq!(g.breaking_moves(5).collect::<Vec<_>>(), vec![(1, 4), (2, 3)]);
    }

    #[test]
    fn test_0d44() {
        let g = Game::from_str("0.44").unwrap();
        assert_eq!(g.breaking_moves(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(g.breaking_moves(1).collect::<Vec<_>>(), vec![]);
        assert_eq!(g.breaking_moves(2).collect::<Vec<_>>(), vec![]);
        assert_eq!(g.breaking_moves(3).collect::<Vec<_>>(), vec![(1, 1)]);
        assert_eq!(g.breaking_moves(4).collect::<Vec<_>>(), vec![(1, 2), (1, 1)]);
        assert_eq!(g.breaking_moves(5).collect::<Vec<_>>(), vec![(1, 3), (2, 2), (1, 2)]);
    }
}