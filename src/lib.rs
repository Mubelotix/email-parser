#![allow(clippy::type_complexity)]

//! The fastest and lightest mail parsing Rust library.  
//! This library is extremely lightweight and does not support common things like header unfolding.  
//! No-panic and zero-copy, written with `nom`.
//! 
//! # Benchmarks
//! 
//! This chart shows the time took to parse a single mail.  
//! The other crates are slower, but they offer more features.
//! 
//! ![Benchmark](https://cdn.discordapp.com/attachments/689171143046987796/727142729934700626/unknown.png)
//! 
//! Run the benchmark by yourself with `rustup run nightly cargo bench`.  
//! Tests require a `mail.txt` file containing a raw mail next to the `Cargo.toml`.

pub mod parser;
pub mod new_parser;