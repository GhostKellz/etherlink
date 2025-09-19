use crate::{EtherlinkError, Result, Address, TxHash, Gas};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// rEVM (Rust Ethereum Virtual Machine) integration for EVM compatibility
#[derive(Debug)]
pub struct REVMClient {
    config: REVMConfig,
    state: EvmState,
}

/// Configuration for rEVM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REVMConfig {
    pub chain_id: u64,
    pub gas_limit: Gas,
    pub gas_price: Gas,
    pub enable_london_hardfork: bool,
    pub enable_shanghai_hardfork: bool,
    pub enable_cancun_hardfork: bool,
    pub precompiles_enabled: bool,
}

impl Default for REVMConfig {
    fn default() -> Self {
        Self {
            chain_id: 1337, // GhostChain testnet
            gas_limit: 30_000_000,
            gas_price: 1_000_000_000, // 1 gwei
            enable_london_hardfork: true,
            enable_shanghai_hardfork: true,
            enable_cancun_hardfork: false,
            precompiles_enabled: true,
        }
    }
}

/// EVM state management
#[derive(Debug, Clone)]
pub struct EvmState {
    pub accounts: HashMap<Address, AccountInfo>,
    pub storage: HashMap<Address, HashMap<String, Vec<u8>>>,
    pub codes: HashMap<Address, Vec<u8>>,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub block_gas_limit: Gas,
}

impl Default for EvmState {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            storage: HashMap::new(),
            codes: HashMap::new(),
            block_number: 0,
            block_timestamp: chrono::Utc::now().timestamp() as u64,
            block_gas_limit: 30_000_000,
        }
    }
}

/// Account information in EVM state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: Option<String>,
    pub storage_root: Option<String>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            code_hash: None,
            storage_root: None,
        }
    }
}

/// EVM transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransaction {
    pub from: Address,
    pub to: Option<Address>,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas_limit: Gas,
    pub gas_price: Gas,
    pub nonce: u64,
    pub chain_id: u64,
    pub signature: EvmSignature,
}

/// EVM transaction signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmSignature {
    pub v: u64,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}

/// EVM execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmExecutionResult {
    pub success: bool,
    pub gas_used: Gas,
    pub gas_refunded: Gas,
    pub output: Vec<u8>,
    pub logs: Vec<EvmLog>,
    pub state_changes: HashMap<Address, AccountChange>,
    pub created_address: Option<Address>,
    pub revert_reason: Option<String>,
}

/// EVM log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmLog {
    pub address: Address,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
}

/// Account state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountChange {
    pub balance_change: Option<i64>,
    pub nonce_change: Option<i64>,
    pub code_change: Option<Vec<u8>>,
    pub storage_changes: HashMap<String, Vec<u8>>,
}

/// EVM call parameters
#[derive(Debug, Clone)]
pub struct EvmCallParams {
    pub caller: Address,
    pub to: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas_limit: Gas,
    pub is_static: bool,
}

impl REVMClient {
    /// Create a new rEVM client
    pub fn new(config: REVMConfig) -> Self {
        Self {
            config,
            state: EvmState::default(),
        }
    }

    /// Create a new rEVM client with default configuration
    pub fn with_defaults() -> Self {
        Self::new(REVMConfig::default())
    }

    /// Initialize the rEVM client
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing rEVM client with chain ID {}", self.config.chain_id);

        // Set up initial state
        self.state.block_number = 0;
        self.state.block_timestamp = chrono::Utc::now().timestamp() as u64;
        self.state.block_gas_limit = self.config.gas_limit;

        // TODO: Load precompiled contracts if enabled
        if self.config.precompiles_enabled {
            self.setup_precompiles().await?;
        }

