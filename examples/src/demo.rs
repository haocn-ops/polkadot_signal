use p2p_messaging::{SignalNetwork, SignalMessage, MessagePayload};
use signal_protocol::{
    keys::{IdentityKeyPair, KeyBundle, SignedPreKey, OneTimePreKey},
    session::Session,
    message::PlaintextContent,
};
use libp2p::PeerId;
use std::sync::Arc;
use tokio::sync::RwLock;

struct SignalApp {
    identity: IdentityKeyPair,
    network: Arc<RwLock<SignalNetwork>>,
    sessions: Arc<RwLock<Vec<Session>>>,
}

impl SignalApp {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let identity = IdentityKeyPair::generate()?;
        let network = SignalNetwork::new(identity.clone()).await?;
        
        Ok(Self {
            identity,
            network: Arc::new(RwLock::new(network)),
            sessions: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut network = self.network.write().await;
        network.listen_on("/ip4/0.0.0.0/tcp/0".parse()?).await?;
        
        println!("🚀 Signal node started");
        println!("📍 Peer ID: {}", network.local_peer_id());
        
        Ok(())
    }
    
    async fn connect_to_peer(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut network = self.network.write().await;
        network.dial(addr.parse()?).await?;
        println!("🔗 Connecting to peer: {}", addr);
        Ok(())
    }
    
    async fn create_key_bundle(&self) -> Result<KeyBundle, Box<dyn std::error::Error>> {
        let signed_pre_key = SignedPreKey::generate(1, &self.identity)?;
        let one_time_pre_key = OneTimePreKey::generate(1)?;
        
        Ok(KeyBundle {
            identity_key: self.identity.public_key.clone(),
            signed_pre_key,
            one_time_pre_key: Some(one_time_pre_key),
        })
    }
    
    async fn start_conversation(&self, recipient_bundle: &KeyBundle) -> Result<(), Box<dyn std::error::Error>> {
        let (session, envelope) = Session::initiate(&self.identity, recipient_bundle)?;
        
        let mut sessions = self.sessions.write().await;
        sessions.push(session);
        
        println!("💬 Started new conversation");
        
        Ok(())
    }
    
    async fn send_message(&self, peer_id: PeerId, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = PlaintextContent::new(text.to_string());
        let plaintext_bytes = plaintext.to_bytes()?;
        
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.first_mut() {
            let envelope = session.encrypt(&plaintext_bytes)?;
            
            let message = SignalMessage::direct(
                self.network.read().await.local_peer_id(),
                peer_id,
                envelope,
            );
            
            let network = self.network.read().await;
            network.send_message(peer_id, message).await?;
            
            println!("📤 Message sent to {}", peer_id);
        }
        
        Ok(())
    }
    
    fn get_peer_id(&self) -> PeerId {
        self.network.try_read().map(|n| n.local_peer_id()).unwrap_or_default()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          Polkadot Signal - Decentralized Messenger         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    let alice = SignalApp::new().await?;
    let bob = SignalApp::new().await?;
    
    alice.start().await?;
    bob.start().await?;
    
    println!("\n📋 Alice's Peer ID: {}", alice.get_peer_id());
    println!("📋 Bob's Peer ID: {}", bob.get_peer_id());
    
    println!("\n🔑 Generating key bundles...");
    let alice_bundle = alice.create_key_bundle().await?;
    let bob_bundle = bob.create_key_bundle().await?;
    
    println!("\n✅ Key bundles generated");
    println!("   Alice identity key: {}...", hex::encode(&alice_bundle.identity_key.public_key[..8]));
    println!("   Bob identity key: {}...", hex::encode(&bob_bundle.identity_key.public_key[..8]));
    
    println!("\n🔐 Starting X3DH key exchange...");
    alice.start_conversation(&bob_bundle).await?;
    println!("✅ Alice initiated session with Bob");
    
    println!("\n📤 Sending encrypted message...");
    let bob_peer_id = bob.get_peer_id();
    alice.send_message(bob_peer_id, "Hello Bob! This is a secure message.").await?;
    
    println!("\n✅ Demo completed successfully!");
    println!("\n📊 Architecture Summary:");
    println!("   ├── Substrate Pallet (signal-keys)");
    println!("   │   └── On-chain identity & key registration");
    println!("   ├── Signal Protocol (protocol/)");
    println!("   │   ├── X3DH key agreement");
    println!("   │   ├── Double Ratchet encryption");
    println!("   │   └── Session management");
    println!("   └── P2P Network (p2p/)");
    println!("       ├── libp2p gossipsub for messaging");
    println!("       ├── Kademlia DHT for peer discovery");
    println!("       └── Direct P2P connections");
    
    Ok(())
}
