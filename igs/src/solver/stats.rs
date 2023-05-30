use std::ops::Deref;
use std::fmt;
use std::fmt::Formatter;

/// Receives reports on search progress.
pub trait ProgressReporter {
    /// Called before search next position or its component.
    #[inline(always)] fn begin(&mut self, _max: u16) {}

    /// Called after search the position or its component.
    #[inline(always)] fn end(&mut self) {}

    /// Called before just before start analyzing next top-level nimber or move.
    /// (`current` is a number of move to be analyzed, and `max` is maximal number of nimber/moves).
    fn progress(&mut self, current: u16);
}

impl ProgressReporter for () {
    #[inline(always)] fn progress(&mut self, _current: u16) {}
}

/// Prints search progress to std-out.
#[derive(Copy, Clone)]
pub struct PrintProgress;

impl ProgressReporter for PrintProgress {
    fn begin(&mut self, max: u16) {
        println!("Searching (max {}):", max);
    }

    fn end(&mut self) {
        println!(" DONE");
    }

    fn progress(&mut self, current: u16) {
        println!(" {}", current);
    }
}

/// Collects statistics for search process.
///
/// Search events are indicated with calling `self` methods by the search algorithm.
///
/// Phases of search of a single position are indicated by calling (in the order):
/// `pre` (which is called for each node), `ETC` (only by the algorithms that use ETC), `recursive`.
///
/// Searching of a single position can finish in any phase and is indicated by calling one of:
/// `exact`, `unknown`, or `db_cut`.
///
/// Reading from databases are indicated by calling `TT_read` and `const_db_read`
/// (which is usually done in pre or ETC phase).
/// Just after `TT_read` or `const_db_read`, `db_cut` or `db_skip` can be called.
///
/// Statistics are collected up to the time of calling `reset`.
/// To enable collecting and averaging statistics from many searches (for many initial positions),
/// solvers never call `reset` (i.e. it must by called manually).
///
/// Default implementations of all `StatsCollector` methods do nothing.
pub trait StatsCollector {

    /// Called by the search algorithm at the begging of each recursive call (for each position).
    /// It enables investigating the shape of the search tree (of depth-first search process).
    /// Visiting of each node is finished by calling one of: exact, unknown, db_cut.
    #[inline(always)] fn pre(&mut self) {}

    /// Called by the search algorithm at the begging of ETC phase.
    #[inline(always)] fn etc(&mut self) {}

    /// Called by the search algorithm just before loop that iterates over moves and recursively calls the algorithm.
    #[inline(always)] fn recursive(&mut self) {}

    /// Called for each reading from transposition table.
    #[inline(always)] fn tt_read(&mut self) {}

    /// Called for each reading from const (end) database.
    #[inline(always)] fn const_db_read(&mut self) {}

    /// Called when value (passed nimber) read from database (transposition table or const database) caused skipping position (usually by ETC).
    /// Type of database can be find by checking which call, TT_read or const_db_read, directly preceded db_cut.
    #[inline(always)] fn db_skip(&mut self, _nimber: u8) {}

    /// Called when value (passed nimber) read from database (transposition table or const database) caused pruning.
    /// Type of database can be find by checking which call, TT_read or const_db_read, directly preceded db_cut.
    #[inline(always)] fn db_cut(&mut self, _nimber: u8) {}

    /// Called for each position which was recursively searched, but whose nimber wasn't established (due to pruning algorithm).
    #[inline(always)] fn unknown(&mut self) {}

    /// Called for each position whose nimber (given as parameter) is calculated and writen to TT; called for each writing to TT.
    #[inline(always)] fn exact(&mut self, _nimber: u8) {}

    /// Reset statistics.
    #[inline(always)] fn reset(&mut self) {}
}

/// It is used by stats collectors to follow the phase (pre, ETC, recursive) of the search.
#[derive(Copy, Clone)]
pub enum SearchPhase { Pre = 0, ETC = 1, Recursive = 2 }

