//! GHOSTD (Blockchain Daemon) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address, TxHash, BlockHeight, Gas};
use crate::clients::{ServiceClient, ApiResponse};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Client for GHOSTD blockchain daemon service
#[derive(Debug, Clone)]
pub struct GhostdClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl GhostdClient {
    /// Create a new GHOSTD client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'));
        Self {
            base_url,
            http_client,
        }
    }

    /// Submit a transaction to the blockchain
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<TxHash> {
        let url = format!("{}/transactions", self.base_url);
        let response: ApiResponse<TransactionResponse> = self.http_client
            .post(&url)
            .json(&tx)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let tx_response = response.into_result()?;
        Ok(TxHash::new(tx_response.tx_hash))
    }

    /// Get a block by height
    pub async fn get_block(&self, height: BlockHeight) -> Result<Block> {
        let url = format!("{}/blockchain/block/{}", self.base_url, height);
        let response: ApiResponse<Block> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get current blockchain height
    pub async fn get_blockchain_height(&self) -> Result<BlockHeight> {
        let url = format!("{}/blockchain/height", self.base_url);
        let response: ApiResponse<HeightResponse> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let height_response = response.into_result()?;
        Ok(height_response.height)
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &Address) -> Result<u64> {
        let url = format!("{}/accounts/{}/balance", self.base_url, address.as_str());
        let response: ApiResponse<BalanceResponse> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let balance_response = response.into_result()?;
        Ok(balance_response.balance)
    }

    /// Get daemon performance metrics
    pub async fn get_metrics(&self) -> Result<DaemonMetrics> {
        let url = format!("{}/performance/metrics", self.base_url);
        let response: ApiResponse<DaemonMetrics> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }
}

#[async_trait::async_trait]
impl ServiceClient for GhostdClient {
    fn service_name(&self) -> &'static str {
        "ghostd"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn health_check(&self) -> Result<serde_json::Value> {
        let url = format!("{}/health", self.base_url);
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        Ok(response)
    }

    async fn status(&self) -> Result<serde_json::Value> {
        let url = format!("{}/status", self.base_url);
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        Ok(response)
    }
}

// Data structures for GHOSTD API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub gas_limit: Gas,
    pub gas_price: u64,
    pub nonce: u64,
    pub data: Option<Vec<u8>>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: BlockHeight,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub merkle_root: String,
    pub gas_used: Gas,
    pub gas_limit: Gas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightResponse {
    pub height: BlockHeight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: u64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonMetrics {
    pub version: String,
    pub chain_id: u64,
    pub peer_count: u32,
    pub uptime_seconds: u64,
    pub transactions_per_second: f64,
    pub blocks_per_second: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
}