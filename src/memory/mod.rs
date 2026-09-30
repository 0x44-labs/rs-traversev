mod compress;
mod fill;
mod init;

pub use fill::fill;
pub use init::initial_blocks;

#[cfg(test)]
mod tests {
    use argon2::{Algorithm, Argon2, Block, Params, Version};

    use crate::common::{BLOCK_SIZE, WORDS};

    use super::*;

    /// Little-endian serialisation, independent of the module's converters.
    fn to_bytes(words: &[u64]) -> Vec<u8> {
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }

    /// A block of distinct, non-zero words.
    fn words(high: u64) -> [u64; WORDS] {
        core::array::from_fn(|i| (high << 32) | i as u64)
    }

    #[test]
    fn fill_matches_argon2d_first_pass() {
        // With one lane and one pass, Argon2d picks reference blocks exactly
        // as the first pass of `fill` does, so the memory must be identical.
        let params = Params::new(8, 1, 1, None).unwrap();
        let argon2 = Argon2::new(Algorithm::Argon2d, Version::V0x13, params);
        let mut blocks = vec![Block::default(); 8];
        argon2
            .fill_memory(b"password", b"some_salt", &mut blocks)
            .unwrap();

        let expected: Vec<u8> = blocks
            .iter()
            .flat_map(|block| {
                let words: &[u64] = block.as_ref();
                to_bytes(words)
            })
            .collect();

        // Start from Argon2's first two blocks and let `fill` do the rest.
        let mut v = vec![0u8; 8 * BLOCK_SIZE];
        v[..2 * BLOCK_SIZE].copy_from_slice(&expected[..2 * BLOCK_SIZE]);
        fill(&mut v, 8, 1);

        assert_eq!(v, expected);
    }

    #[test]
    fn fill_later_passes_recompute_both_blocks() {
        let b0 = words(1);
        let b1 = words(2);
        let mut v = [to_bytes(&b0), to_bytes(&b1)].concat();

        fill(&mut v, 2, 2);

        // With q = 2 every step has exactly one candidate, so J1 cannot
        // matter. Block 0 wraps to the last block as its predecessor and
        // references B0: G(B1, B0). Block 1 follows the new block 0 and
        // references itself: G(B0', B1).
        let new_b0 = compress::compress(&b1, &b0);
        let new_b1 = compress::compress(&new_b0, &b1);
        assert_eq!(v, [to_bytes(&new_b0), to_bytes(&new_b1)].concat());
    }
}
