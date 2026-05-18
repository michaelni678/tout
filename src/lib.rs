//! **Tout** provides utilities that make working with [`proc-macro2`] easier.
//!
//! # Setup
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! proc-macro2 = "1"
//! tout = "0.1.0"
//! ```
//!
//! [`proc-macro2`]: https://crates.io/crates/proc-macro2

pub mod assert;
pub mod extension;
pub mod parser;
pub mod quasi;
