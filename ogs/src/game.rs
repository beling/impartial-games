use crate::{BitSet, SolverEvent};

use std::str::FromStr;

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
        if s.starts_with(b"4.") {
            result.breaking.push(0);
            s = &s[2..];
        } else if s.starts_with(b"0.") {
            s = &s[2..];
        } else if s.starts_with(b".") {
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
}

impl FromStr for Game {
    type Err = &'static str;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii(s.as_bytes()).ok_or("game description must be in format [X.]OOO... where X is 0 or 4 and (up to 255) Os are octal digits")
    }
}


pub struct BreakingMoveIterator<I> {
    current: I,
    n: usize,
    after_take: usize,
    last_i: usize,
    i: usize
}

impl<I> BreakingMoveIterator<I> {
    pub fn for_iter(n: usize, to_take_iterator: I) -> Self {
        Self { current: to_take_iterator, n, after_take: 0, last_i: 0, i: 0 }
    }
}

impl<'a, IU: Into<usize> + Copy> BreakingMoveIterator<std::iter::Copied<std::slice::Iter<'a, IU>>> {
    pub fn for_slice(n: usize, slice: &'a [IU]) -> Self {
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