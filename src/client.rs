use crate::{EtherlinkConfig, EtherlinkError, Result, ConnectionStatus, HealthStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};
use tracing::{info, warn, error};

/// Main Etherlink client for communicating with GhostChain services
#[derive(Debug, Clone)]
pub struct EtherlinkClient {
    config: EtherlinkConfig,
    channel: Option<Channel>,
    status: Arc<RwLock<ConnectionStatus>>,
}

impl EtherlinkClient {
    /// Create a new Etherlink client with the given configuration
    pub fn new(config: EtherlinkConfig) -> Self {
        Self {
            config,
            channel: None,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
        }
    }

    /// Create a new Etherlink client with default configuration
    pub fn with_defaults() -> Self {
        Self::new(EtherlinkConfig::default())
    }

    /// Connect to the GhostChain services
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to GhostChain at {}", self.config.ghostd_endpoint);

        {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Connecting;
        }

        let endpoint = if self.config.enable_tls {
            Endpoint::from_shared(self.config.ghostd_endpoint.clone())?
                .tls_config(tonic::transport::ClientTlsConfig::new())?
        } else {
            Endpoint::from_shared(self.config.ghostd_endpoint.clone())?
        };

        let endpoint = endpoint
            .timeout(std::time::Duration::from_millis(self.config.timeout_ms))
            .tcp_keepalive(Some(std::time::Duration::from_secs(30)));

        match endpoint.connect().await {
            Ok(channel) => {
                self.channel = Some(channel);
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Connected;
                info!("Successfully connected to GhostChain");
                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Error(e.to_string());
                error!("Failed to connect to GhostChain: {}", e);
                Err(EtherlinkError::Transport(e))
            }
        }
    }

    /// Disconnect from GhostChain services
    pub async fn disconnect(&mut self) {
        info!("Disconnecting from GhostChain");
        self.channel = None;
        let mut status = self.status.write().await;
        *status = ConnectionStatus::Disconnected;
    }

    /// Get the current connection status
    pub async fn connection_status(&self) -> ConnectionStatus {
        self.status.read().await.clone()
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.status.read().await, ConnectionStatus::Connected)
    }

    /// Get the gRPC channel (internal use)
    pub(crate) fn channel(&self) -> Result<Channel> {
        self.channel
            .clone()
            .ok_or_else(|| EtherlinkError::Network("Not connected".to_string()))
    }

    /// Ping the service to check connectivity
    pub async fn ping(&self) -> Result<()> {
        if !self.is_connected().await {
            return Err(EtherlinkError::Network("Not connected".to_string()));
        }

        // TODO: Implement actual ping/health check once gRPC service is defined
        Ok(())
    }

    /// Get health status from the service
    pub async fn health_status(&self) -> Result<HealthStatus> {
        if !self.is_connected().await {
            return Err(EtherlinkError::Network("Not connected".to_string()));
        }

        // TODO: Implement actual health check once gRPC service is defined
        Ok(HealthStatus {
            service_name: "ghostd".to_string(),
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 0,
            last_block_height: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Get the client configuration
    pub fn config(&self) -> &EtherlinkConfig {
        &self.config
    }

    /// Update the client configuration
    pub fn update_config(&mut self, config: EtherlinkConfig) {
        self.config = config;
    }
}

impl Default for EtherlinkClient {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Builder pattern for creating Etherlink clients
pub struct EtherlinkClientBuilder {
    config: EtherlinkConfig,
}

impl EtherlinkClientBuilder {
    pub fn new() -> Self {
        Self {
            config: EtherlinkConfig::default(),
        }
    }

    pub fn ghostd_endpoint<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.config.ghostd_endpoint = endpoint.into();
        self
    }

    pub fn cns_endpoint<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.config.cns_endpoint = Some(endpoint.into());
        self
    }

    pub fn ghostplane_endpoint<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.config.ghostplane_endpoint = Some(endpoint.into());
        self
    }

    pub fn use_quic(mut self, enable: bool) -> Self {
        self.config.use_quic = enable;
        self
    }

    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.config.enable_tls = enable;
        self
    }

    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.config.timeout_ms = timeout;
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.config.retry_attempts = attempts;
        self
    }

    pub fn build(self) -> EtherlinkClient {
        EtherlinkClient::new(self.config)
    }
}

impl Default for EtherlinkClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}