use crate::error::{StorageError, StorageResult};
use crate::ipfs::IpfsClient;
use serde::{Deserialize, Serialize};
use signal_protocol::message::Envelope;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredMessage {
    pub message_id: Vec<u8>,
    pub sender: Vec<u8>,
    pub recipient: Vec<u8>,
    pub envelope: Envelope,
    pub timestamp: u64,
    pub expires_at: Option<u64>,
    pub read: bool,
    pub cid: Option<String>,
}

impl StoredMessage {
    pub fn new(sender: Vec<u8>, recipient: Vec<u8>, envelope: Envelope) -> Self {
        let message_id = Self::generate_message_id(&sender, &recipient, &envelope);
        
        Self {
            message_id,
            sender,
            recipient,
            envelope,
            timestamp: current_timestamp(),
            expires_at: None,
            read: false,
            cid: None,
        }
    }
    
    fn generate_message_id(sender: &[u8], recipient: &[u8], envelope: &Envelope) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(sender);
        hasher.update(recipient);
        hasher.update(&current_timestamp().to_le_bytes());
        hasher.update(&envelope.to_bytes().unwrap_or_default());
        
        hasher.finalize().to_vec()
    }
    
    pub fn mark_read(&mut self) {
        self.read = true;
    }
    
    pub fn set_expiry(&mut self, ttl_secs: u64) {
        self.expires_at = Some(current_timestamp() + ttl_secs);
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_timestamp() > expires_at
        } else {
            false
        }
    }
    
    pub fn to_bytes(&self) -> StorageResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
    
    pub fn from_bytes(data: &[u8]) -> StorageResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub message_id: Vec<u8>,
    pub sender: Vec<u8>,
    pub timestamp: u64,
    pub size: usize,
    pub cid: String,
}

pub struct MessageStore {
    ipfs: IpfsClient,
    local_cache: Arc<RwLock<HashMap<Vec<u8>, StoredMessage>>>,
    message_ttl: Option<u64>,
}

impl MessageStore {
    pub fn new(ipfs: IpfsClient, message_ttl: Option<u64>) -> Self {
        Self {
            ipfs,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            message_ttl,
        }
    }
    
    pub fn with_cache() -> Self {
        Self::new(IpfsClient::default(), Some(7 * 24 * 60 * 60))
    }
    
    pub async fn store_message(&self, message: StoredMessage) -> StorageResult<String> {
        let mut message = message;
        
        if let Some(ttl) = self.message_ttl {
            message.set_expiry(ttl);
        }
        
        let data = message.to_bytes()?;
        let cid = self.ipfs.add(&data).await?;
        
        message.cid = Some(cid.clone());
        
        let mut cache = self.local_cache.write().await;
        cache.insert(message.message_id.clone(), message);
        
        tracing::info!("Stored message with CID: {}", cid);
        
        Ok(cid)
    }
    
    pub async fn retrieve_message(&self, cid: &str) -> StorageResult<StoredMessage> {
        let data = self.ipfs.get(cid).await?;
        StoredMessage::from_bytes(&data)
    }
    
    pub async fn get_message_by_id(&self, message_id: &[u8]) -> StorageResult<Option<StoredMessage>> {
        let cache = self.local_cache.read().await;
        
        if let Some(message) = cache.get(message_id) {
            return Ok(Some(message.clone()));
        }
        
        Ok(None)
    }
    
    pub async fn get_messages_for_recipient(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let cache = self.local_cache.read().await;
        
        let messages: Vec<StoredMessage> = cache
            .values()
            .filter(|m| m.recipient == recipient && !m.is_expired())
            .cloned()
            .collect();
        
        Ok(messages)
    }
    
    pub async fn get_unread_messages(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let cache = self.local_cache.read().await;
        
        let messages: Vec<StoredMessage> = cache
            .values()
            .filter(|m| m.recipient == recipient && !m.read && !m.is_expired())
            .cloned()
            .collect();
        
        Ok(messages)
    }
    
    pub async fn mark_as_read(&self, message_id: &[u8]) -> StorageResult<()> {
        let mut cache = self.local_cache.write().await;
        
        if let Some(message) = cache.get_mut(message_id) {
            message.mark_read();
        }
        
        Ok(())
    }
    
    pub async fn delete_message(&self, message_id: &[u8]) -> StorageResult<()> {
        let mut cache = self.local_cache.write().await;
        
        if let Some(message) = cache.remove(message_id) {
            if let Some(cid) = message.cid {
                let _ = self.ipfs.unpin(&cid).await;
            }
        }
        
        Ok(())
    }
    
    pub async fn cleanup_expired(&self) -> StorageResult<usize> {
        let mut cache = self.local_cache.write().await;
        
        let expired: Vec<Vec<u8>> = cache
            .iter()
            .filter(|(_, m)| m.is_expired())
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = expired.len();
        
        for id in expired {
            if let Some(message) = cache.remove(&id) {
                if let Some(cid) = message.cid {
                    let _ = self.ipfs.unpin(&cid).await;
                }
            }
        }
        
        Ok(count)
    }
    
    pub async fn get_message_count(&self) -> usize {
        let cache = self.local_cache.read().await;
        cache.len()
    }
    
    pub async fn get_storage_stats(&self) -> StorageStats {
        let cache = self.local_cache.read().await;
        
        let total_messages = cache.len();
        let unread_count = cache.values().filter(|m| !m.read).count();
        let expired_count = cache.values().filter(|m| m.is_expired()).count();
        
        StorageStats {
            total_messages,
            unread_count,
            expired_count,
        }
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_messages: usize,
    pub unread_count: usize,
    pub expired_count: usize,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_protocol::keys::IdentityKey;
    use signal_protocol::message::MessageType;
    
    #[test]
    fn test_stored_message_creation() {
        let sender = vec![1u8; 32];
        let recipient = vec![2u8; 32];
        let envelope = Envelope {
            message_type: MessageType::Regular,
            sender_identity: IdentityKey::new(sender.clone()).unwrap(),
            receiver_identity: IdentityKey::new(recipient.clone()).unwrap(),
            ephemeral_public_key: vec![0u8; 32],
            used_prekey_id: 0,
            used_one_time_prekey_id: None,
            message: None,
        };
        
        let stored = StoredMessage::new(sender, recipient, envelope);
        assert!(!stored.message_id.is_empty());
        assert!(!stored.read);
        assert!(!stored.is_expired());
    }
    
    #[test]
    fn test_message_expiry() {
        let sender = vec![1u8; 32];
        let recipient = vec![2u8; 32];
        let envelope = Envelope {
            message_type: MessageType::Regular,
            sender_identity: IdentityKey::new(sender.clone()).unwrap(),
            receiver_identity: IdentityKey::new(recipient.clone()).unwrap(),
            ephemeral_public_key: vec![0u8; 32],
            used_prekey_id: 0,
            used_one_time_prekey_id: None,
            message: None,
        };
        
        let mut stored = StoredMessage::new(sender, recipient, envelope);
        stored.set_expiry(1);
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        assert!(stored.is_expired());
    }
}
