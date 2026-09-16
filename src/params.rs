use crate::errors::TraverseVErr;

/// TraverseV parameters.
#[derive(Copy, Clone)]
pub struct Params {
    m_cost: u32,
    t_cost: u32,
    e_cost: u32,
    n_cost: u32,
}

impl Params {
    /// Default memory cost `m`.
    pub const DEFAULT_M: u32 = 19 * 1024;

    const MIN_M: u32 = 2;

    #[allow(dead_code)]
    const MAX_M: u32 = u32::MAX;

    /// Default time cost (number of memory fill iterations) `t`.
    pub const DEFAULT_T: u32 = 3;

    const MIN_T: u32 = 1;

    #[allow(dead_code)]
    const MAX_T: u32 = u32::MAX;

    /// Default number of evaluation mixing rounds `e`.
    pub const DEFAULT_E: u32 = 8;

    const MIN_E: u32 = 1;

    #[allow(dead_code)]
    const MAX_D: u32 = u32::MAX;

    /// Default proof-of-work difficulty `n`.
    pub const DEFAULT_N: u32 = 20;

    const MIN_N: u32 = 1;

    const MAX_N: u32 = u8::MAX as u32;

    /// Default parameters.
    pub const DEFAULT: Self = Params {
        m_cost: Self::DEFAULT_M,
        t_cost: Self::DEFAULT_T,
        e_cost: Self::DEFAULT_E,
        n_cost: Self::DEFAULT_N,
    };

    /// Create new parameters for TraverseV.
    ///
    /// Returns the constructed `Params` on success, or an error when any value
    /// falls outside its accepted range.
    ///
    /// # Arguments
    /// - `m_cost`: memory cost in 1 KiB blocks. Between 2 and (2^32) - 1.
    /// - `t_cost`: number of memory fill iterations. Between 1 and (2^32) - 1.
    /// - `e_cost`: number of evaluation mixing rounds. Between 1 and
    ///   (2^32) - 1.
    /// - `n_cost`: proof-of-work difficulty as a number of leading zero bits.
    ///   Between 1 and (2^8) - 1.
    pub fn new(
        m_cost: u32,
        t_cost: u32,
        e_cost: u32,
        n_cost: u32,
    ) -> Result<Self, TraverseVErr> {
        if m_cost < Self::MIN_M {
            return Err(TraverseVErr::MemoryTooSmall);
        }
        if t_cost < Self::MIN_T {
            return Err(TraverseVErr::TimeTooSmall);
        }
        if e_cost < Self::MIN_E {
            return Err(TraverseVErr::EvaluationTooFew);
        }
        if n_cost < Self::MIN_N {
            return Err(TraverseVErr::DifficultyTooLow);
        }
        if n_cost > Self::MAX_N {
            return Err(TraverseVErr::DifficultyTooHigh);
        }

        Ok(Self {
            m_cost,
            t_cost,
            e_cost,
            n_cost,
        })
    }

    /// Memory size in KiB.
    pub const fn m_cost(&self) -> u32 {
        self.m_cost
    }

    /// Number of memory fill iterations.
    pub const fn t_cost(&self) -> u32 {
        self.t_cost
    }

    /// Number of sequential dependency mixing rounds
    pub const fn e_cost(&self) -> u32 {
        self.e_cost
    }

    /// Proof-of-work difficulty.
    pub const fn n_cost(&self) -> u32 {
        self.n_cost
    }
}

impl Default for Params {
    fn default() -> Self {
        Params::DEFAULT
    }
}
