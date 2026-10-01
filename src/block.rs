#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::common::{BLOCK_SIZE, WORDS};

/// One block of memory as 128 little-endian `u64` words.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Block {
    inner: [u64; WORDS],
}

impl Block {
    /// Read a block from its little-endian byte representation.
    pub(crate) fn from_bytes(bytes: &[u8; BLOCK_SIZE]) -> Self {
        let mut words = [0u64; WORDS];

        for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(8)) {
            #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
            let mut c: [u8; 8] = chunk.try_into().expect(
                "slicing at a fixed aligned offset always yields 8 bytes",
            );
            *word = u64::from_le_bytes(c);

            #[cfg(feature = "zeroize")]
            c.zeroize();
        }

        Self { inner: words }
    }

    /// Write the block as its little-endian byte representation.
    pub(crate) fn to_bytes(&self) -> [u8; BLOCK_SIZE] {
        let mut bytes = [0u8; BLOCK_SIZE];

        for (chunk, word) in bytes.chunks_exact_mut(8).zip(&self.inner) {
            #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
            let mut c = word.to_le_bytes();
            chunk.copy_from_slice(&c);

            #[cfg(feature = "zeroize")]
            c.zeroize();
        }

        bytes
    }

    pub(crate) const fn from_words(words: [u64; WORDS]) -> Self {
        Self { inner: words }
    }

    pub(crate) const fn as_words(&self) -> &[u64; WORDS] {
        &self.inner
    }

    /// Overwrite this block with another.
    pub(crate) fn copy_from(&mut self, rhs: &Block) {
        self.inner.copy_from_slice(&rhs.inner);
    }

    /// J1 (RFC 9106 Deriving J1, J2 in Argon2d): the low 32 bits of the first
    /// word. J2 is not computed as the RFC only uses it to select a lane, and
    /// a single lane makes that selection always 0.
    ///
    /// https://www.rfc-editor.org/info/rfc9106/#section-3.4.1.1
    pub(crate) fn j1(&self) -> u32 {
        self.inner[0] as u32
    }
}

#[cfg(feature = "zeroize")]
impl Zeroize for Block {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes() -> [u8; BLOCK_SIZE] {
        core::array::from_fn(|i| (i * 7 + 3) as u8)
    }

    #[test]
    fn bytes_round_trip() {
        let b = bytes();
        assert_eq!(Block::from_bytes(&b).to_bytes(), b);
    }

    #[test]
    fn words_are_little_endian() {
        let b = bytes();
        let block = Block::from_bytes(&b);
        for i in 0..WORDS {
            #[rustfmt::skip]
            let expected = u64::from_le_bytes(
                b[i * 8..(i + 1) * 8].try_into().unwrap(),
            );
            assert_eq!(block.as_words()[i], expected);
        }
    }

    #[test]
    fn j1_is_first_four_octets_little_endian() {
        let b = bytes();
        let expected = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(Block::from_bytes(&b).j1(), expected);
    }
}
