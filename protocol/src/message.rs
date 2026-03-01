use crate::keys::IdentityKey;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    PreKey,
    Regular,
    KeyUpdate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    pub message_type: MessageType,
    pub sender_identity: IdentityKey,
    pub receiver_identity: IdentityKey,
    pub ephemeral_public_key: Vec<u8>,
    pub used_prekey_id: u32,
    pub used_one_time_prekey_id: Option<u32>,
    pub message: Option<Message>,
}

impl Envelope {
    pub fn to_bytes(&self) -> crate::error::SignalResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
    
    pub fn from_bytes(data: &[u8]) -> crate::error::SignalResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
    
    pub fn to_base64(&self) -> crate::error::SignalResult<String> {
        let bytes = self.to_bytes()?;
        Ok(BASE64.encode(&bytes))
    }
    
    pub fn from_base64(encoded: &str) -> crate::error::SignalResult<Self> {
        let bytes = BASE64.decode(encoded)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))?;
        Self::from_bytes(&bytes)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub message_number: u32,
    pub previous_chain_length: u32,
    pub ephemeral_public_key: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreKeyMessage {
    pub registration_id: u32,
    pub signed_pre_key_id: u32,
    pub one_time_pre_key_id: Option<u32>,
    pub base_key: Vec<u8>,
    pub identity_key: IdentityKey,
    pub message: Message,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenderKeyMessage {
    pub chain_id: Vec<u8>,
    pub chain_key: Vec<u8>,
    pub iteration: u32,
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaintextContent {
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub content_type: String,
    pub data: Vec<u8>,
    pub filename: Option<String>,
}

impl PlaintextContent {
    pub fn new(body: String) -> Self {
        Self {
            body,
            attachments: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    pub fn add_attachment(&mut self, content_type: String, data: Vec<u8>, filename: Option<String>) {
        self.attachments.push(Attachment {
            content_type,
            data,
            filename,
        });
    }
    
    pub fn to_bytes(&self) -> crate::error::SignalResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
    
    pub fn from_bytes(data: &[u8]) -> crate::error::SignalResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
}
