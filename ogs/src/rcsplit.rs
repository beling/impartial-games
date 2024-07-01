use std::fmt::Display;

use crate::{BitSet, stats::NimberStats};

pub(crate) struct RCSplit {
    pub(crate) r: [u64; 1<<(16-6)],
    pub(crate) c: [u64; 1<<(16-6)],
    pub(crate) max_c: u16, // largest nimber in c
    pub(crate) r_positions: Vec<usize>,
}

impl Default for RCSplit {
    #[inline] fn default() -> Self { Self::new(0) }
}

impl RCSplit {
    pub fn new(d: u16) -> Self {
        let mut r = [0; 1<<(16-6)];
        r[0] = d as u64+1;   // adds d to r
        Self { r, c: [0; 1<<(16-6)], max_c: 0, r_positions: Default::default() }
    }

    #[inline] pub fn can_add_to_c(&self, nimber: u16) -> bool {
        for v in 1..=self.max_c {
            if self.c.contain_nimber(v) && self.c.contain_nimber(nimber ^ v) {
                return false;
            }
        }
        true
    }

    #[inline] pub fn can_add_to_c_d(&self, nimber: u16, d: u16) -> bool {
        self.can_add_to_c(nimber ^ d)
    }

    #[inline] pub fn add_to_c(&mut self, nimber: u16) {
        self.c.add_nimber(nimber);
        if nimber > self.max_c { self.max_c = nimber }
    }

    #[inline] pub fn add_to_r(&mut self, nimber: u16) {
        self.r.add_nimber(nimber);
    }

    #[inline] pub fn add_to(&mut self, nimber: u16, to_c: bool) -> bool {
        if to_c {
            self.add_to_c(nimber)
        } else {
            self.add_to_r(nimber)
        }
        to_c
    }

    #[inline] pub fn classify(&mut self, nimber: u16) -> bool {
        self.add_to(nimber, self.can_add_to_c(nimber))
    }

    #[inline] pub fn classify_d(&mut self, nimber: u16, d: u16) -> bool {
        self.add_to(nimber, self.can_add_to_c(nimber ^ d))
    }

    pub fn in_c(&mut self, nimber: u16) -> bool {
        if self.c.contain_nimber(nimber) { return true; }
        if self.r.contain_nimber(nimber) { return false; }
        self.classify(nimber)
    }

    /// Never adds nimber to c.
    pub fn in_r(&mut self, nimber: u16, d: u16) -> bool {
        if self.c.contain_nimber(nimber) { return false; }
        if self.r.contain_nimber(nimber) { return true; }
        if self.can_add_to_c_d(nimber, d) {
            false
        } else {
            self.r.add_nimber(nimber);
            true
        }
    }

    pub fn clear(&mut self) {
        self.c.fill(0);
        self.r.fill(0);
        self.max_c = 0;
        self.r_positions.clear();
    }

    pub fn rebuild(&mut self, stats: &NimberStats, nimbers: &[u16]) {
        self.clear();
        self.r[0] = 1;
        for nimber in stats.nimbers_from_most_common(0) {
            self.classify(nimber);
        }
        for position in 1..nimbers.len() {
            if self.r.contain_nimber(nimbers[position]) {
                self.r_positions.push(position);
            }
        }
    }

    pub fn rebuild_d(&mut self, stats: &NimberStats, nimbers: &[u16], d: u16) {
        self.clear();
        self.r[0] = d as u64+1; // adds (0,0) (i.e. 0th bit) if d=0 and (0,1) (i.e. 1st bit) if d=1 to R
        for nimber in stats.nimbers_from_most_common(d) {   // skip (0,d)
            self.classify_d(nimber, d);
        }
        for position in 1..nimbers.len() {
            if self.r.contain_nimber((nimbers[position]<<1) | (position as u16 & 1)) {
                self.r_positions.push(position);
            }
        }
    }

    pub fn should_rebuild_d(&self, recent_nimber: u16, stats: &NimberStats, rebuild_threshold: u8) -> bool {
        let r_occ = stats.occurences[recent_nimber as usize] + rebuild_threshold as u32;
        for c in 0..=stats.max {
            let c_occ = stats.occurences[c as usize];
            if c_occ == 0 || !self.c.contain_nimber(c) { continue; }
            let c_grater = c > recent_nimber;
            if (c_grater && r_occ == c_occ) || (!c_grater && r_occ == c_occ+1) {
                return true;
            }
        }
        false
    }

    /// rebuild_threshold should be 0
    pub fn should_rebuild(&self, recent_nimber: u16, stats: &NimberStats, rebuild_threshold: u8) -> bool {
        //self.should_rebuild_d(recent_nimber, stats) //OK as 0 is in r
        let r_occ = stats.occurences[recent_nimber as usize] + rebuild_threshold as u32;
        for c in 1..=stats.max {
            let c_occ = stats.occurences[c as usize];
            if c_occ == 0 || !self.c.contain_nimber(c) { continue; }
            let c_grater = c > recent_nimber;
            if (c_grater && r_occ == c_occ) || (!c_grater && r_occ == c_occ+1) {
                return true;
            }
        }
        false
    }
}

impl Display for RCSplit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (as_pairs, name) = if let Some(p) = f.precision() {
            (true, p.to_string())
        } else {
            (false, "".to_owned())
        };
        write!(f, "C{}:", name)?;
        for n in 0..=self.max_c {
            if self.c.contain_nimber(n) {
                if as_pairs { write!(f, " {}.{}", n>>1, n&1)?; } else { write!(f, " {}", n)?; }
            }
        }
        writeln!(f)?;
        write!(f, "R{}:", name)?;
        for n in 0..=u16::MAX {
            if self.r.contain_nimber(n) {
                if as_pairs { write!(f, " {}.{}", n>>1, n&1)?; } else { write!(f, " {}", n)?; }
            }
        }
        write!(f, " at {} pos:", self.r_positions.len())?;
        for p in self.r_positions.iter().take(10) {
            write!(f, " {}", p)?;
        }
        if self.r_positions.len() > 10 { write!(f, "...")?; }
        Ok(())
    }
}
