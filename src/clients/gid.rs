//! GID (Ghost Identity) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address};
use crate::clients::{ServiceClient, ApiResponse};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Client for GID identity management service
#[derive(Debug, Clone)]
pub struct GidClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl GidClient {
    /// Create a new GID client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'));
        Self {
            base_url,
            http_client,
        }
    }

    /// Create a new identity
    pub async fn create_identity(&self, request: CreateIdentityRequest) -> Result<Identity> {
        let url = format!("{}/identities", self.base_url);
        let response: ApiResponse<Identity> = self.http_client
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

    /// Resolve an identity by DID
    pub async fn resolve_identity(&self, did: &str) -> Result<IdentityDocument> {
        let url = format!("{}/identities/resolve/{}", self.base_url, did);
        let response: ApiResponse<IdentityDocument> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Create Guardian access token
    pub async fn guardian_create_token(&self, request: GuardianTokenRequest) -> Result<AccessToken> {
        let url = format!("{}/guardian/tokens", self.base_url);
        let response: ApiResponse<AccessToken> = self.http_client
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

    /// Evaluate Guardian policy
    pub async fn evaluate_policy(&self, request: PolicyRequest) -> Result<PolicyDecision> {
        let url = format!("{}/guardian/evaluate", self.base_url);
        let response: ApiResponse<PolicyDecision> = self.http_client
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

    /// Update identity document
    pub async fn update_identity(&self, did: &str, update: IdentityUpdate) -> Result<IdentityDocument> {
        let url = format!("{}/identities/{}", self.base_url, did);
        let response: ApiResponse<IdentityDocument> = self.http_client
            .put(&url)
            .json(&update)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get identities by address
    pub async fn get_identities_by_address(&self, address: &Address) -> Result<Vec<Identity>> {
        let url = format!("{}/identities/address/{}", self.base_url, address.as_str());
        let response: ApiResponse<Vec<Identity>> = self.http_client
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
impl ServiceClient for GidClient {
    fn service_name(&self) -> &'static str {
        "gid"
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

// Data structures for GID API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityRequest {
    pub address: Address,
    pub identity_type: IdentityType,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub ephemeral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub did: String, // Format: did:ghost:{identifier}
    pub address: Address,
    pub identity_type: IdentityType,
    pub created_at: u64,
    pub updated_at: u64,
    pub ephemeral: bool,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDocument {
    pub context: Vec<String>,
    pub id: String, // DID
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub capability_invocation: Vec<String>,
    pub capability_delegation: Vec<String>,
    pub service: Vec<ServiceEndpoint>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub method_type: String,
    pub controller: String,
    pub public_key_multibase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub service_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianTokenRequest {
    pub identity: String, // DID
    pub permissions: Vec<Permission>,
    pub duration_seconds: Option<u64>,
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token_id: String,
    pub identity: String,
    pub permissions: Vec<Permission>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    pub identity: String,
    pub action: String,
    pub resource: String,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
    pub conditions: Vec<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityUpdate {
    pub verification_method: Option<Vec<VerificationMethod>>,
    pub service: Option<Vec<ServiceEndpoint>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityType {
    Personal,
    Organization,
    Service,
    Device,
    Ephemeral,
}

pub use crate::auth::Permission;