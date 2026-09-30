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

// Compile-time invariants. These fail the build, not a test run.
const _: () = {
    assert!(BLOCK_SIZE % 64 == 0); // 16 sub-blocks of 64 bytes
    assert!(WORDS % 16 == 0); // rows of 16 words
    assert!(WORDS / 16 == 8); // eight rows / column pairs
};
