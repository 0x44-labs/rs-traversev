mod constants;
mod errors;
mod memory;
mod nonce;
mod params;
mod traverse;

use blake3::Hasher;
use subtle::{Choice, ConstantTimeEq};

use crate::constants::BLOCK_SIZE;
use crate::memory::{fill, initial_blocks};
use crate::nonce::Nonce;
pub use crate::params::Params;
use crate::traverse::dependency_chain;

/// TraverseV proof-of-work instance.
///
/// Construction fills a memory buffer using a fill adapted from Argon2d. Each
/// block is derived from its predecessor and a pseudorandomly selected earlier
/// block via Argon2's BLAKE2b-based compression function, repeated over
/// multiple passes.
///
/// Mining iteratively searches for a nonce that satisfies this instance's
/// difficulty. Each candidate nonce is authenticated by applying rounds of
/// the scryptROMix algorithm's second loop, mixing memory blocks into the
/// computation via Salsa20/8-based BlockMix.
pub struct TraverseV {
    inner: Vec<u8>,
    secret: Vec<u8>,
    context: String,
    params: Params,
}

impl TraverseV {
    /// Build a new `TraverseV` instance.
    ///
    /// Creates and fills a memory buffer, and stores this instance's
    /// secret, application context, and parameters.
    pub fn new(secret: &[u8], context: &str, params: Params) -> Self {
        let q = params.m_cost() as usize;
        let t = params.t_cost() as usize;

        let mut v = vec![0u8; q * BLOCK_SIZE];
        let (b0, b1) = initial_blocks(
            secret,
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
            secret: secret.to_vec(),
            context: context.to_string(),
            params,
        }
    }

    /// Mine for a nonce value satisfying this instance's difficulty.
    ///
    /// Iteratively searches for candidates until one satisfies the difficulty.
    /// Runtime is unbounded on high difficulties.
    pub fn mine(&self, input: &[u8]) -> u128 {
        let mut nonce = Nonce::new();

        loop {
            let nonce_u128 = u128::from(nonce);

            let candidate = self.authenticate(input, nonce_u128);
            let satisfied = self.check_n(&candidate);

            // Break loop when the constraint is satisfied
            if bool::from(satisfied) {
                return nonce_u128;
            }

            nonce += 1
        }
    }

    /// Check if a nonce value satisfies this instance's difficulty.
    pub fn verify(&self, input: &[u8], nonce: u128) -> bool {
        let candidate = self.authenticate(input, nonce);
        let satisfied = self.check_n(&candidate);

        bool::from(satisfied)
    }

    fn authenticate(&self, input: &[u8], nonce: u128) -> [u8; 32] {
        let mut hasher = Hasher::new_derive_key(&self.context);
        hasher.update(&self.secret);
        let key: [u8; 32] = hasher.finalize().into();

        let mut hasher = Hasher::new_keyed(&key);
        hasher.update(&u128::from(nonce).to_le_bytes());
        hasher.update(&input);

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
