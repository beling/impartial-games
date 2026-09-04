//! Optional information about the outcome of a position.

/// Optional information about outcome of a game position.
///
/// The solver uses it to stop the search as soon as the outcome (or the nimber)
/// is already known: `is_winning` implies a non-zero nimber, `is_losing` implies nimber 0.
pub trait OptionalOutcome: Copy {
    /// Returns `true` if `self` says that the position is winning.
    #[inline(always)] fn is_winning(self) -> bool { false }
    /// Returns `true` if `self` says that the position is losing.
    #[inline(always)] fn is_losing(self) -> bool { false }
}

/// Outcome is never known.
impl OptionalOutcome for () {}

/// Outcome of a position can be `Some(true)` (winning), `Some(false)` (losing), or `None` (unknown).
impl OptionalOutcome for Option<bool> {
    #[inline(always)] fn is_winning(self) -> bool { self == Some(true) }
    #[inline(always)] fn is_losing(self) -> bool { self == Some(false) }
}