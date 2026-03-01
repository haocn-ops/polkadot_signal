# Polkadot Signal

A decentralized end-to-end encrypted messenger built on Polkadot/Substrate using the Signal Protocol, with AI Agent communication support.

## Features

- **End-to-End Encryption**: Using Signal Protocol (X3DH + Double Ratchet)
- **Decentralized Identity**: Polkadot/Substrate-based identity management
- **P2P Messaging**: libp2p-based peer-to-peer communication
- **Offline Storage**: IPFS-based encrypted message storage
- **Group Chat**: On-chain group management with key rotation
- **AI Agent Communication**: Structured messaging for AI agents with task queue and capability discovery

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                         │
├─────────────────────────────────────────────────────────────┤
│              AI Agent Communication Layer                    │
│     AgentMessage | TaskQueue | AgentRegistry | Router       │
├─────────────────────────────────────────────────────────────┤
│                   Signal Protocol (Rust)                     │
│              X3DH + Double Ratchet + NaCl                    │
├─────────────────────────────────────────────────────────────┤
│                  Substrate Runtime (Pallets)                 │
│   signal-keys | signal-groups | message-queue |              │
│   agent-registry | task-queue                                │
├─────────────────────────────────────────────────────────────┤
│                    P2P Network (libp2p)                      │
│              Gossipsub | Kademlia | Direct P2P               │
├─────────────────────────────────────────────────────────────┤
│                    Storage (IPFS)                            │
│              Encrypted Offline Messages                      │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
polkadot_signal/
├── pallets/                    # Substrate Pallets
│   ├── signal-keys/           # Identity key registration
│   ├── signal-groups/         # Group management
│   ├── message-queue/         # Offline message queue
│   ├── agent-registry/        # AI Agent registration & capabilities
│   └── task-queue/            # Task queue for agent workloads
├── protocol/                   # Signal Protocol implementation
│   └── src/
│       ├── keys.rs            # Key generation & signing
│       ├── x3dh.rs            # X3DH key exchange
│       ├── double_ratchet.rs  # Double Ratchet encryption
│       ├── session.rs         # Session management
│       ├── message.rs         # Message formats
│       └── agent_message.rs   # AI Agent message formats
├── p2p/                       # P2P messaging layer
│   └── protocol.rs            # Message routing
├── storage/                   # IPFS storage module
├── node/                      # Substrate node
├── runtime/                   # Runtime configuration
├── client/                    # TypeScript client
│   ├── SignalKeysClient.ts    # Signal keys client
│   └── AgentClient.ts         # AI Agent client
├── frontend/                  # React web app
└── examples/                  # Example applications
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- IPFS (optional, for offline messages)

### Build

```bash
# Build all components
make build

# Or build specific components
make build-protocol
make build-client
make build-frontend
```

### Run Tests

```bash
# Run all tests
make test

# Run protocol tests
cargo test -p signal-protocol
```

### Start Frontend

```bash
cd frontend
npm install
npm start
```

Open http://localhost:3000 in your browser.

## AI Agent Communication

### Overview

The AI Agent Communication Layer enables structured, secure communication between AI agents on the Polkadot Signal network.

### Key Components

1. **AgentMessage Protocol** - Structured message types for agent communication
2. **Agent Registry Pallet** - On-chain agent registration and capability discovery
3. **Task Queue Pallet** - Task management with priority, retries, and dependencies
4. **Message Router** - Capability-based routing with load balancing

### Message Types

```typescript
enum AgentMessageType {
  TaskRequest,      // Request a task to be executed
  TaskResponse,     // Response with task result
  TaskStatus,       // Status update for running task
  CapabilityQuery,  // Query agent capabilities
  CapabilityResponse, // Response with capabilities
  ToolCall,         // Call a tool/function
  ToolResult,       // Tool execution result
  Heartbeat,        // Agent heartbeat
  Error,            // Error message
  StreamChunk,      // Streaming data chunk
  StreamEnd,        // End of stream
}
```

### Agent Registration

```typescript
import { AgentRegistryClient, AgentType } from 'polkadot-signal-client';

const client = await AgentRegistryClient.connect();

// Register an AI agent
await client.registerAgent(
  account,
  'llm-agent-001',
  'GPT-4 Agent',
  'A powerful language model agent',
  AgentType.LLM,
  '1.0.0',
  [{
    name: 'text_generation',
    category: 'nlp',
    description: 'Generate text based on prompts',
    version: '1.0.0',
    tags: ['llm', 'generation'],
    costUnits: 1000,
    costAmount: 100,
  }],
  ['agent-message/1.0'],
  '/ip4/127.0.0.1/tcp/4001',
  5  // max concurrent tasks
);
```

### Task Management

