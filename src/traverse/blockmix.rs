use salsa20::SalsaCore;
use salsa20::cipher::{StreamCipherCore, consts::U4};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::common::BLOCK_SIZE;

/// The scryptBlockMix Algorithm, specialised for `r = 8`, `128 * r = 1024`.
/// This operates directly on one [BLOCK_SIZE]-sized block with no resizing,
/// with 16 sub-blocks of 64 octets each fixed at compile time.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-4
pub(crate) fn block_mix(b: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    const SUB_BLOCKS: usize = BLOCK_SIZE / 64; // 2r = 16, r = 8

    let mut sub = [[0u8; 64]; SUB_BLOCKS];
    for i in 0..SUB_BLOCKS {
        sub[i].copy_from_slice(&b[i * 64..(i + 1) * 64]);
    }

    let mut x = sub[SUB_BLOCKS - 1];

    let mut y = [[0u8; 64]; SUB_BLOCKS];
    for i in 0..SUB_BLOCKS {
        let mut t = [0u8; 64];
        for k in 0..64 {
            t[k] = x[k] ^ sub[i][k];
        }
        x = salsa(&t);
        y[i] = x;

        #[cfg(feature = "zeroize")]
        t.zeroize();
    }

    let mut out = [0u8; BLOCK_SIZE];
    let mut pos = 0;
    for i in (0..SUB_BLOCKS).step_by(2) {
        out[pos * 64..(pos + 1) * 64].copy_from_slice(&y[i]);
        pos += 1;
    }
    for i in (1..SUB_BLOCKS).step_by(2) {
        out[pos * 64..(pos + 1) * 64].copy_from_slice(&y[i]);
        pos += 1;
    }

    #[cfg(feature = "zeroize")]
    {
        sub.zeroize();
        x.zeroize();
        y.zeroize();
    }

    out
}

/// The Salsa20/8 Core Function.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-3
fn salsa(t: &[u8; 64]) -> [u8; 64] {
    let mut state = [0u32; 16];
    for i in 0..16 {
        #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
        let mut chunk: [u8; 4] = t[i * 4..i * 4 + 4]
            .try_into()
            .expect("slicing at a fixed aligned offset always yields 4 bytes");
        state[i] = u32::from_le_bytes(chunk);
        #[cfg(feature = "zeroize")]
        chunk.zeroize();
    }

    let mut block = [0u8; 64];
    SalsaCore::<U4>::from_raw_state(state)
        .write_keystream_block((&mut block).into());

    #[cfg(feature = "zeroize")]
    state.zeroize();

    block
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

    fn put(block: &mut [u8; BLOCK_SIZE], index: usize, sub: &[u8; 64]) {
        block[index * 64..(index + 1) * 64].copy_from_slice(sub);
    }

    #[test]
    fn matches_rfc_salsa_vector() {
        // With I = INPUT and O = OUTPUT, salsa(I) = O and salsa(0) = 0.
        // Sub-blocks B[0] = 0, B[1] = O ^ I, B[2] = O, B[3..15] = 0 and
        // B[15] = I give the sequence Y = O, O, 0, ..., 0, O.
        let o_xor_i = core::array::from_fn(|k| OUTPUT[k] ^ INPUT[k]);
        let mut b = [0u8; BLOCK_SIZE];
        put(&mut b, 1, &o_xor_i);
        put(&mut b, 2, &OUTPUT);
        put(&mut b, 15, &INPUT);

        // Even-indexed Y come first, then odd-indexed Y, so Y[0], Y[1] and
        // Y[15] land in output sub-blocks 0, 8 and 15.
        let mut expected = [0u8; BLOCK_SIZE];
        put(&mut expected, 0, &OUTPUT);
        put(&mut expected, 8, &OUTPUT);
        put(&mut expected, 15, &OUTPUT);

        assert_eq!(block_mix(&b), expected);
    }
}
