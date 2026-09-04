#[derive(Debug)]
pub enum TraverseVErr {
    MemoryTooSmall,
    TimeTooSmall,
    MixingTooLow,
    DifficultyTooLow,
    DifficultyTooHigh,
}

impl core::fmt::Display for TraverseVErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::MemoryTooSmall => "memory cost is too small",
            Self::TimeTooSmall => "time cost is too small",
            Self::MixingTooLow => "mixing rounds is too low",
            Self::DifficultyTooLow => "difficulty is too low",
            Self::DifficultyTooHigh => "difficulty is too high",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for TraverseVErr {}
