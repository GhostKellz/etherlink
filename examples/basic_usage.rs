//! Basic usage example for Etherlink GhostChain client

use etherlink::{
    EtherlinkClientBuilder, ServiceClients, ServiceClient,
    TransportConfig, HttpTransport,
    AuthCredentials, AuthSecret, Permission, TokenType,
    Address, clients::ghostd::Transaction
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    etherlink::init_with_tracing("info")?;

    // Create client configuration
    let client = EtherlinkClientBuilder::new()
        .ghostd_endpoint("https://testnet.ghostchain.org:8545")
        .cns_endpoint("https://testnet.ghostchain.org:8553")
        .enable_tls(true)
        .timeout_ms(10000)
        .build();

    println!("Created Etherlink client for endpoint: {}", client.config().ghostd_endpoint);

    // Create HTTP transport
    let transport_config = TransportConfig {
        use_gquic: false, // Use HTTP for compatibility
        enable_tls: true,
        timeout_ms: 10000,
        max_connections: 50,
        keepalive_interval_ms: 30000,
    };

    let transport = HttpTransport::new(transport_config)?;
    println!("Created HTTP transport");

    // Create service clients
    let http_client = Arc::new(reqwest::Client::new());
    let services = ServiceClients::new(client.config(), http_client);

    println!("Initialized all 6 GhostChain service clients:");
    println!("  - {} (blockchain daemon)", services.ghostd.service_name());
    println!("  - {} (wallet service)", services.walletd.service_name());
    println!("  - {} (identity service)", services.gid.service_name());
    println!("  - {} (name service)", services.cns.service_name());
    println!("  - {} (signature service)", services.gsig.service_name());
    println!("  - {} (token ledger)", services.gledger.service_name());

    // Example: Check service health (would fail without actual server)
    println!("\nAttempting health checks...");
    match services.ghostd.health_check().await {
        Ok(_) => println!("✅ GHOSTD service is healthy"),
        Err(e) => println!("❌ GHOSTD service unavailable: {}", e),
    }

    // Example: Create sample transaction
    let sample_tx = Transaction {
        from: Address::new("ghost1sender123456789abcdef123456789abcdef123456".to_string()),
        to: Address::new("ghost1receiver123456789abcdef123456789abcdef12345".to_string()),
        amount: 1000,
        gas_limit: 21000,
        gas_price: 100,
        nonce: 1,
        data: None,
        signature: None,
    };

    println!("\nCreated sample transaction:");
    println!("  From: {}", sample_tx.from);
    println!("  To: {}", sample_tx.to);
    println!("  Amount: {}", sample_tx.amount);

    // Example: Authentication credentials
    let auth_credentials = AuthCredentials {
        identity: "did:ghost:example123456789abcdef".to_string(),
        secret: AuthSecret::PrivateKey("sample_private_key".to_string()),
        permissions: vec![
            Permission::ReadBlockchain,
            Permission::TransferTokens(TokenType::GCC),
            Permission::RegisterDomain,
        ],
    };

    println!("\nCreated authentication credentials:");
    println!("  Identity: {}", auth_credentials.identity);
    println!("  Permissions: {} defined", auth_credentials.permissions.len());

    // Example: Token types
    let token_types = vec![
        TokenType::GCC,    // Gas & transaction fees
        TokenType::SPIRIT, // Governance & voting
        TokenType::MANA,   // Utility & rewards
        TokenType::GHOST,  // Brand & collectibles
    ];

    println!("\nSupported token types:");
    for token in token_types {
        println!("  - {:?}", token);
    }

    println!("\n🚀 Etherlink client successfully initialized and ready for GhostChain integration!");
    println!("   Next steps: Configure with actual GhostChain endpoints and authenticate");

    Ok(())
}