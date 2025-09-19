# Getting Started with Etherlink

## Installation

Add Etherlink to your `Cargo.toml`:

```toml
[dependencies]
etherlink = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Basic Usage

### 1. Initialize the Client

```rust
use etherlink::{EtherlinkClientBuilder, ServiceClients};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let client = EtherlinkClientBuilder::new()
        .ghostd_endpoint("https://testnet.ghostchain.org:8545")
        .cns_endpoint("https://testnet.ghostchain.org:8553")
        .enable_tls(true)
        .timeout_ms(10000)
        .build();

    // Create service clients
    let http_client = Arc::new(reqwest::Client::new());
    let services = ServiceClients::new(client.config(), http_client);

    Ok(())
}
```

### 2. Basic Operations

#### Check Service Health
```rust
match services.ghostd.health_check().await {
    Ok(_) => println!("✅ GHOSTD service is healthy"),
    Err(e) => println!("❌ GHOSTD service unavailable: {}", e),
}
```

#### Create a Transaction
```rust
use etherlink::{Transaction, Address};

let tx = Transaction {
    from: Address::new("ghost1sender123...".to_string()),
    to: Address::new("ghost1receiver123...".to_string()),
    amount: 1000,
    gas_limit: 21000,
    gas_price: 100,
    nonce: 1,
    data: None,
    signature: None,
};

// Submit transaction
let tx_hash = services.ghostd.submit_transaction(tx).await?;
println!("Transaction submitted: {}", tx_hash);
```

#### Register a Domain
```rust
let domain = "myapp.ghost";
let owner = Address::new("ghost1owner123...".to_string());

let tx_hash = services.cns.register_domain(domain.to_string(), owner).await?;
println!("Domain registration submitted: {}", tx_hash);
```

#### Check Token Balance
```rust
use etherlink::TokenType;

let address = Address::new("ghost1user123...".to_string());
let balance = services.gledger.get_balance(address, TokenType::GCC).await?;
println!("GCC Balance: {}", balance);
```

### 3. Authentication

```rust
use etherlink::{AuthCredentials, AuthSecret, Permission, TokenType};

let auth_credentials = AuthCredentials {
    identity: "did:ghost:example123456789abcdef".to_string(),
    secret: AuthSecret::PrivateKey("your_private_key".to_string()),
    permissions: vec![
        Permission::ReadBlockchain,
        Permission::TransferTokens(TokenType::GCC),
        Permission::RegisterDomain,
    ],
};
```

### 4. Token Operations

```rust
use etherlink::TokenType;

// Get all token balances
let balances = services.gledger.get_all_balances(address).await?;
println!("Token balances: {:?}", balances);

// Transfer tokens
let tx_hash = services.gledger.transfer_tokens(
    from_address,
    to_address,
    TokenType::SPIRIT,
    1000
).await?;
```

## Advanced Configuration

### Custom Transport Configuration

```rust
use etherlink::{TransportConfig, HttpTransport};

let transport_config = TransportConfig {
    use_gquic: false, // Use HTTP for compatibility
    enable_tls: true,
    timeout_ms: 10000,
    max_connections: 50,
    keepalive_interval_ms: 30000,
};

let transport = HttpTransport::new(transport_config)?;
```

### Enable GQUIC Transport

```toml
[dependencies]
etherlink = { version = "0.1.0", features = ["gquic"] }
```

```rust
let transport_config = TransportConfig {
    use_gquic: true, // Enable high-performance GQUIC
    enable_tls: true,
    timeout_ms: 5000,
    max_connections: 100,
    keepalive_interval_ms: 15000,
};
```

### Enable Enhanced Cryptography

```toml
[dependencies]
etherlink = { version = "0.1.0", features = ["gcrypt"] }
```

## Error Handling

```rust
use etherlink::EtherlinkError;

match services.ghostd.submit_transaction(tx).await {
    Ok(tx_hash) => println!("Success: {}", tx_hash),
    Err(EtherlinkError::Network(msg)) => println!("Network error: {}", msg),
    Err(EtherlinkError::Auth(msg)) => println!("Authentication error: {}", msg),
    Err(EtherlinkError::Api(msg)) => println!("API error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

See the `/examples` directory for complete working examples:

- `basic_usage.rs` - Basic client initialization and operations
- `domain_registration.rs` - CNS domain registration example
- `token_transfer.rs` - Multi-token transfer example
- `identity_management.rs` - GID identity creation and verification
- `multisig_wallet.rs` - Multi-signature wallet operations

## Next Steps

- [API Reference](../api/README.md) - Complete API documentation
- [Architecture Guide](../architecture/system-design.md) - Understanding the system
- [Integration Examples](../integration/examples.md) - Real-world integration patterns
- [Performance Guide](../architecture/performance.md) - Optimization tips