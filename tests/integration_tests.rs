//! Integration tests for Etherlink GhostChain client

use etherlink::{
    EtherlinkClient, EtherlinkConfig, EtherlinkClientBuilder,
    ServiceClients, GhostdClient, GledgerClient, CnsClient,
    Transport, TransportConfig, HttpTransport,
    AuthCredentials, AuthSecret, Permission, TokenType,
    Address, TxHash
};
use std::sync::Arc;
use reqwest::Client as HttpClient;

#[tokio::test]
async fn test_client_creation() {
    let config = EtherlinkConfig::default();
    let client = EtherlinkClient::new(config);

    assert_eq!(client.config().ghostd_endpoint, "http://localhost:8545");
}

#[tokio::test]
async fn test_client_builder() {
    let client = EtherlinkClientBuilder::new()
        .ghostd_endpoint("https://testnet.ghostchain.org:8545")
        .cns_endpoint("https://testnet.ghostchain.org:8553")
        .enable_tls(true)
        .timeout_ms(10000)
        .build();

    assert_eq!(client.config().ghostd_endpoint, "https://testnet.ghostchain.org:8545");
    assert_eq!(client.config().cns_endpoint, Some("https://testnet.ghostchain.org:8553".to_string()));
    assert_eq!(client.config().enable_tls, true);
    assert_eq!(client.config().timeout_ms, 10000);
}

#[tokio::test]
async fn test_service_clients_creation() {
    let config = EtherlinkConfig::default();
    let http_client = Arc::new(HttpClient::new());
    let clients = ServiceClients::new(&config, http_client);

    assert_eq!(clients.ghostd.service_name(), "ghostd");
    assert_eq!(clients.gledger.service_name(), "gledger");
    assert_eq!(clients.cns.service_name(), "cns");
}

#[tokio::test]
async fn test_transport_config() {
    let config = TransportConfig {
        use_gquic: true,
        enable_tls: true,
        timeout_ms: 5000,
        max_connections: 50,
        keepalive_interval_ms: 30000,
    };

    assert_eq!(config.use_gquic, true);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.max_connections, 50);
}

#[tokio::test]
async fn test_http_transport_creation() {
    let config = TransportConfig::default();
    let transport = HttpTransport::new(config);

    assert!(transport.is_ok());
}

#[tokio::test]
async fn test_address_creation() {
    let address = Address::new("ghost1234567890abcdef1234567890abcdef12345678".to_string());
    assert_eq!(address.as_str(), "ghost1234567890abcdef1234567890abcdef12345678");

    let address_display = format!("{}", address);
    assert_eq!(address_display, "ghost1234567890abcdef1234567890abcdef12345678");
}

#[tokio::test]
async fn test_tx_hash_creation() {
    let tx_hash = TxHash::new("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string());
    assert_eq!(tx_hash.as_str(), "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
}

#[tokio::test]
async fn test_auth_credentials() {
    let credentials = AuthCredentials {
        identity: "did:ghost:1234567890abcdef".to_string(),
        secret: AuthSecret::PrivateKey("secret_key".to_string()),
        permissions: vec![
            Permission::ReadBlockchain,
            Permission::TransferTokens(TokenType::GCC),
            Permission::RegisterDomain,
        ],
    };

    assert_eq!(credentials.identity, "did:ghost:1234567890abcdef");
    assert_eq!(credentials.permissions.len(), 3);
}

#[tokio::test]
async fn test_token_types() {
    let tokens = vec![
        TokenType::GCC,
        TokenType::SPIRIT,
        TokenType::MANA,
        TokenType::GHOST,
    ];

    assert_eq!(tokens.len(), 4);
}

#[cfg(test)]
mod mock_server_tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_ghostd_health_check() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Set up mock response
        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "healthy",
                "service": "ghostd",
                "version": "0.1.0"
            })))
            .mount(&mock_server)
            .await;

        // Create client with mock server URL
        let mut config = EtherlinkConfig::default();
        config.ghostd_endpoint = mock_server.uri();

        let http_client = Arc::new(HttpClient::new());
        let ghostd_client = GhostdClient::new(&config, http_client);

        // Test health check
        let result = ghostd_client.health_check().await;
        assert!(result.is_ok());

        let health_data = result.unwrap();
        assert_eq!(health_data["status"], "healthy");
        assert_eq!(health_data["service"], "ghostd");
    }

    #[tokio::test]
    async fn test_gledger_token_balances() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/tokens/balances/ghost1234567890abcdef1234567890abcdef12345678"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": {
                    "address": "ghost1234567890abcdef1234567890abcdef12345678",
                    "gcc": 1000,
                    "spirit": 500,
                    "mana": 2000,
                    "ghost": 10
                }
            })))
            .mount(&mock_server)
            .await;

        let mut config = EtherlinkConfig::default();
        config.ghostd_endpoint = mock_server.uri();

        let http_client = Arc::new(HttpClient::new());
        let gledger_client = GledgerClient::new(&config, http_client);

        let address = Address::new("ghost1234567890abcdef1234567890abcdef12345678".to_string());
        let result = gledger_client.get_all_balances(&address).await;

        assert!(result.is_ok());
        let balances = result.unwrap();
        assert_eq!(balances.gcc, 1000);
        assert_eq!(balances.spirit, 500);
        assert_eq!(balances.mana, 2000);
        assert_eq!(balances.ghost, 10);
    }
}

#[cfg(test)]
mod crypto_tests {
    use super::*;
    use etherlink::auth::crypto::{CryptoProvider, CryptoAlgorithm};

    #[tokio::test]
    async fn test_ed25519_keypair_generation() {
        let provider = CryptoProvider::new();
        let result = provider.generate_keypair(&CryptoAlgorithm::Ed25519);

        assert!(result.is_ok());
        let keypair = result.unwrap();
        assert_eq!(keypair.algorithm, CryptoAlgorithm::Ed25519);
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.public_key.is_empty());
    }

    #[tokio::test]
    async fn test_ed25519_sign_verify() {
        let provider = CryptoProvider::new();
        let keypair = provider.generate_keypair(&CryptoAlgorithm::Ed25519).unwrap();

        let message = b"Hello, GhostChain!";
        let signature = provider.sign_message(message, &keypair.private_key, &CryptoAlgorithm::Ed25519);

        assert!(signature.is_ok());
        let sig = signature.unwrap();

        let verification = provider.verify_signature(message, &sig, &keypair.public_key, &CryptoAlgorithm::Ed25519);
        assert!(verification.is_ok());
        assert_eq!(verification.unwrap(), true);

        // Test with wrong message
        let wrong_message = b"Wrong message";
        let wrong_verification = provider.verify_signature(wrong_message, &sig, &keypair.public_key, &CryptoAlgorithm::Ed25519);
        assert!(wrong_verification.is_ok());
        assert_eq!(wrong_verification.unwrap(), false);
    }
}