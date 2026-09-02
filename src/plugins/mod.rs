//! Optional plugins, gated behind cargo features so the wallet build stays lean.

#[cfg(feature = "irc")]
pub mod irc;
