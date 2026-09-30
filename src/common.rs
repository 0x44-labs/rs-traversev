/// TraverseV block size.
pub const BLOCK_SIZE: usize = 1024;

/// Words per TraverseV block.
pub const WORDS: usize = BLOCK_SIZE / 8;

/// Indicates whether a TraverseV instance produces trustless or permissioned
/// proofs.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Mode {
    /// Proofs are trustless.
    ///
    /// Verifiable by any instance sharing the same application context and
    /// parameters.
    Trustless = 0x54,

    /// Proofs are permissioned.
    ///
    /// Verifiable only by instances sharing the same application context,
    /// parameters, and shared secret.
    Permissioned = 0x50,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time invariants. These fail the build, not a test run.
    const _: () = {
        assert!(BLOCK_SIZE % 64 == 0); // 16 sub-blocks of 64 bytes
        assert!(WORDS % 16 == 0); // rows of 16 words
        assert!(WORDS / 16 == 8); // eight rows / column pairs
    };

    #[test]
    fn block_size_and_words_are_pinned() {
        assert_eq!(BLOCK_SIZE, 1024);
        assert_eq!(WORDS, 128);
    }

    #[test]
    fn mode_discriminants_are_pinned() {
        assert_eq!(Mode::Trustless as u8, 0x54);
        assert_eq!(Mode::Permissioned as u8, 0x50);
    }

    #[test]
    fn mode_discriminants_are_distinct() {
        assert_ne!(Mode::Trustless as u8, Mode::Permissioned as u8);
    }

    #[test]
    fn mode_is_copy_and_clone() {
        for original in [Mode::Trustless, Mode::Permissioned] {
            let copied = original;
            let cloned = original.clone();

            // Original is still usable after the copy.
            assert_eq!(copied as u8, original as u8);
            assert_eq!(cloned as u8, original as u8);
        }
    }
}
