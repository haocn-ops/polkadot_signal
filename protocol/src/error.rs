use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignalError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("No one-time prekey available")]
    NoOneTimePreKey,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Decryption failed")]
    DecryptionFailed,
    
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Key derivation failed")]
    KeyDerivationFailed,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Chain key exhausted")]
    ChainKeyExhausted,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type SignalResult<T> = Result<T, SignalError>;
