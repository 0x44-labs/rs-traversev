use core::ops::AddAssign;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::TraverseVErr;

/// Proof nonce packed into a [u128].
///
/// The upper 64 bits hold the Unix timestamp in seconds from when it was
/// created, and the lower 64 bits hold a counter starting at zero.
#[derive(Clone, Copy)]
pub struct Nonce {
    inner: u128,
}

impl Nonce {
    /// Create a nonce with the current timestamp and a counter of zero
    pub fn new() -> Self {
        let time = Self::timestamp();
        let counter = 0u64;

        Self {
            inner: Self::pack(time, counter),
        }
    }

    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_secs()
    }

    fn validate(time: u64) -> Result<(), TraverseVErr> {
        UNIX_EPOCH
            .checked_add(Duration::from_secs(time))
            .ok_or(TraverseVErr::InvalidNonce)?;
        Ok(())
    }

    fn pack(time: u64, counter: u64) -> u128 {
        (time as u128) << 64 | (counter as u128)
    }

    fn unpack(&self) -> (u64, u64) {
        let time = (self.inner >> 64) as u64;
        let counter = self.inner as u64;

        (time, counter)
    }
}

impl AddAssign<u64> for Nonce {
    /// Performs the `+=` operation on the counter. If the addition would
    /// overflow, the counter restarts at zero under a fresh timestamp instead
    /// of wrapping.
    fn add_assign(&mut self, rhs: u64) {
        let (time, mut counter) = self.unpack();

        if counter <= u64::MAX - rhs {
            counter += rhs;
            self.inner = Self::pack(time, counter);
        } else {
            let time = Self::timestamp();
            let counter = 0u64;
            self.inner = Self::pack(time, counter);
        }
    }
}

impl TryFrom<u128> for Nonce {
    type Error = TraverseVErr;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let time = (value >> 64) as u64;

        Self::validate(time)?;
        Ok(Self { inner: value })
    }
}

impl From<Nonce> for u128 {
    fn from(nonce: Nonce) -> Self {
        nonce.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Error = TraverseVErr;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_secs()
    }

    #[test]
    fn new_uses_current_time_and_zero_counter() {
        let before = now();
        let nonce = Nonce::new();
        let after = now();

        let (time, counter) = nonce.unpack();
        assert!(before <= time && time <= after);
        assert_eq!(counter, 0);
    }

    #[test]
    fn add_assign_increments_counter() {
        let mut nonce = Nonce::new();
        let (time, _) = nonce.unpack();

        nonce += 42;

        assert_eq!(nonce.unpack(), (time, 42));
    }

    #[test]
    fn add_assign_restarts_counter_on_overflow() {
        let time = now();
        let mut nonce = Nonce {
            inner: Nonce::pack(time, u64::MAX),
        };

        let before = now();
        nonce += 1;
        let after = now();

        let (_, counter) = nonce.unpack();
        assert_eq!(counter, 0);
        assert!(before <= time && time <= after);
    }

    #[test]
    fn try_from_accepts_valid_timestamp() {
        let raw = Nonce::pack(now(), 42);
        let result = Nonce::try_from(raw);

        assert!(result.is_ok());
    }

    #[test]
    fn try_from_rejects_unrepresentable_timestamp() {
        let raw = Nonce::pack(u64::MAX, 42);
        let result = Nonce::try_from(raw);

        assert_eq!(result.err(), Some(Error::InvalidNonce));
    }

    #[test]
    fn from_nonce_returns_u128() {
        let raw = Nonce::pack(now(), 42);
        let nonce = Nonce { inner: raw };

        assert_eq!(u128::from(nonce), raw);
    }
}
