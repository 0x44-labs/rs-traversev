use blake3::Hasher;
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::block::Block;
use crate::common::{BLOCK_SIZE, Mode};

/// Compute the starting blocks B0 and B1. Similar to RFC 9106's Lane Starting
/// Blocks and Second Lane Blocks, but specialised for TraverseV lacking a
/// degree of parallelism and use of BLAKE3.
///
/// `B[0] = init(H_0 || LE32(0))`
///
/// `B[1] = init(H_0 || LE32(1))`
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
pub fn initial_blocks(
    mode: Mode,
    m_cost: u32,
    t_cost: u32,
    e_cost: u32,
    n_cost: u32,
    context: &str,
    secret: Option<&[u8]>,
) -> (Block, Block) {
    #[rustfmt::skip]
    #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
    let mut h_0 = preimage(
        mode,
        m_cost,
        t_cost,
        e_cost,
        n_cost,
        context,
        secret
    );

    let b0 = init(&h_0, 0);
    let b1 = init(&h_0, 1);

    #[cfg(feature = "zeroize")]
    h_0.zeroize();

    (b0, b1)
}

/// Hash preimage of the parameters and secret. Delivers a consistent 64 byte
/// preimage despite unbounded secret length to avoid re-hashing potentially
/// large input once per output block with [init].
///
/// Similar to RFC 9106's H_0 Generation but specialised for TraverseV's
/// parameters and with BLAKE3 instead of BLAKE2b.
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
fn preimage(
    mode: Mode,
    m_cost: u32,
    t_cost: u32,
    e_cost: u32,
    n_cost: u32,
    context: &str,
    secret: Option<&[u8]>,
) -> [u8; 64] {
    let mut hasher = Hasher::new();
    hasher.update(&[mode as u8]);
    hasher.update(&m_cost.to_le_bytes());
    hasher.update(&t_cost.to_le_bytes());
    hasher.update(&e_cost.to_le_bytes());
    hasher.update(&n_cost.to_le_bytes());

    let context_bytes = context.as_bytes();
    #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
    let mut context_len = (context_bytes.len() as u32).to_le_bytes();
    hasher.update(&context_len);
    hasher.update(context_bytes);

    #[cfg(feature = "zeroize")]
    context_len.zeroize();

    if let Some(secret) = secret {
        #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
        let mut secret_len = (secret.len() as u32).to_le_bytes();
        hasher.update(&secret_len);
        hasher.update(secret);

        #[cfg(feature = "zeroize")]
        secret_len.zeroize();
    }

    let mut out = [0u8; 64];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut out);

    #[cfg(feature = "zeroize")]
    {
        hasher.zeroize();
        reader.zeroize();
    }

    out
}

/// Wrapper for the BLAKE3 XOF, producing one [Block] of output.
///
/// Replaces RFC 9106's Function H' for Tag and Initial Block Computations.
fn init(preimage: &[u8; 64], index: u32) -> Block {
    #[cfg_attr(not(feature = "zeroize"), allow(unused_mut))]
    let mut input = [preimage.as_slice(), &index.to_le_bytes()].concat();

    let mut hasher = Hasher::new();
    hasher.update(&input);

    let mut buf = [0u8; BLOCK_SIZE];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut buf);

    let block = Block::from_bytes(&buf);

    #[cfg(feature = "zeroize")]
    {
        input.zeroize();
        hasher.zeroize();
        reader.zeroize();
        buf.zeroize();
    }

    block
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTEXT: &str = "TRAVERSEV_TEST";
    const SECRET: &[u8] = b"This is a secret.";

    /// BLAKE3 XOF over flat bytes
    fn xof<const N: usize>(bytes: &[u8]) -> [u8; N] {
        let mut out = [0u8; N];
        Hasher::new().update(bytes).finalize_xof().fill(&mut out);
        out
    }

    /// H_0 preimage layout up to and including the context.
    fn layout(mode: Mode) -> Vec<u8> {
        let mut bytes = vec![mode as u8];
        for cost in [19 * 1024u32, 3, 8, 20] {
            bytes.extend(cost.to_le_bytes());
        }
        bytes.extend((CONTEXT.len() as u32).to_le_bytes());
        bytes.extend(CONTEXT.as_bytes());
        bytes
    }

    #[test]
    fn preimage_without_secret() {
        let expected = xof::<64>(&layout(Mode::Trustless));

        let h_0 = preimage(Mode::Trustless, 19 * 1024, 3, 8, 20, CONTEXT, None);
        assert_eq!(h_0, expected);
    }

    #[test]
    fn preimage_with_secret() {
        let mut bytes = layout(Mode::Permissioned);
        bytes.extend((SECRET.len() as u32).to_le_bytes());
        bytes.extend(SECRET);
        let expected = xof::<64>(&bytes);

        let h_0 = preimage(
            Mode::Permissioned,
            19 * 1024,
            3,
            8,
            20,
            CONTEXT,
            Some(SECRET),
        );
        assert_eq!(h_0, expected);
    }

    #[test]
    fn init_hashes_preimage_and_index() {
        let h_0 = [0x00; 64];
        let mut bytes = h_0.to_vec();
        bytes.extend(0u32.to_le_bytes());

        let expected = Block::from_bytes(&xof::<BLOCK_SIZE>(&bytes));

        assert_eq!(init(&h_0, 0), expected);
    }

    #[test]
    fn initial_blocks_produces_two_blocks() {
        let h_0 = preimage(Mode::Trustless, 19 * 1024, 3, 8, 20, CONTEXT, None);
        #[rustfmt::skip]
        let (b0, b1) = initial_blocks(
            Mode::Trustless,
            19 * 1024,
            3,
            8,
            20,
            CONTEXT,
            None
        );

        assert_eq!(b0, init(&h_0, 0));
        assert_eq!(b1, init(&h_0, 1));
    }
}
