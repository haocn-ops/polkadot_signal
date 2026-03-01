use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentMessageType {
    TaskRequest,
    TaskResponse,
    TaskStatus,
    CapabilityQuery,
    CapabilityResponse,
    Heartbeat,
    Error,
    ToolCall,
    ToolResult,
    StreamChunk,
    StreamEnd,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentMessage {
    pub version: String,
    pub message_id: String,
    pub message_type: AgentMessageType,
    pub sender: AgentIdentifier,
    pub recipient: Option<AgentIdentifier>,
    pub timestamp: u64,
    pub payload: AgentPayload,
    pub metadata: HashMap<String, String>,
    pub correlation_id: Option<String>,
    pub ttl: Option<u64>,
}

impl AgentMessage {
    pub fn new(
        message_type: AgentMessageType,
        sender: AgentIdentifier,
        payload: AgentPayload,
    ) -> Self {
        use rand::Rng;
        
        Self {
            version: "1.0.0".to_string(),
            message_id: Self::generate_message_id(),
            message_type,
            sender,
            recipient: None,
            timestamp: current_timestamp(),
            payload,
            metadata: HashMap::new(),
            correlation_id: None,
            ttl: None,
        }
    }
    
    pub fn with_recipient(mut self, recipient: AgentIdentifier) -> Self {
        self.recipient = Some(recipient);
        self
    }
    
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl = Some(current_timestamp() + ttl_secs);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            current_timestamp() > ttl
        } else {
            false
        }
    }
    
    pub fn task_request(
        sender: AgentIdentifier,
        recipient: AgentIdentifier,
        task: TaskDefinition,
    ) -> Self {
        Self::new(AgentMessageType::TaskRequest, sender, AgentPayload::TaskRequest(task))
            .with_recipient(recipient)
    }
    
    pub fn task_response(
        sender: AgentIdentifier,
        recipient: AgentIdentifier,
        correlation_id: String,
        result: TaskResult,
    ) -> Self {
        Self::new(AgentMessageType::TaskResponse, sender, AgentPayload::TaskResponse(result))
            .with_recipient(recipient)
            .with_correlation_id(correlation_id)
    }
    
    pub fn capability_query(sender: AgentIdentifier, query: CapabilityQuery) -> Self {
        Self::new(AgentMessageType::CapabilityQuery, sender, AgentPayload::CapabilityQuery(query))
    }
    
    pub fn capability_response(
        sender: AgentIdentifier,
        correlation_id: String,
        capabilities: AgentCapabilities,
    ) -> Self {
        Self::new(
            AgentMessageType::CapabilityResponse,
            sender,
            AgentPayload::CapabilityResponse(capabilities),
        )
        .with_correlation_id(correlation_id)
    }
    
    pub fn tool_call(
        sender: AgentIdentifier,
        recipient: AgentIdentifier,
        tool_call: ToolCall,
    ) -> Self {
        Self::new(AgentMessageType::ToolCall, sender, AgentPayload::ToolCall(tool_call))
            .with_recipient(recipient)
    }
    
    pub fn tool_result(
        sender: AgentIdentifier,
        recipient: AgentIdentifier,
        correlation_id: String,
        result: ToolResult,
    ) -> Self {
        Self::new(AgentMessageType::ToolResult, sender, AgentPayload::ToolResult(result))
            .with_recipient(recipient)
            .with_correlation_id(correlation_id)
    }
    
    pub fn error(
        sender: AgentIdentifier,
        correlation_id: Option<String>,
        error: AgentError,
    ) -> Self {
        let mut msg = Self::new(AgentMessageType::Error, sender, AgentPayload::Error(error));
        msg.correlation_id = correlation_id;
        msg
    }
    
    pub fn to_bytes(&self) -> crate::error::SignalResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
    
    pub fn from_bytes(data: &[u8]) -> crate::error::SignalResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| crate::error::SignalError::Serialization(e.to_string()))
    }
    
    fn generate_message_id() -> String {
        use rand::Rng;
        let random_bytes: [u8; 16] = rand::thread_rng().gen();
        hex::encode(random_bytes)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentIdentifier {
    pub agent_id: String,
    pub account_address: Option<String>,
    pub peer_id: Option<String>,
    pub agent_type: AgentType,
}

impl AgentIdentifier {
    pub fn new(agent_id: String, agent_type: AgentType) -> Self {
        Self {
            agent_id,
            account_address: None,
            peer_id: None,
            agent_type,
        }
    }
    
    pub fn with_account(mut self, address: String) -> Self {
        self.account_address = Some(address);
        self
    }
    
    pub fn with_peer_id(mut self, peer_id: String) -> Self {
        self.peer_id = Some(peer_id);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentType {
    LLM,
    Tool,
    Orchestrator,
    Worker,
    Coordinator,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentPayload {
    TaskRequest(TaskDefinition),
    TaskResponse(TaskResult),
    TaskStatus(TaskStatusUpdate),
    CapabilityQuery(CapabilityQuery),
    CapabilityResponse(AgentCapabilities),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
    Heartbeat(HeartbeatData),
    Error(AgentError),
    StreamChunk(StreamChunkData),
    StreamEnd(StreamEndData),
    Custom(serde_json::Value),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub task_id: String,
    pub task_type: String,
    pub priority: TaskPriority,
    pub input: serde_json::Value,
    pub deadline: Option<u64>,
    pub max_retries: u32,
    pub dependencies: Vec<String>,
    pub required_capabilities: Vec<String>,
}

impl TaskDefinition {
    pub fn new(task_type: String, input: serde_json::Value) -> Self {
        use rand::Rng;
        let random_bytes: [u8; 8] = rand::thread_rng().gen();
        
        Self {
            task_id: format!("task_{}", hex::encode(random_bytes)),
            task_type,
            priority: TaskPriority::Normal,
            input,
            deadline: None,
            max_retries: 3,
            dependencies: Vec::new(),
            required_capabilities: Vec::new(),
        }
    }
    
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }
    
    pub fn with_capability(mut self, capability: String) -> Self {
        self.required_capabilities.push(capability);
        self
    }
    
    pub fn with_dependency(mut self, task_id: String) -> Self {
        self.dependencies.push(task_id);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskCompletionStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub resources_used: ResourceUsage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskCompletionStatus {
    Success,
    PartialSuccess,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_time_ms: u64,
    pub memory_bytes: u64,
    pub tokens_used: Option<u64>,
    pub api_calls: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskStatusUpdate {
    pub task_id: String,
    pub status: TaskExecutionStatus,
    pub progress: f32,
    pub message: Option<String>,
    pub estimated_remaining_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskExecutionStatus {
    Pending,
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityQuery {
    pub query_type: CapabilityQueryType,
    pub filter: Option<CapabilityFilter>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CapabilityQueryType {
    All,
    ByName(String),
    ByCategory(String),
    ByVersion { name: String, min_version: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityFilter {
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub min_reliability: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub supported_protocols: Vec<String>,
    pub max_concurrent_tasks: u32,
    pub average_response_time_ms: u64,
    pub reliability_score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub category: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub cost: Option<CapabilityCost>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityCost {
    pub unit: String,
    pub amount: f64,
    pub per_request: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

impl ToolCall {
    pub fn new(tool_name: String, arguments: serde_json::Value) -> Self {
        use rand::Rng;
        let random_bytes: [u8; 8] = rand::thread_rng().gen();
        
        Self {
            tool_id: format!("tool_{}", hex::encode(random_bytes)),
            tool_name,
            arguments,
            timeout_ms: None,
        }
    }
    
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub status: AgentStatus,
    pub active_tasks: u32,
    pub queue_size: u32,
    pub load_average: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Overloaded,
    Maintenance,
    Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub recoverable: bool,
}

impl AgentError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            recoverable: false,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamChunkData {
    pub stream_id: String,
    pub sequence: u32,
    pub content: serde_json::Value,
    pub is_final: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamEndData {
    pub stream_id: String,
    pub total_chunks: u32,
    pub final_result: Option<serde_json::Value>,
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
    fn test_task_request_creation() {
        let sender = AgentIdentifier::new("agent_1".to_string(), AgentType::LLM);
        let recipient = AgentIdentifier::new("agent_2".to_string(), AgentType::Worker);
        let task = TaskDefinition::new(
            "text_generation".to_string(),
            serde_json::json!({"prompt": "Hello"}),
        );
        
        let message = AgentMessage::task_request(sender, recipient, task);
        
        assert_eq!(message.message_type, AgentMessageType::TaskRequest);
        assert!(message.recipient.is_some());
        assert!(message.correlation_id.is_none());
    }
    
    #[test]
    fn test_capability_query() {
        let sender = AgentIdentifier::new("agent_1".to_string(), AgentType::Orchestrator);
        let query = CapabilityQuery {
            query_type: CapabilityQueryType::ByCategory("nlp".to_string()),
            filter: None,
        };
        
        let message = AgentMessage::capability_query(sender, query);
        
        assert_eq!(message.message_type, AgentMessageType::CapabilityQuery);
    }
    
    #[test]
    fn test_message_serialization() {
        let sender = AgentIdentifier::new("agent_1".to_string(), AgentType::LLM);
        let task = TaskDefinition::new(
            "test_task".to_string(),
            serde_json::json!({"input": "test"}),
        );
        let message = AgentMessage::task_request(
            sender,
            AgentIdentifier::new("agent_2".to_string(), AgentType::Worker),
            task,
        );
        
        let bytes = message.to_bytes().unwrap();
        let decoded = AgentMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.message_type, AgentMessageType::TaskRequest);
    }
    
    #[test]
    fn test_tool_call() {
        let sender = AgentIdentifier::new("agent_1".to_string(), AgentType::LLM);
        let recipient = AgentIdentifier::new("tool_agent".to_string(), AgentType::Tool);
        let tool_call = ToolCall::new(
            "search".to_string(),
            serde_json::json!({"query": "test"}),
        );
        
        let message = AgentMessage::tool_call(sender, recipient, tool_call);
        
        assert_eq!(message.message_type, AgentMessageType::ToolCall);
    }
}
