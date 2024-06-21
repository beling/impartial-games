mod set;
use set::BitSet;

use std::str::FromStr;

#[derive(Default)]
struct Game {
    taking_all: [u64; 4],   // set
    taking: Vec<u8>,
    breaking: Vec<u8>,
    //breaking_even: Vec<u8>,
    //breaking_odd: Vec<u8>
}

impl Game {
    fn parse_octal(c: u8) -> Option<u8> {
        (b'0' <= c && c <= b'8').then(|| c - b'0')
    }

    fn from_ascii(mut s: &[u8]) -> Option<Game> {
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
        Self::from_ascii(s.as_bytes()).ok_or("game description must be in format [X.]OOO... where X is 0 or 4 and Os are octal digits")
    }
}

impl Game {
    #[inline] fn can_take_all(&self, n: usize) -> bool {
        self.taking_all.try_get_bit(n).unwrap_or(false)
    }
}

const MAX_N: usize = 100;

fn main() {
    let game = Game::from_ascii(b"4.007").unwrap();
    let mut nimbers = [0u16; MAX_N];
    for n in 0..MAX_N {
        let mut option_nimbers = [0u64; 1<<(16-6)]; // 2**16 bits
        if game.can_take_all(n) { option_nimbers.set_nimber(0) }
        for t in &game.taking {
            let t = *t as usize;
            if t >= n { break }
            option_nimbers.set_nimber(nimbers[n-t]);
        }
        for b in &game.breaking {
            let b = *b as usize;
            if b >= n { break }
            let after_take = n - b;
            for i in 1 .. after_take/2 + 1 {
                option_nimbers.set_nimber(nimbers[i] ^ nimbers[after_take-i]);
            }
        }
        nimbers[n] = option_nimbers.mex();
        print !("{} ", nimbers[n])
    }
}