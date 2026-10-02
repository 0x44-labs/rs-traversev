use std::ops::{BitXor, BitXorAssign, Deref, DerefMut};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// One block of memory as 128 little-endian `u64` words.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub struct Block([u64; Self::WORDS]);

impl Block {
    /// TraverseV block size.
    pub const SIZE: usize = 1024;

    /// Words per TraverseV block.
    pub const WORDS: usize = Self::SIZE / 8;

    /// Create a new block of zero words.
    pub(crate) const fn new() -> Self {
        Self([0u64; Self::WORDS])
    }

    /// Read a block from its little-endian byte representation, in place.
    pub(crate) fn copy_from_bytes(&mut self, bytes: &[u8; Self::SIZE]) {
        for (word, chunk) in self.0.iter_mut().zip(bytes.chunks_exact(8)) {
            let mut c: [u8; 8] = chunk.try_into().expect(
                "slicing at a fixed aligned offset always yields 8 bytes",
            );
            *word = u64::from_le_bytes(c);

            #[cfg(feature = "zeroize")]
            c.zeroize();
        }
    }

    /// Write the block as its little-endian byte representation, in place.
    pub(crate) fn copy_to_bytes(&self, bytes: &mut [u8; Self::SIZE]) {
        for (chunk, word) in bytes.chunks_exact_mut(8).zip(&self.0) {
            let mut c = word.to_le_bytes();
            chunk.copy_from_slice(&c);

            #[cfg(feature = "zeroize")]
            c.zeroize();
        }
    }

    /// Overwrite this block with another.
    pub(crate) fn copy_from(&mut self, rhs: &Block) {
        self.0.copy_from_slice(&rhs.0);
    }

    /// J1 (RFC 9106 Deriving J1, J2 in Argon2d): the low 32 bits of the first
    /// word. J2 is not computed as the RFC only uses it to select a lane, and
    /// a single lane makes that selection always 0.
    ///
    /// https://www.rfc-editor.org/info/rfc9106/#section-3.4.1.1
    pub(crate) fn j1(&self) -> u32 {
        self.0[0] as u32
    }
}

impl BitXorAssign<&Block> for Block {
    fn bitxor_assign(&mut self, rhs: &Block) {
        for (a, b) in self.0.iter_mut().zip(&rhs.0) {
            *a ^= *b;
        }
    }
}

impl BitXor<&Block> for &Block {
    type Output = Block;

    fn bitxor(self, rhs: &Block) -> Self::Output {
        Block(core::array::from_fn(|i| self.0[i] ^ rhs.0[i]))
    }
}

impl Deref for Block {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "zeroize")]
impl Zeroize for Block {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes() -> [u8; Block::SIZE] {
        core::array::from_fn(|i| (i + (i >> 8)) as u8)
    }

    #[test]
    fn bytes_round_trip() {
        let b = bytes();
        let mut block = Block([u64::MAX; Block::WORDS]);
        block.copy_from_bytes(&b);

        let mut out = [0xFF; Block::SIZE];
        block.copy_to_bytes(&mut out);

        assert_eq!(out, b);
    }

    #[test]
    fn words_are_little_endian() {
        let b = bytes();
        let mut block = Block([u64::MAX; Block::WORDS]);
        block.copy_from_bytes(&b);

        for i in 0..Block::WORDS {
            #[rustfmt::skip]
            let expected = u64::from_le_bytes(
                b[i * 8..(i + 1) * 8].try_into().unwrap(),
            );
            assert_eq!(block[i], expected);
        }
    }

    #[test]
    fn j1_is_first_four_octets_little_endian() {
        let b = bytes();
        let mut block = Block::new();
        block.copy_from_bytes(&b);

        let expected = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(block.j1(), expected);
    }
}
