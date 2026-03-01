use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Libp2p error: {0}")]
    Libp2p(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Message send failed: {0}")]
    SendFailed(String),
    
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Signal protocol error: {0}")]
    SignalProtocol(#[from] signal_protocol::error::SignalError),
}

pub type P2PResult<T> = Result<T, P2PError>;
