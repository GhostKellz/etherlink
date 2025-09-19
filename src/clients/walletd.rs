//! WALLETD (Wallet Service) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address, TxHash};
use crate::clients::{ServiceClient, ApiResponse};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Client for WALLETD wallet management service
#[derive(Debug, Clone)]
pub struct WalletdClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl WalletdClient {
    /// Create a new WALLETD client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'));
        Self {
            base_url,
            http_client,
        }
    }

    /// Create a new wallet
    pub async fn create_wallet(&self, request: CreateWalletRequest) -> Result<WalletInfo> {
        let url = format!("{}/wallets", self.base_url);
        let response: ApiResponse<WalletInfo> = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// List all wallets
    pub async fn list_wallets(&self) -> Result<Vec<WalletInfo>> {
        let url = format!("{}/wallets", self.base_url);
        let response: ApiResponse<Vec<WalletInfo>> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Sign a transaction
    pub async fn sign_transaction(&self, request: SignTransactionRequest) -> Result<SignedTransaction> {
        let url = format!("{}/wallets/{}/sign", self.base_url, request.wallet_id);
        let response: ApiResponse<SignedTransaction> = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get wallet addresses
    pub async fn get_addresses(&self, wallet_id: &str) -> Result<Vec<WalletAddress>> {
        let url = format!("{}/wallets/{}/addresses", self.base_url, wallet_id);
        let response: ApiResponse<Vec<WalletAddress>> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Generate new address for wallet
    pub async fn generate_address(&self, wallet_id: &str, derivation_path: Option<String>) -> Result<WalletAddress> {
        let url = format!("{}/wallets/{}/addresses", self.base_url, wallet_id);
        let request = GenerateAddressRequest { derivation_path };
        let response: ApiResponse<WalletAddress> = self.http_client
            .post(&url)
            .json(&request)
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
impl ServiceClient for WalletdClient {
    fn service_name(&self) -> &'static str {
        "walletd"
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

// Data structures for WALLETD API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub algorithm: CryptoAlgorithm,
    pub mnemonic: Option<String>, // If provided, restore from mnemonic
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub algorithm: CryptoAlgorithm,
    pub created_at: u64,
    pub address_count: u32,
    pub is_hardware: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignTransactionRequest {
    pub wallet_id: String,
    pub transaction: crate::clients::ghostd::Transaction,
    pub address_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: crate::clients::ghostd::Transaction,
    pub signature: String,
    pub public_key: String,
    pub signature_algorithm: CryptoAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub address: Address,
    pub derivation_path: String,
    pub public_key: String,
    pub address_index: u32,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAddressRequest {
    pub derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoAlgorithm {
    Ed25519,
    Secp256k1,
    Bls12381,
}