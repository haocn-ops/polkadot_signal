use crate::behaviour::SignalBehaviour;
use crate::error::{P2PError, P2PResult};
use crate::handler::MessageHandler;
use crate::protocol::{GOSSIP_TOPIC, PeerInfo, SignalMessage};
use libp2p::{
    core::upgrade,
    gossipsub::Event as GossipsubEvent,
    identify::Event as IdentifyEvent,
    kad::Event as KademliaEvent,
    ping::Event as PingEvent,
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId, SwarmBuilder,
};
use signal_protocol::keys::IdentityKeyPair;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct SignalNetwork {
    swarm: Swarm<SignalBehaviour>,
    handler: Arc<RwLock<MessageHandler>>,
    local_peer_id: PeerId,
    command_sender: mpsc::Sender<NetworkCommand>,
    command_receiver: Option<mpsc::Receiver<NetworkCommand>>,
}

#[derive(Debug)]
pub enum NetworkCommand {
    SendMessage {
        to: PeerId,
        message: SignalMessage,
    },
    Broadcast {
        message: SignalMessage,
    },
    Dial {
        address: Multiaddr,
    },
    Subscribe {
        topic: String,
    },
    AnnounceKeys,
    RequestKeys {
        peer_id: PeerId,
    },
}

impl SignalNetwork {
    pub async fn new(identity: IdentityKeyPair) -> P2PResult<Self> {
        let handler = Arc::new(RwLock::new(MessageHandler::new(identity)));
        
        let swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .map_err(|e| P2PError::Libp2p(e.to_string()))?
            .with_quic()
            .with_behaviour(|key| {
                let peer_id = key.public().to_peer_id();
                SignalBehaviour::new(peer_id)
            })
            .map_err(|e| P2PError::Libp2p(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();
        
        let local_peer_id = *swarm.local_peer_id();
        
        let (command_sender, command_receiver) = mpsc::channel(100);
        
        Ok(Self {
            swarm,
            handler,
            local_peer_id,
            command_sender,
            command_receiver: Some(command_receiver),
        })
    }
    
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }
    
    pub fn command_sender(&self) -> mpsc::Sender<NetworkCommand> {
        self.command_sender.clone()
    }
    
    pub async fn start(&mut self) -> P2PResult<()> {
        let topic = GOSSIP_TOPIC.to_string();
        self.swarm.behaviour_mut().subscribe_to_topic(&topic);
        
        let mut command_receiver = self.command_receiver.take()
            .ok_or(P2PError::InvalidMessage("Network already started".into()))?;
        
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
                
                command = command_receiver.recv() => {
                    if let Some(cmd) = command {
                        self.handle_command(cmd).await?;
                    }
                }
            }
        }
    }
    
    async fn handle_swarm_event(&mut self, event: SwarmEvent<crate::behaviour::SignalEvent>) -> P2PResult<()> {
        match event {
            SwarmEvent::Behaviour(crate::behaviour::SignalEvent::Gossipsub(event)) => {
                self.handle_gossipsub_event(event).await?;
            }
            SwarmEvent::Behaviour(crate::behaviour::SignalEvent::Identify(event)) => {
                self.handle_identify_event(event).await?;
            }
            SwarmEvent::Behaviour(crate::behaviour::SignalEvent::Kademlia(event)) => {
                self.handle_kademlia_event(event).await?;
            }
            SwarmEvent::Behaviour(crate::behaviour::SignalEvent::Ping(event)) => {
                self.handle_ping_event(event).await?;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                tracing::info!("Connected to {} via {}", peer_id, endpoint.get_remote_address());
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                tracing::info!("Disconnected from {}: {:?}", peer_id, cause);
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                tracing::warn!("Failed to connect to {:?}: {}", peer_id, error);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_gossipsub_event(&mut self, event: GossipsubEvent) -> P2PResult<()> {
        match event {
            GossipsubEvent::Message {
                propagation_source: peer_id,
                message_id: _,
                message,
            } => {
                let signal_message = SignalMessage::from_bytes(&message.data)?;
                let handler = self.handler.read().await;
                handler.handle_message(signal_message).await?;
            }
            GossipsubEvent::Subscribed { peer_id, topic } => {
                tracing::info!("Peer {} subscribed to {}", peer_id, topic);
            }
            GossipsubEvent::Unsubscribed { peer_id, topic } => {
                tracing::info!("Peer {} unsubscribed from {}", peer_id, topic);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_identify_event(&mut self, event: IdentifyEvent) -> P2PResult<()> {
        match event {
            IdentifyEvent::Received { peer_id, info } => {
                tracing::info!(
                    "Identified peer {} with agent {}",
                    peer_id,
                    info.agent_version
                );
                
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_kademlia_event(&mut self, event: KademliaEvent) -> P2PResult<()> {
        match event {
            KademliaEvent::RoutingUpdated { peer, is_new_peer, .. } => {
                if is_new_peer {
                    tracing::info!("Kademlia routing updated with new peer: {}", peer);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_ping_event(&mut self, event: PingEvent) -> P2PResult<()> {
        match event {
            PingEvent::Success { peer, rtt } => {
                tracing::debug!("Ping to {} successful: {}ms", peer, rtt.as_millis());
            }
            PingEvent::Failure { peer, error } => {
                tracing::warn!("Ping to {} failed: {}", peer, error);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_command(&mut self, command: NetworkCommand) -> P2PResult<()> {
        match command {
            NetworkCommand::SendMessage { to, message } => {
                let message_bytes = message.to_bytes()?;
                self.swarm.behaviour_mut().publish_message(GOSSIP_TOPIC, message_bytes);
            }
            NetworkCommand::Broadcast { message } => {
                let message_bytes = message.to_bytes()?;
                self.swarm.behaviour_mut().publish_message(GOSSIP_TOPIC, message_bytes);
            }
            NetworkCommand::Dial { address } => {
                self.swarm.dial(address).map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;
            }
            NetworkCommand::Subscribe { topic } => {
                self.swarm.behaviour_mut().subscribe_to_topic(&topic);
            }
            NetworkCommand::AnnounceKeys => {
                tracing::info!("Announcing keys to network");
            }
            NetworkCommand::RequestKeys { peer_id } => {
                tracing::info!("Requesting keys from {}", peer_id);
            }
        }
        
        Ok(())
    }
    
    pub async fn listen_on(&mut self, addr: Multiaddr) -> P2PResult<()> {
        self.swarm.listen_on(addr).map_err(|e| P2PError::Libp2p(e.to_string()))?;
        Ok(())
    }
    
    pub async fn dial(&mut self, addr: Multiaddr) -> P2PResult<()> {
        self.swarm.dial(addr).map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }
    
    pub async fn send_message(&self, to: PeerId, message: SignalMessage) -> P2PResult<()> {
        self.command_sender
            .send(NetworkCommand::SendMessage { to, message })
            .await
            .map_err(|_| P2PError::ChannelClosed)?;
        Ok(())
    }
    
    pub async fn broadcast(&self, message: SignalMessage) -> P2PResult<()> {
        self.command_sender
            .send(NetworkCommand::Broadcast { message })
            .await
            .map_err(|_| P2PError::ChannelClosed)?;
        Ok(())
    }
    
    pub async fn get_handler(&self) -> Arc<RwLock<MessageHandler>> {
        self.handler.clone()
    }
}
