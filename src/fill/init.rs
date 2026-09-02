use blake2::{Blake2b512, Digest};

const BLOCK_SIZE: usize = 1024;

/// Compute the starting blocks B0 and B1 (RFC 9106 Lane Starting Blocks and
/// Second Lane Blocks) specialised for TraverseV lacking a degree of
/// parallelism.
///
/// `B[0] = H'^(1024)(H_0 || LE32(0))`
///
/// `B[1] = H'^(1024)(H_0 || LE32(1))`
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
pub(crate) fn starting_blocks(
    key: &[u8],
    m_cost: u32,
    t_cost: u32,
) -> ([u8; BLOCK_SIZE], [u8; BLOCK_SIZE]) {
    let preimage = h_0(key, m_cost, t_cost);

    let b0 = h_prime(&[preimage.as_slice(), &0u32.to_le_bytes()].concat());
    let b1 = h_prime(&[preimage.as_slice(), &1u32.to_le_bytes()].concat());

    (b0, b1)
}

/// Hash preimage of the parameters and key (RFC 9106 H_0 Generation)
/// specialised for TraverseV's parameters.
///
/// The key is not length prefixed as no salt is included in the preimage.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn h_0(key: &[u8], m_cost: u32, t_cost: u32) -> [u8; 64] {
    let mut hasher = Blake2b512::new();
    hasher.update(m_cost.to_le_bytes());
    hasher.update(t_cost.to_le_bytes());
    hasher.update(key);

    hasher.finalize().into()
}

/// Variable-length hash function (RFC 9106 Function H' for Tag and Initial
/// Block Computations) specialised for `T = BLOCK_SIZE`.
///
/// Every call in the chain is an untruncated [Blake2b512] output. Blake2b's
/// output length is folded into its internal state. A truncated Blake2b-512
/// is not interchangeable with a shorter Blake2b digest computed directly.
///
/// For `T = 1024` this specialisation produces output identical to the RFC's
/// Function H'.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.3
fn h_prime(input: &[u8]) -> [u8; BLOCK_SIZE] {
    let r = (BLOCK_SIZE + 31) / 32 - 2;

    let mut out = [0u8; BLOCK_SIZE];
    let mut offset = 0;

    let mut v: [u8; 64] = Blake2b512::new()
        .chain_update((BLOCK_SIZE as u32).to_le_bytes())
        .chain_update(input)
        .finalize()
        .into();
    out[offset..offset + 32].copy_from_slice(&v[..32]);
    offset += 32;

    for _ in 1..r {
        v = Blake2b512::new().chain_update(v).finalize().into();
        out[offset..offset + 32].copy_from_slice(&v[..32]);
        offset += 32;
    }

    let last: [u8; 64] = Blake2b512::new().chain_update(v).finalize().into();
    out[offset..offset + 64].copy_from_slice(&last);

    out
}
