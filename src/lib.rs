#![allow(clippy::type_complexity)]

//! The fastest and lightest mail parsing Rust library!\
//! The goal of this library is to fully comply with RFC 5322. However, this library does not support the obsolete syntax because it has been obsolete for 12 years.\
//! This library has no dependency.
//!
//! # Benchmarks
//!
//! This chart shows the time took to parse a single mail.\
//! This library is still the fastest, but this benchmark is not up to date.
//!
//! ![Benchmark](https://cdn.discordapp.com/attachments/689171143046987796/727142729934700626/unknown.png)
//!
//! Run the benchmark by yourself with `rustup run nightly cargo bench`.  
//! Tests require a `mail.txt` file containing a raw mail next to the `Cargo.toml`.

pub mod error;
pub(crate) mod parsing;
pub mod prelude;
pub mod string;