impl SearchPhase {
    /// Should be called in pre.
    #[inline(always)] fn pre(&mut self) { *self = Self::Pre; }
    /// Should be called in ETC.
    #[inline(always)] fn etc(&mut self) { *self = Self::ETC; }
    /// Should be called in recursive.
    #[inline(always)] fn recursive(&mut self) { *self = Self::Recursive; }
    /// Should be called in: db_cut, unknown, exact.
    #[inline(always)] fn end(&mut self) { *self = Self::Recursive; }
}

impl Default for SearchPhase {
    #[inline(always)] fn default() -> Self { Self::Pre }
}

/// Event generated during search.
#[derive(Copy, Clone)]
pub enum EventType {
    Exact = 0, Unknown = 1, TTCut = 2, ConstDbCut = 3, TTSkip = 4, ConstDbSkip = 5, TTRead = 6, ConstDbRead = 7
}

impl EventType {
    const NAMES: [&'static str; 8] = [ "exact value", "undetermined/cut", "cut by TT", "cut by const db",
        "skipped by TT", "skipped by const db", "TT reads", "const db reads" ];

    /// Returns event for database cut: TTCut if tt is true, ConstDbSkip otherwise.
    #[inline(always)]
    fn db_cut(tt: bool) -> Self {
        if tt { EventType::TTCut } else { EventType::ConstDbCut }
    }

    /// Returns event for database skip: TTSkip if tt is true, ConstDbSkip otherwise.
    #[inline(always)]
    fn db_skip(tt: bool) -> Self {
        if tt { EventType::TTSkip } else { EventType::ConstDbSkip }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(Self::NAMES[*self as usize])
    }
}

/// Count events, separately for various types and search phases.
#[derive(Default, Copy, Clone)]
pub struct EventCounters {
    /// Counts any event in any phase.
    events_count: [[u64; 3]; 8]
}

macro_rules! fs { () => ("{:17} {:>10} {:>10} {:>10} {:>10}") }
macro_rules! fs_title { () => ("{:>17} {:>10} {:>10} {:>10} {:>10}") }

impl EventCounters {
    /// Count event revealed in given search phase.
    #[inline(always)]
    pub fn register_event(&mut self, phase: SearchPhase, event: EventType) {
        self.events_count[event as usize][phase as usize] += 1;
    }

    /// Returns number of events of given type in given search phase.
    #[inline(always)]
    pub fn number_of_events(&self, phase: SearchPhase, event: EventType) -> u64 {
        self.events_count[event as usize][phase as usize]
    }

    /// Returns number of nodes visited during search, which equals to number of returning all events.
    pub fn nodes_visited(&self) -> u64 {
        self.returns_in_phase(SearchPhase::Pre) + self.returns_in_phase(SearchPhase::ETC) + self.returns_in_phase(SearchPhase::Recursive)
    }

    /// Returns number of hits to TT
    pub fn tt_hits(&self, phase: SearchPhase) -> u64 {
        self.number_of_events(phase, EventType::TTCut) + self.number_of_events(phase, EventType::TTSkip)
    }

    /// Returns number of hits to const db
    pub fn const_db_hits(&self, phase: SearchPhase) -> u64 {
        self.number_of_events(phase, EventType::ConstDbCut) + self.number_of_events(phase, EventType::ConstDbSkip)
    }

    /// Returns number of returns (of any reason: exact, unknown, TT_cut, const_db_cut) in given search phase.
    pub fn returns_in_phase(&self, phase: SearchPhase) -> u64 {
        let p = phase as usize;
        let e = &self.events_count;
        e[EventType::Exact as usize][p] + e[EventType::Unknown as usize][p]
            + e[EventType::TTCut as usize][p] + e[EventType::ConstDbCut as usize][p]
    }

    /// Returns numbers of returns (of any reason) in all search phases
    /// (returned array is indexed by the search phase).
    /*pub fn returns_in_phases(&self) -> [u64; 3] {
        [self.returns_in_phase(SearchPhase::pre), self.returns_in_phase(SearchPhase::ETC), self.returns_in_phase(SearchPhase::recursive)]
    }*/

