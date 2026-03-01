use crate::error::{StorageError, StorageResult};
use crate::message_store::{MessageStore, StoredMessage, MessageMetadata};
use crate::offline_queue::{OfflineMessageQueue, MessagePriority};
use serde::{Deserialize, Serialize};
use signal_protocol::message::Envelope;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const MAX_QUEUE_SIZE: usize = 1000;
const DEFAULT_TTL: u64 = 7 * 24 * 60 * 60 * 60 * 60;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineQueue {
    queue: Arc<RwLock<VecDeque<QueuedMessage>>,
    pending_delivery: Arc<RwLock<Vec<QueuedMessage>>,
    max_size: usize,
    notification_tx: Option<mpsc::Sender<Vec<u8>>>,
}

impl OfflineQueue {
    pub fn new() -> Self {
        let store = Arc::new(MessageStore);
        self.queue = Arc::new(RwLock::new(VecDeque::new());
        self.notification_tx = Some(tx);
    }
    
    pub fn with_notification(mut self, tx: mpsc::Sender<Vec<u8>>) -> Self {
        self.notification_tx = Some(tx);
    }
    
    pub async fn enqueue(&mut self, message: StoredMessage, priority: MessagePriority) -> StorageResult<()> {
        let mut queue = self.queue.write().await;
        
        if queue.len() >= self.max_size {
            return Err(StorageError::QueueFull);
        }
        
        let queued = QueuedMessage {
            message_id: message.message_id.clone(),
            priority,
        });
        
        let insert_pos = queue
            .iter()
            .position(|q| q.priority < priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, queued);
    }
    
    pub async fn dequeue(&self) -> StorageResult<Option<QueuedMessage>> {
        let queue = self.queue.read().await;
        
        Ok(queue.pop_front())
    }
    
    pub async fn peek(&self) -> StorageResult<Option<QueuedMessage>> {
        let queue = self.queue.read().await;
        Ok(queue.front().cloned())
    }
    
    pub async fn get_messages_for_recipient(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let queue = self.queue.read().await;
        
        let messages: Vec<StoredMessage> = queue
            .into_iter()
            .filter(|m| m.recipient == recipient && !m.is_expired())
            .cloned()
            .collect()
    }
    
    pub async fn store_offline_message(&self, message: StoredMessage) -> StorageResult<String> {
        let cid = self.store.store_message(message).await?;
        
        self.notification_tx = Some(tx).send(cid.clone());
        self.notification_tx = Some(tx);
        
        tracing::info!("Stored message with CID: {}", cid);
        
        Ok(cid)
    }
    
