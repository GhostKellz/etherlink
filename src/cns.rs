use crate::{EtherlinkError, Result, Address};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

/// CNS (Cryptographic Name Service) client for domain resolution
#[derive(Debug, Clone)]
pub struct CNSClient {
    config: CNSConfig,
    cache: std::sync::Arc<RwLock<DomainCache>>,
}

/// CNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CNSConfig {
    pub endpoint: String,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub max_cache_entries: usize,
    pub supported_tlds: Vec<String>,
    pub enable_ens_bridge: bool,
    pub enable_unstoppable_bridge: bool,
}

impl Default for CNSConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8553".to_string(),
            enable_cache: true,
            cache_ttl_seconds: 3600,
            max_cache_entries: 10000,
            supported_tlds: vec![
                "ghost".to_string(),
                "gcc".to_string(),
                "warp".to_string(),
                "arc".to_string(),
                "gcp".to_string(),
            ],
            enable_ens_bridge: true,
            enable_unstoppable_bridge: true,
        }
    }
}

/// Domain cache for performance
#[derive(Debug, Clone)]
struct DomainCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    resolution: DomainResolution,
    expires_at: u64,
}

impl DomainCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
        }
    }

    fn get(&self, domain: &str) -> Option<DomainResolution> {
        let now = chrono::Utc::now().timestamp() as u64;
        if let Some(entry) = self.entries.get(domain) {
            if entry.expires_at > now {
                return Some(entry.resolution.clone());
            }
        }
        None
    }

    fn insert(&mut self, domain: String, resolution: DomainResolution, ttl: u64) {
        let now = chrono::Utc::now().timestamp() as u64;

        // Simple LRU eviction
        if self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(domain, CacheEntry {
            resolution,
            expires_at: now + ttl,
        });
    }

    fn clear_expired(&mut self) {
        let now = chrono::Utc::now().timestamp() as u64;
        self.entries.retain(|_, entry| entry.expires_at > now);
    }
}

/// Domain resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResolution {
    pub domain: String,
    pub owner: Address,
    pub records: BTreeMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub expires_at: u64,
    pub service_type: ServiceType,
    pub blockchain_address: Option<Address>,
    pub ipfs_hash: Option<String>,
    pub web5_did: Option<String>,
}

/// Service type for domain routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    Blockchain,
    Wallet,
    L2,
    Storage,
    Web5,
    Bridge,
}

/// DNS record types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

/// Domain registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistration {
    pub domain: String,
    pub owner: Address,
    pub initial_records: Vec<DnsRecord>,
    pub metadata: HashMap<String, String>,
    pub payment_token: crate::TokenType,
    pub payment_amount: u64,
}

/// Domain subscription for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSubscription {
    pub domains: Vec<String>,
    pub record_types: Vec<String>,
    pub include_metadata: bool,
}

/// Domain change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainChangeEvent {
    pub domain: String,
    pub event_type: ChangeEventType,
    pub timestamp: u64,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeEventType {
    Registered,
    Updated,
    Transferred,
    Expired,
    Renewed,
}

impl CNSClient {
    /// Create a new CNS client
    pub fn new(config: CNSConfig) -> Self {
        let cache = DomainCache::new(config.max_cache_entries);
        Self {
            config,
            cache: std::sync::Arc::new(RwLock::new(cache)),
        }
    }

