/// Optional information about outcome.
pub trait OptionalOutcome: Copy {
    #[inline(always)] fn is_winning(self) -> bool { false }
    #[inline(always)] fn is_losing(self) -> bool { false }
}

impl OptionalOutcome for () {}

impl OptionalOutcome for Option<bool> {
    #[inline(always)] fn is_winning(self) -> bool { self == Some(true) }
    #[inline(always)] fn is_losing(self) -> bool { self == Some(false) }
}