use thiserror::Error;

/// Custom error types for the Solana client application
#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),

    #[error("Borsh deserialization error: {0}")]
    BorshError(String),

    #[error("Solana keypair error: {0}")]
    KeypairError(String),

    #[error("Program deployment error: {0}")]
    DeploymentError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Program not found at {0}")]
    ProgramNotFound(String),

    #[error("Failed to connect to validator: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;
