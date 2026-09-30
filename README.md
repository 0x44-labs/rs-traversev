# TraverseV

![Crates.io Version](https://img.shields.io/crates/v/traversev)
![Crates.io License](https://img.shields.io/crates/l/traversev)
![docs.rs](https://img.shields.io/docsrs/traversev)

Pure Rust implementation of a memory-hard proof-of-work construction, producing trustless and permissioned proofs.

[Documentation](https://docs.rs/traversev/)

## About

TraverseV builds a memory-hard structure once through a multi-pass fill in which each block is derived from previously computed blocks using data-dependent addressing and a mixing function adapted from Argon2d. Candidate nonce values are then evaluated against that structure through a sequential, read-only traversal derived from scrypt, in which each round's memory access depends on the result of the previous round.

Whether a TraverseV instance produces trustless or permissioned proofs is fixed at construction.

- A **Trustless** instance is built from application context and parameters. Its proofs are based on the BLAKE3 regular hash function, and are verifiable by any instance sharing the same configuration.
- A **Permissioned** instance is built from a shared secret, application context, and parameters. Its proofs are based on the BLAKE3 keyed hash function, and are verifiable only by instances sharing the same configuration and shared secret.

Mining searches over nonce values until one meets the target difficulty, while verifying a candidate is cheap and does not require repeating that search.

## Minimum Supported Rust Version

Rust **1.85.1** or higher.

## License

Licensed under the [MIT License](https://opensource.org/license/MIT).
