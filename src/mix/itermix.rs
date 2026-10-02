#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::block::Block;
use crate::mix::blockmix::block_mix;

/// The scryptROMix algorithm's second loop, computed in place on `x`. The
/// loop runs for `k` rounds and is independent of q, compared to RFC 7914
/// sizing both the array and iteration count from N.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-5
pub fn iter_mix(x: &mut Block, v: &[Block], q: usize, k: usize) {
    let mut tmp = Block::new();

    for _ in 0..k {
        let j = (integerify(x) % q as u64) as usize;

        *x ^= &v[j];
        block_mix(x, &mut tmp);
    }

    #[cfg(feature = "zeroize")]
    tmp.zeroize();
}

/// Integerify, narrowed to the last 8 octets of X rather than the full last
/// 64-octet sub-block. q is not constrained to a power of two the way N is,
/// and scryptBlockMix's Salsa20/8 core binds every output byte to the full
/// input regardless of width.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-5
fn integerify(x: &Block) -> u64 {
    x[Block::WORDS - 1]
}

#[cfg(test)]
mod tests {
    use crate::block::Block;

    use super::*;

    /// `q` blocks, with block `b` filled with the byte `b + 1`.
    fn memory(q: usize) -> Vec<Block> {
        (0..q)
            .map(|b| {
                let mut block = Block::new();
                block.copy_from_bytes(&[b as u8 + 1; Block::SIZE]);
                block
            })
            .collect()
    }

    fn mixed(mut x: Block, v: &[Block], q: usize, k: usize) -> Block {
        iter_mix(&mut x, v, q, k);
        x
    }

    #[test]
    fn single_round_selects_block_from_last_octets() {
        // The last 8 octets encode 10 little-endian, so j = 10 % 7 = 3.
        // A mask would give 2, a big-endian read 5, and reading the first
        // 8 octets (11) would give 4.
        let v = memory(7);
        let mut bytes = [0u8; Block::SIZE];
        bytes[..8].copy_from_slice(&11u64.to_le_bytes());
        bytes[Block::SIZE - 8..].copy_from_slice(&10u64.to_le_bytes());
        let mut x = Block::new();
        x.copy_from_bytes(&bytes);

        let mut dst = x.clone();
        dst ^= &v[3];
        let mut tmp = Block::new();
        block_mix(&mut dst, &mut tmp);

        assert_eq!(mixed(x, &v, 7, 1), dst);
    }

    #[test]
    fn state_carries_between_rounds() {
        let v = memory(7);
        let mut bytes = [0u8; Block::SIZE];
        bytes[Block::SIZE - 8..].copy_from_slice(&10u64.to_le_bytes());
        let mut x = Block::new();
        x.copy_from_bytes(&bytes);

        let once = mixed(x.clone(), &v, 7, 1);

        assert_eq!(mixed(x, &v, 7, 2), mixed(once, &v, 7, 1));
    }
}
