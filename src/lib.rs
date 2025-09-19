//! # Etherlink
//!
//! A Rust-native bridge and gRPC client powering GhostChain's hybrid Rust ↔ Zig ecosystem.
//!
//! Etherlink provides secure and performant communication between Rust-based services
//! (GhostChain Core, GWallet, GhostBridge) and Zig-based execution layers like GhostPlane.

pub mod client;
pub mod ffi;
pub mod ghostplane;
pub mod rvm;
pub mod revm;
pub mod cns;
pub mod error;
pub mod types;

// Re-export commonly used types
pub use client::*;
pub use cns::CNSClient;
pub use ghostplane::GhostPlaneClient;
pub use error::{EtherlinkError, Result};
pub use types::*;

/// Initialize the Etherlink library with default configuration
pub fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}

/// Initialize the Etherlink library with custom tracing configuration
pub fn init_with_tracing(filter: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    Ok(())
}