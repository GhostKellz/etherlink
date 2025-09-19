//! Guardian authentication provider implementation

use crate::auth::{AuthProvider, AuthCredentials, AuthToken, Permission};
use crate::clients::gid::{GidClient, GuardianTokenRequest, AccessToken};
use crate::{Result, EtherlinkError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;

/// Guardian authentication provider for zero-trust access control
#[derive(Debug, Clone)]
pub struct GuardianAuthProvider {
    gid_client: Arc<GidClient>,
    current_token: Option<AuthToken>,
}

impl GuardianAuthProvider {
    /// Create a new Guardian authentication provider
    pub fn new(gid_client: Arc<GidClient>) -> Self {
        Self {
            gid_client,
            current_token: None,
        }
    }

    /// Convert Guardian access token to auth token
    fn convert_access_token(&self, access_token: AccessToken) -> AuthToken {
        AuthToken {
            token_id: access_token.token_id,
            identity: access_token.identity,
            permissions: access_token.permissions,
            issued_at: access_token.issued_at,
            expires_at: access_token.expires_at,
            signature: access_token.signature,
            algorithm: "Guardian".to_string(),
        }
    }

    /// Get current token if valid
    pub fn get_current_token(&self) -> Option<&AuthToken> {
        if let Some(token) = &self.current_token {
            if !token.is_expired() {
                return Some(token);
            }
        }
        None
    }

    /// Check if token needs refresh
    pub fn needs_refresh(&self, threshold_seconds: u64) -> bool {
        if let Some(token) = &self.current_token {
            let now = Utc::now().timestamp() as u64;
            let time_until_expiry = token.expires_at.saturating_sub(now);
            return time_until_expiry <= threshold_seconds;
        }
        true
    }
}

#[async_trait]
impl AuthProvider for GuardianAuthProvider {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthToken> {
        // Create Guardian token request
        let request = GuardianTokenRequest {
            identity: credentials.identity.clone(),
            permissions: credentials.permissions.clone(),
            duration_seconds: Some(3600), // 1 hour default
            resource: None,
        };

        // Request token from GID service
        let access_token = self.gid_client
            .guardian_create_token(request)
            .await
            .map_err(|e| EtherlinkError::Authentication(format!("Failed to create Guardian token: {}", e)))?;

        // Convert to auth token
        let auth_token = self.convert_access_token(access_token);

        Ok(auth_token)
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // Create new credentials with existing identity and permissions
        let credentials = AuthCredentials {
            identity: token.identity.clone(),
            secret: crate::auth::AuthSecret::Password("refresh".to_string()), // Placeholder
            permissions: token.permissions.clone(),
        };

        // Re-authenticate to get new token
        self.authenticate(&credentials).await
    }

    async fn validate_token(&self, token: &AuthToken) -> Result<bool> {
        // Check expiration
        if token.is_expired() {
            return Ok(false);
        }

        // TODO: Implement server-side token validation via GID service
        // For now, just check expiration and algorithm
        Ok(token.algorithm == "Guardian")
    }

    fn get_auth_headers(&self, token: &AuthToken) -> Result<HashMap<String, String>> {
        let mut headers = HashMap::new();

        // Add Guardian-specific headers
        headers.insert("Authorization".to_string(), token.as_bearer());
        headers.insert("X-Guardian-Token".to_string(), token.token_id.clone());
        headers.insert("X-Guardian-Identity".to_string(), token.identity.clone());
        headers.insert("X-Guardian-Signature".to_string(), token.signature.clone());

        // Add timestamp for request validation
        let timestamp = Utc::now().timestamp().to_string();
        headers.insert("X-Guardian-Timestamp".to_string(), timestamp);

        Ok(headers)
    }
}

/// Guardian authentication manager with automatic token refresh
#[derive(Debug)]
pub struct GuardianAuthManager {
    provider: GuardianAuthProvider,
    config: crate::auth::AuthConfig,
    current_token: tokio::sync::RwLock<Option<AuthToken>>,
}

impl GuardianAuthManager {
    /// Create a new Guardian authentication manager
    pub fn new(gid_client: Arc<GidClient>, config: crate::auth::AuthConfig) -> Self {
        Self {
            provider: GuardianAuthProvider::new(gid_client),
            config,
            current_token: tokio::sync::RwLock::new(None),
        }
    }

    /// Authenticate and store token
    pub async fn authenticate(&self, credentials: &AuthCredentials) -> Result<()> {
        let token = self.provider.authenticate(credentials).await?;

        let mut current_token = self.current_token.write().await;
        *current_token = Some(token);

        Ok(())
    }

    /// Get authentication headers, refreshing token if needed
    pub async fn get_auth_headers(&self) -> Result<HashMap<String, String>> {
        // Check if we need to refresh the token
        if self.config.auto_refresh {
            let should_refresh = {
                let token_guard = self.current_token.read().await;
                if let Some(token) = token_guard.as_ref() {
                    let now = Utc::now().timestamp() as u64;
                    let time_until_expiry = token.expires_at.saturating_sub(now);
                    time_until_expiry <= self.config.refresh_threshold_seconds
                } else {
                    true
                }
            };

            if should_refresh {
                // Try to refresh token
                let current_token_clone = {
                    let token_guard = self.current_token.read().await;
                    token_guard.clone()
                };

                if let Some(current_token) = current_token_clone {
                    if let Ok(new_token) = self.provider.refresh_token(&current_token).await {
                        let mut token_guard = self.current_token.write().await;
                        *token_guard = Some(new_token);
                    }
                }
            }
        }

        // Get current token and generate headers
        let token_guard = self.current_token.read().await;
        if let Some(token) = token_guard.as_ref() {
            if !token.is_expired() {
                return self.provider.get_auth_headers(token);
            }
        }

        Err(EtherlinkError::Authentication("No valid token available".to_string()))
    }

    /// Check if authenticated
    pub async fn is_authenticated(&self) -> bool {
        let token_guard = self.current_token.read().await;
        if let Some(token) = token_guard.as_ref() {
            !token.is_expired()
        } else {
            false
        }
    }

    /// Get current token (if valid)
    pub async fn get_current_token(&self) -> Option<AuthToken> {
        let token_guard = self.current_token.read().await;
        if let Some(token) = token_guard.as_ref() {
            if !token.is_expired() {
                return Some(token.clone());
            }
        }
        None
    }
}