//! GQUIC transport implementation using the gquic crate

use crate::{Result, EtherlinkError};
use crate::transport::{Transport, TransportConfig, TransportStats};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[cfg(feature = "gquic")]
use gquic::prelude::*;

/// GQUIC transport implementation for high-performance communication
#[derive(Debug, Clone)]
pub struct GQuicTransport {
    #[cfg(feature = "gquic")]
    client: Arc<QuicClient>,
    #[cfg(feature = "gquic")]
    pool: Arc<ConnectionPool>,
    config: TransportConfig,
    stats: Arc<RwLock<TransportStats>>,
}

impl GQuicTransport {
    /// Create a new GQUIC transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        #[cfg(feature = "gquic")]
        {
            let quic_config = QuicClientConfig::builder()
                .server_name("ghostchain.local".to_string())
                .with_alpn("ghostchain-v1")
                .with_alpn("grpc")
                .max_idle_timeout(config.timeout_ms as u32)
                .build();

            let client = QuicClient::new(quic_config)
                .map_err(|e| EtherlinkError::Transport(e.into()))?;

            let pool_config = PoolConfig::default();
            let pool = ConnectionPool::new(pool_config);

            let stats = TransportStats {
                active_connections: 0,
                total_requests: 0,
                failed_requests: 0,
                average_latency_ms: 0.0,
                bytes_sent: 0,
                bytes_received: 0,
            };

            Ok(Self {
                client: Arc::new(client),
                pool: Arc::new(pool),
                config,
                stats: Arc::new(RwLock::new(stats)),
            })
        }

        #[cfg(not(feature = "gquic"))]
        {
            Err(EtherlinkError::Configuration("GQUIC feature not enabled".to_string()))
        }
    }

    #[cfg(feature = "gquic")]
    async fn get_connection(&self, addr: SocketAddr) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        // Try to get existing connection from pool
        if let Some(conn) = self.pool.get_connection(addr).await {
            return Ok(conn);
        }

        // Create new connection
        let conn = self.client.connect(addr).await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let conn_arc = Arc::new(conn) as Arc<dyn std::any::Any + Send + Sync>;

        // Return connection to pool for reuse
        self.pool.return_connection(addr, conn_arc.clone()).await;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.active_connections += 1;

        Ok(conn_arc)
    }
}

#[async_trait]
impl Transport for GQuicTransport {
    async fn send_json_request(&self, endpoint: &str, request: serde_json::Value) -> Result<serde_json::Value> {
        #[cfg(feature = "gquic")]
        {
            let start_time = Instant::now();

            // Parse endpoint to socket address
            let addr: SocketAddr = endpoint.parse()
                .map_err(|e| EtherlinkError::Configuration(format!("Invalid endpoint: {}", e)))?;

            // Get connection
            let conn = self.get_connection(addr).await?;

            // Serialize request
            let request_data = serde_json::to_vec(&request)
                .map_err(|e| EtherlinkError::Serialization(e))?;

            // Open bidirectional stream
            let mut stream = self.client.open_bi_stream(&conn).await
                .map_err(|e| EtherlinkError::Network(e.to_string()))?;

            // Send request
            stream.write_all(&request_data).await
                .map_err(|e| EtherlinkError::Network(e.to_string()))?;
            stream.finish().await
                .map_err(|e| EtherlinkError::Network(e.to_string()))?;

            // Read response
            let response_data = stream.read_to_end(64 * 1024).await
                .map_err(|e| EtherlinkError::Network(e.to_string()))?;

            // Deserialize response
            let response: serde_json::Value = serde_json::from_slice(&response_data)
                .map_err(|e| EtherlinkError::Serialization(e))?;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.bytes_sent += request_data.len() as u64;
            stats.bytes_received += response_data.len() as u64;

            let latency = start_time.elapsed().as_millis() as f64;
            stats.average_latency_ms = (stats.average_latency_ms * (stats.total_requests - 1) as f64 + latency) / stats.total_requests as f64;

            Ok(response)
        }

        #[cfg(not(feature = "gquic"))]
        {
            Err(EtherlinkError::Configuration("GQUIC feature not enabled".to_string()))
        }
    }

    async fn health_check(&self, endpoint: &str) -> Result<()> {
        #[cfg(feature = "gquic")]
        {
            let addr: SocketAddr = endpoint.parse()
                .map_err(|e| EtherlinkError::Configuration(format!("Invalid endpoint: {}", e)))?;

            // Try to establish connection
            let _conn = self.get_connection(addr).await?;
            Ok(())
        }

        #[cfg(not(feature = "gquic"))]
        {
            Err(EtherlinkError::Configuration("GQUIC feature not enabled".to_string()))
        }
    }

    async fn get_stats(&self) -> Result<TransportStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

// Mock implementations for when gquic feature is not enabled
#[cfg(not(feature = "gquic"))]
mod mock_gquic {
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::sync::Arc;

    pub struct QuicClientConfig {
        pub server_name: String,
        pub alpn: Vec<String>,
        pub max_idle_timeout: u32,
    }

    impl QuicClientConfig {
        pub fn builder() -> QuicClientConfigBuilder {
            QuicClientConfigBuilder::default()
        }
    }

    #[derive(Default)]
    pub struct QuicClientConfigBuilder {
        server_name: String,
        alpn: Vec<String>,
        max_idle_timeout: u32,
    }

    impl QuicClientConfigBuilder {
        pub fn server_name(mut self, name: String) -> Self {
            self.server_name = name;
            self
        }

        pub fn with_alpn(mut self, protocol: &str) -> Self {
            self.alpn.push(protocol.to_string());
            self
        }

        pub fn max_idle_timeout(mut self, timeout: u32) -> Self {
            self.max_idle_timeout = timeout;
            self
        }

        pub fn build(self) -> QuicClientConfig {
            QuicClientConfig {
                server_name: self.server_name,
                alpn: self.alpn,
                max_idle_timeout: self.max_idle_timeout,
            }
        }
    }

    pub struct QuicClient;

    impl QuicClient {
        pub fn new(_config: QuicClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self)
        }

        pub async fn connect(&self, _addr: SocketAddr) -> Result<QuicConnection, Box<dyn std::error::Error>> {
            Ok(QuicConnection)
        }

        pub async fn open_bi_stream(&self, _conn: &Arc<dyn std::any::Any + Send + Sync>) -> Result<QuicStream, Box<dyn std::error::Error>> {
            Ok(QuicStream)
        }
    }

    pub struct QuicConnection;
    pub struct QuicStream;

    impl QuicStream {
        pub async fn write_all(&mut self, _data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        pub async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        pub async fn read_to_end(&mut self, _max_len: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            Ok(Vec::new())
        }
    }

    #[derive(Default)]
    pub struct PoolConfig;

    pub struct ConnectionPool;

    impl ConnectionPool {
        pub fn new(_config: PoolConfig) -> Self {
            Self
        }

        pub async fn get_connection(&self, _addr: SocketAddr) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
            None
        }

        pub async fn return_connection(&self, _addr: SocketAddr, _conn: Arc<dyn std::any::Any + Send + Sync>) {
            // No-op
        }
    }
}