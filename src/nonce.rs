use core::ops::AddAssign;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::TraverseVErr;

pub struct Nonce {
    inner: u128,
}

impl Nonce {
    pub fn new() -> Self {
        let time = Self::timestamp();
        let nonce = 0u64;

        Self {
            inner: Self::pack(time, nonce),
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

    fn pack(time: u64, nonce: u64) -> u128 {
        (time as u128) << 64 | (nonce as u128)
    }

    fn unpack(&self) -> (u64, u64) {
        let time = (self.inner >> 64) as u64;
        let nonce = self.inner as u64;

        (time, nonce)
    }
}

impl AddAssign<u64> for Nonce {
    fn add_assign(&mut self, rhs: u64) {
        let (time, mut nonce) = self.unpack();

        if nonce <= u64::MAX - rhs {
            nonce += rhs;
            self.inner = Self::pack(time, nonce);
        } else {
            let time = Self::timestamp();
            let nonce = 0u64;
            self.inner = Self::pack(time, nonce);
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
