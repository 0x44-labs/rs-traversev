use core::ops::AddAssign;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Preimage {
    time: u64,
    nonce: u64,
    data: Vec<u8>,
}

impl Preimage {
    pub fn new(data: &[u8]) -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_secs();

        Self {
            time: time,
            nonce: 0u64,
            data: data.to_vec(),
        }
    }
}

impl AddAssign<u64> for Preimage {
    fn add_assign(&mut self, rhs: u64) {
        if self.nonce <= u64::MAX - rhs {
            self.nonce += rhs;
        } else {
            self.nonce = 0u64;
            self.time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_secs();
        }
    }
}
