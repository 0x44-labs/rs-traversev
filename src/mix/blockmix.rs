use salsa20::SalsaCore;
use salsa20::cipher::{StreamCipherCore, consts::U4};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::block::Block;

/// The scryptBlockMix Algorithm, specialised for `r = 8`, `128 * r = 1024`.
/// This operates directly on one [BLOCK_SIZE]-sized block with no resizing,
/// with 16 sub-blocks of 8 words each fixed at compile time.
///
/// `dst` holds the input and receives the output. `tmp` is scratch space that
/// is fully overwritten so the call site can reuse one buffer across rounds.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-4
pub(crate) fn block_mix(dst: &mut Block, tmp: &mut Block) {
    const SUB_BLOCKS: usize = Block::SIZE / 64; // 2r = 16, r = 8
    const SUB_WORDS: usize = 8;

    let mut x: [u64; SUB_WORDS] = dst[(SUB_BLOCKS - 1) * SUB_WORDS..]
        .try_into()
        .expect("slicing at a fixed aligned offset always yields 8 words");

    for (i, sub) in dst.chunks_exact(SUB_WORDS).enumerate() {
        for (xk, bk) in x.iter_mut().zip(sub) {
            *xk ^= *bk;
        }
        salsa(&mut x);

        // Even-indexed outputs come first, then odd-indexed outputs.
        let pos = if i % 2 == 0 {
            i / 2
        } else {
            SUB_BLOCKS / 2 + i / 2
        };
        tmp[pos * SUB_WORDS..(pos + 1) * SUB_WORDS].copy_from_slice(&x);
    }

    #[cfg(feature = "zeroize")]
    x.zeroize();

    dst.copy_from(tmp);
}

/// The Salsa20/8 Core Function.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-3
fn salsa(t: &mut [u64; 8]) {
    let mut state = [0u32; 16];
    for (pair, word) in state.chunks_exact_mut(2).zip(t.iter()) {
        pair[0] = *word as u32;
        pair[1] = (*word >> 32) as u32;
    }

    let mut block = [0u8; 64];
    SalsaCore::<U4>::from_raw_state(state)
        .write_keystream_block((&mut block).into());

    for (word, chunk) in t.iter_mut().zip(block.chunks_exact(8)) {
        let mut c: [u8; 8] = chunk
            .try_into()
            .expect("slicing at a fixed aligned offset always yields 8 bytes");
        *word = u64::from_le_bytes(c);
        #[cfg(feature = "zeroize")]
        c.zeroize();
    }

    #[cfg(feature = "zeroize")]
    {
        state.zeroize();
        block.zeroize();
    }
}

#[cfg(feature = "zeroize")]
const _: () = {
    const fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}
    assert_zeroize_on_drop::<SalsaCore<U4>>();
};

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7914 section 8, Salsa20/8 Core INPUT test vector.
    ///
    /// https://www.rfc-editor.org/info/rfc7914/#section-8
    #[rustfmt::skip]
    const INPUT: [u8; 64] = [
        0x7e, 0x87, 0x9a, 0x21, 0x4f, 0x3e, 0xc9, 0x86,
        0x7c, 0xa9, 0x40, 0xe6, 0x41, 0x71, 0x8f, 0x26,
        0xba, 0xee, 0x55, 0x5b, 0x8c, 0x61, 0xc1, 0xb5,
        0x0d, 0xf8, 0x46, 0x11, 0x6d, 0xcd, 0x3b, 0x1d,
        0xee, 0x24, 0xf3, 0x19, 0xdf, 0x9b, 0x3d, 0x85,
        0x14, 0x12, 0x1e, 0x4b, 0x5a, 0xc5, 0xaa, 0x32,
        0x76, 0x02, 0x1d, 0x29, 0x09, 0xc7, 0x48, 0x29,
        0xed, 0xeb, 0xc6, 0x8d, 0xb8, 0xb8, 0xc2, 0x5e,
    ];

    /// RFC 7914 section 8, Salsa20/8 Core OUTPUT test vector.
    ///
    /// https://www.rfc-editor.org/info/rfc7914/#section-8
    #[rustfmt::skip]
    const OUTPUT: [u8; 64] = [
        0xa4, 0x1f, 0x85, 0x9c, 0x66, 0x08, 0xcc, 0x99,
        0x3b, 0x81, 0xca, 0xcb, 0x02, 0x0c, 0xef, 0x05,
        0x04, 0x4b, 0x21, 0x81, 0xa2, 0xfd, 0x33, 0x7d,
        0xfd, 0x7b, 0x1c, 0x63, 0x96, 0x68, 0x2f, 0x29,
        0xb4, 0x39, 0x31, 0x68, 0xe3, 0xc9, 0xe6, 0xbc,
        0xfe, 0x6b, 0xc5, 0xb7, 0xa0, 0x6d, 0x96, 0xba,
        0xe4, 0x24, 0xcc, 0x10, 0x2c, 0x91, 0x74, 0x5c,
        0x24, 0xad, 0x67, 0x3d, 0xc7, 0x61, 0x8f, 0x81,
    ];

    fn put(block: &mut [u8; Block::SIZE], index: usize, sub: &[u8; 64]) {
        block[index * 64..(index + 1) * 64].copy_from_slice(sub);
    }

    #[test]
    fn matches_rfc_salsa_vector() {
        // With I = INPUT and O = OUTPUT, salsa(I) = O and salsa(0) = 0.
        // Sub-blocks B[0] = 0, B[1] = O ^ I, B[2] = O, B[3..15] = 0 and
        // B[15] = I give the sequence Y = O, O, 0, ..., 0, O.
        let o_xor_i = core::array::from_fn(|k| OUTPUT[k] ^ INPUT[k]);
        let mut b = [0u8; Block::SIZE];
        put(&mut b, 1, &o_xor_i);
        put(&mut b, 2, &OUTPUT);
        put(&mut b, 15, &INPUT);

        // Even-indexed Y come first, then odd-indexed Y, so Y[0], Y[1] and
        // Y[15] land in output sub-blocks 0, 8 and 15.
        let mut e = [0u8; Block::SIZE];
        put(&mut e, 0, &OUTPUT);
        put(&mut e, 8, &OUTPUT);
        put(&mut e, 15, &OUTPUT);

        let mut dst = Block::new();
        dst.copy_from_bytes(&b);
        let mut tmp = Block::new();
        block_mix(&mut dst, &mut tmp);

        let mut expected = Block::new();
        expected.copy_from_bytes(&e);
        assert_eq!(dst, expected);
    }
}
