use crate::constants::{BLOCK_SIZE, WORDS};
use crate::memory::compress::{bytes_to_words, compress, words_to_bytes};

/// Fill every block of V after the initial two blocks (RFC 9106  Further Block
/// Generation and Further Passes), specialised for a single lane.
///
/// The RFC computes this once per lane in parallel where lanes do not depend
/// n each other's work. With a single lane, there is only one sequence of
/// blocks to fill, so this is a single loop over the whole array.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
pub fn fill(v: &mut [u8], q: usize, t: usize) {
    let mut words = vec![0u64; q * WORDS];
    for i in 0..q {
        let block_bytes: [u8; BLOCK_SIZE] = v
            [i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]
            .try_into()
            .expect("v is laid out in fixed BLOCK_SIZE blocks");
        words[i * WORDS..(i + 1) * WORDS]
            .copy_from_slice(&bytes_to_words(&block_bytes));
    }

    for pass in 0..t {
        let start = if pass == 0 { 2 } else { 0 };
        for j in start..q {
            fill_block(&mut words, q, pass, j);
        }
    }

    for i in 0..q {
        let block_words: [u64; WORDS] = words[i * WORDS..(i + 1) * WORDS]
            .try_into()
            .expect("words is laid out in fixed WORDS-sized blocks");
        v[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]
            .copy_from_slice(&words_to_bytes(&block_words));
    }
}

/// Compute a single block of V as the compression of the block at [prev_index]
/// with the block at [reference_index] (RFC 9106 Further Block Generation).
///
/// After the first pass the RFC XORs this result back into the block's
/// existing value; this function always overwrites.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn fill_block(v: &mut [u64], q: usize, pass: usize, j: usize) {
    let prev = prev_index(pass, j, q);
    let prev_words: [u64; WORDS] = v[prev * WORDS..(prev + 1) * WORDS]
        .try_into()
        .expect("v is laid out in fixed WORDS-sized blocks");

    let len = w_len(pass, j, q);
    let z = reference_index(prev, len, &prev_words);
    let ref_words: [u64; WORDS] = v[z * WORDS..(z + 1) * WORDS]
        .try_into()
        .expect("v is laid out in fixed WORDS-sized blocks");

    let result = compress(&prev_words, &ref_words);
    v[j * WORDS..(j + 1) * WORDS].copy_from_slice(&result);
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
/// parllel without referencing each other's work. With a single lane there is
/// nothing to protect against, so the whole array is used instead.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn w_len(pass: usize, j: usize, q: usize) -> usize {
    if pass == 0 { j - 1 } else { q - 1 }
}

/// Reference block index. RFC 9106 pairs this reference block with the lane it
/// belongs to; with a single lane this is alwys the same (see [j1]), so only a
/// block's position within the array is returned.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.1.1
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.2
fn reference_index(
    prev: usize,
    len: usize,
    prev_words: &[u64; WORDS],
) -> usize {
    let pos = select(len, j1(prev_words));
    w_index(prev, pos)
}

/// J1 (RFC 9106 Deriving J1, J2 in Argon2d). J2 is not computed as the RFC
/// only uses it to select a lane, and a single lane makes that selection
/// always 0.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.4.1.1
fn j1(prev: &[u64; WORDS]) -> u32 {
    prev[0] as u32
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
