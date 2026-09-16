# TraverseV

![Crates.io Version](https://img.shields.io/crates/v/traversev)
![Crates.io License](https://img.shields.io/crates/l/traversev)
![docs.rs](https://img.shields.io/docsrs/traversev)

Pure Rust implementation of a memory-hard proof-of-work construction, producing trustless or permissioned proofs.

[Documentation](https://docs.rs/traversev/)

# About

A memory-hard structure is built once through a multi-pass fill in which each block is derived from previously computed blocks using data-dependent addressing and a mixing function derived from Argon2d. Candidate nonce values are then evaluated against that structure through a sequential, read-only traversal derived from scrypt, in which each round's memory access depends on the result of the previous round.

Mining searches over nonce values until one meets the target difficulty, while verifying a candidate is cheap and does not require repeating that search. Whether an instance produces trustless or permissioned proofs is fixed at construction. A trustless instance is built from application context and parameters, while a permissioned additionally requires a shared secret.

Proofs mined by a trustless instance are based on the BLAKE3 regular hash function, and are verifiable by any instance sharing the same context and configuration. Proofs mined by a permissioned instance are based on the BLAKE3 keyed hash function, and are verifiable only by instances sharing the same context, configuration, and shared secret.

# Minimum Supported Rust Version

Rust **1.85.1** or higher.

# License

Licensed under the [MIT License](https://opensource.org/license/MIT).
