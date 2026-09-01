use crate::errors::TraverseVErr;

pub struct Params {
    m_cost: u32,
    t_cost: u32,
}

impl Params {
    pub const DEFAULT_M: u32 = 19 * 1024;

    pub const MIN_M: u32 = 2;

    pub const MAX_M: u32 = u32::MAX;

    pub const DEFAULT_T: u32 = 2;

    pub const MIN_T: u32 = 1;

    pub const MAX_T: u32 = u32::MAX;

    pub const DEFAULT: Self = Params {
        m_cost: Self::DEFAULT_M,
        t_cost: Self::DEFAULT_T,
    };

    pub fn new(m_cost: u32, t_cost: u32) -> Result<Self, TraverseVErr> {
        if m_cost < Self::MIN_M {
            return Err(TraverseVErr::MemoryTooSmall);
        }
        if t_cost < Self::MIN_T {
            return Err(TraverseVErr::TimeTooSmall);
        }

        Ok(Self { m_cost, t_cost })
    }
}

impl Default for Params {
    fn default() -> Self {
        Params::DEFAULT
    }
}
