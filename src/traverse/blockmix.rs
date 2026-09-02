use salsa20::SalsaCore;
use salsa20::cipher::{StreamCipherCore, consts::U4};

const BLOCK_SIZE: usize = 1024;

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

    out
}

/// The Salsa20/8 Core Function.
///
/// https://www.rfc-editor.org/info/rfc7914/#section-3
fn salsa(t: &[u8; 64]) -> [u8; 64] {
    let mut state = [0u32; 16];
    for i in 0..16 {
        state[i] =
            u32::from_le_bytes(t[i * 4..i * 4 + 4].try_into().expect(
                "slicing at a fixed aligned offset always yields 4 bytes",
            ))
    }

    let mut block = [0u8; 64];
    SalsaCore::<U4>::from_raw_state(state)
        .write_keystream_block((&mut block).into());
    block
}
