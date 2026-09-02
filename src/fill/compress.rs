//! This file contains logic sourced from RustCrypto/password-hashes@a5df39e.
//! The `compress` function is adapted from the method of the same name from
//! the `Block` struct of argon2/src/block.rs, and the TRUNC constant along
//! with the permutation macros are direct copies of what appears in block.rs.
//!
//! https://github.com/RustCrypto/password-hashes
//!
//! Copyright (c) 2021-2026 The RustCrypto Project Developers
//!
//! Permission is hereby granted, free of charge, to any
//! person obtaining a copy of this software and associated
//! documentation files (the "Software"), to deal in the
//! Software without restriction, including without
//! limitation the rights to use, copy, modify, merge,
//! publish, distribute, sublicense, and/or sell copies of
//! the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following
//! conditions:
//!
//! The above copyright notice and this permission notice
//! shall be included in all copies or substantial portions
//! of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//! ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//! TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//! SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//! OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//! IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.
use core::num::Wrapping;

const BLOCK_SIZE: usize = 1024;

pub const WORDS: usize = BLOCK_SIZE / 8;

const TRUNC: u64 = u32::MAX as u64;

#[rustfmt::skip]
macro_rules! permute_step {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        $a = (Wrapping($a) + Wrapping($b) + (Wrapping(2) * Wrapping(($a & TRUNC) * ($b & TRUNC)))).0;
        $d = ($d ^ $a).rotate_right(32);
        $c = (Wrapping($c) + Wrapping($d) + (Wrapping(2) * Wrapping(($c & TRUNC) * ($d & TRUNC)))).0;
        $b = ($b ^ $c).rotate_right(24);

        $a = (Wrapping($a) + Wrapping($b) + (Wrapping(2) * Wrapping(($a & TRUNC) * ($b & TRUNC)))).0;
        $d = ($d ^ $a).rotate_right(16);
        $c = (Wrapping($c) + Wrapping($d) + (Wrapping(2) * Wrapping(($c & TRUNC) * ($d & TRUNC)))).0;
        $b = ($b ^ $c).rotate_right(63);
    };
}

macro_rules! permute {
    (
        $v0:expr, $v1:expr, $v2:expr, $v3:expr,
        $v4:expr, $v5:expr, $v6:expr, $v7:expr,
        $v8:expr, $v9:expr, $v10:expr, $v11:expr,
        $v12:expr, $v13:expr, $v14:expr, $v15:expr,
    ) => {
        permute_step!($v0, $v4, $v8, $v12);
        permute_step!($v1, $v5, $v9, $v13);
        permute_step!($v2, $v6, $v10, $v14);
        permute_step!($v3, $v7, $v11, $v15);
        permute_step!($v0, $v5, $v10, $v15);
        permute_step!($v1, $v6, $v11, $v12);
        permute_step!($v2, $v7, $v8, $v13);
        permute_step!($v3, $v4, $v9, $v14);
    };
}

/// Compression Function G of RFC 9106 built upon the BLAKE2b-based
/// transformation P.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.5
pub fn compress(rhs: &[u64; WORDS], lhs: &[u64; WORDS]) -> [u64; WORDS] {
    let mut r = [0u64; WORDS];
    for i in 0..WORDS {
        r[i] = rhs[i] ^ lhs[i];
    }

    // Apply permutations rowwise
    let mut q = r;
    for chunk in q.chunks_exact_mut(16) {
        #[rustfmt::skip]
        permute!(
            chunk[0], chunk[1], chunk[2], chunk[3],
            chunk[4], chunk[5], chunk[6], chunk[7],
            chunk[8], chunk[9], chunk[10], chunk[11],
            chunk[12], chunk[13], chunk[14], chunk[15],
        );
    }

    // Apply permutations columnwise
    for i in 0..8 {
        let b = i * 2;

        #[rustfmt::skip]
        permute!(
            q[b], q[b + 1],
            q[b + 16], q[b + 17],
            q[b + 32], q[b + 33],
            q[b + 48], q[b + 49],
            q[b + 64], q[b + 65],
            q[b + 80], q[b + 81],
            q[b + 96], q[b + 97],
            q[b + 112], q[b + 113],
        );
    }

    for i in 0..WORDS {
        q[i] ^= r[i];
    }
    q
}

pub fn bytes_to_words(bytes: &[u8; BLOCK_SIZE]) -> [u64; WORDS] {
    let mut words = [0u64; WORDS];
    for i in 0..WORDS {
        words[i] =
            u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().expect(
                "slicing at a fixed aligned offset always yields 8 bytes",
            ));
    }

    words
}

pub fn words_to_bytes(words: &[u64; WORDS]) -> [u8; BLOCK_SIZE] {
    let mut bytes = [0u8; BLOCK_SIZE];
    for i in 0..WORDS {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&words[i].to_le_bytes());
    }

    bytes
}
