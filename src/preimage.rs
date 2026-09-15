use core::ops::AddAssign;
use std::time::SystemTime;

pub struct Preimage {
    time: SystemTime,
    nonce: u128,
    data: Vec<u8>,
}

impl Preimage {
    pub fn new(data: &[u8]) -> Self {
        Self {
            time: SystemTime::now(),
            nonce: 0u128,
            data: data.to_vec(),
        }
    }
}

impl AddAssign<u128> for Preimage {
    fn add_assign(&mut self, rhs: u128) {
        if self.nonce <= u128::MAX - rhs {
            self.nonce += rhs;
        } else {
            self.nonce = 0u128;
            self.time = SystemTime::now();
        }
    }
}