    /// Returns number of events of given type (in any search phase).
    pub fn events_of_type(&self, event_type: EventType) -> u64 {
        self.events_count[event_type as usize].iter().sum()
    }

    /// Returns numbers of returns of all reasons (in any search phase).
    /// (returned array is indexed by the return reason).
    /*pub fn returns_of_reasons(&self, reason: EventType) ->[u64; 4] {
        [self.returns_of_reason(EventType::exact), self.returns_of_reason(EventType::unknown),
            self.returns_of_reason(EventType::TT_cut), self.returns_of_reason(EventType::const_db_cut)]
    }*/

    /// Set all event counters to zero.
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    fn print_row<T: fmt::Display>(f: &mut fmt::Formatter<'_>, title: T, pre: u64, etc: u64, recursive: u64) -> fmt::Result {
        writeln!(f, fs!(), title, pre, etc, recursive, pre+etc+recursive)
    }

    fn print_event_stats(&self, f: &mut fmt::Formatter<'_>, event: EventType) -> fmt::Result {
        Self::print_row(f, event,
                        self.number_of_events(SearchPhase::Pre, event),
                        self.number_of_events(SearchPhase::ETC, event),
                        self.number_of_events(SearchPhase::Recursive, event))
    }

    fn print_summarize<T: fmt::Display>(f: &mut fmt::Formatter<'_>, title: T, pre: u64, etc: u64, recursive: u64) -> fmt::Result {
        writeln!(f, fs_title!(), title, pre, etc, recursive, pre+etc+recursive)
    }
}

impl fmt::Display for EventCounters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, fs_title!(), "phase" , "pre", "ETC", "recursive", "total")?;
        for r in &[EventType::Exact, EventType::Unknown, EventType::TTCut, EventType::ConstDbCut] {
            self.print_event_stats(f, *r)?;
        }
        EventCounters::print_summarize(f, "total",
                                       self.returns_in_phase(SearchPhase::Pre),
                                       self.returns_in_phase(SearchPhase::ETC),
                                       self.returns_in_phase(SearchPhase::Recursive)
        )?;
        writeln!(f)?;
        Self::print_row(f, "TT hits",
                        self.tt_hits(SearchPhase::Pre),
                        self.tt_hits(SearchPhase::ETC),
                        self.tt_hits(SearchPhase::Recursive))?;
        self.print_event_stats(f, EventType::TTRead)?;
        Self::print_row(f, "const db hits",
                        self.const_db_hits(SearchPhase::Pre),
                        self.const_db_hits(SearchPhase::ETC),
                        self.const_db_hits(SearchPhase::Recursive))?;
        self.print_event_stats(f, EventType::ConstDbRead)?;
        Ok(())
    }
}

impl std::ops::AddAssign for EventCounters {
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl std::ops::AddAssign<&Self> for EventCounters {
    fn add_assign(&mut self, rhs: &Self) {
        for (ts, tr) in self.events_count.iter_mut().zip(rhs.events_count.iter()) {
            for (s, r) in ts.iter_mut().zip(tr.iter()) {
                *s += r;
            }
        }
    }
}

impl std::ops::Add for EventCounters {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self + &rhs
    }
}

impl std::ops::Add<&Self> for EventCounters {
    type Output = Self;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs; self
    }
}

impl<'a> std::iter::Sum<&'a EventCounters> for EventCounters {
    fn sum<I: Iterator<Item=&'a Self>>(iter: I) -> Self {
        let mut result = EventCounters::default();
        for e in iter { result += e; };
        result
    }
}

#[derive(Default)]
pub struct EventStats {
    events: EventCounters,
    phase: SearchPhase,
    read_was_from_tt: bool
}

impl Deref for EventStats {
    type Target = EventCounters;
    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl fmt::Display for EventStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.events.fmt(f)
    }
}

impl StatsCollector for EventStats {
    #[inline(always)] fn pre(&mut self) { self.phase.pre(); }

    #[inline(always)] fn etc(&mut self) { self.phase.etc(); }

