mod constants;
mod errors;
mod memory;
mod params;
mod traverse;

use crate::constants::BLOCK_SIZE;
use crate::memory::{fill, initial_blocks};
use crate::params::Params;

pub struct TraverseV {
    inner: Vec<u8>,
    params: Params,
}

impl TraverseV {
    pub fn new(params: Params) -> Self {
        let q = params.m_cost() as usize;
        let v = vec![0u8; q * BLOCK_SIZE];

        Self { inner: v, params }
    }

    pub fn fill(&mut self, key: &[u8]) {
        let q = self.params.m_cost() as usize;
        let t = self.params.t_cost() as usize;

        let (b0, b1) = initial_blocks(
            key,
            self.params.m_cost(),
            self.params.t_cost(),
            self.params.d_cost(),
            self.params.n_cost(),
        );
        self.inner[0..BLOCK_SIZE].copy_from_slice(&b0);
        self.inner[BLOCK_SIZE..2 * BLOCK_SIZE].copy_from_slice(&b1);

        fill(&mut self.inner, q, t);
    }
}
