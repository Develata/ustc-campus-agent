//! Affairs clock abstraction. The M71 application service uses this to read
//! `now()` when `as_of` is `None` (a new authorized read). The fixed clock is
//! the test fixture; production wires a real clock.

use time::OffsetDateTime;

/// Read-only clock abstraction for the M71 application service.
pub trait AffairsClock: Send + Sync {
    /// Returns the current instant.
    fn now(&self) -> OffsetDateTime;
}

/// Fixed clock for tests and fixtures. Always returns the same instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock {
    fixed_at: OffsetDateTime,
}

impl FixedClock {
    /// Builds one fixed clock.
    #[must_use]
    pub const fn new(fixed_at: OffsetDateTime) -> Self {
        Self { fixed_at }
    }

    /// Returns the fixed instant.
    #[must_use]
    pub const fn fixed_at(&self) -> OffsetDateTime {
        self.fixed_at
    }
}

impl AffairsClock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.fixed_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_clock_returns_same_instant() {
        let t = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        let clock = FixedClock::new(t);
        assert_eq!(clock.now(), t);
        assert_eq!(clock.now(), t);
    }
}
