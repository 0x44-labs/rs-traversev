#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::common::BLOCK_SIZE;
use crate::traverse::blockmix::block_mix;

/// The scryptROMix algorithm's second loop. The loop runs for `k` rounds
/// and is independent of q, compared to RFC 7914 sizing both the array and
/// iteration count from N.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-5
pub fn dependency_chain(
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

        #[cfg(feature = "zeroize")]
        t.zeroize();
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
    #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
    let mut chunk: [u8; 8] = x[BLOCK_SIZE - 8..]
        .try_into()
        .expect("slicing at a fixed aligned offset always yields 8 bytes");
    let j = u64::from_le_bytes(chunk);

    #[cfg(feature = "zeroize")]
    chunk.zeroize();

    j
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `q` blocks, with block `b` filled with the byte `b + 1`.
    fn memory(q: usize) -> Vec<u8> {
        (0..q).flat_map(|b| [b as u8 + 1; BLOCK_SIZE]).collect()
    }

    #[test]
    fn single_round_selects_block_from_last_octets() {
        // The last 8 octets encode 10 little-endian, so j = 10 % 7 = 3.
        // A mask would give 2, a big-endian read 5, and reading the first
        // 8 octets (11) would give 4.
        let v = memory(7);
        let mut x = [0u8; BLOCK_SIZE];
        x[..8].copy_from_slice(&11u64.to_le_bytes());
        x[BLOCK_SIZE - 8..].copy_from_slice(&10u64.to_le_bytes());

        let t = core::array::from_fn(|i| x[i] ^ v[3 * BLOCK_SIZE + i]);

        assert_eq!(dependency_chain(x, &v, 7, 1), block_mix(&t));
    }

    #[test]
    fn state_carries_between_rounds() {
        let v = memory(7);
        let mut x = [0u8; BLOCK_SIZE];
        x[BLOCK_SIZE - 8..].copy_from_slice(&10u64.to_le_bytes());

        let once = dependency_chain(x, &v, 7, 1);

        assert_eq!(
            dependency_chain(x, &v, 7, 2),
            dependency_chain(once, &v, 7, 1)
        );
    }
}
