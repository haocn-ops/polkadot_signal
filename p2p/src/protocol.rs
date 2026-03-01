use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use signal_protocol::message::Envelope;

pub const PROTOCOL_NAME: &str = "/polkadot-signal/1.0.0";
pub const GOSSIP_TOPIC: &str = "polkadot-signal-messages";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalMessage {
    pub from: PeerId,
    pub to: Option<PeerId>,
    pub payload: MessagePayload,
    pub timestamp: u64,
    pub message_id: Vec<u8>,
}

impl SignalMessage {
    pub fn new(from: PeerId, to: Option<PeerId>, payload: MessagePayload) -> Self {
        use rand::Rng;
        
        Self {
            from,
            to,
            payload,
            timestamp: current_timestamp(),
            message_id: rand::thread_rng().gen::<[u8; 16]>().to_vec(),
        }
    }
    
    pub fn direct(from: PeerId, to: PeerId, envelope: Envelope) -> Self {
        Self::new(
            from,
            Some(to),
            MessagePayload::DirectMessage { envelope },
        )
    }
    
    pub fn broadcast(from: PeerId, envelope: Envelope) -> Self {
        Self::new(
            from,
            None,
            MessagePayload::Broadcast { envelope },
        )
    }
    
    pub fn key_announcement(from: PeerId, identity_key: Vec<u8>, signed_prekey: Vec<u8>) -> Self {
        Self::new(
            from,
            None,
            MessagePayload::KeyAnnouncement {
                identity_key,
                signed_prekey,
            },
        )
    }
    
    pub fn to_bytes(&self) -> crate::error::P2PResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::P2PError::Serialization(e.to_string()))
    }
    
    pub fn from_bytes(data: &[u8]) -> crate::error::P2PResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| crate::error::P2PError::Serialization(e.to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessagePayload {
    DirectMessage {
        envelope: Envelope,
    },
    Broadcast {
        envelope: Envelope,
    },
    KeyAnnouncement {
        identity_key: Vec<u8>,
        signed_prekey: Vec<u8>,
    },
    KeyRequest {
        peer_id: PeerId,
    },
    KeyResponse {
        identity_key: Vec<u8>,
        signed_prekey: Vec<u8>,
        one_time_prekeys: Vec<Vec<u8>>,
    },
    DeliveryReceipt {
        message_id: Vec<u8>,
        status: DeliveryStatus,
    },
    TypingIndicator {
        is_typing: bool,
    },
    ReadReceipt {
        message_ids: Vec<Vec<u8>>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryStatus {
    Delivered,
    Read,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub identity_key: Option<Vec<u8>>,
    pub signed_prekey: Option<Vec<u8>>,
    pub last_seen: u64,
    pub addresses: Vec<String>,
}

impl PeerInfo {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            identity_key: None,
            signed_prekey: None,
            last_seen: current_timestamp(),
            addresses: Vec::new(),
        }
    }
    
    pub fn update_keys(&mut self, identity_key: Vec<u8>, signed_prekey: Vec<u8>) {
        self.identity_key = Some(identity_key);
        self.signed_prekey = Some(signed_prekey);
        self.last_seen = current_timestamp();
    }
    
    pub fn add_address(&mut self, address: String) {
        if !self.addresses.contains(&address) {
            self.addresses.push(address);
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
