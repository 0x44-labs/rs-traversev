/// Errors returned by TraverseV.
#[derive(Debug)]
pub enum TraverseVErr {
    /// Memory cost is below the minimum of 2 blocks.
    MemoryTooSmall,

    /// Time cost is below the minimum of 1 memory fill iteration.
    TimeTooSmall,

    /// Number of evaluation mixing rounds is below the minimum of 1.
    EvaluationTooFew,

    /// Difficulty is below the minimum of 1 leading zero bit.
    DifficultyTooLow,

    /// Difficulty is above the maximum of (2^8) - 1 leading zero bits.
    DifficultyTooHigh,

    /// A nonce's timestamp is too far after the Unix epoch to be represented
    /// as a [SystemTime](std::time::SystemTime).
    InvalidNonce,
}

impl core::fmt::Display for TraverseVErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::MemoryTooSmall => "memory cost is too small",
            Self::TimeTooSmall => "time cost is too small",
            Self::EvaluationTooFew => "evaluation rounds is too few",
            Self::DifficultyTooLow => "difficulty is too low",
            Self::DifficultyTooHigh => "difficulty is too high",
            Self::InvalidNonce => "nonce time before unix epoch",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for TraverseVErr {}
