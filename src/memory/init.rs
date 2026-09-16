use blake3::Hasher;

use crate::common::BLOCK_SIZE;

/// Compute the starting blocks B0 and B1. Similar to RFC 9106's Lane Starting
/// Blocks and Second Lane Blocks, but specialised for TraverseV lacking a
/// degree of parallelism and use of BLAKE3.
///
/// `B[0] = init(H_0 || LE32(0))`
///
/// `B[1] = init(H_0 || LE32(1))`
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
pub fn initial_blocks(
    secret: &[u8],
    m_cost: u32,
    t_cost: u32,
    d_cost: u32,
    n_cost: u32,
) -> ([u8; BLOCK_SIZE], [u8; BLOCK_SIZE]) {
    let preimage = h_0(secret, m_cost, t_cost, d_cost, n_cost);

    let b0 = init(&preimage, 0);
    let b1 = init(&preimage, 1);

    (b0, b1)
}

/// Hash preimage of the parameters and secret. Delivers a consistent 64 byte
/// preimage despite unbounded secret length to avoid re-hashing potentially
/// large input once per output block with [init].
///
/// Similar to RFC 9106's H_0 Generation but specialised for TraverseV's
/// parameters and with BLAKE3 instead of BLAKE2b.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn h_0(
    secret: &[u8],
    m_cost: u32,
    t_cost: u32,
    d_cost: u32,
    n_cost: u32,
) -> [u8; 64] {
    let mut hasher = Hasher::new();
    hasher.update(&m_cost.to_le_bytes());
    hasher.update(&t_cost.to_le_bytes());
    hasher.update(&d_cost.to_le_bytes());
    hasher.update(&n_cost.to_le_bytes());
    hasher.update(secret);

    let mut out = [0u8; 64];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut out);

    out
}

/// Wrapper for the BLAKE3 XOF, producing one [BLOCK_SIZE]-sized output for
/// initial block computations.
///
/// Replaces RFC 9106's Function H' for Tag and Initial Block Computations.
fn init(preimage: &[u8; 64], index: u32) -> [u8; BLOCK_SIZE] {
    let input = [preimage.as_slice(), &index.to_le_bytes()].concat();

    let mut hasher = Hasher::new();
    hasher.update(&input);

    let mut out = [0u8; BLOCK_SIZE];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut out);

    out
}
