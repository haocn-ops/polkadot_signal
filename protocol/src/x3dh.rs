use crate::error::{SignalError, SignalResult};
use crate::keys::{derive_key, IdentityKey, IdentityKeyPair, KeyBundle};
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

const KDF_INFO: &[u8] = b"Signal X3DH";

pub struct X3DH;

#[derive(Debug)]
pub struct X3DHResult {
    pub root_key: Vec<u8>,
    pub ephemeral_public_key: Vec<u8>,
    pub used_one_time_prekey_id: Option<u32>,
}

impl X3DH {
    pub fn initiate(
        identity_key: &IdentityKeyPair,
        recipient_bundle: &KeyBundle,
    ) -> SignalResult<X3DHResult> {
        let mut csprng = rand::rngs::OsRng;
        
        let ik_a = identity_key.get_x25519_private_key()?;
        
        let ek_a = EphemeralSecret::random_from_rng(&mut csprng);
        let ek_a_public = PublicKey::from(&ek_a);
        
        let ik_b = PublicKey::from(
            <[u8; 32]>::try_from(&recipient_bundle.identity_key.public_key[..])
                .map_err(|_| SignalError::InvalidKey("Invalid recipient identity key".into()))?
        );
        
        let spk_b = PublicKey::from(
            <[u8; 32]>::try_from(&recipient_bundle.signed_pre_key.public_key[..])
                .map_err(|_| SignalError::InvalidKey("Invalid recipient signed prekey".into()))?
        );
        
        let dh1 = ik_a.diffie_hellman(&spk_b);
        
        let master_secret = if let Some(opk) = &recipient_bundle.one_time_pre_key {
            let opk_b = PublicKey::from(
                <[u8; 32]>::try_from(&opk.public_key[..])
                    .map_err(|_| SignalError::InvalidKey("Invalid recipient one-time prekey".into()))?
            );
            
            let ek_a2 = EphemeralSecret::random_from_rng(&mut csprng);
            let ek_a3 = EphemeralSecret::random_from_rng(&mut csprng);
            let ek_a4 = EphemeralSecret::random_from_rng(&mut csprng);
            
            let dh2 = ek_a2.diffie_hellman(&ik_b);
            let dh3 = ek_a3.diffie_hellman(&spk_b);
            let dh4 = ek_a4.diffie_hellman(&opk_b);
            
            let mut secret = Vec::new();
            secret.extend_from_slice(dh1.as_bytes());
            secret.extend_from_slice(dh2.as_bytes());
            secret.extend_from_slice(dh3.as_bytes());
            secret.extend_from_slice(dh4.as_bytes());
            
            let root_key = derive_key(&secret, KDF_INFO, 32)?;
            
            X3DHResult {
                root_key,
                ephemeral_public_key: ek_a_public.as_bytes().to_vec(),
                used_one_time_prekey_id: recipient_bundle.one_time_pre_key.as_ref().map(|k| k.key_id),
            }
        } else {
            let ek_a2 = EphemeralSecret::random_from_rng(&mut csprng);
            let ek_a3 = EphemeralSecret::random_from_rng(&mut csprng);
            
            let dh2 = ek_a2.diffie_hellman(&ik_b);
            let dh3 = ek_a3.diffie_hellman(&spk_b);
            
            let mut secret = Vec::new();
            secret.extend_from_slice(dh1.as_bytes());
            secret.extend_from_slice(dh2.as_bytes());
            secret.extend_from_slice(dh3.as_bytes());
            
            let root_key = derive_key(&secret, KDF_INFO, 32)?;
            
            X3DHResult {
                root_key,
                ephemeral_public_key: ek_a_public.as_bytes().to_vec(),
                used_one_time_prekey_id: None,
            }
        };
        
        Ok(master_secret)
    }
    
    pub fn respond(
        identity_key: &IdentityKeyPair,
        signed_pre_key: &StaticSecret,
        one_time_pre_key: Option<&StaticSecret>,
        initiator_identity_public: &IdentityKey,
        initiator_ephemeral_public: &[u8],
    ) -> SignalResult<X3DHResult> {
        let ik_b = identity_key.get_x25519_private_key()?;
        let spk_b = signed_pre_key;
        
        let ik_a = PublicKey::from(
            <[u8; 32]>::try_from(&initiator_identity_public.public_key[..])
                .map_err(|_| SignalError::InvalidKey("Invalid initiator identity key".into()))?
        );
        
        let ek_a = PublicKey::from(
            <[u8; 32]>::try_from(initiator_ephemeral_public)
                .map_err(|_| SignalError::InvalidKey("Invalid initiator ephemeral key".into()))?
        );
        
        let dh1 = spk_b.diffie_hellman(&ik_a);
        let dh2 = ik_b.diffie_hellman(&ek_a);
        let dh3 = spk_b.diffie_hellman(&ek_a);
        
        let master_secret = if let Some(opk_b) = one_time_pre_key {
            let dh4 = opk_b.diffie_hellman(&ek_a);
            
            let mut secret = Vec::new();
            secret.extend_from_slice(dh1.as_bytes());
            secret.extend_from_slice(dh2.as_bytes());
            secret.extend_from_slice(dh3.as_bytes());
            secret.extend_from_slice(dh4.as_bytes());
            
            derive_key(&secret, KDF_INFO, 32)?
        } else {
            let mut secret = Vec::new();
            secret.extend_from_slice(dh1.as_bytes());
            secret.extend_from_slice(dh2.as_bytes());
            secret.extend_from_slice(dh3.as_bytes());
            
            derive_key(&secret, KDF_INFO, 32)?
        };
        
        Ok(X3DHResult {
            root_key: master_secret,
            ephemeral_public_key: vec![],
            used_one_time_prekey_id: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{OneTimePreKey, SignedPreKey};
    
    #[test]
    fn test_x3dh_key_exchange() {
        let alice_identity = IdentityKeyPair::generate().unwrap();
        let bob_identity = IdentityKeyPair::generate().unwrap();
        
        let bob_signed_pre_key = SignedPreKey::generate(1, &bob_identity).unwrap();
        let bob_one_time_pre_key = OneTimePreKey::generate(1).unwrap();
        
        let bob_bundle = KeyBundle {
            identity_key: bob_identity.public_key.clone(),
            signed_pre_key: bob_signed_pre_key,
            one_time_pre_key: Some(bob_one_time_pre_key),
        };
        
        let alice_result = X3DH::initiate(&alice_identity, &bob_bundle).unwrap();
        
        assert_eq!(alice_result.root_key.len(), 32);
        assert_eq!(alice_result.ephemeral_public_key.len(), 32);
        assert_eq!(alice_result.used_one_time_prekey_id, Some(1));
    }
}