    #[inline(always)] fn recursive(&mut self) { self.phase.recursive(); }

    #[inline]
    fn tt_read(&mut self) {
        self.events.register_event(self.phase, EventType::TTRead);
        self.read_was_from_tt = true;
    }

    #[inline]
    fn const_db_read(&mut self) {
        self.events.register_event(self.phase, EventType::ConstDbRead);
        self.read_was_from_tt = false;
    }

    #[inline]
    fn db_skip(&mut self, _nimber: u8) {
        self.events.register_event(self.phase, EventType::db_skip(self.read_was_from_tt));
    }

    #[inline]
    fn db_cut(&mut self, _nimber: u8) {
        self.events.register_event(self.phase, EventType::db_cut(self.read_was_from_tt));
        self.phase.end();
    }

    #[inline]
    fn unknown(&mut self) {
        self.events.register_event(self.phase, EventType::Unknown);
        self.phase.end();
    }

    #[inline]
    fn exact(&mut self, _nimber: u8) {
        self.events.register_event(self.phase, EventType::Exact);
        self.phase.end();
    }

    #[inline]
    fn reset(&mut self) {
        self.events.reset();
    }
}

#[derive(Default)]
pub struct EventStatsAtLevels {
    events: Vec<EventCounters>,
    phase: SearchPhase,
    level: usize,   // 0 before/after search, 1 for root, etc.
    read_was_from_tt: bool
}

/// Enlarge (with defaults values) vector vec to have given index and return vec[index].
#[inline] fn enlarge_to_index<T: Default>(vec: &mut Vec<T>, index: usize) -> &mut T {
    if index >= vec.len() {
        vec.resize_with(index+1, Default::default);
    }
    unsafe { vec.get_unchecked_mut(index) }
}

impl EventStatsAtLevels {
    /// Returns total number of events (summed over all levels).
    pub fn total(&self) -> EventCounters {
        self.events.iter().sum()
    }

    fn register_event(&mut self, event: EventType) {
        enlarge_to_index(&mut self.events, self.level-1).register_event(self.phase, event);
    }

    fn register_return_event(&mut self, event: EventType) {
        self.register_event(event);
        self.phase.end();
        self.level -= 1;
    }
}

impl StatsCollector for EventStatsAtLevels {
    #[inline(always)]
    fn pre(&mut self) {
        self.phase.pre();
        self.level += 1;
    }

    #[inline(always)] fn etc(&mut self) { self.phase.etc(); }

    #[inline(always)] fn recursive(&mut self) { self.phase.recursive(); }

    #[inline(always)]
    fn tt_read(&mut self) {
        self.register_event(EventType::TTRead);
        self.read_was_from_tt = true;
    }

    #[inline(always)]
    fn const_db_read(&mut self) {
        self.register_event(EventType::ConstDbRead);
        self.read_was_from_tt = false;
    }

    #[inline(always)]
    fn db_skip(&mut self, _nimber: u8) {
        self.register_event(EventType::db_skip(self.read_was_from_tt));
    }

    #[inline(always)]
    fn db_cut(&mut self, _nimber: u8) {
        self.register_return_event(EventType::db_cut(self.read_was_from_tt));
    }

    #[inline(always)]
    fn unknown(&mut self) {
        self.register_return_event(EventType::Unknown);
    }

    #[inline(always)]
    fn exact(&mut self, _nimber: u8) {
        self.register_return_event(EventType::Exact);
    }

    #[inline]
    fn reset(&mut self) {
        assert_eq!(self.level, 0);
        self.events.clear();
    }
}

macro_rules! fs_level { () => ("{:4} {:>10}") }

impl fmt::Display for EventStatsAtLevels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.total().fmt(f)?;
        writeln!(f)?;
        writeln!(f, fs_level!(), "deep", "nodes")?;
        for (i, e) in self.events.iter().enumerate() {
            writeln!(f, fs_level!(), i, e.nodes_visited())?;
        }
        Ok(())
    }
}

macro_rules! ncf { () => ("{:>6} {:>10} {:>10} {:>10}") }
//macro_rules! ncf_r { () => (concat!(" " ncf_l!())) }

