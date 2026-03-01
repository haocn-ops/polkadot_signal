use crate::error::{SignalError, SignalResult};
use crate::keys::derive_key;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

const MAX_SKIP: usize = 2000;
const ROOT_KEY_INFO: &[u8] = b"Signal Root Key";
const CHAIN_KEY_INFO: &[u8] = b"Signal Chain Key";
const MESSAGE_KEY_INFO: &[u8] = b"Signal Message Key";

#[derive(Debug)]
pub struct DoubleRatchet {
    root_key: Vec<u8>,
    sending_chain_key: Option<Vec<u8>>,
    receiving_chain_key: Option<Vec<u8>>,
    sending_ratchet_public: Option<Vec<u8>>,
    receiving_ratchet_public: Option<Vec<u8>>,
    send_message_number: u32,
    receive_message_number: u32,
    previous_sending_chain_length: u32,
    skipped_message_keys: Vec<(Vec<u8>, u32, Vec<u8>)>,
}

impl DoubleRatchet {
    pub fn new_initiator(root_key: Vec<u8>, recipient_ratchet_public: Vec<u8>) -> SignalResult<Self> {
        let mut csprng = OsRng;
        let sending_ratchet = StaticSecret::random_from_rng(&mut csprng);
        let sending_ratchet_public_key = PublicKey::from(&sending_ratchet);
        
        let recipient_public = PublicKey::from(
            <[u8; 32]>::try_from(recipient_ratchet_public.as_slice())
                .map_err(|_| SignalError::InvalidKey("Invalid recipient public key".into()))?
        );
        
        let (new_root_key, sending_chain_key) = Self::ratchet_step(
            &root_key,
            sending_ratchet.as_bytes(),
            &recipient_public,
        )?;
        
        Ok(Self {
            root_key: new_root_key,
            sending_chain_key: Some(sending_chain_key),
            receiving_chain_key: None,
            sending_ratchet_public: Some(sending_ratchet_public_key.as_bytes().to_vec()),
            receiving_ratchet_public: Some(recipient_ratchet_public),
            send_message_number: 0,
            receive_message_number: 0,
            previous_sending_chain_length: 0,
            skipped_message_keys: Vec::new(),
        })
    }
    
