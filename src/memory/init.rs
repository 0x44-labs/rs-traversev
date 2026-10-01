use blake3::Hasher;
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::block::{BLOCK_SIZE, Block, WORDS};
use crate::params::Params;

/// Indicates whether a TraverseV instance produces trustless or permissioned
/// proofs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// Proofs are trustless.
    ///
    /// Verifiable by any instance sharing the same application context and
    /// parameters.
    Trustless = 0x54,

    /// Proofs are permissioned.
    ///
    /// Verifiable only by instances sharing the same application context,
    /// parameters, and shared secret.
    Permissioned = 0x50,
}

// Compile-time invariants
const _: () = {
    assert!(BLOCK_SIZE % 64 == 0); // 16 sub-blocks of 64 bytes
    assert!(WORDS % 16 == 0); // rows of 16 words
    assert!(WORDS / 16 == 8); // eight rows / column pairs
};

/// Compute the starting blocks B0 and B1. Similar to RFC 9106's Lane Starting
/// Blocks and Second Lane Blocks, but specialised for TraverseV lacking a
/// degree of parallelism and use of BLAKE3.
///
/// `B[0] = init(H_0 || LE32(0))`
///
/// `B[1] = init(H_0 || LE32(1))`
///
/// https://www.rfc-editor.org/info/rfc9106/#section-3.2
pub(crate) fn initial_blocks(
    mode: Mode,
    params: Params,
    context: &str,
    secret: Option<&[u8]>,
) -> (Block, Block) {
    let mut h_0 = preimage(mode, params, context, secret);

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
    params: Params,
    context: &str,
    secret: Option<&[u8]>,
) -> [u8; 64] {
    let mut hasher = Hasher::new();
    hasher.update(&[mode as u8]);
    hasher.update(&params.m_cost().to_le_bytes());
    hasher.update(&params.t_cost().to_le_bytes());
    hasher.update(&params.e_cost().to_le_bytes());
    hasher.update(&params.n_cost().to_le_bytes());

    let context_bytes = context.as_bytes();
    let mut context_len = (context_bytes.len() as u32).to_le_bytes();
    hasher.update(&context_len);
    hasher.update(context_bytes);

    #[cfg(feature = "zeroize")]
    context_len.zeroize();

    if let Some(secret) = secret {
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

    /// BLAKE3 XOF over flat bytes/
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
        let params = Params::default();
        let expected = xof::<64>(&layout(Mode::Trustless));

        let h_0 = preimage(Mode::Trustless, params, CONTEXT, None);
        assert_eq!(h_0, expected);
    }

    #[test]
    fn preimage_with_secret() {
        let params = Params::default();
        let mut bytes = layout(Mode::Permissioned);
        bytes.extend((SECRET.len() as u32).to_le_bytes());
        bytes.extend(SECRET);
        let expected = xof::<64>(&bytes);

        let h_0 = preimage(Mode::Permissioned, params, CONTEXT, Some(SECRET));
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
        let params = Params::default();
        let h_0 = preimage(Mode::Trustless, params, CONTEXT, None);
        let (b0, b1) = initial_blocks(Mode::Trustless, params, CONTEXT, None);

        assert_eq!(b0, init(&h_0, 0));
        assert_eq!(b1, init(&h_0, 1));
    }
}
