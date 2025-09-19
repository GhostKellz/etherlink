use thiserror::Error;

pub type Result<T> = std::result::Result<T, EtherlinkError>;

#[derive(Error, Debug)]
pub enum EtherlinkError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),

    #[error("QUIC connection error: {0}")]
    Quic(#[from] quinn::ConnectionError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("FFI error: {0}")]
    Ffi(String),

    #[error("CNS resolution error: {0}")]
    CnsResolution(String),

    #[error("RVM execution error: {0}")]
    RvmExecution(String),

    #[error("Contract execution error: {0}")]
    ContractExecution(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("General error: {0}")]
    General(#[from] anyhow::Error),
}