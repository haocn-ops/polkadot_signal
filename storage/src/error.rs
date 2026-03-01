use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IPFS error: {0}")]
    Ipfs(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Message not found: {0}")]
    NotFound(String),
    
    #[error("Invalid CID: {0}")]
    InvalidCid(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Signal protocol error: {0}")]
    SignalProtocol(#[from] signal_protocol::error::SignalError),
}

pub type StorageResult<T> = Result<T, StorageError>;
