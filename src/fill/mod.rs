mod compress;
mod fill;
mod init;

const BLOCK_SIZE: usize = 1024;

pub(crate) fn fill_memory(key: &[u8], m_cost: u32, t_cost: u32) -> Vec<u8> {
    let q = m_cost as usize;
    let mut v = vec![0u8; q * BLOCK_SIZE];

    let (b0, b1) = init::starting_blocks(key, m_cost, t_cost);
    v[0..BLOCK_SIZE].copy_from_slice(&b0);
    v[BLOCK_SIZE..2 * BLOCK_SIZE].copy_from_slice(&b1);

    fill::fill(&mut v, q, t_cost);

    v
}
