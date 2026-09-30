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

// Compile-time invariants
const _: () = {
    assert!(BLOCK_SIZE.is_multiple_of(64)); // 16 sub-blocks of 64 bytes
    assert!(WORDS.is_multiple_of(16)); // rows of 16 words
    assert!(WORDS / 16 == 8); // eight rows / column pairs
};
