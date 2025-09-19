//! Transport layer implementations for GhostChain communication

pub mod gquic;
pub mod http;

pub use gquic::GQuicTransport;
pub use http::HttpTransport;

use crate::{Result, EtherlinkError};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::net::SocketAddr;

/// Transport trait for different communication protocols
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON request and return JSON response
    async fn send_json_request(&self, endpoint: &str, request: serde_json::Value) -> Result<serde_json::Value>;

    /// Health check the transport connection
    async fn health_check(&self, endpoint: &str) -> Result<()>;

    /// Get connection statistics
    async fn get_stats(&self) -> Result<TransportStats>;
}

/// Transport statistics
#[derive(Debug, Clone)]
pub struct TransportStats {
    pub active_connections: u32,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Configuration for transport layer
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub use_gquic: bool,
    pub enable_tls: bool,
    pub timeout_ms: u64,
    pub max_connections: u32,
    pub keepalive_interval_ms: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            use_gquic: true,
            enable_tls: true,
            timeout_ms: 30000,
            max_connections: 100,
            keepalive_interval_ms: 30000,
        }
    }
}

/// Create the appropriate transport based on configuration
pub fn create_transport(config: &TransportConfig) -> Result<Box<dyn Transport>> {
    if config.use_gquic {
        #[cfg(feature = "gquic")]
        {
            let transport = GQuicTransport::new(config.clone())?;
            Ok(Box::new(transport))
        }
        #[cfg(not(feature = "gquic"))]
        {
            return Err(EtherlinkError::Configuration("GQUIC feature not enabled".to_string()));
        }
    } else {
        let transport = HttpTransport::new(config.clone())?;
        Ok(Box::new(transport))
    }
}