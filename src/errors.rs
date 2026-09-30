/// Errors returned by TraverseV.
#[derive(Debug, PartialEq)]
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
            Self::InvalidNonce => "cannot represent timestamp as SystemTime",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for TraverseVErr {}

#[cfg(test)]
mod tests {
    use super::*;

    type Error = TraverseVErr;

    /// Expected [Display](core::fmt::Display) output.
    fn expected_display(err: &Error) -> &'static str {
        match err {
            Error::MemoryTooSmall => "memory cost is too small",
            Error::TimeTooSmall => "time cost is too small",
            Error::EvaluationTooFew => "evaluation rounds is too few",
            Error::DifficultyTooLow => "difficulty is too low",
            Error::DifficultyTooHigh => "difficulty is too high",
            Error::InvalidNonce => "cannot represent timestamp as SystemTime",
        }
    }

    fn all_variants() -> Vec<(Error, &'static str)> {
        vec![
            (Error::MemoryTooSmall, "MemoryTooSmall"),
            (Error::TimeTooSmall, "TimeTooSmall"),
            (Error::EvaluationTooFew, "EvaluationTooFew"),
            (Error::DifficultyTooLow, "DifficultyTooLow"),
            (Error::DifficultyTooHigh, "DifficultyTooHigh"),
            (Error::InvalidNonce, "InvalidNonce"),
        ]
    }

    #[test]
    fn display_matches_for_every_variant() {
        for (err, _) in all_variants() {
            assert_eq!(err.to_string(), expected_display(&err));
        }
    }

    #[test]
    fn debug_output_is_the_variant_name() {
        for (err, name) in all_variants() {
            assert_eq!(format!("{err:?}"), name);
        }
    }

    #[test]
    fn conforms_to_std_error() {
        fn assert_bounds<T: Send + Sync + 'static>() {}
        assert_bounds::<TraverseVErr>();

        for (err, _) in all_variants() {
            let expected = expected_display(&err);
            let dyn_err: &dyn std::error::Error = &err;
            assert!(dyn_err.source().is_none());
            assert_eq!(dyn_err.to_string(), expected);
        }

        // Converts into a boxed error via `?`
        fn fallible() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err(TraverseVErr::DifficultyTooHigh)?
        }
        let boxed = fallible().unwrap_err();
        assert_eq!(boxed.to_string(), "difficulty is too high");
    }
}
