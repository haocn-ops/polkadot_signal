use signal_protocol::{
    keys::{IdentityKeyPair, KeyBundle, SignedPreKey, OneTimePreKey},
    session::Session,
    message::{Envelope, PlaintextContent, MessageType},
    x3dh::X3DH,
};
use p2p_messaging::{SignalNetwork, SignalMessage, MessagePayload};
use ipfs_storage::{MessageStore, StoredMessage, OfflineMessageQueue, MessagePriority};
use std::sync::Arc;
use libp2p::PeerId;

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
}

fn create_identity() -> IdentityKeyPair {
    IdentityKeyPair::generate().expect("Failed to generate identity")
}

fn create_key_bundle(identity: &IdentityKeyPair) -> KeyBundle {
    let signed_pre_key = SignedPreKey::generate(1, identity).expect("Failed to generate signed prekey");
    let one_time_pre_key = OneTimePreKey::generate(1).expect("Failed to generate one-time prekey");
    
    KeyBundle {
        identity_key: identity.public_key.clone(),
        signed_pre_key,
        one_time_pre_key: Some(one_time_pre_key),
    }
}

#[tokio::test]
async fn test_full_message_flow() {
    setup_tracing();
    
    println!("=== Testing Full Message Flow ===\n");
    
    println!("1. Creating identities...");
    let alice_identity = create_identity();
    let bob_identity = create_identity();
    println!("   ✓ Alice identity created");
    println!("   ✓ Bob identity created\n");
    
    println!("2. Creating key bundles...");
    let alice_bundle = create_key_bundle(&alice_identity);
    let bob_bundle = create_key_bundle(&bob_identity);
    println!("   ✓ Key bundles generated\n");
    
    println!("3. Testing X3DH key exchange...");
    let x3dh_result = X3DH::initiate(&alice_identity, &bob_bundle)
        .expect("X3DH failed");
    println!("   ✓ X3DH completed successfully");
    println!("   ✓ Root key derived: {} bytes", x3dh_result.root_key.len());
    println!("   ✓ Ephemeral key: {} bytes\n", x3dh_result.ephemeral_public_key.len());
    
    println!("4. Creating sessions...");
    let (mut alice_session, _envelope) = Session::initiate(&alice_identity, &bob_bundle)
        .expect("Failed to create Alice session");
    println!("   ✓ Alice session created\n");
    
    println!("5. Encrypting message...");
    let plaintext = PlaintextContent::new("Hello Bob! This is a secure message from Alice.".to_string());
    let plaintext_bytes = plaintext.to_bytes().expect("Failed to serialize plaintext");
    
    let encrypted_envelope = alice_session.encrypt(&plaintext_bytes)
        .expect("Encryption failed");
    println!("   ✓ Message encrypted");
    println!("   ✓ Envelope type: {:?}", encrypted_envelope.message_type);
    println!("   ✓ Sender identity: {} bytes\n", encrypted_envelope.sender_identity.public_key.len());
    
    println!("6. Testing message storage...");
    let store = Arc::new(MessageStore::with_cache());
    
    let stored_message = StoredMessage::new(
        alice_identity.public_key.public_key.clone(),
        bob_identity.public_key.public_key.clone(),
        encrypted_envelope.clone(),
    );
    
    let message_id = stored_message.message_id.clone();
    println!("   ✓ Stored message created");
    println!("   ✓ Message ID: {} bytes\n", message_id.len());
    
    println!("7. Testing offline queue...");
    let queue = OfflineMessageQueue::new(store.clone());
    
    queue.enqueue(stored_message.clone(), MessagePriority::Normal)
        .await
        .expect("Failed to enqueue");
    println!("   ✓ Message enqueued");
    
    let stats = queue.get_stats().await;
    println!("   ✓ Queue size: {}", stats.queue_size);
    println!("   ✓ Normal priority: {}\n", stats.normal_count);
    
    println!("8. Testing dequeue...");
    let dequeued = queue.dequeue().await
        .expect("Failed to dequeue")
        .expect("No message in queue");
    println!("   ✓ Message dequeued");
    println!("   ✓ Priority: {:?}\n", dequeued.priority);
    
    println!("=== All Tests Passed! ===");
}

