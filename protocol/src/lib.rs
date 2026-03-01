pub mod error;
pub mod keys;
pub mod x3dh;
pub mod double_ratchet;
pub mod session;
pub mod message;
pub mod agent_message;

pub use error::{SignalError, SignalResult};
pub use keys::{
    IdentityKeyPair, IdentityKey, SignedPreKey, OneTimePreKey,
    KeyBundle, PreKeySignature,
};
pub use x3dh::X3DH;
pub use double_ratchet::DoubleRatchet;
pub use session::Session;
pub use message::{Envelope, Message, MessageType};
pub use agent_message::{
    AgentMessage, AgentMessageType, AgentIdentifier, AgentType,
    AgentPayload, TaskDefinition, TaskResult, TaskPriority,
    AgentCapabilities, Capability, ToolCall, ToolResult, AgentError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{IdentityKeyPair, SignedPreKey, OneTimePreKey};
    
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
