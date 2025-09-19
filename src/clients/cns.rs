//! CNS (Crypto Name Server) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address, TxHash};
use crate::clients::{ServiceClient, ApiResponse};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Client for CNS domain resolution service
#[derive(Debug, Clone)]
pub struct CnsClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl CnsClient {
    /// Create a new CNS client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = if let Some(cns_endpoint) = &config.cns_endpoint {
            format!("{}/api/v1", cns_endpoint.trim_end_matches('/'))
        } else {
            format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'))
        };
        Self {
            base_url,
            http_client,
        }
    }

    /// Resolve a domain to get its information
    pub async fn resolve_domain(&self, domain: &str) -> Result<DomainResolution> {
        let url = format!("{}/domains/resolve/{}", self.base_url, domain);
        let response: ApiResponse<DomainResolution> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Register a new domain
    pub async fn register_domain(&self, registration: DomainRegistration) -> Result<TxHash> {
        let url = format!("{}/domains/register", self.base_url);
        let response: ApiResponse<RegistrationResponse> = self.http_client
            .post(&url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let registration_response = response.into_result()?;
        Ok(TxHash::new(registration_response.tx_hash))
    }

    /// Update domain records
    pub async fn update_domain_records(&self, domain: &str, records: DomainRecords) -> Result<TxHash> {
        let url = format!("{}/domains/{}/records", self.base_url, domain);
        let response: ApiResponse<RegistrationResponse> = self.http_client
            .put(&url)
            .json(&records)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let update_response = response.into_result()?;
        Ok(TxHash::new(update_response.tx_hash))
    }

    /// Get domain ownership information
    pub async fn get_domain_info(&self, domain: &str) -> Result<DomainInfo> {
        let url = format!("{}/domains/{}", self.base_url, domain);
        let response: ApiResponse<DomainInfo> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get domains owned by an address
    pub async fn get_domains_by_owner(&self, address: &Address) -> Result<Vec<String>> {
        let url = format!("{}/domains/owner/{}", self.base_url, address.as_str());
        let response: ApiResponse<DomainsResponse> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let domains_response = response.into_result()?;
        Ok(domains_response.domains)
    }

    /// Check if a domain is available for registration
    pub async fn check_domain_availability(&self, domain: &str) -> Result<bool> {
        let url = format!("{}/domains/available/{}", self.base_url, domain);
        let response: ApiResponse<AvailabilityResponse> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        let availability_response = response.into_result()?;
        Ok(availability_response.available)
    }

    /// Get supported TLDs and their pricing
    pub async fn get_supported_tlds(&self) -> Result<Vec<TldInfo>> {
        let url = format!("{}/domains/tlds", self.base_url);
        let response: ApiResponse<Vec<TldInfo>> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Bridge resolution (ENS, Unstoppable, etc.)
    pub async fn bridge_resolve(&self, domain: &str, bridge_type: BridgeType) -> Result<DomainResolution> {
        let url = format!("{}/bridge/{:?}/resolve/{}", self.base_url, bridge_type, domain);
        let response: ApiResponse<DomainResolution> = self.http_client
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
impl ServiceClient for CnsClient {
    fn service_name(&self) -> &'static str {
        "cns"
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

// Data structures for CNS API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistration {
    pub domain: String,
    pub owner: Address,
    pub duration_years: u32,
    pub records: DomainRecords,
    pub payment_token: crate::TokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecords {
    pub addresses: HashMap<String, String>, // Chain -> Address mapping
    pub content_hash: Option<String>,
    pub text_records: HashMap<String, String>,
    pub avatar: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResolution {
    pub domain: String,
    pub owner: Address,
    pub records: DomainRecords,
    pub expires_at: u64,
    pub created_at: u64,
    pub last_updated: u64,
    pub resolver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub domain: String,
    pub owner: Address,
    pub expires_at: u64,
    pub created_at: u64,
    pub is_expired: bool,
    pub tld: String,
    pub registration_fee: u64,
    pub renewal_fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub tx_hash: String,
    pub domain: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainsResponse {
    pub domains: Vec<String>,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityResponse {
    pub domain: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TldInfo {
    pub tld: String,
    pub native: bool, // true for .ghost, .gcc, etc.
    pub bridged: bool, // true for .eth, .crypto, etc.
    pub registration_fee: u64,
    pub renewal_fee: u64,
    pub min_length: u32,
    pub max_length: u32,
    pub supported_records: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeType {
    ENS,         // .eth domains
    Unstoppable, // .crypto, .nft, .x domains
    Web5,        // did: identifiers
    Handshake,   // .hns domains
}