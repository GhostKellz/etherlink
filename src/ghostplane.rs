use crate::{ffi::ZigBridge, EtherlinkError, Result, Address, TxHash, BlockHeight};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// GhostPlane L2 client for high-performance Zig-based execution
#[derive(Debug)]
pub struct GhostPlaneClient {
    bridge: ZigBridge,
    config: GhostPlaneConfig,
    state: RwLock<GhostPlaneState>,
}

/// Configuration for GhostPlane L2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostPlaneConfig {
    pub endpoint: String,
    pub chain_id: u64,
    pub batch_size: usize,
    pub finalization_timeout_ms: u64,
    pub enable_zk_proofs: bool,
}

impl Default for GhostPlaneConfig {
    fn default() -> Self {
        Self {
            endpoint: "localhost:9090".to_string(),
            chain_id: 1337,
            batch_size: 1000,
            finalization_timeout_ms: 30000,
            enable_zk_proofs: true,
        }
    }
}

/// GhostPlane L2 state tracker
#[derive(Debug, Clone)]
pub struct GhostPlaneState {
    pub current_block: BlockHeight,
    pub pending_transactions: HashMap<TxHash, L2Transaction>,
    pub finalized_batches: Vec<BatchInfo>,
    pub total_transactions: u64,
}

impl Default for GhostPlaneState {
    fn default() -> Self {
        Self {
            current_block: 0,
            pending_transactions: HashMap::new(),
            finalized_batches: Vec::new(),
            total_transactions: 0,
        }
    }
}

/// Layer 2 transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Transaction {
    pub from: Address,
    pub to: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
}

/// Batch information for L1 commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInfo {
    pub batch_id: String,
    pub transactions: Vec<TxHash>,
    pub merkle_root: String,
    pub zk_proof: Option<Vec<u8>>,
    pub l1_commitment_hash: Option<String>,
    pub finalized_at: u64,
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2ExecutionResult {
    pub tx_hash: TxHash,
    pub success: bool,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<String>,
    pub state_changes: HashMap<String, Vec<u8>>,
}

impl GhostPlaneClient {
    /// Create a new GhostPlane client
    pub fn new(config: GhostPlaneConfig) -> Self {
        Self {
            bridge: ZigBridge::new(),
            config,
            state: RwLock::new(GhostPlaneState::default()),
        }
    }

