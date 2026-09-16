/// TraverseV block size.
pub const BLOCK_SIZE: usize = 1024;

/// Words per TraverseV block.
pub const WORDS: usize = BLOCK_SIZE / 8;

/// Indicates whether a `TraverseV` instance produces trustless or permissioned
/// proofs.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Mode {
    /// Proofs are trustless.
    ///
    /// Verifiable by any instance sharing the same application context and
    /// configuration.
    Trustless = 0x54,

    /// Proofs are permissioned.
    ///
    /// Verifiable only by instances sharing the same application context,
    /// configuration, and shared secret.
    Permissioned = 0x50,
}
