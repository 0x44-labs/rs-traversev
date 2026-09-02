#[derive(Debug)]
pub enum TraverseVErr {
    MemoryTooSmall,
    TimeTooSmall,
    DependencyTooSmall,
    DifficultyTooSmall,
    DifficultyTooBig,
}

impl core::fmt::Display for TraverseVErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::MemoryTooSmall => "memory cost is too small",
            Self::TimeTooSmall => "time cost is too small",
            Self::DependencyTooSmall => "dependency cost is too small",
            Self::DifficultyTooSmall => "difficulty cost is too small",
            Self::DifficultyTooBig => "difficulty cost is too big",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for TraverseVErr {}
