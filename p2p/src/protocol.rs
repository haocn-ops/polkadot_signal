use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use signal_protocol::agent_message::{
    AgentMessage, AgentMessageType, AgentIdentifier, AgentType,
    TaskDefinition, TaskPriority, AgentCapabilities,
};
use signal_protocol::message::Envelope;
use std::collections::HashMap;

pub const PROTOCOL_NAME: &str = "/polkadot-signal/1.0.0";
pub const AGENT_PROTOCOL_NAME: &str = "/polkadot-signal-agent/1.0.0";
pub const GOSSIP_TOPIC: &str = "polkadot-signal-messages";
pub const AGENT_GOSSIP_TOPIC: &str = "polkadot-signal-agent-messages";

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSignalMessage {
    pub from: PeerId,
    pub to: Option<PeerId>,
    pub agent_message: AgentMessage,
    pub routing_info: RoutingInfo,
    pub timestamp: u64,
    pub message_id: Vec<u8>,
}

impl AgentSignalMessage {
    pub fn new(from: PeerId, to: Option<PeerId>, agent_message: AgentMessage) -> Self {
        use rand::Rng;
        
        let routing_info = RoutingInfo::from_message(&agent_message);
        
        Self {
            from,
            to,
            agent_message,
            routing_info,
            timestamp: current_timestamp(),
            message_id: rand::thread_rng().gen::<[u8; 16]>().to_vec(),
        }
    }
    
    pub fn direct(from: PeerId, to: PeerId, message: AgentMessage) -> Self {
        Self::new(from, Some(to), message)
    }
    
