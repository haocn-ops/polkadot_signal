use crate::error::{P2PError, P2PResult};
use crate::protocol::{DeliveryStatus, MessagePayload, PeerInfo, SignalMessage};
use libp2p::PeerId;
use signal_protocol::message::Envelope;
use signal_protocol::session::{Session, SessionManager};
use signal_protocol::keys::IdentityKeyPair;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub type MessageCallback = Box<dyn Fn(SignalMessage) + Send + Sync>;
pub type PeerCallback = Box<dyn Fn(PeerId, PeerInfo) + Send + Sync>;

pub struct MessageHandler {
    sessions: Arc<RwLock<SessionManager>>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    identity: IdentityKeyPair,
    pending_messages: Arc<RwLock<HashMap<Vec<u8>, SignalMessage>>>,
    message_callback: Option<MessageCallback>,
    peer_callback: Option<PeerCallback>,
}

impl MessageHandler {
    pub fn new(identity: IdentityKeyPair) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(SessionManager::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            identity,
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            message_callback: None,
            peer_callback: None,
        }
    }
    
    pub fn on_message(&mut self, callback: MessageCallback) {
        self.message_callback = Some(callback);
    }
    
    pub fn on_peer_update(&mut self, callback: PeerCallback) {
        self.peer_callback = Some(callback);
    }
    
    pub async fn handle_message(&self, message: SignalMessage) -> P2PResult<()> {
        match &message.payload {
            MessagePayload::DirectMessage { envelope } => {
                self.handle_direct_message(&message.from, envelope).await?;
            }
            MessagePayload::Broadcast { envelope } => {
                self.handle_broadcast(&message.from, envelope).await?;
            }
            MessagePayload::KeyAnnouncement { identity_key, signed_prekey } => {
                self.handle_key_announcement(&message.from, identity_key.clone(), signed_prekey.clone()).await?;
            }
            MessagePayload::KeyRequest { peer_id } => {
                self.handle_key_request(*peer_id).await?;
            }
            MessagePayload::KeyResponse { identity_key, signed_prekey, one_time_prekeys } => {
                self.handle_key_response(&message.from, identity_key.clone(), signed_prekey.clone(), one_time_prekeys.clone()).await?;
            }
            MessagePayload::DeliveryReceipt { message_id, status } => {
                self.handle_delivery_receipt(message_id, status).await?;
            }
            MessagePayload::TypingIndicator { is_typing } => {
                self.handle_typing_indicator(&message.from, *is_typing).await?;
            }
            MessagePayload::ReadReceipt { message_ids } => {
                self.handle_read_receipt(&message.from, message_ids).await?;
            }
        }
        
        if let Some(ref callback) = self.message_callback {
            callback(message);
        }
        
        Ok(())
    }
    
    async fn handle_direct_message(&self, from: &PeerId, envelope: &Envelope) -> P2PResult<()> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get_session(&from.to_bytes()) {
            let mut session = session.clone();
            let plaintext = session.decrypt(envelope)?;
            
            tracing::info!(
                "Received direct message from {}: {} bytes",
                from,
                plaintext.len()
            );
        } else {
            tracing::warn!("Received message from unknown peer: {}", from);
            
            let mut pending = self.pending_messages.write().await;
            pending.insert(from.to_bytes(), SignalMessage::new(*from, None, MessagePayload::DirectMessage {
                envelope: envelope.clone(),
            }));
        }
        
        Ok(())
    }
    
    async fn handle_broadcast(&self, from: &PeerId, envelope: &Envelope) -> P2PResult<()> {
        tracing::info!("Received broadcast from {}", from);
        
        Ok(())
    }
    
    async fn handle_key_announcement(
        &self,
        from: &PeerId,
        identity_key: Vec<u8>,
        signed_prekey: Vec<u8>,
    ) -> P2PResult<()> {
        let mut peers = self.peers.write().await;
        
        let peer_info = peers.entry(*from).or_insert_with(|| PeerInfo::new(*from));
        peer_info.update_keys(identity_key, signed_prekey);
        
        tracing::info!("Updated keys for peer {}", from);
        
        if let Some(ref callback) = self.peer_callback {
            callback(*from, peer_info.clone());
        }
        
        Ok(())
    }
    
    async fn handle_key_request(&self, peer_id: PeerId) -> P2PResult<()> {
        tracing::info!("Key request from {}", peer_id);
        Ok(())
    }
    
    async fn handle_key_response(
        &self,
        from: &PeerId,
        identity_key: Vec<u8>,
        signed_prekey: Vec<u8>,
        one_time_prekeys: Vec<Vec<u8>>,
    ) -> P2PResult<()> {
        let mut peers = self.peers.write().await;
        
        let peer_info = peers.entry(*from).or_insert_with(|| PeerInfo::new(*from));
        peer_info.update_keys(identity_key, signed_prekey);
        
        tracing::info!("Received key response from {} with {} one-time prekeys", from, one_time_prekeys.len());
        
        Ok(())
    }
    
    async fn handle_delivery_receipt(&self, message_id: &[u8], status: &DeliveryStatus) -> P2PResult<()> {
        tracing::info!("Delivery receipt for message {:?}: {:?}", message_id, status);
        Ok(())
    }
    
    async fn handle_typing_indicator(&self, from: &PeerId, is_typing: bool) -> P2PResult<()> {
        tracing::info!("Peer {} is {}", from, if is_typing { "typing" } else { "not typing" });
        Ok(())
    }
    
    async fn handle_read_receipt(&self, from: &PeerId, message_ids: &[Vec<u8>]) -> P2PResult<()> {
        tracing::info!("Peer {} read {} messages", from, message_ids.len());
        Ok(())
    }
    
    pub async fn get_peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }
    
    pub async fn get_all_peers(&self) -> Vec<(PeerId, PeerInfo)> {
        let peers = self.peers.read().await;
        peers.iter().map(|(k, v)| (*k, v.clone())).collect()
    }
    
    pub async fn create_direct_message(&self, to: &PeerId, plaintext: &[u8]) -> P2PResult<SignalMessage> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get_session(to.to_bytes()) {
            let mut session = session.clone();
            let envelope = session.encrypt(plaintext)?;
            
            Ok(SignalMessage::direct(self.identity.public_key.clone().public_key.into(), *to, envelope))
        } else {
            Err(P2PError::PeerNotFound(to.to_string()))
        }
    }
    
    pub async fn register_session(&self, peer_id: PeerId, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.create_session(session);
    }
}
