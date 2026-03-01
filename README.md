# Polkadot Signal

A decentralized end-to-end encrypted messenger built on Polkadot/Substrate using the Signal Protocol.

## Features

- **End-to-End Encryption**: Using Signal Protocol (X3DH + Double Ratchet)
- **Decentralized Identity**: Polkadot/Substrate-based identity management
- **P2P Messaging**: libp2p-based peer-to-peer communication
- **Offline Storage**: IPFS-based encrypted message storage
- **Group Chat**: On-chain group management with key rotation

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                         │
├─────────────────────────────────────────────────────────────┤
│                   Signal Protocol (Rust)                     │
│              X3DH + Double Ratchet + NaCl                    │
├─────────────────────────────────────────────────────────────┤
│                  Substrate Runtime (Pallets)                 │
│        signal-keys | signal-groups | message-queue           │
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
│   └── message-queue/         # Offline message queue
├── protocol/                   # Signal Protocol implementation
│   └── src/
│       ├── keys.rs            # Key generation & signing
│       ├── x3dh.rs            # X3DH key exchange
│       ├── double_ratchet.rs  # Double Ratchet encryption
│       ├── session.rs         # Session management
│       └── message.rs         # Message formats
├── p2p/                       # P2P messaging layer
├── storage/                   # IPFS storage module
├── node/                      # Substrate node
├── runtime/                   # Runtime configuration
├── client/                    # TypeScript client
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

## Signal Protocol

### X3DH Key Exchange

```
Alice (Initiator)                    Bob (Receiver)
       │                                    │
       │  1. Get Bob's key bundle           │
       │     - Identity Key (IK_B)          │
       │     - Signed Prekey (SPK_B)        │
       │     - One-time Prekey (OPK_B)      │
       │                                    │
       │  2. Generate Ephemeral Key (EK_A)  │
       │                                    │
       │  3. Compute DH                     │
       │     DH1 = DH(IK_A, SPK_B)          │
       │     DH2 = DH(EK_A, IK_B)           │
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

## TypeScript Client

```typescript
import { SignalKeysClient } from './SignalKeysClient';

const client = await SignalKeysClient.connect();

// Register identity
await client.registerIdentity(account, identityKey, signedPrekey, signature);

// Get recipient's keys
const bobKeys = await client.getIdentity(bobAddress);
const prekey = await client.getOneTimePrekey(bobAddress);
```

## Security

- **Forward Secrecy**: Double Ratchet ensures past messages stay secure
- **Identity Authentication**: On-chain identity keys prevent MITM attacks
- **Metadata Protection**: P2P direct connections reduce server exposure
- **Offline Security**: IPFS encrypted storage, only recipient can decrypt

## License

Apache-2.0

## Contributing

Contributions are welcome! Please read our contributing guidelines.

## Resources

- [Signal Protocol](https://signal.org/docs/)
- [Substrate Documentation](https://docs.substrate.io/)
- [Polkadot Wiki](https://wiki.polkadot.network/)
- [libp2p](https://libp2p.io/)