    /// Create CNS client with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CNSConfig::default())
    }

    /// Connect to CNS service
    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to CNS service at {}", self.config.endpoint);

        // TODO: Establish connection to CNS gRPC service
        // For now, just validate configuration

        if self.config.supported_tlds.is_empty() {
            return Err(EtherlinkError::Configuration("No supported TLDs configured".to_string()));
        }

        info!("CNS client connected successfully");
        Ok(())
    }

    /// Resolve a domain name
    pub async fn resolve_domain(&self, domain: &str) -> Result<DomainResolution> {
        debug!("Resolving domain: {}", domain);

        // Check cache first
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(domain) {
                debug!("Domain {} resolved from cache", domain);
                return Ok(cached);
            }
        }

        // Route to appropriate resolver based on TLD
        let resolution = self.resolve_domain_by_tld(domain).await?;

        // Cache the result
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(domain.to_string(), resolution.clone(), self.config.cache_ttl_seconds);
        }

        debug!("Domain {} resolved successfully", domain);
        Ok(resolution)
    }

    /// Resolve domain based on TLD
    async fn resolve_domain_by_tld(&self, domain: &str) -> Result<DomainResolution> {
        let tld = domain.split('.').last()
            .ok_or_else(|| EtherlinkError::CnsResolution("Invalid domain format".to_string()))?;

        match tld {
            "ghost" | "gcc" | "warp" | "arc" | "gcp" => {
                self.resolve_native_domain(domain).await
            }
            "eth" if self.config.enable_ens_bridge => {
                self.resolve_ens_domain(domain).await
            }
            "crypto" | "nft" | "x" if self.config.enable_unstoppable_bridge => {
                self.resolve_unstoppable_domain(domain).await
            }
            _ => {
                Err(EtherlinkError::CnsResolution(format!("Unsupported TLD: {}", tld)))
            }
        }
    }

    /// Resolve native GhostChain domain
    async fn resolve_native_domain(&self, domain: &str) -> Result<DomainResolution> {
        debug!("Resolving native domain: {}", domain);

        // TODO: Query actual CNS service via gRPC
        // For now, return a placeholder resolution

        Ok(DomainResolution {
            domain: domain.to_string(),
            owner: Address::new("0x1234567890123456789012345678901234567890".to_string()),
            records: {
                let mut records = BTreeMap::new();
                records.insert("A".to_string(), "127.0.0.1".to_string());
                records.insert("AAAA".to_string(), "::1".to_string());
                records
            },
            metadata: HashMap::new(),
            expires_at: (chrono::Utc::now().timestamp() + 365 * 24 * 3600) as u64,
            service_type: ServiceType::Blockchain,
            blockchain_address: Some(Address::new("0x1234567890123456789012345678901234567890".to_string())),
            ipfs_hash: None,
            web5_did: None,
        })
    }

    /// Resolve ENS domain (.eth)
    async fn resolve_ens_domain(&self, domain: &str) -> Result<DomainResolution> {
        debug!("Resolving ENS domain: {}", domain);

        // TODO: Bridge to ENS resolver
        Err(EtherlinkError::CnsResolution("ENS bridge not implemented".to_string()))
    }

    /// Resolve Unstoppable Domains (.crypto, .nft, etc.)
    async fn resolve_unstoppable_domain(&self, domain: &str) -> Result<DomainResolution> {
        debug!("Resolving Unstoppable domain: {}", domain);

        // TODO: Bridge to Unstoppable Domains resolver
        Err(EtherlinkError::CnsResolution("Unstoppable bridge not implemented".to_string()))
    }

    /// Register a new domain
    pub async fn register_domain(&self, registration: DomainRegistration) -> Result<String> {
        info!("Registering domain: {}", registration.domain);

        // Validate domain format
        self.validate_domain_format(&registration.domain)?;

        // Check if domain is available
        if self.is_domain_available(&registration.domain).await? {
            return Err(EtherlinkError::CnsResolution(
                format!("Domain {} is not available", registration.domain)
            ));
        }

        // TODO: Submit registration via gRPC
        let tx_hash = "0xabcdef1234567890".to_string();

        info!("Domain {} registered with tx hash: {}", registration.domain, tx_hash);
        Ok(tx_hash)
    }

    /// Check if a domain is available for registration
    pub async fn is_domain_available(&self, domain: &str) -> Result<bool> {
        debug!("Checking availability for domain: {}", domain);

        match self.resolve_domain(domain).await {
            Ok(_) => Ok(false), // Domain exists, not available
            Err(EtherlinkError::CnsResolution(_)) => Ok(true), // Domain not found, available
            Err(e) => Err(e), // Other error
        }
    }

    /// Update domain records
    pub async fn update_domain_records(
        &self,
        domain: &str,
        owner: &Address,
        records: Vec<DnsRecord>,
    ) -> Result<String> {
        info!("Updating records for domain: {}", domain);

        // Verify ownership
        let resolution = self.resolve_domain(domain).await?;
        if resolution.owner != *owner {
            return Err(EtherlinkError::CnsResolution("Not domain owner".to_string()));
        }

        // TODO: Submit update via gRPC
        let tx_hash = "0xfedcba0987654321".to_string();

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.entries.remove(domain);
        }

        info!("Domain {} records updated with tx hash: {}", domain, tx_hash);
        Ok(tx_hash)
    }

    /// Subscribe to domain changes
    pub async fn subscribe_domain_changes(
        &self,
        subscription: DomainSubscription,
    ) -> crate::Result<impl StreamExt<Item = std::result::Result<DomainChangeEvent, Status>>> {
        info!("Subscribing to changes for {} domains", subscription.domains.len());

        // TODO: Implement actual gRPC streaming subscription
        // For now, return an empty stream
        Ok(tokio_stream::empty())
    }

    /// Transfer domain ownership
    pub async fn transfer_domain(
        &self,
        domain: &str,
        current_owner: &Address,
        new_owner: &Address,
    ) -> Result<String> {
        info!("Transferring domain {} from {} to {}", domain, current_owner, new_owner);

        // Verify current ownership
        let resolution = self.resolve_domain(domain).await?;
        if resolution.owner != *current_owner {
            return Err(EtherlinkError::CnsResolution("Not domain owner".to_string()));
        }

        // TODO: Submit transfer via gRPC
        let tx_hash = "0x1122334455667788".to_string();

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.entries.remove(domain);
        }

        info!("Domain {} transferred with tx hash: {}", domain, tx_hash);
        Ok(tx_hash)
    }

    /// Renew domain registration
    pub async fn renew_domain(
        &self,
        domain: &str,
        owner: &Address,
        years: u32,
        payment_amount: u64,
    ) -> Result<String> {
        info!("Renewing domain {} for {} years", domain, years);

        // Verify ownership
        let resolution = self.resolve_domain(domain).await?;
        if resolution.owner != *owner {
            return Err(EtherlinkError::CnsResolution("Not domain owner".to_string()));
        }

        // TODO: Submit renewal via gRPC
        let tx_hash = "0x9988776655443322".to_string();

        info!("Domain {} renewed with tx hash: {}", domain, tx_hash);
        Ok(tx_hash)
    }

    /// Validate domain format
    fn validate_domain_format(&self, domain: &str) -> Result<()> {
        if domain.is_empty() {
            return Err(EtherlinkError::CnsResolution("Domain cannot be empty".to_string()));
        }

        if !domain.contains('.') {
            return Err(EtherlinkError::CnsResolution("Domain must contain a TLD".to_string()));
        }

        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return Err(EtherlinkError::CnsResolution("Invalid domain format".to_string()));
        }

        let tld = parts.last().unwrap();
        if !self.config.supported_tlds.contains(&tld.to_string())
            && *tld != "eth"
            && !["crypto", "nft", "x"].contains(tld) {
            return Err(EtherlinkError::CnsResolution(format!("Unsupported TLD: {}", tld)));
        }

        Ok(())
    }

    /// Clear expired cache entries
    pub async fn cleanup_cache(&self) {
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.clear_expired();
        }
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            (cache.entries.len(), cache.max_entries)
        } else {
            (0, 0)
        }
    }

    /// Get configuration
    pub fn config(&self) -> &CNSConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: CNSConfig) {
        self.config = config;
    }
}

impl Default for CNSClient {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Builder for CNS client
pub struct CNSClientBuilder {
    config: CNSConfig,
}

impl CNSClientBuilder {
    pub fn new() -> Self {
        Self {
            config: CNSConfig::default(),
        }
    }

    pub fn endpoint<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.config.enable_cache = enable;
        self
    }

    pub fn cache_ttl_seconds(mut self, ttl: u64) -> Self {
        self.config.cache_ttl_seconds = ttl;
        self
    }

    pub fn max_cache_entries(mut self, max: usize) -> Self {
        self.config.max_cache_entries = max;
        self
    }

    pub fn supported_tlds(mut self, tlds: Vec<String>) -> Self {
        self.config.supported_tlds = tlds;
        self
    }

    pub fn enable_ens_bridge(mut self, enable: bool) -> Self {
        self.config.enable_ens_bridge = enable;
        self
    }

    pub fn enable_unstoppable_bridge(mut self, enable: bool) -> Self {
        self.config.enable_unstoppable_bridge = enable;
        self
    }

    pub fn build(self) -> CNSClient {
        CNSClient::new(self.config)
    }
}

impl Default for CNSClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}