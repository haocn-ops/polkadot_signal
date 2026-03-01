use libp2p::{
    gossipsub::{self, Behaviour as Gossipsub, Config as GossipsubConfig, Event as GossipsubEvent, MessageAuthenticity, Topic, ValidationMode},
    identify::{Behaviour as Identify, Config as IdentifyConfig, Event as IdentifyEvent},
    kad::{Behaviour as Kademlia, Config as KademliaConfig, Event as KademliaEvent, store::MemoryStore},
    ping::{Behaviour as Ping, Event as PingEvent},
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use std::time::Duration;

const IDENTIFY_PROTOCOL_VERSION: &str = "polkadot-signal/1.0.0";
const IDENTIFY_AGENT_VERSION: &str = "polkadot-signal-node/0.1.0";

#[derive(NetworkBehaviour)]
pub struct SignalBehaviour {
    pub gossipsub: Gossipsub,
    pub identify: Identify,
    pub kademlia: Kademlia<MemoryStore>,
    pub ping: Ping,
}

impl SignalBehaviour {
    pub fn new(local_peer_id: PeerId) -> Self {
        let gossipsub = Self::create_gossipsub(local_peer_id);
        let identify = Self::create_identify();
        let kademlia = Self::create_kademlia(local_peer_id);
        let ping = Ping::default();
        
        Self {
            gossipsub,
            identify,
            kademlia,
            ping,
        }
    }
    
    fn create_gossipsub(peer_id: PeerId) -> Gossipsub {
        let config = GossipsubConfig::builder()
            .validation_mode(ValidationMode::Strict)
            .heartbeat_interval(Duration::from_secs(1))
            .max_transmit_size(1024 * 1024)
            .build()
            .expect("Valid gossipsub config");
        
        Gossipsub::new(
            MessageAuthenticity::Author(peer_id),
            config,
        ).expect("Valid gossipsub")
    }
    
    fn create_identify() -> Identify {
        let config = IdentifyConfig::new(
            IDENTIFY_PROTOCOL_VERSION.to_string(),
            "/polkadot-signal/1.0.0".parse().unwrap(),
        )
        .with_agent_version(IDENTIFY_AGENT_VERSION.to_string());
        
        Identify::new(config)
    }
    
    fn create_kademlia(peer_id: PeerId) -> Kademlia<MemoryStore> {
        let store = MemoryStore::new(peer_id);
        let mut config = KademliaConfig::default();
        config.set_protocol_name(StreamProtocol::new("/polkadot-signal/kad/1.0.0"));
        
        Kademlia::with_config(peer_id, store, config)
    }
    
    pub fn subscribe_to_topic(&mut self, topic: &str) -> bool {
        let topic = Topic::new(topic);
        self.gossipsub.subscribe(&topic).is_ok()
    }
    
    pub fn publish_message(&mut self, topic: &str, message: Vec<u8>) -> libp2p::gossipsub::MessageId {
        let topic = Topic::new(topic);
        self.gossipsub.publish(topic, message).expect("Failed to publish")
    }
}

#[derive(Debug)]
pub enum SignalEvent {
    Gossipsub(GossipsubEvent),
    Identify(IdentifyEvent),
    Kademlia(KademliaEvent),
    Ping(PingEvent),
}

impl From<GossipsubEvent> for SignalEvent {
    fn from(event: GossipsubEvent) -> Self {
        SignalEvent::Gossipsub(event)
    }
}

impl From<IdentifyEvent> for SignalEvent {
    fn from(event: IdentifyEvent) -> Self {
        SignalEvent::Identify(event)
    }
}

impl From<KademliaEvent> for SignalEvent {
    fn from(event: KademliaEvent) -> Self {
        SignalEvent::Kademlia(event)
    }
}

impl From<PingEvent> for SignalEvent {
    fn from(event: PingEvent) -> Self {
        SignalEvent::Ping(event)
    }
}
