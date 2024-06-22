use crate::BitSet;

use std::str::FromStr;

#[derive(Default)]
pub struct Game {
    pub taking_all: [u64; 4],   // set
    pub taking: Vec<u8>,
    pub breaking: Vec<u8>,
    //breaking_even: Vec<u8>,
    //breaking_odd: Vec<u8>
}

impl Game {
    fn parse_octal(c: u8) -> Option<u8> {
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
            if oct & 1 != 0 { result.taking_all.set_nimber(position as u16) }
            if oct & 2 != 0 { result.taking.push(position) }
            if oct & 4 != 0 { result.breaking.push(position) }
        }
        Some(result)
    }
}

impl FromStr for Game {
    type Err = &'static str;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii(s.as_bytes()).ok_or("game description must be in format [X.]OOO... where X is 0 or 4 and (up to 255) Os are octal digits")
    }
}

impl Game {
    #[inline] pub fn can_take_all(&self, n: usize) -> bool {
        self.taking_all.try_get_bit(n).unwrap_or(false)
    }
}