# API Reference

## Core Client

### `EtherlinkClient`
Main client entry point for GhostChain ecosystem integration.

```rust
use etherlink::{EtherlinkClientBuilder, EtherlinkClient};

let client = EtherlinkClientBuilder::new()
    .ghostd_endpoint("https://mainnet.ghostchain.org:8545")
    .enable_tls(true)
    .timeout_ms(10000)
    .build();
```

## Service Clients

### GHOSTD - Blockchain Daemon
- `submit_transaction(tx: Transaction) -> Result<TxHash>`
- `get_block(height: u64) -> Result<Block>`
- `get_blockchain_height() -> Result<u64>`
- `get_balance(address: Address) -> Result<u64>`

### WALLETD - Wallet Service
- `create_wallet(config: WalletConfig) -> Result<WalletId>`
- `get_balance(wallet_id: WalletId, token_type: TokenType) -> Result<u64>`
- `transfer(from: Address, to: Address, amount: u64) -> Result<TxHash>`

### GID - Identity Service
- `create_identity(config: IdentityConfig) -> Result<DidIdentifier>`
- `verify_identity(did: DidIdentifier) -> Result<bool>`
- `get_identity_info(did: DidIdentifier) -> Result<IdentityInfo>`

### CNS - Crypto Name Server
- `register_domain(domain: String, owner: Address) -> Result<TxHash>`
- `resolve_domain(domain: String) -> Result<DomainInfo>`
- `transfer_domain(domain: String, new_owner: Address) -> Result<TxHash>`

### GSIG - Signature Service
- `sign_message(message: Vec<u8>, key_id: String) -> Result<Signature>`
- `verify_signature(message: Vec<u8>, signature: Signature, public_key: String) -> Result<bool>`
- `create_multisig(participants: Vec<Address>, threshold: u32) -> Result<MultisigAddress>`

### GLEDGER - Token Ledger
- `transfer_tokens(from: Address, to: Address, token_type: TokenType, amount: u64) -> Result<TxHash>`
- `get_balance(address: Address, token_type: TokenType) -> Result<u64>`
- `get_all_balances(address: Address) -> Result<TokenBalances>`
- `get_token_economics(token_type: TokenType) -> Result<TokenEconomics>`

## Transport Layer

### Transport Trait
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_json_request(&self, endpoint: &str, request: serde_json::Value) -> Result<serde_json::Value>;
    async fn health_check(&self, endpoint: &str) -> Result<()>;
    async fn get_stats(&self) -> Result<TransportStats>;
}
```

### GQUIC Transport
High-performance QUIC transport implementation.

### HTTP Transport
Fallback HTTP transport for compatibility.

## Authentication

### Guardian Authentication Provider
- `authenticate(credentials: AuthCredentials) -> Result<AuthToken>`
- `refresh_token(token: AuthToken) -> Result<AuthToken>`
- `validate_permissions(token: AuthToken, permission: Permission) -> Result<bool>`

### Crypto Provider
- `generate_keypair(algorithm: CryptoAlgorithm) -> Result<KeyPair>`
- `sign_message(message: &[u8], private_key: &str, algorithm: CryptoAlgorithm) -> Result<String>`
- `verify_signature(message: &[u8], signature: &str, public_key: &str, algorithm: CryptoAlgorithm) -> Result<bool>`

## Token Types

### GCC (⚡) - Gas & Transaction Fees
Deflationary token used for transaction fees and gas.

### SPIRIT (🗳️) - Governance & Voting
Fixed supply token used for governance and voting rights.

### MANA (✨) - Utility & Rewards
Inflationary token used for utility functions and rewards.

### GHOST (👻) - Brand & Collectibles
Burn-to-mint token used for brand identity and collectibles.

## Error Handling

### EtherlinkError
```rust
pub enum EtherlinkError {
    Network(String),
    Auth(String),
    Crypto(String),
    Api(String),
    Config(String),
    Transport(String),
}
```