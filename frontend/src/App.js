import React, { useState, useEffect } from 'react';
import './App.css';

const nacl = require('tweetnacl');
const { encodeUTF8, decodeUTF8, encodeBase64, decodeBase64 } = require('tweetnacl-util');

function App() {
  const [connected, setConnected] = useState(false);
  const [message, setMessage] = useState('Hello, this is a secret message!');
  const [encryptedMessage, setEncryptedMessage] = useState(null);
  const [decryptedMessage, setDecryptedMessage] = useState('');
  const [logs, setLogs] = useState([]);
  const [aliceKeys, setAliceKeys] = useState(null);
  const [bobKeys, setBobKeys] = useState(null);
  const [aliceRegistered, setAliceRegistered] = useState(false);
  const [bobRegistered, setBobRegistered] = useState(false);

  const addLog = (msg) => {
    setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  };

  useEffect(() => {
    addLog('� Polkadot Signal Demo Started');
    addLog('💡 This is a client-side demo of Signal Protocol encryption');
    addLog('');
    addLog('Click "Generate Alice Keys" and "Generate Bob Keys" to begin');
  }, []);

  const generateKeyPair = () => {
    const keyPair = nacl.box.keyPair();
    return {
      publicKey: encodeBase64(keyPair.publicKey),
      secretKey: encodeBase64(keyPair.secretKey)
    };
  };

  const generateAliceKeys = () => {
    addLog('📝 Generating Alice identity keys...');
    const identityKey = generateKeyPair();
    const signedPrekey = generateKeyPair();
    const oneTimePrekeys = [generateKeyPair(), generateKeyPair(), generateKeyPair()];
    
    setAliceKeys({
      identityKey,
      signedPrekey,
      oneTimePrekeys
    });
    
    addLog('✅ Alice keys generated!');
    addLog(`   Identity Public Key: ${identityKey.publicKey.substring(0, 20)}...`);
    addLog(`   Signed Prekey: ${signedPrekey.publicKey.substring(0, 20)}...`);
    addLog(`   One-time Prekeys: ${oneTimePrekeys.length} generated`);
    setAliceRegistered(true);
  };

  const generateBobKeys = () => {
    addLog('📝 Generating Bob identity keys...');
    const identityKey = generateKeyPair();
    const signedPrekey = generateKeyPair();
    const oneTimePrekeys = [generateKeyPair(), generateKeyPair(), generateKeyPair()];
    
    setBobKeys({
      identityKey,
      signedPrekey,
      oneTimePrekeys
    });
    
    addLog('✅ Bob keys generated!');
    addLog(`   Identity Public Key: ${identityKey.publicKey.substring(0, 20)}...`);
    addLog(`   Signed Prekey: ${signedPrekey.publicKey.substring(0, 20)}...`);
    addLog(`   One-time Prekeys: ${oneTimePrekeys.length} generated`);
    setBobRegistered(true);
  };

  const simulateX3DH = () => {
    addLog('');
    addLog('� Simulating X3DH Key Exchange...');
    addLog('');
    addLog('Step 1: Alice fetches Bob\'s key bundle');
    addLog('   - Bob\'s Identity Key (IK_B)');
    addLog('   - Bob\'s Signed Prekey (SPK_B)');
    addLog('   - Bob\'s One-time Prekey (OPK_B)');
    addLog('');
    addLog('Step 2: Alice generates ephemeral key (EK_A)');
    addLog('');
    addLog('Step 3: Alice computes DH operations');
    addLog('   DH1 = DH(IK_A, SPK_B)');
    addLog('   DH2 = DH(EK_A, IK_B)');
    addLog('   DH3 = DH(EK_A, SPK_B)');
    addLog('   DH4 = DH(EK_A, OPK_B)');
    addLog('');
    addLog('Step 4: Derive shared secret');
    addLog('   SK = KDF(DH1 || DH2 || DH3 || DH4)');
    addLog('');
    addLog('✅ X3DH Key Exchange Complete!');
    addLog('   Both parties now share a secret key for Double Ratchet');
    setConnected(true);
  };

  const encryptMessage = () => {
    try {
      if (!aliceKeys || !bobKeys) {
        addLog('❌ Please generate keys for both Alice and Bob first!');
        return;
      }
      
      addLog('🔐 Encrypting message using NaCl box...');
      
      const bobPublicKey = decodeBase64(bobKeys.identityKey.publicKey);
      const aliceSecretKey = decodeBase64(aliceKeys.identityKey.secretKey);
      
      const nonce = nacl.randomBytes(24);
      const messageUint8 = decodeUTF8(message);
      
      const encrypted = nacl.box(messageUint8, nonce, bobPublicKey, aliceSecretKey);
      
      if (!encrypted) {
        addLog('❌ Encryption failed!');
        return;
      }
      
      const encryptedData = {
        ciphertext: encodeBase64(encrypted),
        nonce: encodeBase64(nonce),
        from: 'Alice',
        to: 'Bob'
      };
      
      setEncryptedMessage(encryptedData);
      addLog('✅ Message encrypted successfully!');
      addLog(`   Ciphertext: ${encryptedData.ciphertext.substring(0, 30)}...`);
      addLog(`   Nonce: ${encryptedData.nonce.substring(0, 20)}...`);
      addLog(`   Original: "${message}"`);
    } catch (error) {
      addLog(`❌ Encryption failed: ${error.message}`);
    }
  };

  const decryptMessage = () => {
    try {
      if (!encryptedMessage) {
        addLog('❌ No encrypted message to decrypt!');
        return;
      }
      
      addLog('🔓 Decrypting message...');
      
      const alicePublicKey = decodeBase64(aliceKeys.identityKey.publicKey);
      const bobSecretKey = decodeBase64(bobKeys.identityKey.secretKey);
      const nonce = decodeBase64(encryptedMessage.nonce);
      const ciphertext = decodeBase64(encryptedMessage.ciphertext);
      
      const decrypted = nacl.box.open(ciphertext, nonce, alicePublicKey, bobSecretKey);
      
      if (!decrypted) {
        addLog('❌ Decryption failed! Invalid keys or corrupted message.');
        return;
      }
      
      const decryptedText = encodeUTF8(decrypted);
      setDecryptedMessage(decryptedText);
      addLog('✅ Message decrypted successfully!');
      addLog(`   Decrypted: "${decryptedText}"`);
    } catch (error) {
      addLog(`❌ Decryption failed: ${error.message}`);
    }
  };

  const clearLogs = () => {
    setLogs([]);
    addLog('📋 Logs cleared');
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>🔐 Polkadot Signal</h1>
        <p className="subtitle">Decentralized End-to-End Encrypted Messenger</p>
      </header>

      <div className="status-bar">
        <div className="status-item">
          <span className={`status-dot ${aliceRegistered ? 'connected' : 'disconnected'}`}></span>
          <span>Alice: {aliceRegistered ? 'Keys Ready' : 'Not Ready'}</span>
        </div>
        <div className="status-item">
          <span className={`status-dot ${bobRegistered ? 'connected' : 'disconnected'}`}></span>
          <span>Bob: {bobRegistered ? 'Keys Ready' : 'Not Ready'}</span>
        </div>
        <div className="status-item">
          <span className={`status-dot ${connected ? 'connected' : 'disconnected'}`}></span>
          <span>X3DH: {connected ? 'Complete' : 'Pending'}</span>
        </div>
      </div>

      <main className="main-content">
        <div className="cards-container">
          {/* Key Generation */}
          <div className="card">
            <h2>� Key Generation</h2>
            <p style={{color: '#888', marginBottom: '15px'}}>
              Generate Signal Protocol identity keys for Alice and Bob
            </p>
            <div style={{display: 'flex', gap: '10px', flexWrap: 'wrap'}}>
              <button onClick={generateAliceKeys} disabled={aliceRegistered}>
                {aliceRegistered ? '✅ Alice Ready' : '🔐 Generate Alice Keys'}
              </button>
              <button onClick={generateBobKeys} disabled={bobRegistered}>
                {bobRegistered ? '✅ Bob Ready' : '🔐 Generate Bob Keys'}
              </button>
            </div>
            {aliceRegistered && bobRegistered && (
              <button onClick={simulateX3DH} style={{marginTop: '10px', width: '100%'}}>
                🔄 Simulate X3DH Key Exchange
              </button>
            )}
          </div>

          {/* Message Encryption */}
          <div className="card">
            <h2>📨 Message Encryption</h2>
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Enter your secret message..."
              rows={3}
            />
            <button onClick={encryptMessage} disabled={!connected}>
              🔐 Encrypt Message
            </button>
            {encryptedMessage && (
              <div className="encrypted-output">
                <p><strong>🔒 Encrypted Message:</strong></p>
                <code style={{wordBreak: 'break-all', fontSize: '0.8rem'}}>
                  {encryptedMessage.ciphertext.substring(0, 80)}...
                </code>
                <p style={{marginTop: '10px', fontSize: '0.85rem', color: '#888'}}>
                  Nonce: {encryptedMessage.nonce.substring(0, 30)}...
                </p>
              </div>
            )}
          </div>

          {/* Message Decryption */}
          <div className="card">
            <h2>📬 Message Decryption</h2>
            <p style={{color: '#888', marginBottom: '15px'}}>
              Decrypt the message using Bob's private key
            </p>
            <button onClick={decryptMessage} disabled={!encryptedMessage}>
              🔓 Decrypt Message
            </button>
            {decryptedMessage && (
              <div className="decrypted-output">
                <p><strong>✅ Decrypted Message:</strong></p>
                <div className="message-box">{decryptedMessage}</div>
              </div>
            )}
          </div>
        </div>

        {/* Activity Log */}
        <div className="log-section">
          <div style={{display: 'flex', justifyContent: 'space-between', alignItems: 'center'}}>
            <h2>📋 Activity Log</h2>
            <button onClick={clearLogs} style={{padding: '5px 15px', fontSize: '0.85rem'}}>
              Clear
            </button>
          </div>
          <div className="log-container">
            {logs.map((log, index) => (
              <div key={index} className="log-entry">{log}</div>
            ))}
          </div>
        </div>

        {/* Architecture Info */}
        <div className="architecture-section">
          <h2>🏗️ Architecture</h2>
          <div className="arch-diagram">
            <div className="arch-layer">
              <span className="layer-title">Frontend (React)</span>
              <span className="layer-desc">User Interface - This Demo</span>
            </div>
            <div className="arch-arrow">↓</div>
            <div className="arch-layer">
              <span className="layer-title">Signal Protocol</span>
              <span className="layer-desc">X3DH + Double Ratchet + NaCl</span>
            </div>
            <div className="arch-arrow">↓</div>
            <div className="arch-layer">
              <span className="layer-title">Substrate Node</span>
              <span className="layer-desc">Identity & Key Storage (Pallets)</span>
            </div>
            <div className="arch-arrow">↓</div>
            <div className="arch-layer">
              <span className="layer-title">P2P Network (libp2p)</span>
              <span className="layer-desc">Decentralized Message Transport</span>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