    pub fn broadcast(from: PeerId, message: AgentMessage) -> Self {
        Self::new(from, None, message)
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
pub struct RoutingInfo {
    pub routing_type: RoutingType,
    pub priority: u8,
    pub requires_ack: bool,
    pub ttl_secs: Option<u64>,
    pub route: Vec<String>,
    pub capabilities_required: Vec<String>,
}

impl RoutingInfo {
    pub fn from_message(message: &AgentMessage) -> Self {
        let routing_type = match &message.message_type {
            AgentMessageType::TaskRequest => RoutingType::Direct,
            AgentMessageType::TaskResponse => RoutingType::Direct,
            AgentMessageType::TaskStatus => RoutingType::Direct,
            AgentMessageType::CapabilityQuery => RoutingType::Broadcast,
            AgentMessageType::CapabilityResponse => RoutingType::Direct,
            AgentMessageType::Heartbeat => RoutingType::Broadcast,
            AgentMessageType::Error => RoutingType::Direct,
            AgentMessageType::ToolCall => RoutingType::Direct,
            AgentMessageType::ToolResult => RoutingType::Direct,
            AgentMessageType::StreamChunk => RoutingType::Direct,
            AgentMessageType::StreamEnd => RoutingType::Direct,
        };
        
        let priority = match &message.message_type {
            AgentMessageType::TaskRequest => {
                if let signal_protocol::agent_message::AgentPayload::TaskRequest(task) = &message.payload {
                    match task.priority {
                        TaskPriority::Critical => 4,
                        TaskPriority::High => 3,
                        TaskPriority::Normal => 2,
                        TaskPriority::Low => 1,
                    }
                } else {
                    2
                }
            }
            AgentMessageType::Error => 4,
            AgentMessageType::Heartbeat => 0,
            _ => 2,
        };
        
        let requires_ack = matches!(
            message.message_type,
            AgentMessageType::TaskRequest |
            AgentMessageType::ToolCall
        );
        
        let capabilities_required = if let signal_protocol::agent_message::AgentPayload::TaskRequest(task) = &message.payload {
            task.required_capabilities.clone()
        } else {
            Vec::new()
        };
        
        Self {
            routing_type,
            priority,
            requires_ack,
            ttl_secs: None,
            route: Vec::new(),
            capabilities_required,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoutingType {
    Direct,
    Broadcast,
    Multicast(Vec<String>),
    CapabilityBased(Vec<String>),
    RoundRobin(String),
    LoadBalanced,
}

pub struct MessageRouter {
    agent_registry: HashMap<String, AgentRouteInfo>,
    capability_index: HashMap<String, Vec<String>>,
    load_tracker: HashMap<String, AgentLoad>,
    routing_rules: Vec<RoutingRule>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            agent_registry: HashMap::new(),
            capability_index: HashMap::new(),
            load_tracker: HashMap::new(),
            routing_rules: Vec::new(),
        }
    }
    
    pub fn register_agent(
        &mut self,
        agent_id: String,
        peer_id: PeerId,
        capabilities: Vec<String>,
        max_concurrent: u32,
    ) {
        let route_info = AgentRouteInfo {
            agent_id: agent_id.clone(),
            peer_id,
            capabilities: capabilities.clone(),
            max_concurrent,
            current_load: 0,
            last_routed: 0,
            reliability_score: 1.0,
        };
        
        self.agent_registry.insert(agent_id.clone(), route_info);
        
        for cap in capabilities {
            self.capability_index
                .entry(cap)
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }
        
        self.load_tracker.insert(agent_id, AgentLoad::default());
    }
    
    pub fn unregister_agent(&mut self, agent_id: &str) {
        if let Some(info) = self.agent_registry.remove(agent_id) {
            for cap in info.capabilities {
                if let Some(agents) = self.capability_index.get_mut(&cap) {
                    agents.retain(|a| a != agent_id);
                }
            }
        }
        self.load_tracker.remove(agent_id);
    }
    
    pub fn route_message(&mut self, message: &AgentMessage) -> Option<RouteDecision> {
        match &message.message_type {
            AgentMessageType::TaskRequest => {
                self.route_task_request(message)
            }
            AgentMessageType::CapabilityQuery => {
                Some(RouteDecision::Broadcast)
            }
            AgentMessageType::Heartbeat => {
                Some(RouteDecision::Broadcast)
            }
            _ => {
                if let Some(recipient) = &message.recipient {
                    Some(RouteDecision::Direct(recipient.agent_id.clone()))
                } else {
                    Some(RouteDecision::Broadcast)
                }
            }
        }
    }
    
    fn route_task_request(&mut self, message: &AgentMessage) -> Option<RouteDecision> {
        if let Some(recipient) = &message.recipient {
            return Some(RouteDecision::Direct(recipient.agent_id.clone()));
        }
        
        if let signal_protocol::agent_message::AgentPayload::TaskRequest(task) = &message.payload {
            let capable_agents = self.find_capable_agents(&task.required_capabilities);
            
            if capable_agents.is_empty() {
                return None;
            }
            
            let best_agent = self.select_best_agent(&capable_agents)?;
            
            self.load_tracker.entry(best_agent.clone()).and_modify(|load| {
                load.current_tasks += 1;
                load.last_assigned = current_timestamp();
            });
            
            return Some(RouteDecision::Direct(best_agent));
        }
        
        None
    }
    
    fn find_capable_agents(&self, required_capabilities: &[String]) -> Vec<String> {
        if required_capabilities.is_empty() {
            return self.agent_registry.keys().cloned().collect();
        }
        
        let mut agent_capability_count: HashMap<String, usize> = HashMap::new();
        
        for cap in required_capabilities {
            if let Some(agents) = self.capability_index.get(cap) {
                for agent in agents {
                    *agent_capability_count.entry(agent.clone()).or_insert(0) += 1;
                }
            }
        }
        
        agent_capability_count
            .into_iter()
            .filter(|(_, count)| *count == required_capabilities.len())
            .map(|(agent, _)| agent)
            .filter(|agent| {
                if let Some(info) = self.agent_registry.get(agent) {
                    info.current_load < info.max_concurrent
                } else {
                    false
                }
            })
            .collect()
    }
    
    fn select_best_agent(&self, candidates: &[String]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|agent_id| {
                self.agent_registry.get(agent_id).map(|info| {
                    let load_score = 1.0 - (info.current_load as f32 / info.max_concurrent as f32);
                    let score = info.reliability_score * load_score;
                    (agent_id.clone(), score)
                })
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(agent_id, _)| agent_id)
    }
    
    pub fn update_agent_load(&mut self, agent_id: &str, delta: i32) {
        if let Some(load) = self.load_tracker.get_mut(agent_id) {
            if delta > 0 {
                load.current_tasks = load.current_tasks.saturating_add(delta as u32);
            } else {
                load.current_tasks = load.current_tasks.saturating_sub((-delta) as u32);
            }
        }
        
        if let Some(info) = self.agent_registry.get_mut(agent_id) {
            if delta > 0 {
                info.current_load = info.current_load.saturating_add(delta as u32);
            } else {
                info.current_load = info.current_load.saturating_sub((-delta) as u32);
            }
        }
    }
    
    pub fn update_agent_reliability(&mut self, agent_id: &str, success: bool) {
        if let Some(info) = self.agent_registry.get_mut(agent_id) {
            let alpha = 0.1;
            if success {
                info.reliability_score = alpha + (1.0 - alpha) * info.reliability_score;
            } else {
                info.reliability_score = (1.0 - alpha) * info.reliability_score;
            }
        }
    }
    
    pub fn add_routing_rule(&mut self, rule: RoutingRule) {
        self.routing_rules.push(rule);
    }
    
    pub fn get_agent_stats(&self) -> RouterStats {
        let total_agents = self.agent_registry.len();
        let active_agents = self.agent_registry.values()
            .filter(|info| info.current_load < info.max_concurrent)
            .count();
        let total_capabilities = self.capability_index.len();
        let total_load: u32 = self.load_tracker.values()
            .map(|load| load.current_tasks)
            .sum();
        
        RouterStats {
            total_agents,
            active_agents,
            total_capabilities,
            total_load,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentRouteInfo {
    pub agent_id: String,
    pub peer_id: PeerId,
    pub capabilities: Vec<String>,
    pub max_concurrent: u32,
    pub current_load: u32,
    pub last_routed: u64,
    pub reliability_score: f32,
}

#[derive(Clone, Debug, Default)]
pub struct AgentLoad {
    pub current_tasks: u32,
    pub last_assigned: u64,
    pub avg_response_time_ms: u64,
}

#[derive(Clone, Debug)]
pub struct RoutingRule {
    pub name: String,
    pub condition: RoutingCondition,
    pub action: RoutingAction,
    pub priority: u8,
}

#[derive(Clone, Debug)]
pub enum RoutingCondition {
    MessageType(AgentMessageType),
    CapabilityRequired(String),
    LoadAbove(f32),
    LoadBelow(f32),
    Always,
}

#[derive(Clone, Debug)]
pub enum RoutingAction {
    RouteTo(String),
    Broadcast,
    RoundRobin(Vec<String>),
    LoadBalance,
    Reject,
}

#[derive(Clone, Debug)]
pub enum RouteDecision {
    Direct(String),
    Broadcast,
    Multicast(Vec<String>),
    Reject(String),
}

#[derive(Debug)]
pub struct RouterStats {
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_capabilities: usize,
    pub total_load: u32,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_registration() {
        let mut router = MessageRouter::new();
        let peer_id = PeerId::random();
        
        router.register_agent(
            "agent_1".to_string(),
            peer_id,
            vec!["text_generation".to_string(), "translation".to_string()],
            5,
        );
        
        let stats = router.get_agent_stats();
        assert_eq!(stats.total_agents, 1);
        assert_eq!(stats.active_agents, 1);
    }
    
    #[test]
    fn test_capability_based_routing() {
        let mut router = MessageRouter::new();
        let peer_id = PeerId::random();
        
        router.register_agent(
            "agent_1".to_string(),
            peer_id,
            vec!["text_generation".to_string()],
            5,
        );
        
        let task = TaskDefinition::new(
            "generate".to_string(),
            serde_json::json!({"prompt": "test"}),
        ).with_capability("text_generation".to_string());
        
        let sender = AgentIdentifier::new("sender".to_string(), AgentType::Orchestrator);
        let message = AgentMessage::task_request(
            sender,
            AgentIdentifier::new("agent_1".to_string(), AgentType::LLM),
            task,
        );
        
        let decision = router.route_message(&message);
        assert!(decision.is_some());
    }
}
