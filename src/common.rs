/// TraverseV block size.
pub const BLOCK_SIZE: usize = 1024;

/// Words per TraverseV block.
pub const WORDS: usize = BLOCK_SIZE / 8;

pub enum Mode {
    Trustless,
    Permissioned,
}
