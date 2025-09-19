//! GSIG (Ghost Signature) client implementation

use crate::{Result, EtherlinkConfig, EtherlinkError, Address};
use crate::clients::{ServiceClient, ApiResponse};
use crate::clients::walletd::CryptoAlgorithm;
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Client for GSIG signature verification service
#[derive(Debug, Clone)]
pub struct GsigClient {
    base_url: String,
    http_client: Arc<HttpClient>,
}

impl GsigClient {
    /// Create a new GSIG client
    pub fn new(config: &EtherlinkConfig, http_client: Arc<HttpClient>) -> Self {
        let base_url = format!("{}/api/v1", config.ghostd_endpoint.trim_end_matches('/'));
        Self {
            base_url,
            http_client,
        }
    }

    /// Sign a message
    pub async fn sign(&self, request: SignRequest) -> Result<SignatureResponse> {
        let url = format!("{}/signatures/sign", self.base_url);
        let response: ApiResponse<SignatureResponse> = self.http_client
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

    /// Verify a signature
    pub async fn verify(&self, request: VerifyRequest) -> Result<VerificationResult> {
        let url = format!("{}/signatures/verify", self.base_url);
        let response: ApiResponse<VerificationResult> = self.http_client
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

    /// Batch verify multiple signatures
    pub async fn batch_verify(&self, requests: Vec<VerifyRequest>) -> Result<Vec<VerificationResult>> {
        let url = format!("{}/signatures/batch/verify", self.base_url);
        let response: ApiResponse<Vec<VerificationResult>> = self.http_client
            .post(&url)
            .json(&requests)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Create a threshold signature scheme
    pub async fn create_threshold_signature(&self, request: ThresholdSignatureRequest) -> Result<ThresholdSignatureResponse> {
        let url = format!("{}/signatures/threshold", self.base_url);
        let response: ApiResponse<ThresholdSignatureResponse> = self.http_client
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

    /// Get supported signature algorithms
    pub async fn get_supported_algorithms(&self) -> Result<Vec<AlgorithmInfo>> {
        let url = format!("{}/signatures/algorithms", self.base_url);
        let response: ApiResponse<Vec<AlgorithmInfo>> = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| EtherlinkError::Network(e.to_string()))?;

        response.into_result()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<SignatureMetrics> {
        let url = format!("{}/signatures/metrics", self.base_url);
        let response: ApiResponse<SignatureMetrics> = self.http_client
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
impl ServiceClient for GsigClient {
    fn service_name(&self) -> &'static str {
        "gsig"
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

// Data structures for GSIG API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequest {
    pub message: Vec<u8>,
    pub algorithm: CryptoAlgorithm,
    pub private_key: Option<String>, // For client-side signing
    pub key_id: Option<String>,      // For server-side signing
    pub address: Option<Address>,    // For wallet-based signing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub public_key: String,
    pub algorithm: CryptoAlgorithm,
    pub message_hash: String,
    pub signature_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub message: Vec<u8>,
    pub signature: String,
    pub public_key: String,
    pub algorithm: CryptoAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub algorithm: CryptoAlgorithm,
    pub message_hash: String,
    pub verification_time_ms: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSignatureRequest {
    pub message: Vec<u8>,
    pub threshold: u32,
    pub participants: Vec<ThresholdParticipant>,
    pub algorithm: CryptoAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdParticipant {
    pub id: String,
    pub public_key: String,
    pub partial_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSignatureResponse {
    pub signature: String,
    pub threshold: u32,
    pub participants_count: u32,
    pub algorithm: CryptoAlgorithm,
    pub signature_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmInfo {
    pub algorithm: CryptoAlgorithm,
    pub name: String,
    pub description: String,
    pub key_size_bits: u32,
    pub signature_size_bytes: u32,
    pub post_quantum: bool,
    pub supported_operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetrics {
    pub total_signatures_generated: u64,
    pub total_verifications_performed: u64,
    pub average_sign_time_ms: f64,
    pub average_verify_time_ms: f64,
    pub verifications_per_second: f64,
    pub supported_algorithms: Vec<CryptoAlgorithm>,
    pub uptime_seconds: u64,
}