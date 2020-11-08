#![allow(clippy::type_complexity)]

//! The fastest and lightest email parsing Rust library!\
//! This library has no dependency.
//!
//! # Goal
//!
//! The goal of this library is to be fully compliant with RFC 5322. However, this library does not intend to support the obsolete syntax because it has been obsolete for 12 years, and it would slow down everything.\
//! I plan to add optional support to the Multipurpose Internet Mail Extensions and for PGP.
//!
//! # Pay for what you use
//!
//! Mails can be elaborated. No matter what you are building, you are certainly not using all of its features.\
//! So why would you pay the parsing cost of header fields you are not using? This library allows you to enable headers you need so that other header values will be parsed as an unstructured header, which is much faster.\
//! By disabling all header value parsing, this library can parse an entire mail twice faster! But don't worry if you need everything enabled; this library is blazing fast anyway!
//!
//! # Zero-Copy
//!
//! This library is using a custom `String` enum, which is intended to store `String`s as a reference when possible and to clone them when needed.\
//! Thanks to this enum, this library avoids 90% of string clones.\
//! I plan to add many methods to this struct, so you can use it as if it were an `&str`.
//!
//! # Benchmarks
//!
//! This chart shows the time took to parse a single mail.\
//!
//! ![Benchmark](https://cdn.discordapp.com/attachments/770283472988143616/774711170208104448/Screenshot_2020-11-07_Performance_comparison1.png)
//!
//! Run the benchmark by yourself with `rustup run nightly cargo bench`.  
//! Tests require a `mail.txt` file containing a raw mail next to the `Cargo.toml`.
//! Some libraries suffer from huge performance variations depending on the content of the mail.

pub mod error;
pub(crate) mod parsing;
pub mod prelude;
pub mod string;

pub use crate::parsing::fields::Field;
pub use crate::parsing::time::{Day, Month};
pub use crate::parsing::message::parse_message;
