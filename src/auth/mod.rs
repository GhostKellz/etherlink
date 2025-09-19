//! Authentication and authorization for GhostChain services

pub mod guardian;
pub mod crypto;

pub use guardian::*;
pub use crypto::*;

use crate::{Result, EtherlinkError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Authentication provider trait
#[async_trait::async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate and get access token
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthToken>;

    /// Refresh an existing token
    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken>;

    /// Validate a token
    async fn validate_token(&self, token: &AuthToken) -> Result<bool>;

    /// Get authentication headers for requests
    fn get_auth_headers(&self, token: &AuthToken) -> Result<HashMap<String, String>>;
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub identity: String,
    pub secret: AuthSecret,
    pub permissions: Vec<Permission>,
}

/// Authentication secret types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthSecret {
    PrivateKey(String),
    Mnemonic(String),
    Password(String),
    Certificate(String),
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token_id: String,
    pub identity: String,
    pub permissions: Vec<Permission>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub signature: String,
    pub algorithm: String,
}

impl AuthToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now >= self.expires_at
    }

    /// Check if token has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Get token as bearer string
    pub fn as_bearer(&self) -> String {
        format!("Bearer {}", self.token_id)
    }
}

/// Permission types for GhostChain services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    // Blockchain permissions
    ReadBlockchain,
    WriteBlockchain,
    SubmitTransaction,

    // Wallet permissions
    ReadWallet,
    WriteWallet,
    SignTransaction,

    // Token permissions
    ReadTokens,
    TransferTokens(crate::TokenType),
    MintTokens(crate::TokenType),
    BurnTokens(crate::TokenType),

    // Domain permissions
    ReadDomains,
    RegisterDomain,
    UpdateDomain,

    // Identity permissions
    ReadIdentity,
    WriteIdentity,
    CreateIdentity,

    // Signature permissions
    Sign,
    Verify,
    ThresholdSign,

    // Administrative permissions
    Admin,
    SystemRead,
    SystemWrite,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub provider: AuthProviderType,
    pub token_duration_seconds: u64,
    pub auto_refresh: bool,
    pub refresh_threshold_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthProviderType {
    Guardian,
    BasicAuth,
    ApiKey,
    Certificate,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            provider: AuthProviderType::Guardian,
            token_duration_seconds: 3600, // 1 hour
            auto_refresh: true,
            refresh_threshold_seconds: 300, // 5 minutes
        }
    }
}