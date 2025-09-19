//! HTTP transport implementation as fallback

use crate::{Result, EtherlinkError};
use crate::transport::{Transport, TransportConfig, TransportStats};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use reqwest::Client;

/// HTTP transport implementation for standard REST API communication
#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    config: TransportConfig,
    stats: Arc<RwLock<TransportStats>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .tcp_keepalive(Duration::from_millis(config.keepalive_interval_ms))
            .build()
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let stats = TransportStats {
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
        };

        Ok(Self {
            client,
            config,
            stats: Arc::new(RwLock::new(stats)),
        })
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_json_request(&self, endpoint: &str, request: serde_json::Value) -> Result<serde_json::Value> {
        let start_time = Instant::now();

        // Send HTTP POST request
        let response = self.client
            .post(endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                // Update failed request stats
                tokio::spawn({
                    let stats = self.stats.clone();
                    async move {
                        let mut stats = stats.write().await;
                        stats.failed_requests += 1;
                    }
                });
                EtherlinkError::Network(e.to_string())
            })?;

        // Check if request was successful
        if !response.status().is_success() {
            let mut stats = self.stats.write().await;
            stats.failed_requests += 1;
            return Err(EtherlinkError::Network(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        // Get response size for stats
        let content_length = response.content_length().unwrap_or(0);

        // Parse response
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.bytes_received += content_length;

        let latency = start_time.elapsed().as_millis() as f64;
        stats.average_latency_ms = (stats.average_latency_ms * (stats.total_requests - 1) as f64 + latency) / stats.total_requests as f64;

        Ok(result)
    }

    async fn health_check(&self, endpoint: &str) -> Result<()> {
        let health_url = if endpoint.ends_with('/') {
            format!("{}health", endpoint)
        } else {
            format!("{}/health", endpoint)
        };

        let response = self.client
            .get(&health_url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(EtherlinkError::Network(format!(
                "Health check failed with status: {}",
                response.status()
            )))
        }
    }

    async fn get_stats(&self) -> Result<TransportStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}