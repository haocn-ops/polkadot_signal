use crate::error::{SignalError, SignalResult};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

pub use x25519_dalek::StaticSecret as X25519StaticSecret;
pub use x25519_dalek::PublicKey as X25519PublicKeyType;

pub const KEY_LENGTH: usize = 32;
pub const SIGNATURE_LENGTH: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityKeyPair {
    pub public_key: IdentityKey,
    private_key: Vec<u8>,
}

impl IdentityKeyPair {
    pub fn generate() -> SignalResult<Self> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        let public_key = IdentityKey {
            public_key: verifying_key.to_bytes().to_vec(),
        };
        
        let private_key = signing_key.to_bytes().to_vec();
        
        Ok(Self {
            public_key,
            private_key,
        })
    }
    
    pub fn sign(&self, message: &[u8]) -> SignalResult<PreKeySignature> {
        let signing_key = SigningKey::from_bytes(
            &self.private_key.clone().try_into().map_err(|_| SignalError::InvalidKey("Invalid private key length".into()))?
        );
        
        let signature = signing_key.sign(message);
        
        Ok(PreKeySignature {
            signature: signature.to_bytes().to_vec(),
        })
    }
    
    pub fn get_x25519_public_key(&self) -> SignalResult<X25519PublicKey> {
        let secret = self.get_x25519_private_key()?;
        Ok(X25519PublicKey::from(&secret))
    }
    
    pub fn get_x25519_private_key(&self) -> SignalResult<StaticSecret> {
        let signing_key = SigningKey::from_bytes(
            &self.private_key.clone().try_into().map_err(|_| SignalError::InvalidKey("Invalid private key length".into()))?
        );
        
        Ok(StaticSecret::from(signing_key.to_bytes()))
    }
}

impl Drop for IdentityKeyPair {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityKey {
    pub public_key: Vec<u8>,
}

impl IdentityKey {
    pub fn new(public_key: Vec<u8>) -> SignalResult<Self> {
        if public_key.len() != KEY_LENGTH {
            return Err(SignalError::InvalidKey(format!(
                "Invalid key length: expected {}, got {}",
                KEY_LENGTH,
                public_key.len()
            )));
        }
        Ok(Self { public_key })
    }
    
    pub fn verify(&self, message: &[u8], signature: &PreKeySignature) -> SignalResult<bool> {
        let verifying_key = VerifyingKey::from_bytes(
            &self.public_key.clone().try_into().map_err(|_| SignalError::InvalidKey("Invalid public key length".into()))?
        ).map_err(|_| SignalError::InvalidKey("Invalid public key".into()))?;
        
        let sig = Signature::from_slice(&signature.signature)
            .map_err(|_| SignalError::InvalidSignature)?;
        
        Ok(verifying_key.verify_strict(message, &sig).is_ok())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPreKey {
    pub key_id: u32,
    pub public_key: Vec<u8>,
    pub signature: PreKeySignature,
    pub timestamp: u64,
}

impl SignedPreKey {
    pub fn generate(key_id: u32, identity: &IdentityKeyPair) -> SignalResult<Self> {
        let mut csprng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut csprng);
        let public = X25519PublicKey::from(&secret);
        
        let public_key = public.as_bytes().to_vec();
        let signature = identity.sign(&public_key)?;
        
        Ok(Self {
            key_id,
            public_key,
            signature,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    pub fn verify(&self, identity_key: &IdentityKey) -> SignalResult<bool> {
        identity_key.verify(&self.public_key, &self.signature)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePreKey {
    pub key_id: u32,
    pub public_key: Vec<u8>,
}

impl OneTimePreKey {
    pub fn generate(key_id: u32) -> SignalResult<Self> {
        let mut csprng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut csprng);
        let public = X25519PublicKey::from(&secret);
        
        Ok(Self {
            key_id,
            public_key: public.as_bytes().to_vec(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreKeySignature {
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyBundle {
    pub identity_key: IdentityKey,
    pub signed_pre_key: SignedPreKey,
    pub one_time_pre_key: Option<OneTimePreKey>,
}

impl KeyBundle {
    pub fn verify(&self) -> SignalResult<bool> {
        self.signed_pre_key.verify(&self.identity_key)
    }
}

pub fn derive_key(secret: &[u8], info: &[u8], output_len: usize) -> SignalResult<Vec<u8>> {
    use hkdf::Hkdf;
    
    let hkdf = Hkdf::<Sha256>::new(None, secret);
    let mut output = vec![0u8; output_len];
    
    hkdf.expand(info, &mut output)
        .map_err(|_| SignalError::KeyDerivationFailed)?;
    
    Ok(output)
}

pub fn kdf(input: &[u8], salt: Option<&[u8]>, info: &[u8]) -> SignalResult<[u8; 32]> {
    let mut hasher = Sha256::new();
    
    if let Some(s) = salt {
        hasher.update(s);
    }
    
    hasher.update(input);
    hasher.update(info);
    
    let result = hasher.finalize();
    
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_key_pair_generation() {
        let pair = IdentityKeyPair::generate().unwrap();
        assert_eq!(pair.public_key.public_key.len(), KEY_LENGTH);
        assert_eq!(pair.private_key.len(), KEY_LENGTH);
    }
    
    #[test]
    fn test_sign_and_verify() {
        let pair = IdentityKeyPair::generate().unwrap();
        let message = b"test message";
        
        let signature = pair.sign(message).unwrap();
        let valid = pair.public_key.verify(message, &signature).unwrap();
        
        assert!(valid);
    }
    
    #[test]
    fn test_signed_pre_key_generation() {
        let identity = IdentityKeyPair::generate().unwrap();
        let signed_pre_key = SignedPreKey::generate(1, &identity).unwrap();
        
        assert!(signed_pre_key.verify(&identity.public_key).unwrap());
    }
    
    #[test]
    fn test_one_time_pre_key_generation() {
        let pre_key = OneTimePreKey::generate(1).unwrap();
        assert_eq!(pre_key.public_key.len(), KEY_LENGTH);
    }
}
