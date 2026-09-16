#[derive(Debug)]
pub enum TraverseVErr {
    MemoryTooSmall,
    TimeTooSmall,
    EvaluationTooFew,
    DifficultyTooLow,
    DifficultyTooHigh,
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