        info!("rEVM client initialized successfully");
        Ok(())
    }

    /// Execute an EVM transaction
    pub async fn execute_transaction(&mut self, tx: EvmTransaction) -> Result<EvmExecutionResult> {
        debug!("Executing EVM transaction from {} to {:?}", tx.from, tx.to);

        // Validate transaction
        self.validate_transaction(&tx)?;

        // Check account balance and nonce
        let sender_account = self.get_or_create_account(&tx.from);
        if sender_account.nonce != tx.nonce {
            return Err(EtherlinkError::ContractExecution(
                format!("Invalid nonce: expected {}, got {}", sender_account.nonce, tx.nonce)
            ));
        }

        let total_cost = tx.value + (tx.gas_limit * tx.gas_price);
        if sender_account.balance < total_cost {
            return Err(EtherlinkError::ContractExecution("Insufficient balance".to_string()));
        }

        // Execute transaction
        let result = if tx.to.is_some() {
            self.execute_call(&tx).await?
        } else {
            self.execute_create(&tx).await?
        };

        // Apply state changes
        if result.success {
            self.apply_state_changes(&tx, &result).await?;
        }

        debug!("EVM transaction executed, gas used: {}", result.gas_used);
        Ok(result)
    }

    /// Call a contract method (read-only)
    pub async fn call_contract(&self, params: EvmCallParams) -> Result<Vec<u8>> {
        debug!("Calling EVM contract at {} (read-only)", params.to);

        // Get contract code
        let code = self.state.codes.get(&params.to)
            .ok_or_else(|| EtherlinkError::ContractExecution("Contract not found".to_string()))?;

        if code.is_empty() {
            return Err(EtherlinkError::ContractExecution("Contract has no code".to_string()));
        }

        // Execute read-only call
        let result = self.execute_code(&params, code).await?;

        if result.success {
            Ok(result.output)
        } else {
            Err(EtherlinkError::ContractExecution(
                result.revert_reason.unwrap_or_else(|| "Call reverted".to_string())
            ))
        }
    }

    /// Deploy a new contract
    pub async fn deploy_contract(
        &mut self,
        deployer: Address,
        bytecode: Vec<u8>,
        constructor_data: Vec<u8>,
        gas_limit: Gas,
        value: u64,
    ) -> Result<(Address, EvmExecutionResult)> {
        info!("Deploying EVM contract from {}", deployer);

        // Create deployment transaction
        let tx = EvmTransaction {
            from: deployer.clone(),
            to: None, // Contract creation
            value,
            data: [bytecode, constructor_data].concat(),
            gas_limit,
            gas_price: self.config.gas_price,
            nonce: self.get_account_nonce(&deployer),
            chain_id: self.config.chain_id,
            signature: EvmSignature {
                v: 0,
                r: vec![],
                s: vec![],
            },
        };

        let result = self.execute_transaction(tx).await?;

        if let Some(address) = &result.created_address {
            info!("Contract deployed successfully at {}", address);
            Ok((address.clone(), result))
        } else {
            Err(EtherlinkError::ContractExecution("Contract deployment failed".to_string()))
        }
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.state.accounts.get(address)
            .map(|acc| acc.balance)
            .unwrap_or(0)
    }

    /// Get account nonce
    pub fn get_account_nonce(&self, address: &Address) -> u64 {
        self.state.accounts.get(address)
            .map(|acc| acc.nonce)
            .unwrap_or(0)
    }

    /// Set account balance (for testing)
    pub fn set_balance(&mut self, address: Address, balance: u64) {
        let account = self.get_or_create_account(&address);
        account.balance = balance;
    }

    /// Get contract code
    pub fn get_code(&self, address: &Address) -> Option<&Vec<u8>> {
        self.state.codes.get(address)
    }

    /// Get storage value
    pub fn get_storage(&self, address: &Address, key: &str) -> Option<&Vec<u8>> {
        self.state.storage.get(address)?.get(key)
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: &EvmTransaction) -> Result<Gas> {
        debug!("Estimating gas for EVM transaction");

        // TODO: Implement actual gas estimation
        // For now, return a conservative estimate based on transaction type
        let base_gas = if tx.to.is_none() {
            53000 // Contract creation
        } else {
            21000 // Simple transfer
        };

        let data_gas = tx.data.len() as Gas * 16; // 16 gas per byte of data
        Ok(base_gas + data_gas)
    }

    /// Execute a contract call transaction
    async fn execute_call(&self, tx: &EvmTransaction) -> Result<EvmExecutionResult> {
        let to = tx.to.as_ref().unwrap();

        // Get contract code
        let code = self.state.codes.get(to);

        if let Some(code) = code {
            if !code.is_empty() {
                // Contract call
                let params = EvmCallParams {
                    caller: tx.from.clone(),
                    to: to.clone(),
                    value: tx.value,
                    data: tx.data.clone(),
                    gas_limit: tx.gas_limit,
                    is_static: false,
                };
                return self.execute_code(&params, code).await;
            }
        }

        // Simple transfer
        Ok(EvmExecutionResult {
            success: true,
            gas_used: 21000,
            gas_refunded: 0,
            output: Vec::new(),
            logs: Vec::new(),
            state_changes: HashMap::new(),
            created_address: None,
            revert_reason: None,
        })
    }

    /// Execute a contract creation transaction
    async fn execute_create(&self, tx: &EvmTransaction) -> Result<EvmExecutionResult> {
        // Generate contract address
        let contract_address = self.generate_contract_address(&tx.from, tx.nonce);

        // TODO: Execute constructor and deploy code
        debug!("Creating contract at {}", contract_address);

        Ok(EvmExecutionResult {
            success: true,
            gas_used: 53000,
            gas_refunded: 0,
            output: Vec::new(),
            logs: Vec::new(),
            state_changes: HashMap::new(),
            created_address: Some(contract_address),
            revert_reason: None,
        })
    }

    /// Execute contract code
    async fn execute_code(&self, params: &EvmCallParams, code: &[u8]) -> Result<EvmExecutionResult> {
        debug!("Executing {} bytes of EVM bytecode", code.len());

        // TODO: Implement actual EVM bytecode execution
        // For now, return a placeholder result

        Ok(EvmExecutionResult {
            success: true,
            gas_used: 50000,
            gas_refunded: 0,
            output: Vec::new(),
            logs: Vec::new(),
            state_changes: HashMap::new(),
            created_address: None,
            revert_reason: None,
        })
    }

    /// Validate transaction
    fn validate_transaction(&self, tx: &EvmTransaction) -> Result<()> {
        if tx.gas_limit == 0 {
            return Err(EtherlinkError::ContractExecution("Gas limit cannot be zero".to_string()));
        }

        if tx.gas_limit > self.config.gas_limit {
            return Err(EtherlinkError::ContractExecution("Gas limit too high".to_string()));
        }

        if tx.chain_id != self.config.chain_id {
            return Err(EtherlinkError::ContractExecution("Invalid chain ID".to_string()));
        }

        Ok(())
    }

    /// Get or create account
    fn get_or_create_account(&mut self, address: &Address) -> &mut AccountInfo {
        self.state.accounts.entry(address.clone()).or_insert_with(AccountInfo::default)
    }

    /// Generate contract address
    fn generate_contract_address(&self, deployer: &Address, nonce: u64) -> Address {
        // TODO: Implement proper CREATE address generation
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(deployer.as_str().as_bytes());
        hasher.update(nonce.to_le_bytes());
        let hash = hasher.finalize();
        Address::new(format!("0x{}", hex::encode(&hash[..20])))
    }

    /// Apply state changes after successful execution
    async fn apply_state_changes(&mut self, tx: &EvmTransaction, result: &EvmExecutionResult) -> Result<()> {
        // Update sender account
        let sender = self.get_or_create_account(&tx.from);
        sender.nonce += 1;
        sender.balance -= tx.gas_limit * tx.gas_price; // Deduct gas cost
        sender.balance += (tx.gas_limit - result.gas_used) * tx.gas_price; // Refund unused gas

        if let Some(to) = &tx.to {
            // Update recipient balance
            let recipient = self.get_or_create_account(to);
            recipient.balance += tx.value;
        }

        // Apply other state changes
        for (address, change) in &result.state_changes {
            let account = self.get_or_create_account(address);

            if let Some(balance_change) = change.balance_change {
                account.balance = (account.balance as i64 + balance_change) as u64;
            }

            if let Some(nonce_change) = change.nonce_change {
                account.nonce = (account.nonce as i64 + nonce_change) as u64;
            }

            if let Some(code) = &change.code_change {
                self.state.codes.insert(address.clone(), code.clone());
            }

            for (key, value) in &change.storage_changes {
                self.state.storage
                    .entry(address.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    /// Setup precompiled contracts
    async fn setup_precompiles(&mut self) -> Result<()> {
        debug!("Setting up EVM precompiled contracts");

        // TODO: Implement precompiled contracts (ecrecover, sha256, ripemd160, etc.)

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &REVMConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: REVMConfig) {
        self.config = config;
    }
}

impl Default for REVMClient {
    fn default() -> Self {
        Self::with_defaults()
    }
}