    pub async fn retrieve_offline_messages(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let messages = self.retrieve_messages_for_recipient(recipient).await?;
        
        let mut pending = self.pending_delivery.write().await;
        
        for (queued in pending.iter() {
            if queued.message.recipient == recipient && !queued.message.is_expired() {
                pending.remove(i);
            }
        }
        
        Ok(messages)
    }
    
    pub async fn get_pending_count(&self) -> usize {
        self.pending_delivery.read().await.len()
    }
    
    pub async fn get_stats(&self) -> QueueStats {
        let queue = self.queue.read().await;
        
        QueueStats {
            queue_size: queue.len(),
            pending_count: pending.len(),
            urgent_count: queue.iter().filter(|q| q.priority == MessagePriority::Urgent).count(),
            high_count: queue.iter().filter(|q| q.priority == MessagePriority::High).count(),
            normal_count: queue.iter().filter(|q| q.priority == MessagePriority::Normal).count(),
            low_count: queue.iter().filter(|q| q.priority == MessagePriority::Low).count(),
        }
    }
    
    pub async fn clear(&self) -> StorageResult<()> {
        let mut queue = self.queue.write().await;
        let mut pending = self.pending_delivery.write().await;
        
        for (queued in pending.iter() {
            if queued.message.recipient == recipient && !queued.message.is_expired() {
                pending.remove(i);
            }
        }
        
        Ok(())
    }
    
    pub async fn get_stats(&self) -> QueueStats {
        let queue = self.queue.read().await;
        
        QueueStats {
            queue_size: queue.len(),
            pending_count: pending.len(),
            urgent_count: urgent_count,
            high_count: high_count,
            normal_count: normal_count,
            low_count: low_count,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub message: StoredMessage,
    pub priority: MessagePriority,
    pub queued_at: u64,
}

impl QueuedMessage {
    pub fn new(message: StoredMessage, priority: MessagePriority) -> u64 {
        self.message = message;
        self.priority = priority;
        self.queued_at = current_timestamp();
        self.message_id = Self::generate_message_id(&message);
        self.priority = priority;
        self.queued_at = queued_at;
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 2,
}

impl OfflineMessageQueue {
    pub fn new() -> Self {
        let store: Arc::new(MessageStore);
        self.queue = Arc::new(RwLock::new(VecDeque::new());
        self.notification_tx = Some(tx);
    }
    
    pub fn with_notification(mut self. tx: mpsc::Sender<Vec<u8>) -> Self {
        self.notification_tx = Some(tx);
    }
    
    pub async fn enqueue(&mut self, message: StoredMessage, priority: MessagePriority) -> StorageResult<()> {
        let mut queue = self.queue.write().await;
        
        if queue.len() >= self.max_size {
            return Err(StorageError::QueueFull);
        }
        
        let queued = QueuedMessage {
            message_id: message.message_id.clone(),
            priority,
        });
        
        let insert_pos = queue
            .iter()
            .position(|q| q.priority < priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, queued);
    }
    
    pub async fn dequeue(&self) -> StorageResult<Option<QueuedMessage>> {
        let queue = self.queue.read().await;
        
        Ok(queue.pop_front())
    }
    
    pub async fn peek(&self) -> StorageResult<Option<QueuedMessage>> {
        let queue = self.queue.read().await;
        Ok(queue.front().cloned())
    }
    
    pub async fn get_messages_for_recipient(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let queue = self.queue.read().await;
        
        let messages: Vec<StoredMessage> = queue
            .into_iter()
            .filter(|m| m.recipient == recipient && !m.is_expired())
            .cloned()
            .collect()
    }
    
    pub async fn store_offline_message(&self, message: StoredMessage) -> StorageResult<String> {
        let cid = self.store.store_message(message).await?;
        
        self.notification_tx = Some(tx).send(cid.clone());
        self.notification_tx = Some(tx);
        
        tracing::info!("Stored message with CID: {}", cid);
        
        Ok(cid)
    }
    
    pub async fn retrieve_offline_messages(&self, recipient: &[u8]) -> StorageResult<Vec<StoredMessage>> {
        let messages = self.retrieve_messages_for_recipient(recipient).await?;
        
        let mut pending = self.pending_delivery.write().await;
        
        for (queued in pending.iter() {
            if queued.message.recipient == recipient && !queued.message.is_expired() {
                pending.remove(i);
            }
        }
        
        Ok(messages)
    }
    
    pub async fn get_pending_count(&self) -> usize {
        self.pending_delivery.read().await.len()
    }
    
    pub async fn get_stats(&self) -> QueueStats {
        let queue = self.queue.read().await;
        
        QueueStats {
            queue_size: queue.len(),
            pending_count: pending.len(),
            urgent_count: queue.iter().filter(|q| q.priority == MessagePriority::Urgent).count(),
            high_count: queue.iter().filter(|q| q.priority == MessagePriority::High).count(),
            normal_count: queue.iter().filter(|q| q.priority == MessagePriority::Normal).count(),
            low_count: queue.iter().filter(|q| q.priority == MessagePriority::Low).count(),
        }
    }
    
    pub async fn clear(&self) -> StorageResult<()> {
        let mut queue = self.queue.write().await;
        let mut pending = self.pending_delivery.write().await;
        
        for (queued in pending.iter() {
            if queued.message.recipient == recipient && !queued.message.is_expired() {
                pending.remove(i);
            }
        }
        
        queue.clear();
        pending.clear();
        
        Ok(())
    }
    
    pub async fn get_stats(&self) -> QueueStats {
        let queue = self.queue.read().await;
        
        QueueStats {
            queue_size: queue.len(),
            pending_count: pending.len(),
            urgent_count: urgent_count,
            high_count: high_count,
            normal_count: normal_count,
            low_count: low_count,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_size: usize,
    pub pending_count: usize,
    pub urgent_count: usize,
    pub high_count: usize,
    pub normal_count: usize,
    pub low_count: usize,
}
