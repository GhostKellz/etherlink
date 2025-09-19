//! GLEDGER (Token Ledger) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address, TxHash, TokenType};
use crate::clients::{ServiceClient, ApiResponse};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Client for GLEDGER token operations service
#[derive(Debug, Clone)]
pub struct GledgerClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl GledgerClient {
    /// Create a new GLEDGER client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'));
        Self {
            base_url,
            http_client,
        }
    }

    /// Transfer tokens between accounts
    pub async fn transfer_tokens(&self, transfer: TokenTransfer) -> Result<TxHash> {
        let url = format!("{}/tokens/transfer", self.base_url);
        let response: ApiResponse<TransferResponse> = self.http_client
            .post(&url)
            .json(&transfer)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let transfer_response = response.into_result()?;
        Ok(TxHash::new(transfer_response.tx_hash))
    }

    /// Get token balance for a specific token type
    pub async fn get_balance(&self, address: &Address, token_type: TokenType) -> Result<u64> {
        let url = format!("{}/tokens/balance/{}/{:?}", self.base_url, address.as_str(), token_type);
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

    /// Get all token balances for an address
    pub async fn get_all_balances(&self, address: &Address) -> Result<TokenBalances> {
        let url = format!("{}/tokens/balances/{}", self.base_url, address.as_str());
        let response: ApiResponse<TokenBalances> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Mint tokens (requires appropriate permissions)
    pub async fn mint_tokens(&self, mint: TokenMint) -> Result<TxHash> {
        let url = format!("{}/tokens/mint", self.base_url);
        let response: ApiResponse<TransferResponse> = self.http_client
            .post(&url)
            .json(&mint)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let mint_response = response.into_result()?;
        Ok(TxHash::new(mint_response.tx_hash))
    }

    /// Burn tokens
    pub async fn burn_tokens(&self, burn: TokenBurn) -> Result<TxHash> {
        let url = format!("{}/tokens/burn", self.base_url);
        let response: ApiResponse<TransferResponse> = self.http_client
            .post(&url)
            .json(&burn)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let burn_response = response.into_result()?;
        Ok(TxHash::new(burn_response.tx_hash))
    }

    /// Get token economics information
    pub async fn get_token_economics(&self) -> Result<TokenEconomics> {
        let url = format!("{}/tokens/economics", self.base_url);
        let response: ApiResponse<TokenEconomics> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get transaction history for an address
    pub async fn get_transaction_history(&self, address: &Address, limit: Option<u32>) -> Result<Vec<TokenTransaction>> {
        let mut url = format!("{}/tokens/history/{}", self.base_url, address.as_str());
        if let Some(limit) = limit {
            url.push_str(&format!("?limit={}", limit));
        }

        let response: ApiResponse<Vec<TokenTransaction>> = self.http_client
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
impl ServiceClient for GledgerClient {
    fn service_name(&self) -> &'static str {
        "gledger"
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

// Data structures for GLEDGER API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub from: Address,
    pub to: Address,
    pub token_type: TokenType,
    pub amount: u64,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    pub to: Address,
    pub token_type: TokenType,
    pub amount: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBurn {
    pub from: Address,
    pub token_type: TokenType,
    pub amount: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: u64,
    pub token_type: TokenType,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalances {
    pub address: String,
    pub gcc: u64,
    pub spirit: u64,
    pub mana: u64,
    pub ghost: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    pub gcc: TokenEconomicsInfo,
    pub spirit: TokenEconomicsInfo,
    pub mana: TokenEconomicsInfo,
    pub ghost: TokenEconomicsInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomicsInfo {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub max_supply: Option<u64>,
    pub inflation_rate: Option<f64>,
    pub burn_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransaction {
    pub tx_hash: String,
    pub from: Address,
    pub to: Address,
    pub token_type: TokenType,
    pub amount: u64,
    pub timestamp: u64,
    pub block_height: u64,
    pub memo: Option<String>,
}