# Etherlink - GhostChain Rust Client SDK

A high-performance Rust client SDK for the GhostChain ecosystem, providing secure and efficient communication with all GhostChain services using GQUIC transport and comprehensive authentication.

![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)
![gRPC](https://img.shields.io/badge/protocol-gRPC-blue?logo=grpc)
![QUIC](https://img.shields.io/badge/transport-QUIC%2FHTTP3-teal?logo=quic)
![License](https://img.shields.io/badge/license-Apache--2.0-lightgrey)

## 🚀 Features

### Complete Service Coverage
- **GHOSTD** - Blockchain daemon client (port 8545)
- **WALLETD** - Wallet management service (port 8548)
- **GID** - Ghost Identity system (port 8552)
- **CNS** - Crypto Name Server (port 8553)
- **GSIG** - Signature verification service (port 8554)
- **GLEDGER** - Token ledger operations (port 8555)

### Advanced Transport Layer
- **GQUIC** - High-performance QUIC transport from [gquic](https://github.com/ghostkellz/gquic)
- **HTTP/REST** - Fallback HTTP transport for compatibility
- **Connection pooling** and automatic retry with exponential backoff
- **TLS/SSL** support with certificate validation

### Authentication & Security
- **Guardian Framework** - Zero-trust policy-based authentication
- **Multi-algorithm crypto** - Ed25519, Secp256k1, BLS12-381 via [gcrypt](https://github.com/ghostkellz/gcrypt)
- **Token-based permissions** - Fine-grained access control
- **DID-compatible** identity management

### Token Economy Integration
- **GCC (⚡)** - Gas & transaction fees (deflationary)
- **SPIRIT (🗳️)** - Governance & voting (fixed supply)
- **MANA (✨)** - Utility & rewards (inflationary)
- **GHOST (👻)** - Brand & collectibles (burn-to-mint)
