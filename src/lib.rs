mod constants;
mod errors;
mod memory;
mod params;
mod traverse;

use blake3::Hasher;
use subtle::{Choice, ConstantTimeEq};

use crate::constants::BLOCK_SIZE;
use crate::memory::{fill, initial_blocks};
pub use crate::params::Params;
use crate::traverse::dependency_chain;

pub struct TraverseV {
    inner: Vec<u8>,
    key: Vec<u8>,
    context: String,
    params: Params,
}

impl TraverseV {
    pub fn new(key: &[u8], context: &str, params: Params) -> Self {
        let q = params.m_cost() as usize;
        let t = params.t_cost() as usize;

        let mut v = vec![0u8; q * BLOCK_SIZE];
        let (b0, b1) = initial_blocks(
            key,
            params.m_cost(),
            params.t_cost(),
            params.d_cost(),
            params.n_cost(),
        );
        v[0..BLOCK_SIZE].copy_from_slice(&b0);
        v[BLOCK_SIZE..2 * BLOCK_SIZE].copy_from_slice(&b1);
        fill(&mut v, q, t);

        Self {
            inner: v,
            key: key.to_vec(),
            context: context.to_string(),
            params,
        }
    }

    pub fn mine(&self) -> u128 {
        let mut nonce = 0u128;

        loop {
            let candidate = self.authenticate(nonce);
            let satisfied = self.check_n(&candidate);

            // Break loop when the constraint is satisfied
            if bool::from(satisfied) {
                return nonce;
            }

            nonce += 1
        }
    }

    pub fn verify(&self, nonce: u128) -> bool {
        let candidate = self.authenticate(nonce);
        let satisfied = self.check_n(&candidate);

        bool::from(satisfied)
    }

    fn authenticate(&self, nonce: u128) -> [u8; 32] {
        let mut hasher = Hasher::new_derive_key(&self.context);
        hasher.update(&self.key);
        let dk: [u8; 32] = hasher.finalize().into();

        let mut hasher = Hasher::new_keyed(&dk);
        hasher.update(&nonce.to_le_bytes());

        let mut x = [0u8; BLOCK_SIZE];
        let mut reader = hasher.finalize_xof();
        reader.fill(&mut x);

        let q = self.params.m_cost() as usize;
        let k = self.params.d_cost() as usize;
        x = dependency_chain(x, &self.inner, q, k);
        blake3::hash(&x).into()
    }

    fn check_n(&self, candidate: &[u8; 32]) -> Choice {
        let n = self.params.n_cost() as u8;
        let bytes = n / 8;
        let bits = n % 8;

        // Verify first N bytes are zero, following N bits are zero
        let mut satisfied = Choice::from(1u8);
        for i in 0..bytes {
            satisfied &= candidate[i as usize].ct_eq(&0u8);
        }
        if bits > 0 {
            let mask = (0xFF << (8 - bits)) as u8;
            satisfied &= (candidate[bytes as usize] & mask).ct_eq(&0u8);
        }

        satisfied
    }
}