    /// Create a new GhostPlane client with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GhostPlaneConfig::default())
    }

    /// Initialize the GhostPlane client and Zig bridge
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing GhostPlane client");

        self.bridge.initialize()?;

        // Initialize L2 state
        {
            let mut state = self.state.write().await;
            *state = GhostPlaneState::default();
        }

        info!("GhostPlane client initialized successfully");
        Ok(())
    }

    /// Submit a transaction to GhostPlane L2
    pub async fn submit_transaction(&self, tx: L2Transaction) -> Result<TxHash> {
        debug!("Submitting L2 transaction from {} to {}", tx.from, tx.to);

        // Serialize transaction for Zig
        let tx_bytes = serde_json::to_vec(&tx)
            .map_err(|e| EtherlinkError::Serialization(e))?;

        // Submit via FFI bridge
        let tx_hash_str = self.bridge.submit_ghostplane_transaction(&tx_bytes).await?;
        let tx_hash = TxHash::new(tx_hash_str);

        // Update local state
        {
            let mut state = self.state.write().await;
            state.pending_transactions.insert(tx_hash.clone(), tx);
            state.total_transactions += 1;
        }

        debug!("L2 transaction submitted with hash: {}", tx_hash.as_str());
        Ok(tx_hash)
    }

    /// Execute a transaction on GhostPlane and get the result
    pub async fn execute_transaction(&self, tx: L2Transaction) -> Result<L2ExecutionResult> {
        let tx_hash = self.submit_transaction(tx).await?;

        // TODO: Wait for execution and get result
        // For now, return a placeholder result
        Ok(L2ExecutionResult {
            tx_hash,
            success: true,
            gas_used: 21000,
            output: Vec::new(),
            logs: Vec::new(),
            state_changes: HashMap::new(),
        })
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_hash: &TxHash) -> Result<Option<L2ExecutionResult>> {
        let state = self.state.read().await;

        if state.pending_transactions.contains_key(tx_hash) {
            // Transaction is pending
            Ok(None)
        } else {
            // TODO: Query finalized transaction status from GhostPlane
            Ok(None)
        }
    }

    /// Create a batch of pending transactions
    pub async fn create_batch(&self) -> Result<BatchInfo> {
        let mut state = self.state.write().await;

        let pending_txs: Vec<TxHash> = state.pending_transactions.keys().cloned().collect();

        if pending_txs.is_empty() {
            return Err(EtherlinkError::General(anyhow::anyhow!("No pending transactions for batch")));
        }

        let batch_id = uuid::Uuid::new_v4().to_string();
        let merkle_root = self.calculate_merkle_root(&pending_txs).await?;

        let batch = BatchInfo {
            batch_id,
            transactions: pending_txs.clone(),
            merkle_root,
            zk_proof: None,
            l1_commitment_hash: None,
            finalized_at: 0,
        };

        // Clear pending transactions (they're now in batch)
        for tx_hash in &pending_txs {
            state.pending_transactions.remove(tx_hash);
        }

        debug!("Created batch with {} transactions", pending_txs.len());
        Ok(batch)
    }

    /// Generate ZK proof for a batch (via Zig)
    pub async fn generate_batch_proof(&self, batch: &BatchInfo) -> Result<Vec<u8>> {
        if !self.config.enable_zk_proofs {
            warn!("ZK proofs disabled in configuration");
            return Ok(Vec::new());
        }

        debug!("Generating ZK proof for batch {}", batch.batch_id);

        // TODO: Generate actual ZK proof via Zig bridge
        // For now, return placeholder proof
        Ok(vec![0u8; 128])
    }

    /// Submit batch to L1 for finalization
    pub async fn finalize_batch(&self, mut batch: BatchInfo, proof: Vec<u8>) -> Result<String> {
        batch.zk_proof = Some(proof);
        batch.finalized_at = chrono::Utc::now().timestamp() as u64;

        // TODO: Submit to L1 via bridge
        let l1_commitment = format!("0x{}", hex::encode(&batch.batch_id));
        batch.l1_commitment_hash = Some(l1_commitment.clone());

        // Update state
        {
            let mut state = self.state.write().await;
            state.finalized_batches.push(batch);
            state.current_block += 1;
        }

        info!("Batch finalized with L1 commitment: {}", l1_commitment);
        Ok(l1_commitment)
    }

    /// Get current L2 state information
    pub async fn get_state_info(&self) -> GhostPlaneState {
        self.state.read().await.clone()
    }

    /// Query L2 state via Zig bridge
    pub async fn query_state(&self, query: &str) -> Result<String> {
        debug!("Querying GhostPlane state: {}", query);
        self.bridge.query_ghostplane_state(query).await
    }

    /// Get pending transaction count
    pub async fn pending_transaction_count(&self) -> usize {
        self.state.read().await.pending_transactions.len()
    }

    /// Get total transaction count
    pub async fn total_transaction_count(&self) -> u64 {
        self.state.read().await.total_transactions
    }

    /// Calculate merkle root for transactions (placeholder implementation)
    async fn calculate_merkle_root(&self, tx_hashes: &[TxHash]) -> Result<String> {
        // TODO: Implement proper merkle tree calculation
        let combined = tx_hashes.iter()
            .map(|h| h.as_str())
            .collect::<Vec<_>>()
            .join("");

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();

        Ok(format!("0x{}", hex::encode(result)))
    }

    /// Shutdown the GhostPlane client
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down GhostPlane client");
        self.bridge.shutdown()?;
        Ok(())
    }
}

impl Default for GhostPlaneClient {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Builder for GhostPlane client
pub struct GhostPlaneClientBuilder {
    config: GhostPlaneConfig,
}

impl GhostPlaneClientBuilder {
    pub fn new() -> Self {
        Self {
            config: GhostPlaneConfig::default(),
        }
    }

    pub fn endpoint<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.config.chain_id = chain_id;
        self
    }

    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    pub fn finalization_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.finalization_timeout_ms = timeout;
        self
    }

    pub fn enable_zk_proofs(mut self, enable: bool) -> Self {
        self.config.enable_zk_proofs = enable;
        self
    }

    pub fn build(self) -> GhostPlaneClient {
        GhostPlaneClient::new(self.config)
    }
}

impl Default for GhostPlaneClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}