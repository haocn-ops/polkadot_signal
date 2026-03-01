use crate::double_ratchet::{DoubleRatchet, EncryptedMessage};
use crate::error::{SignalError, SignalResult};
use crate::keys::{IdentityKey, IdentityKeyPair, KeyBundle, X25519StaticSecret};
use crate::message::{Envelope, Message, MessageType};
use crate::x3dh::{X3DH, X3DHResult};

#[derive(Debug)]
pub struct SessionState {
    pub session_id: Vec<u8>,
    pub their_identity_key: IdentityKey,
    pub double_ratchet: DoubleRatchet,
    pub created_at: u64,
    pub last_received: u64,
    pub last_sent: u64,
}

pub struct Session {
    state: SessionState,
}

impl Session {
    pub fn initiate(
        identity: &IdentityKeyPair,
        recipient_bundle: &KeyBundle,
    ) -> SignalResult<(Self, Envelope)> {
        let x3dh_result = X3DH::initiate(identity, recipient_bundle)?;
        
        let double_ratchet = DoubleRatchet::new_initiator(
            x3dh_result.root_key.clone(),
            recipient_bundle.signed_pre_key.public_key.clone(),
        )?;
        
        let session_id = Self::generate_session_id(
            &identity.public_key,
            &recipient_bundle.identity_key,
        );
        
        let state = SessionState {
            session_id,
            their_identity_key: recipient_bundle.identity_key.clone(),
            double_ratchet,
            created_at: current_timestamp(),
            last_received: current_timestamp(),
            last_sent: current_timestamp(),
        };
        
        let session = Self { state };
        
        let envelope = Envelope {
            message_type: MessageType::PreKey,
            sender_identity: identity.public_key.clone(),
            receiver_identity: recipient_bundle.identity_key.clone(),
            ephemeral_public_key: x3dh_result.ephemeral_public_key,
            used_prekey_id: recipient_bundle.signed_pre_key.key_id,
            used_one_time_prekey_id: x3dh_result.used_one_time_prekey_id,
            message: None,
        };
        
        Ok((session, envelope))
    }
    
    pub fn respond(
        identity: &IdentityKeyPair,
        signed_pre_key: &X25519StaticSecret,
        one_time_pre_key: Option<&X25519StaticSecret>,
        envelope: &Envelope,
    ) -> SignalResult<Self> {
        let x3dh_result = X3DH::respond(
            identity,
            signed_pre_key,
            one_time_pre_key,
            &envelope.sender_identity,
            &envelope.ephemeral_public_key,
        )?;
        
        let double_ratchet = DoubleRatchet::new_responder(x3dh_result.root_key)?;
        
        let session_id = Self::generate_session_id(
            &envelope.sender_identity,
            &identity.public_key,
        );
        
        let state = SessionState {
            session_id,
            their_identity_key: envelope.sender_identity.clone(),
            double_ratchet,
            created_at: current_timestamp(),
            last_received: current_timestamp(),
            last_sent: current_timestamp(),
        };
        
        Ok(Self { state })
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8]) -> SignalResult<Envelope> {
        let encrypted = self.state.double_ratchet.encrypt(plaintext)?;
        
        let message = Message {
            ciphertext: encrypted.ciphertext,
            nonce: encrypted.nonce,
            message_number: encrypted.message_number,
            previous_chain_length: encrypted.previous_chain_length,
            ephemeral_public_key: encrypted.ephemeral_public_key,
        };
        
        self.state.last_sent = current_timestamp();
        
        Ok(Envelope {
            message_type: MessageType::Regular,
            sender_identity: IdentityKey::new(vec![0u8; 32])?,
            receiver_identity: self.state.their_identity_key.clone(),
            ephemeral_public_key: vec![],
            used_prekey_id: 0,
            used_one_time_prekey_id: None,
            message: Some(message),
        })
    }
    
    pub fn decrypt(&mut self, envelope: &Envelope) -> SignalResult<Vec<u8>> {
        let message = envelope.message.as_ref()
            .ok_or_else(|| SignalError::InvalidMessage("No message in envelope".into()))?;
        
        let encrypted = EncryptedMessage {
            ephemeral_public_key: message.ephemeral_public_key.clone(),
            message_number: message.message_number,
            previous_chain_length: message.previous_chain_length,
            ciphertext: message.ciphertext.clone(),
            nonce: message.nonce.clone(),
        };
        
        let plaintext = self.state.double_ratchet.decrypt(&encrypted)?;
        
        self.state.last_received = current_timestamp();
        
        Ok(plaintext)
    }
    
    fn generate_session_id(identity_a: &IdentityKey, identity_b: &IdentityKey) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(&identity_a.public_key);
        hasher.update(&identity_b.public_key);
        
        hasher.finalize().to_vec()
    }
    
    pub fn get_state(&self) -> &SessionState {
        &self.state
    }
    
    pub fn get_session_id(&self) -> &[u8] {
        &self.state.session_id
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct SessionManager {
    sessions: Vec<(Vec<u8>, Session)>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }
    
    pub fn create_session(&mut self, session: Session) {
        let session_id = session.get_session_id().to_vec();
        self.sessions.push((session_id, session));
    }
    
    pub fn get_session(&self, session_id: &[u8]) -> Option<&Session> {
        self.sessions.iter()
            .find(|(id, _)| id == session_id)
            .map(|(_, session)| session)
    }
    
    pub fn get_session_mut(&mut self, session_id: &[u8]) -> Option<&mut Session> {
        self.sessions.iter_mut()
            .find(|(id, _)| id == session_id)
            .map(|(_, session)| session)
    }
    
    pub fn remove_session(&mut self, session_id: &[u8]) -> Option<Session> {
        if let Some(pos) = self.sessions.iter().position(|(id, _)| id == session_id) {
            Some(self.sessions.remove(pos).1)
        } else {
            None
        }
    }
    
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{OneTimePreKey, SignedPreKey};
    
    #[test]
    fn test_session_creation() {
        let alice = IdentityKeyPair::generate().unwrap();
        let bob = IdentityKeyPair::generate().unwrap();
        
        let bob_signed_pre_key = SignedPreKey::generate(1, &bob).unwrap();
        let bob_one_time_pre_key = OneTimePreKey::generate(1).unwrap();
        
        let bob_bundle = KeyBundle {
            identity_key: bob.public_key.clone(),
            signed_pre_key: bob_signed_pre_key,
            one_time_pre_key: Some(bob_one_time_pre_key),
        };
        
        let (alice_session, _envelope) = Session::initiate(&alice, &bob_bundle).unwrap();
        
        assert!(!alice_session.get_session_id().is_empty());
    }
}