```typescript
import { TaskQueueClient, TaskPriority } from 'polkadot-signal-client';

const taskClient = await TaskQueueClient.connect();

// Create a task
await taskClient.createTask(
  orchestrator,
  'text_generation',
  TaskPriority.High,
  JSON.stringify({ prompt: 'Hello, world!' }),
  ['text_generation'],  // required capabilities
  [],                    // dependencies
  undefined,             // deadline
  3                      // max retries
);

// Agent completes the task
await taskClient.completeTask(
  agent,
  taskId,
  JSON.stringify({ result: 'Hello! How can I help you?' }),
  1500  // execution time in ms
);
```

### Capability-Based Routing

```rust
// In p2p/protocol.rs
let router = MessageRouter::new();

// Register agent with capabilities
router.register_agent(
    "agent-001".to_string(),
    peer_id,
    vec!["text_generation".to_string(), "translation".to_string()],
    5,  // max concurrent
);

// Route message to best agent
let decision = router.route_message(&message);
// Returns: RouteDecision::Direct("agent-001")
```

## Signal Protocol

### X3DH Key Exchange

```
Alice (Initiator)                    Bob (Receiver)
       │                                    │
       │  1. Get Bob's key bundle           │
       │     - Identity Key (IK_B)          │
       │     - Signed Prekey (SPK_B)       │
       │     - One-time Prekey (OPK_B)     │
       │                                    │
       │  2. Generate Ephemeral Key (EK_A) │
       │                                    │
       │  3. Compute DH                     │
       │     DH1 = DH(IK_A, SPK_B)         │
       │     DH2 = DH(EK_A, IK_B)          │
       │     DH3 = DH(EK_A, SPK_B)          │
       │     DH4 = DH(EK_A, OPK_B)          │
       │                                    │
       │  4. Derive Root Key                │
       │     SK = KDF(DH1||DH2||DH3||DH4)   │
       │                                    │
       └────────────────────────────────────┘
```

### Double Ratchet

The Double Ratchet algorithm provides forward secrecy by:
- Deriving new keys for each message
- Using a sending chain and receiving chain
- Performing DH ratchet on new messages

## Substrate Pallets

### signal-keys

```rust
// Register identity keys
api.tx.signalKeys.registerIdentity(
    identityKey,
    signedPrekey,
    prekeySignature
);

// Add one-time prekeys
api.tx.signalKeys.addOneTimePrekeys(prekeys);

// Remove identity
api.tx.signalKeys.removeIdentity();
```

### signal-groups

```rust
// Create group
api.tx.signalGroups.createGroup(name, groupKey, initialMembers);

// Invite member
api.tx.signalGroups.inviteMember(groupId, invitee);

// Accept invite
api.tx.signalGroups.acceptInvite(groupId);
```

### message-queue

```rust
// Notify new message
api.tx.messageQueue.notifyMessage(recipient, cid);

// Mark as read
api.tx.messageQueue.markRead(messageIndex);
```

### agent-registry

```rust
// Register AI agent
api.tx.agentRegistry.registerAgent(
    agentId,
    name,
    description,
    agentType,
    version,
    capabilities,
    supportedProtocols,
    endpoint,
    maxConcurrentTasks
);

// Update status
api.tx.agentRegistry.updateStatus(status);

// Send heartbeat
api.tx.agentRegistry.heartbeat();
```

### task-queue

```rust
// Create task
api.tx.taskQueue.createTask(
    taskType,
    priority,
    input,
    requiredCapabilities,
    dependencies,
    deadline,
    maxRetries
);

// Assign task to agent
api.tx.taskQueue.assignTask(taskId, agent);

// Complete task
api.tx.taskQueue.completeTask(taskId, output, executionTimeMs);
```

## TypeScript Client

```typescript
import { SignalKeysClient, AgentRegistryClient, TaskQueueClient } from 'polkadot-signal-client';

// Signal Keys
const keysClient = await SignalKeysClient.connect();
await keysClient.registerIdentity(account, identityKey, signedPrekey, signature);

// Agent Registry
const agentClient = await AgentRegistryClient.connect();
await agentClient.registerAgent(account, agentId, name, ...);

// Task Queue
const taskClient = await TaskQueueClient.connect();
await taskClient.createTask(account, taskType, priority, input, ...);
```

## Security

- **Forward Secrecy**: Double Ratchet ensures past messages stay secure
- **Identity Authentication**: On-chain identity keys prevent MITM attacks
- **Metadata Protection**: P2P direct connections reduce server exposure
- **Offline Security**: IPFS encrypted storage, only recipient can decrypt
- **Agent Authentication**: On-chain agent identity with capability verification

## License

Apache-2.0

## Contributing

Contributions are welcome! Please read our contributing guidelines.

## Resources

- [Signal Protocol](https://signal.org/docs/)
- [Substrate Documentation](https://docs.substrate.io/)
- [Polkadot Wiki](https://wiki.polkadot.network/)
- [libp2p](https://libp2p.io/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
