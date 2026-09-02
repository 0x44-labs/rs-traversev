mod blockmix;
mod chain;

use blake3::Hasher;

const BLOCK_SIZE: usize = 1024;

pub(crate) fn authenticate(
    key: &[u8; 32],
    nonce: u128,
    v: &[u8],
    q: usize,
    k: usize,
) -> [u8; 32] {
    let mut hasher = Hasher::new_keyed(key);
    hasher.update(&nonce.to_le_bytes());

    let mut x = [0u8; BLOCK_SIZE];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut x);

    x = chain::dependency_chain(x, v, q, k);
    blake3::hash(&x).into()
}