#[derive(Default)]
struct NimberOccurrences {
    calculated: u64,
    tt: u64,
    const_db: u64
}

#[derive(Default)]
pub struct NimberStats {
    number_of_nimber: Vec<NimberOccurrences>,
    read_was_from_tt: bool
}

/// Calculates statistics for nimbers.
impl NimberStats {
    fn register_nimber_from_db(&mut self, nimber: u8) {
        let c = enlarge_to_index(&mut self.number_of_nimber, nimber as usize);
        if self.read_was_from_tt { c.tt += 1; } else { c.const_db += 1; }
    }

    fn write_header(f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, ncf!(), "nimber", "calculated", "from: TT", "const db")
    }

    fn write_nimber(&self, f: &mut Formatter<'_>, nimber: u8) -> fmt::Result {
        write!(f, ncf!(), nimber,
               self.number_of_nimber[nimber as usize].calculated,
               self.number_of_nimber[nimber as usize].tt,
               self.number_of_nimber[nimber as usize].const_db)
    }
}

impl fmt::Display for NimberStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.number_of_nimber.len() == 0 { return Ok(()); }
        Self::write_header(f)?;
        if self.number_of_nimber.len() == 1 { return self.write_nimber(f, 1); }
        write!(f, "     ")?; Self::write_header(f)?; writeln!(f)?;
        let half = (self.number_of_nimber.len()+1) / 2;
        for i in 0..half {
            self.write_nimber(f, i as u8)?;
            let r = i + half;
            if r < self.number_of_nimber.len() {
                write!(f, "     ")?; self.write_nimber(f, r as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl StatsCollector for NimberStats {
    #[inline(always)] fn tt_read(&mut self) { self.read_was_from_tt = true; }
    #[inline(always)] fn const_db_read(&mut self) { self.read_was_from_tt = false; }

    #[inline(always)] fn db_skip(&mut self, nimber: u8) { self.register_nimber_from_db(nimber); }
    #[inline(always)] fn db_cut(&mut self, nimber: u8) { self.register_nimber_from_db(nimber); }

    #[inline(always)] fn exact(&mut self, nimber: u8) {
        enlarge_to_index(&mut self.number_of_nimber, nimber as usize).calculated += 1;
    }

    #[inline(always)] fn reset(&mut self) { self.number_of_nimber.clear(); }
}


impl StatsCollector for () {}

impl<SC1: StatsCollector> StatsCollector for (SC1,) {
    #[inline(always)] fn pre(&mut self) { self.0.pre(); }
    #[inline(always)] fn etc(&mut self) { self.0.etc(); }
    #[inline(always)] fn recursive(&mut self) { self.0.recursive(); }

    #[inline(always)] fn tt_read(&mut self) { self.0.tt_read(); }
    #[inline(always)] fn const_db_read(&mut self) { self.0.const_db_read(); }

    #[inline(always)] fn db_skip(&mut self, nimber: u8) { self.0.db_skip(nimber); }

    #[inline(always)] fn db_cut(&mut self, nimber: u8) { self.0.db_cut(nimber); }
    #[inline(always)] fn unknown(&mut self) { self.0.unknown(); }
    #[inline(always)] fn exact(&mut self, nimber: u8) { self.0.exact(nimber); }

    #[inline(always)] fn reset(&mut self) { self.0.reset(); }
}

impl<SC1: StatsCollector, SC2: StatsCollector> StatsCollector for (SC1, SC2) {
    #[inline(always)] fn pre(&mut self) { self.0.pre(); self.1.pre(); }
    #[inline(always)] fn etc(&mut self) { self.0.etc(); self.1.etc(); }
    #[inline(always)] fn recursive(&mut self) { self.0.recursive(); self.1.recursive(); }

    #[inline(always)] fn tt_read(&mut self) { self.0.tt_read(); self.1.tt_read(); }
    #[inline(always)] fn const_db_read(&mut self) { self.0.const_db_read(); self.1.const_db_read(); }

    #[inline(always)] fn db_skip(&mut self, nimber: u8) { self.0.db_skip(nimber); self.1.db_skip(nimber); }

    #[inline(always)] fn db_cut(&mut self, nimber: u8) { self.0.db_cut(nimber); self.1.db_cut(nimber); }
    #[inline(always)] fn unknown(&mut self) { self.0.unknown(); self.1.unknown(); }
    #[inline(always)] fn exact(&mut self, nimber: u8) { self.0.exact(nimber); self.1.exact(nimber); }

    #[inline(always)] fn reset(&mut self) { self.0.reset(); self.1.reset(); }
}

impl<SC1: StatsCollector, SC2: StatsCollector, SC3: StatsCollector> StatsCollector for (SC1, SC2, SC3) {
    #[inline] fn pre(&mut self) {
        self.0.pre(); self.1.pre(); self.2.pre();
    }
    #[inline] fn etc(&mut self) {
        self.0.etc(); self.1.etc(); self.2.etc();
    }
    #[inline] fn recursive(&mut self) {
        self.0.recursive(); self.1.recursive(); self.2.recursive();
    }

    #[inline] fn tt_read(&mut self) {
        self.0.tt_read(); self.1.tt_read(); self.2.tt_read();
    }
    #[inline] fn const_db_read(&mut self) {
        self.0.const_db_read(); self.1.const_db_read(); self.2.const_db_read();
    }

    #[inline] fn db_skip(&mut self, nimber: u8) {
        self.0.db_skip(nimber); self.1.db_skip(nimber); self.2.db_skip(nimber);
    }

    #[inline] fn db_cut(&mut self, nimber: u8) {
        self.0.db_cut(nimber); self.1.db_cut(nimber); self.2.db_cut(nimber);
    }
    #[inline] fn unknown(&mut self) {
        self.0.unknown(); self.1.unknown(); self.2.unknown();
    }
    #[inline] fn exact(&mut self, nimber: u8) {
        self.0.exact(nimber); self.1.exact(nimber); self.2.exact(nimber);
    }

    #[inline] fn reset(&mut self) {
        self.0.reset(); self.1.reset(); self.2.reset();
    }
}

