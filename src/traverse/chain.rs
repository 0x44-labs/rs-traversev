use crate::traverse::blockmix::block_mix;

const BLOCK_SIZE: usize = 1024;

/// The scryptROMix algorithm's second loop. The loop runs for `k` rounds
/// and is independent of q, compared to RFC 7914 sizing both the array and
/// iteration count from N.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-5
pub(crate) fn dependency_chain(
    mut x: [u8; BLOCK_SIZE],
    v: &[u8],
    q: usize,
    k: usize,
) -> [u8; BLOCK_SIZE] {
    for _ in 0..k {
        let j = (integerify(&x) % q as u64) as usize;

        let mut t = [0u8; BLOCK_SIZE];
        for k in 0..BLOCK_SIZE {
            t[k] = x[k] ^ v[j * BLOCK_SIZE + k];
        }

        x = block_mix(&t);
    }

    x
}

/// Integerify, narrowed to the last 8 octets of X rather than the full last
/// 64-octet sub-block. q is not constrained to a power of two the way N is,
/// and scryptBlockMix's Salsa20/8 core binds every output byte to the full
/// input regardless of width.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-5
fn integerify(x: &[u8; BLOCK_SIZE]) -> u64 {
    u64::from_le_bytes(
        x[BLOCK_SIZE - 8..]
            .try_into()
            .expect("slicing at a fixed aligned offset always yields 8 bytes"),
    )
}
