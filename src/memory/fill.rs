#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::block::Block;
use crate::memory::{
    compress::compress,
    init::{Mode, initial_blocks},
};
use crate::params::Params;

/// Allocate the memory buffer V of `m_cost` blocks and fill it.
///
/// The starting blocks B0 and B1 are derived from the mode, parameters,
/// context, and optional secret. The starting blocks are placed at the start
/// of V, and the remaining blocks are computed over `t_cost` passes by [fill].
pub fn build_buffer(
    mode: Mode,
    secret: Option<&[u8]>,
    context: &str,
    params: Params,
) -> Vec<Block> {
    let q = params.m_cost() as usize;
    let t = params.t_cost() as usize;

    let mut v = vec![Block::new(); q];
    let dst = v.first_chunk_mut::<2>().expect("m_cost is at least 2");
    initial_blocks(dst, mode, params, context, secret);

    fill(&mut v, q, t);

    v
}

/// Fill every block of V after the initial two blocks (RFC 9106 Further Block
/// Generation and Further Passes), specialised for a single lane.
///
/// The RFC computes this once per lane in parallel where lanes do not depend
/// on each other's work. With a single lane, there is only one sequence of
/// blocks to fill, so this is a single loop over the whole array.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn fill(v: &mut [Block], q: usize, t: usize) {
    for pass in 0..t {
        let start = if pass == 0 { 2 } else { 0 };
        for j in start..q {
            fill_block(v, q, pass, j);
        }
    }
}

/// Compute a single block of V as the compression of the block at [prev_index]
/// with the block at [reference_index] (RFC 9106 Further Block Generation).
///
/// After the first pass the RFC XORs this result back into the block's
/// existing value; this function always overwrites.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn fill_block(v: &mut [Block], q: usize, pass: usize, j: usize) {
    let prev = prev_index(pass, j, q);
    let len = w_len(pass, j, q);
    let z = reference_index(prev, len, &v[prev]);

    let mut r = &v[prev] ^ &v[z];
    v[j].copy_from(&r);
    compress(&mut v[j], &r);

    #[cfg(feature = "zeroize")]
    r.zeroize();
}

/// Index of the block immediately preceding the one being computed.
///
/// `B[i][j-1]` and `B[i][q-1]` of RFC 9106 Further Block Generation
/// and Further Passes.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn prev_index(pass: usize, j: usize, q: usize) -> usize {
    if pass > 0 && j == 0 { q - 1 } else { j - 1 }
}

/// Size of candidate set W (RFC 9106 Mapping J_1 and J_2 to Reference Block
/// Index), specialised for a single lane.
///
/// The RFC restricts W to a few segments to allow multiple lanes to compute in
/// parallel without referencing each other's work. With a single lane there is
/// nothing to protect against, so the whole array is used instead.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn w_len(pass: usize, j: usize, q: usize) -> usize {
    if pass == 0 { j - 1 } else { q - 1 }
}

/// Reference block index. RFC 9106 pairs this reference block with the lane it
/// belongs to; with a single lane this is always the same (see [Block::j1]),
/// so only a block's position within the array is returned.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.1.1
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn reference_index(prev: usize, len: usize, prev_block: &Block) -> usize {
    let pos = select(len, prev_block.j1());
    w_index(prev, pos)
}

/// Position within candidate set W selected by J1 (RFC 9106 Computing J1 and
/// Computing J1, Part 2).
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn select(w_len: usize, j1: u32) -> usize {
    let x = (u64::from(j1) * u64::from(j1)) >> 32;
    let y = (w_len as u64 * x) >> 32;
    w_len - 1 - y as usize
}

/// Translate a position within candidate set W into the block index it refers
/// to. RFC 9106 does not give this mapping explicitly since it treats W only
/// as an abstract set.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn w_index(prev: usize, pos: usize) -> usize {
    if pos < prev { pos } else { pos + 1 }
}

#[cfg(test)]
mod tests {
    use argon2::{
        Algorithm, Argon2, Block as ArgonBlock, Params as ArgonParams, Version,
    };

    use crate::block::Block;
    use crate::params::Params;

    use super::*;

    const CONTEXT: &str = "TRAVERSEV_TEST";
    const SECRET: &[u8] = b"This is a secret.";

    #[test]
    fn runs_passes_over_seeded_memory() {
        let g = |x: &Block, y: &Block| -> Block {
            let r = x ^ y;
            let mut q = r.clone();
            compress(&mut q, &r);
            q
        };

        for (mode, secret) in
            [(Mode::Trustless, None), (Mode::Permissioned, Some(SECRET))]
        {
            // One pass: the initial blocks are untouched and every later block
            // is the compression of its predecessor with some earlier block.
            let params =
                Params::new(8, 1, 1, 1).expect("test parameters are valid");

            let mut dst = [Block::new(), Block::new()];
            initial_blocks(&mut dst, mode, params, CONTEXT, secret);
            let v = build_buffer(mode, secret, CONTEXT, params);

            assert_eq!(v.len(), 8);
            assert_eq!(v[0], dst[0]);
            assert_eq!(v[1], dst[1]);
            for j in 2..v.len() {
                assert!((0..j - 1).any(|i| g(&v[j - 1], &v[i]) == v[j]));
            }

            // Two passes: the second pass overwrites both seeds.
            let params =
                Params::new(8, 2, 1, 1).expect("test parameters are valid");
            let mut dst = [Block::new(), Block::new()];
            initial_blocks(&mut dst, mode, params, CONTEXT, secret);
            let v = build_buffer(mode, secret, CONTEXT, params);

            assert_ne!(v[0], dst[0]);
            assert_ne!(v[1], dst[1]);
        }
    }

    #[test]
    fn matches_argon2d_first_pass() {
        // With one lane and one pass, Argon2d picks reference blocks exactly
        // as the first pass of `fill` does, so the memory must be identical.
        let params = ArgonParams::new(8, 1, 1, None).unwrap();
        let argon2 = Argon2::new(Algorithm::Argon2d, Version::V0x13, params);
        let mut blocks = vec![ArgonBlock::default(); 8];
        argon2
            .fill_memory(b"password", b"some_salt", &mut blocks)
            .unwrap();

        let expected: Vec<Block> = blocks
            .iter()
            .map(|block| {
                let words: &[u64] = block.as_ref();
                let mut b = Block::new();
                b.copy_from_slice(words);
                b
            })
            .collect();

        // Start from Argon2's first two blocks and let `fill` do the rest.
        let mut v = vec![Block::new(); 8];
        v[0].copy_from(&expected[0]);
        v[1].copy_from(&expected[1]);
        fill(&mut v, 8, 1);

        assert_eq!(v, expected);
    }

    #[test]
    fn later_passes_wrap_and_self_ref() {
        // With q = 2 J1 cannot matter. Block 0 wraps to the last block as its
        // predecessor and references itself: G(B1, B0). Block 1 follows the
        // new block 0 and references itself: G(B0', B1).
        let mut b0 = Block::new();
        b0.fill(1);
        let mut b1 = Block::new();
        b1.fill(2);

        let mut v = vec![b0.clone(), b1.clone()];
        fill(&mut v, 2, 2);

        let r = &b1 ^ &b0;
        let mut new_b0 = r.clone();
        compress(&mut new_b0, &r);

        let r = &new_b0 ^ &b1;
        let mut new_b1 = r.clone();
        compress(&mut new_b1, &r);

        assert_eq!(v, [new_b0, new_b1]);
    }
}
