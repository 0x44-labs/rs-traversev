# TraverseV

![Crates.io Version](https://img.shields.io/crates/v/traversev)
![Crates.io License](https://img.shields.io/crates/l/traversev)
![docs.rs](https://img.shields.io/docsrs/traversev)

Pure Rust implementation of an authenticated, memory-hard proof-of-work construction.

[Documentation](https://docs.rs/traversev/)

# About

This crate provides an authenticated, memory-hard proof-of-work construction. A memory-hard structure is built once through a multi-pass fill in which each block is derived from previously computed blocks using data-dependent addressing and a mixing function derived from Argon2d. Candidate nonce values are then checked against that structure through a sequential, read-only traversal derived from scrypt, in which each round's memory access depends on the result of the previous round.

Mining searches over nonce values until one meets the target difficulty, while verifying a candidate is cheap and does not require repeating that search. Authentication is provided by BLAKE3, binding every check to the same secret used to build the structure, so a proof produced under one secret cannot be substituted for another.

# Minimum Supported Rust Version

Rust **1.85.1** or higher.

# License

Licensed under the [MIT License](https://opensource.org/license/MIT).