    pub fn new_responder(root_key: Vec<u8>) -> SignalResult<Self> {
        Ok(Self {
            root_key,
            sending_chain_key: None,
            receiving_chain_key: None,
            sending_ratchet_public: None,
            receiving_ratchet_public: None,
            send_message_number: 0,
            receive_message_number: 1,
            previous_sending_chain_length: 0,
            skipped_message_keys: Vec::new(),
        })
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8]) -> SignalResult<EncryptedMessage> {
        let chain_key = self.sending_chain_key.as_mut()
            .ok_or_else(|| SignalError::InvalidState("No sending chain key".into()))?;
        
        let message_key = Self::derive_message_key(chain_key)?;
        *chain_key = Self::advance_chain_key(chain_key)?;
        
        let encrypted = Self::encrypt_with_key(&message_key, plaintext)?;
        
        let ephemeral_public = self.sending_ratchet_public.clone()
            .ok_or_else(|| SignalError::InvalidState("No ratchet key".into()))?;
        
        let message = EncryptedMessage {
            ephemeral_public_key: ephemeral_public,
            message_number: self.send_message_number,
            previous_chain_length: self.previous_sending_chain_length,
            ciphertext: encrypted.ciphertext,
            nonce: encrypted.nonce,
        };
        
        self.send_message_number += 1;
        
        Ok(message)
    }
    
    pub fn decrypt(&mut self, message: &EncryptedMessage) -> SignalResult<Vec<u8>> {
        let message_key = self.try_get_message_key(
            &message.ephemeral_public_key,
            message.message_number,
        )?;
        
        Self::decrypt_with_key(&message_key, &message.ciphertext, &message.nonce)
    }
    
    fn try_get_message_key(&mut self, ephemeral_public: &[u8], message_number: u32) -> SignalResult<Vec<u8>> {
        let public_key = PublicKey::from(
            <[u8; 32]>::try_from(ephemeral_public)
                .map_err(|_| SignalError::InvalidKey("Invalid ephemeral key".into()))?
        );
        
        if let Some(ref mut chain_key) = self.receiving_chain_key {
            let current_n = self.receive_message_number;
            
            if message_number < current_n {
                for (i, (pk, n, mk)) in self.skipped_message_keys.iter().enumerate() {
                    if pk == ephemeral_public && n == &message_number {
                        let mk = mk.clone();
                        self.skipped_message_keys.remove(i);
                        return Ok(mk);
                    }
                }
                return Err(SignalError::InvalidMessage("Message number too old".into()));
            }
            
            let skip = message_number - current_n;
            if skip > MAX_SKIP as u32 {
                return Err(SignalError::InvalidMessage("Skipped too many messages".into()));
            }
            
            for _ in 0..skip {
                let mk = Self::derive_message_key(chain_key)?;
                *chain_key = Self::advance_chain_key(chain_key)?;
                self.skipped_message_keys.push((
                    ephemeral_public.to_vec(),
                    self.receive_message_number,
                    mk,
                ));
                self.receive_message_number += 1;
            }
            
            let mk = Self::derive_message_key(chain_key)?;
            *chain_key = Self::advance_chain_key(chain_key)?;
            self.receive_message_number += 1;
            
            return Ok(mk);
        }
        
        self.ratchet_receive(&public_key)?;
        self.try_get_message_key(ephemeral_public, message_number)
    }
    
    fn ratchet_receive(&mut self, their_public: &PublicKey) -> SignalResult<()> {
        let sending_ratchet_secret = StaticSecret::random_from_rng(&mut OsRng);
        let sending_ratchet_public = PublicKey::from(&sending_ratchet_secret);
        
        let (new_root_key, receiving_chain_key) = Self::ratchet_step(
            &self.root_key,
            sending_ratchet_secret.as_bytes(),
            their_public,
        )?;
        
        let (final_root_key, sending_chain_key) = Self::ratchet_step(
            &new_root_key,
            sending_ratchet_secret.as_bytes(),
            their_public,
        )?;
        
        self.root_key = final_root_key;
        self.receiving_chain_key = Some(receiving_chain_key);
        self.sending_chain_key = Some(sending_chain_key);
        self.sending_ratchet_public = Some(sending_ratchet_public.as_bytes().to_vec());
        self.receiving_ratchet_public = Some(their_public.as_bytes().to_vec());
        self.previous_sending_chain_length = self.send_message_number;
        self.send_message_number = 1;
        self.receive_message_number = 1;
        
        Ok(())
    }
    
    fn ratchet_step(root_key: &[u8], our_secret: &[u8], their_public: &PublicKey) -> SignalResult<(Vec<u8>, Vec<u8>)> {
        let our_secret = StaticSecret::from(<[u8; 32]>::try_from(our_secret)
            .map_err(|_| SignalError::InvalidKey("Invalid secret".into()))?);
        
        let dh_output = our_secret.diffie_hellman(their_public);
        
        let derived = derive_key(dh_output.as_bytes(), root_key, 64)?;
        
        let new_root_key = derived[..32].to_vec();
        let chain_key = derived[32..].to_vec();
        
        Ok((new_root_key, chain_key))
    }
    
    fn derive_message_key(chain_key: &[u8]) -> SignalResult<Vec<u8>> {
        derive_key(chain_key, MESSAGE_KEY_INFO, 32)
    }
    
    fn advance_chain_key(chain_key: &[u8]) -> SignalResult<Vec<u8>> {
        derive_key(chain_key, CHAIN_KEY_INFO, 32)
    }
    
    fn encrypt_with_key(key: &[u8], plaintext: &[u8]) -> SignalResult<EncryptedData> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| SignalError::EncryptionFailed)?;
        
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|_| SignalError::EncryptionFailed)?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }
    
    fn decrypt_with_key(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> SignalResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| SignalError::DecryptionFailed)?;
        
        let nonce_array: [u8; 12] = nonce.try_into()
            .map_err(|_| SignalError::InvalidMessage("Invalid nonce".into()))?;
        let nonce = Nonce::from_slice(&nonce_array);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| SignalError::DecryptionFailed)
    }
}

impl Drop for DoubleRatchet {
    fn drop(&mut self) {
        self.root_key.zeroize();
        if let Some(ref mut key) = self.sending_chain_key {
            key.zeroize();
        }
        if let Some(ref mut key) = self.receiving_chain_key {
            key.zeroize();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub ephemeral_public_key: Vec<u8>,
    pub message_number: u32,
    pub previous_chain_length: u32,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

struct EncryptedData {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::IdentityKeyPair;
    
    #[test]
    fn test_double_ratchet_init() {
        let alice_identity = IdentityKeyPair::generate().unwrap();
        let bob_identity = IdentityKeyPair::generate().unwrap();
        
        let bob_secret = StaticSecret::random_from_rng(&mut OsRng);
        let bob_public = PublicKey::from(&bob_secret);
        
        let alice_ratchet = DoubleRatchet::new_initiator(vec![0u8; 32], bob_public.as_bytes().to_vec()).unwrap();
        let _bob_ratchet = DoubleRatchet::new_responder(vec![0u8; 32]).unwrap();
        
        assert!(alice_ratchet.sending_chain_key.is_some());
    }
}
