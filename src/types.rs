use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Address type for blockchain addresses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: String) -> Self {
        Self(addr)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction hash type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(pub String);

impl TxHash {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Block height type
pub type BlockHeight = u64;

/// Gas limit and gas used types
pub type Gas = u64;

/// Configuration for Etherlink client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherlinkConfig {
    pub ghostd_endpoint: String,
    pub cns_endpoint: Option<String>,
    pub ghostplane_endpoint: Option<String>,
    pub use_quic: bool,
    pub enable_tls: bool,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
}

impl Default for EtherlinkConfig {
    fn default() -> Self {
        Self {
            ghostd_endpoint: "http://localhost:8545".to_string(),
            cns_endpoint: None,
            ghostplane_endpoint: None,
            use_quic: false,
            enable_tls: true,
            timeout_ms: 30000,
            retry_attempts: 3,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    Local,
    Testnet,
    Mainnet,
    Custom(String),
}

/// Token types supported by GhostChain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    GCC,    // Gas & transaction fees
    SPIRIT, // Governance & voting
    MANA,   // Utility & rewards
    GHOST,  // Brand & collectibles
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub tx_hash: TxHash,
    pub block_height: BlockHeight,
    pub gas_used: Gas,
    pub success: bool,
    pub logs: Vec<String>,
}

/// Connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service_name: String,
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub last_block_height: Option<BlockHeight>,
    pub metadata: HashMap<String, String>,
}