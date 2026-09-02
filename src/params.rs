use crate::errors::TraverseVErr;

pub struct Params {
    m_cost: u32,
    t_cost: u32,
    d_cost: u32,
    n_cost: u32,
}

impl Params {
    pub const DEFAULT_M: u32 = 19 * 1024;

    pub const MIN_M: u32 = 2;

    pub const MAX_M: u32 = u32::MAX;

    pub const DEFAULT_T: u32 = 2;

    pub const MIN_T: u32 = 1;

    pub const MAX_T: u32 = u32::MAX;

    pub const DEFAULT_D: u32 = 4;

    pub const MIN_D: u32 = 1;

    pub const MAX_D: u32 = u32::MAX;

    pub const DEFAULT_N: u32 = 16;

    pub const MIN_N: u32 = 1;

    pub const MAX_N: u32 = u8::MAX as u32;

    pub const DEFAULT: Self = Params {
        m_cost: Self::DEFAULT_M,
        t_cost: Self::DEFAULT_T,
        d_cost: Self::DEFAULT_D,
        n_cost: Self::DEFAULT_N,
    };

    pub fn new(
        m_cost: u32,
        t_cost: u32,
        d_cost: u32,
        n_cost: u32,
    ) -> Result<Self, TraverseVErr> {
        if m_cost < Self::MIN_M {
            return Err(TraverseVErr::MemoryTooSmall);
        }
        if t_cost < Self::MIN_T {
            return Err(TraverseVErr::TimeTooSmall);
        }
        if d_cost < Self::MIN_D {
            return Err(TraverseVErr::DependencyTooSmall);
        }
        if n_cost < Self::MIN_N {
            return Err(TraverseVErr::DifficultyTooSmall);
        }
        if n_cost > Self::MAX_N {
            return Err(TraverseVErr::DifficultyTooBig);
        }

        Ok(Self {
            m_cost,
            t_cost,
            d_cost,
            n_cost,
        })
    }

    pub const fn m_cost(&self) -> u32 {
        self.m_cost
    }

    pub const fn t_cost(&self) -> u32 {
        self.t_cost
    }

    pub const fn d_cost(&self) -> u32 {
        self.d_cost
    }

    pub const fn n_cost(&self) -> u32 {
        self.n_cost
    }
}

impl Default for Params {
    fn default() -> Self {
        Params::DEFAULT
    }
}
