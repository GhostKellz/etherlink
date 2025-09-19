//! GhostChain service clients
//!
//! This module contains client implementations for all GhostChain services

pub mod ghostd;
pub mod walletd;
pub mod gid;
pub mod cns;
pub mod gsig;
pub mod gledger;

pub use ghostd::GhostdClient;
pub use walletd::WalletdClient;
pub use gid::GidClient;
pub use cns::CnsClient;
pub use gsig::GsigClient;
pub use gledger::GledgerClient;

use crate::{Result, EtherlinkConfig};
use reqwest::Client as HttpClient;
use std::sync::Arc;

/// Collection of all GhostChain service clients
#[derive(Debug, Clone)]
pub struct ServiceClients {
    pub ghostd: GhostdClient,
    pub walletd: WalletdClient,
    pub gid: GidClient,
    pub cns: CnsClient,
    pub gsig: GsigClient,
    pub gledger: GledgerClient,
}

impl ServiceClients {
    /// Create new service clients with the given configuration
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        Self {
            ghostd: GhostdClient::new(config, http_client.clone()),
            walletd: WalletdClient::new(config, http_client.clone()),
            gid: GidClient::new(config, http_client.clone()),
            cns: CnsClient::new(config, http_client.clone()),
            gsig: GsigClient::new(config, http_client.clone()),
            gledger: GledgerClient::new(config, http_client),
        }
    }
}

/// Base trait for all service clients
#[async_trait::async_trait]
pub trait ServiceClient {
    /// Get the service name
    fn service_name(&self) -> &'static str;

    /// Get the base URL for the service
    fn base_url(&self) -> &str;

    /// Health check endpoint
    async fn health_check(&self) -> Result<serde_json::Value>;

    /// Get service status
    async fn status(&self) -> Result<serde_json::Value>;
}

/// Common API response format used by GhostChain services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T> {
        if self.success {
            self.data.ok_or_else(|| crate::EtherlinkError::Api("Missing data in successful response".to_string()))
        } else {
            Err(crate::EtherlinkError::Api(
                self.error.unwrap_or_else(|| "Unknown API error".to_string())
            ))
        }
    }
}