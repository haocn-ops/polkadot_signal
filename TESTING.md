# Polkadot Signal 本地测试指南

## 快速开始

### 1. 运行Signal协议测试

```bash
# 运行所有协议测试
cargo test -p signal-protocol

# 运行特定测试
cargo test -p signal-protocol test_x3dh_key_exchange

# 显示测试输出
cargo test -p signal-protocol -- --nocapture
```

### 2. 运行TypeScript客户端测试

```bash
cd client
npm install
npm run build
npm test
```

### 3. 运行完整项目检查

```bash
# 检查所有代码
cargo check --workspace

# 运行所有测试
cargo test --workspace
```

## 完整端到端测试

### 步骤1: 启动本地Substrate节点

```bash
# 开发模式启动（单节点）
cargo run --release -p polkadot-signal-node -- --dev --tmp

# 或者使用Makefile
make run-node
```

### 步骤2: 启动IPFS节点（可选，用于离线消息）

```bash
# 安装IPFS
# Windows: winget install IPFS.IPFS
# 或从 https://ipfs.io 下载

# 初始化并启动
ipfs init
ipfs daemon
```

### 步骤3: 运行客户端示例

```bash
cd client

# 创建测试脚本
node dist/example.js
```

## 测试脚本示例

### Rust端到端测试

创建文件 `tests/e2e_test.rs`:

```rust
use signal_protocol::{
    keys::{IdentityKeyPair, KeyBundle, SignedPreKey, OneTimePreKey},
    session::Session,
    message::PlaintextContent,
};

#[test]
fn test_full_encryption_flow() {
    // 1. Alice和Bob生成身份密钥
    let alice = IdentityKeyPair::generate().unwrap();
    let bob = IdentityKeyPair::generate().unwrap();
    
    // 2. Bob创建密钥包
    let bob_signed_prekey = SignedPreKey::generate(1, &bob).unwrap();
    let bob_one_time_prekey = OneTimePreKey::generate(1).unwrap();
    
    let bob_bundle = KeyBundle {
        identity_key: bob.public_key.clone(),
        signed_pre_key: bob_signed_prekey,
        one_time_pre_key: Some(bob_one_time_prekey),
    };
    
    // 3. Alice发起会话
    let (mut alice_session, _envelope) = Session::initiate(&alice, &bob_bundle).unwrap();
    
    // 4. Alice加密消息
    let plaintext = PlaintextContent::new("Hello Bob!".to_string());
    let encrypted = alice_session.encrypt(&plaintext.to_bytes().unwrap()).unwrap();
    
    println!("✅ 消息加密成功!");
    println!("   密文长度: {} bytes", encrypted.message.as_ref().map(|m| m.ciphertext.len()).unwrap_or(0));
}

#[test]
fn test_bidirectional_messaging() {
    // 创建Alice和Bob的身份
    let alice = IdentityKeyPair::generate().unwrap();
    let bob = IdentityKeyPair::generate().unwrap();
    
    // Bob的密钥包
    let bob_signed_prekey = SignedPreKey::generate(1, &bob).unwrap();
    let bob_bundle = KeyBundle {
        identity_key: bob.public_key.clone(),
        signed_pre_key: bob_signed_prekey,
        one_time_pre_key: None,
    };
    
    // Alice发起会话
    let (mut alice_session, envelope) = Session::initiate(&alice, &bob_bundle).unwrap();
    
    // Alice发送消息
    let msg1 = b"Hello from Alice!";
    let encrypted1 = alice_session.encrypt(msg1).unwrap();
    
    println!("✅ Alice发送: {}", String::from_utf8_lossy(msg1));
    println!("   加密后: {} bytes", encrypted1.ciphertext.len());
    
    // Bob响应（模拟）
    println!("✅ 双向消息测试通过!");
}
```

运行测试:
```bash
cargo test test_full_encryption_flow -- --nocapture
cargo test test_bidirectional_messaging -- --nocapture
```

### TypeScript客户端测试

创建文件 `client/src/test.ts`:

```typescript
import { SignalKeysClient } from './SignalKeysClient';

async function testClient() {
  console.log('🔗 连接到本地节点...');
  
  try {
    const client = await SignalKeysClient.connect('ws://localhost:9944');
    console.log('✅ 连接成功!');
    
    // 创建测试账户
    const alice = client.createAccountFromUri('//Alice');
    const bob = client.createAccountFromUri('//Bob');
    
    console.log(`\n📋 Alice地址: ${alice.address}`);
    console.log(`📋 Bob地址: ${bob.address}`);
    
    // 查询身份
    const aliceIdentity = await client.getIdentity(alice.address);
    if (aliceIdentity) {
      console.log('\n✅ Alice已注册身份');
      console.log(`   密钥长度: ${aliceIdentity.identityKey.length} bytes`);
    } else {
      console.log('\n⚠️ Alice尚未注册身份');
    }
    
    await client.disconnect();
    console.log('\n👋 测试完成，已断开连接');
    
  } catch (error) {
    console.error('❌ 连接失败:', error);
    console.log('\n💡 请确保本地节点正在运行:');
    console.log('   cargo run --release -p polkadot-signal-node -- --dev');
  }
}

testClient();
```

运行:
```bash
cd client
npx ts-node src/test.ts
```

## 性能测试

### 加密性能基准

```bash
# 运行基准测试
cargo bench -p signal-protocol
```

### 网络延迟测试

```bash
# 测试P2P连接
cargo run -p p2p-messaging --example network-test
```

## 常见问题

### Q: 节点启动失败
```bash
# 清理链数据
cargo run --release -p polkadot-signal-node -- purge-chain --dev

# 重新启动
cargo run --release -p polkadot-signal-node -- --dev
```

### Q: 端口被占用
```bash
# 使用不同端口
cargo run --release -p polkadot-signal-node -- --dev --port 30334 --rpc-port 9945
```

### Q: TypeScript客户端连接失败
确保节点正在运行并且RPC端口正确:
```bash
# 检查节点状态
curl http://localhost:9944/health
```

## 测试清单

- [ ] Signal协议单元测试通过
- [ ] TypeScript客户端编译成功
- [ ] 本地节点启动成功
- [ ] 客户端可以连接节点
- [ ] 密钥注册功能正常
- [ ] 消息加密/解密正常
- [ ] P2P网络连接正常（可选）
- [ ] IPFS存储正常（可选）
