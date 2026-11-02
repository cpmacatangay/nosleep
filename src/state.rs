//! The user-visible state machine.
//!
//! Deliberately pure: no Win32, no clocks, no I/O. Everything that can be
//! decided without touching the operating system is decided here, which is why
//! it can be unit tested.

/// Whether the machine is being held awake by this process.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Off,
    On,
}

impl State {
    /// A new process always starts Off. State is never persisted.
    pub const START: State = State::Off;

    pub fn is_on(self) -> bool {
        matches!(self, State::On)
    }

    /// The single transition the whole product is built on.
    pub fn toggled(self) -> State {
        match self {
            State::Off => State::On,
            State::On => State::Off,
        }
    }

    /// Tooltip text. States must be legible in words, not only in artwork.
    pub fn tooltip(self) -> &'static str {
        match self {
            State::On => "NoSleep: keeping this PC awake - click to allow sleep",
            State::Off => "NoSleep: normal sleep - click to keep this PC awake",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn starts_off() {
        assert_eq!(State::START, State::Off);
        assert!(!State::START.is_on());
    }

    #[test]
    fn toggle_round_trips() {
        assert_eq!(State::Off.toggled(), State::On);
        assert_eq!(State::On.toggled(), State::Off);
        assert_eq!(State::Off.toggled().toggled(), State::Off);
    }

    #[test]
    fn is_on_matches_variant() {
        assert!(State::On.is_on());
        assert!(!State::Off.is_on());
    }

    #[test]
    fn tooltips_are_distinct_and_bounded() {
        let on = State::On.tooltip();
        let off = State::Off.tooltip();
        assert_ne!(on, off, "the two states must not share a tooltip");
        // NOTIFYICONDATAW.szTip is 128 UTF-16 code units including the NUL.
        assert!(on.encode_utf16().count() < 128);
        assert!(off.encode_utf16().count() < 128);
    }
}