impl<SC1: StatsCollector, SC2: StatsCollector, SC3: StatsCollector, SC4: StatsCollector> StatsCollector for (SC1, SC2, SC3, SC4) {
    #[inline] fn pre(&mut self) {
        self.0.pre(); self.1.pre(); self.2.pre(); self.3.pre();
    }
    #[inline] fn etc(&mut self) {
        self.0.etc(); self.1.etc(); self.2.etc(); self.3.etc();
    }
    #[inline] fn recursive(&mut self) {
        self.0.recursive(); self.1.recursive(); self.2.recursive(); self.3.recursive();
    }

    #[inline] fn tt_read(&mut self) {
        self.0.tt_read(); self.1.tt_read(); self.2.tt_read(); self.3.tt_read();
    }
    #[inline] fn const_db_read(&mut self) {
        self.0.const_db_read(); self.1.const_db_read(); self.2.const_db_read(); self.3.const_db_read();
    }

    #[inline] fn db_skip(&mut self, nimber: u8) {
        self.0.db_skip(nimber); self.1.db_skip(nimber); self.2.db_skip(nimber); self.3.db_skip(nimber);
    }

    #[inline] fn db_cut(&mut self, nimber: u8) {
        self.0.db_cut(nimber); self.1.db_cut(nimber); self.2.db_cut(nimber); self.3.db_cut(nimber);
    }
    #[inline] fn unknown(&mut self) {
        self.0.unknown(); self.1.unknown(); self.2.unknown(); self.3.unknown();
    }
    #[inline] fn exact(&mut self, nimber: u8) {
        self.0.exact(nimber); self.1.exact(nimber); self.2.exact(nimber); self.3.exact(nimber);
    }

    #[inline] fn reset(&mut self) {
        self.0.reset(); self.1.reset(); self.2.reset(); self.3.reset();
    }
}