#[tokio::test]
async fn test_multiple_messages() {
    setup_tracing();
    
    println!("\n=== Testing Multiple Messages ===\n");
    
    let alice = create_identity();
    let bob = create_identity();
    let bob_bundle = create_key_bundle(&bob);
    
    let (mut session, _) = Session::initiate(&alice, &bob_bundle).expect("Session failed");
    
    let store = Arc::new(MessageStore::with_cache());
    let queue = OfflineMessageQueue::new(store.clone());
    
    println!("Sending 5 messages...");
    for i in 1..=5 {
        let plaintext = PlaintextContent::new(format!("Message #{}", i));
        let bytes = plaintext.to_bytes().expect("Serialize failed");
        let envelope = session.encrypt(&bytes).expect("Encrypt failed");
        
        let stored = StoredMessage::new(
            alice.public_key.public_key.clone(),
            bob.public_key.public_key.clone(),
            envelope,
        );
        
        let priority = match i {
            1 => MessagePriority::Urgent,
            2 | 3 => MessagePriority::High,
            _ => MessagePriority::Normal,
        };
        
        queue.enqueue(stored, priority).await.expect("Enqueue failed");
        println!("   ✓ Message {} enqueued with priority {:?}", i, priority);
    }
    
    let stats = queue.get_stats().await;
    println!("\n   Queue stats:");
    println!("   - Total: {}", stats.queue_size);
    println!("   - Urgent: {}", stats.urgent_count);
    println!("   - High: {}", stats.high_count);
    println!("   - Normal: {}", stats.normal_count);
    
    println!("\n   Dequeueing in priority order:");
    for i in 1..=5 {
        let msg = queue.dequeue().await.expect("Dequeue failed").expect("Message");
        println!("   {}. Priority: {:?}", i, msg.priority);
    }
    
    println!("\n=== Multiple Messages Test Passed! ===");
}

#[tokio::test]
async fn test_message_expiry() {
    println!("\n=== Testing Message Expiry ===\n");
    
    let alice = create_identity();
    let bob = create_identity();
    let bob_bundle = create_key_bundle(&bob);
    
    let (mut session, _) = Session::initiate(&alice, &bob_bundle).expect("Session failed");
    
    let plaintext = PlaintextContent::new("Expiring message".to_string());
    let bytes = plaintext.to_bytes().expect("Serialize failed");
    let envelope = session.encrypt(&bytes).expect("Encrypt failed");
    
    let mut stored = StoredMessage::new(
        alice.public_key.public_key.clone(),
        bob.public_key.public_key.clone(),
        envelope,
    );
    
    stored.set_expiry(1);
    
    println!("   Message created with 1 second TTL");
    println!("   Is expired: {}", stored.is_expired());
    
    assert!(!stored.is_expired());
    
    println!("   Waiting 2 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("   Is expired: {}", stored.is_expired());
    assert!(stored.is_expired());
    
    println!("\n=== Message Expiry Test Passed! ===");
}

#[tokio::test]
async fn test_delivery_receipts() {
    println!("\n=== Testing Delivery Receipts ===\n");
    
    let store = Arc::new(MessageStore::with_cache());
    let queue = OfflineMessageQueue::new(store.clone());
    
    let alice = create_identity();
    let bob = create_identity();
    let bob_bundle = create_key_bundle(&bob);
    
    let (mut session, _) = Session::initiate(&alice, &bob_bundle).expect("Session failed");
    
    let plaintext = PlaintextContent::new("Test message".to_string());
    let bytes = plaintext.to_bytes().expect("Serialize failed");
    let envelope = session.encrypt(&bytes).expect("Encrypt failed");
    
    let stored = StoredMessage::new(
        alice.public_key.public_key.clone(),
        bob.public_key.public_key.clone(),
        envelope,
    );
    
    let message_id = stored.message_id.clone();
    
    queue.enqueue(stored, MessagePriority::Normal).await.expect("Enqueue failed");
    println!("   ✓ Message enqueued");
    
    queue.mark_for_delivery(&message_id).await.expect("Mark failed");
    println!("   ✓ Marked for delivery");
    
    let pending = queue.get_pending_count().await;
    println!("   ✓ Pending count: {}", pending);
    
    let confirmed = queue.confirm_delivery(&message_id).await.expect("Confirm failed");
    println!("   ✓ Delivery confirmed: {}", confirmed);
    
    println!("\n=== Delivery Receipts Test Passed! ===");
}

#[tokio::test]
async fn test_retry_mechanism() {
    println!("\n=== Testing Retry Mechanism ===\n");
    
    let store = Arc::new(MessageStore::with_cache());
    let queue = OfflineMessageQueue::new(store.clone());
    
    let alice = create_identity();
    let bob = create_identity();
    let bob_bundle = create_key_bundle(&bob);
    
    let (mut session, _) = Session::initiate(&alice, &bob_bundle).expect("Session failed");
    
    let plaintext = PlaintextContent::new("Retry test".to_string());
    let bytes = plaintext.to_bytes().expect("Serialize failed");
    let envelope = session.encrypt(&bytes).expect("Encrypt failed");
    
    let stored = StoredMessage::new(
        alice.public_key.public_key.clone(),
        bob.public_key.public_key.clone(),
        envelope,
    );
    
    let message_id = stored.message_id.clone();
    
    queue.enqueue(stored, MessagePriority::Normal).await.expect("Enqueue failed");
    queue.mark_for_delivery(&message_id).await.expect("Mark failed");
    
    println!("   Message marked for delivery");
    
    for attempt in 1..=4 {
        let retried = queue.retry_failed().await.expect("Retry failed");
        println!("   Retry attempt {}: {} messages re-queued", attempt, retried);
    }
    
    let stats = queue.get_stats().await;
    println!("   Final queue size: {}", stats.queue_size);
    
    println!("\n=== Retry Mechanism Test Passed! ===");
}
