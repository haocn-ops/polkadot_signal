import React, { useState, useEffect } from 'react';
import './App.css';
import './components/AgentComponents.css';

const nacl = require('tweetnacl');
const { encodeUTF8, decodeUTF8, encodeBase64, decodeBase64 } = require('tweetnacl-util');

function App() {
  const [activeTab, setActiveTab] = useState('signal');
  const [connected, setConnected] = useState(false);
  const [message, setMessage] = useState('Hello, this is a secret message!');
  const [encryptedMessage, setEncryptedMessage] = useState(null);
  const [decryptedMessage, setDecryptedMessage] = useState('');
  const [logs, setLogs] = useState([]);
  const [aliceKeys, setAliceKeys] = useState(null);
  const [bobKeys, setBobKeys] = useState(null);
  const [aliceRegistered, setAliceRegistered] = useState(false);
  const [bobRegistered, setBobRegistered] = useState(false);

  const [agents, setAgents] = useState([]);
  const [tasks, setTasks] = useState([]);
  const [selectedAgent, setSelectedAgent] = useState(null);

  const addLog = (msg) => {
    setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  };

  useEffect(() => {
    addLog('🔐 Polkadot Signal Demo Started');
    addLog('💡 This is a client-side demo of Signal Protocol encryption');
    addLog('');
    addLog('Click "Generate Alice Keys" and "Generate Bob Keys" to begin');
    
    const demoAgents = [
      {
        agentId: 'llm-agent-001',
        name: 'GPT-4 Agent',
        description: 'A powerful language model agent for text generation and analysis',
        agentType: 'LLM',
        version: '1.0.0',
        capabilities: [
          { name: 'text_generation', category: 'nlp', description: 'Generate text based on prompts' },
          { name: 'text_analysis', category: 'nlp', description: 'Analyze text for sentiment and entities' },
        ],
        endpoint: '/ip4/127.0.0.1/tcp/4001',
        maxConcurrentTasks: 5,
        reliabilityScore: 980,
        totalTasksCompleted: 156,
        totalTasksFailed: 3,
        averageResponseTimeMs: 1200,
        status: 'Active',
      },
      {
        agentId: 'tool-agent-001',
        name: 'Search Tool',
        description: 'Web search and code execution capabilities',
        agentType: 'Tool',
        version: '1.0.0',
        capabilities: [
          { name: 'web_search', category: 'search', description: 'Search the web for information' },
          { name: 'code_execution', category: 'compute', description: 'Execute code in sandbox' },
        ],
        endpoint: '/ip4/127.0.0.1/tcp/4002',
        maxConcurrentTasks: 10,
        reliabilityScore: 950,
        totalTasksCompleted: 89,
        totalTasksFailed: 5,
        averageResponseTimeMs: 800,
        status: 'Idle',
      },
      {
        agentId: 'orchestrator-001',
        name: 'Task Orchestrator',
        description: 'Coordinates multiple agents for complex workflows',
        agentType: 'Orchestrator',
        version: '1.0.0',
        capabilities: [
          { name: 'task_orchestration', category: 'workflow', description: 'Orchestrate multi-agent tasks' },
          { name: 'workflow_planning', category: 'planning', description: 'Plan and optimize workflows' },
        ],
        endpoint: '/ip4/127.0.0.1/tcp/4003',
        maxConcurrentTasks: 3,
        reliabilityScore: 990,
        totalTasksCompleted: 45,
        totalTasksFailed: 0,
        averageResponseTimeMs: 500,
        status: 'Active',
      },
    ];
    setAgents(demoAgents);

    const demoTasks = [
      {
        taskId: 'task-001-abc123',
        taskType: 'text_generation',
        priority: 'High',
        input: JSON.stringify({ prompt: 'Explain quantum computing' }),
        requiredCapabilities: ['text_generation'],
        status: 'Completed',
        requester: 'Alice',
        assignedAgent: 'llm-agent-001',
        output: JSON.stringify({ result: 'Quantum computing uses quantum bits...' }),
        executionTimeMs: 1500,
      },
      {
        taskId: 'task-002-def456',
        taskType: 'web_search',
        priority: 'Normal',
        input: JSON.stringify({ query: 'latest AI research' }),
        requiredCapabilities: ['web_search'],
        status: 'Running',
        requester: 'Bob',
        assignedAgent: 'tool-agent-001',
      },
      {
        taskId: 'task-003-ghi789',
        taskType: 'data_analysis',
        priority: 'Low',
        input: JSON.stringify({ dataset: 'sales_2024.csv' }),
        requiredCapabilities: ['data_analysis'],
        status: 'Pending',
        requester: 'Alice',
      },
    ];
    setTasks(demoTasks);
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
    
    setAliceKeys({ identityKey, signedPrekey, oneTimePrekeys });
    
    addLog('✅ Alice keys generated!');
    addLog(`   Identity Public Key: ${identityKey.publicKey.substring(0, 20)}...`);
    setAliceRegistered(true);
  };

  const generateBobKeys = () => {
    addLog('📝 Generating Bob identity keys...');
    const identityKey = generateKeyPair();
    const signedPrekey = generateKeyPair();
    const oneTimePrekeys = [generateKeyPair(), generateKeyPair(), generateKeyPair()];
    
    setBobKeys({ identityKey, signedPrekey, oneTimePrekeys });
    
    addLog('✅ Bob keys generated!');
    addLog(`   Identity Public Key: ${identityKey.publicKey.substring(0, 20)}...`);
    setBobRegistered(true);
  };

  const simulateX3DH = () => {
    addLog('');
    addLog('🔄 Simulating X3DH Key Exchange...');
    addLog('   DH1 = DH(IK_A, SPK_B)');
    addLog('   DH2 = DH(EK_A, IK_B)');
    addLog('   DH3 = DH(EK_A, SPK_B)');
    addLog('   DH4 = DH(EK_A, OPK_B)');
    addLog('✅ X3DH Key Exchange Complete!');
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
        addLog('❌ Decryption failed!');
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

  const handleRegisterAgent = (agentData) => {
    const newAgent = {
      ...agentData,
      reliabilityScore: 1000,
      totalTasksCompleted: 0,
      totalTasksFailed: 0,
      averageResponseTimeMs: 0,
      status: 'Active',
    };
    setAgents([...agents, newAgent]);
    addLog(`✅ Agent "${agentData.name}" registered successfully!`);
  };

  const handleCreateTask = (taskData) => {
    const newTask = {
      taskId: `task-${Date.now().toString(36)}`,
      ...taskData,
      status: 'Pending',
      requester: 'User',
    };
    setTasks([newTask, ...tasks]);
    addLog(`✅ Task "${taskData.taskType}" created successfully!`);
  };

  const handleAssignTask = (taskId, agentId) => {
    setTasks(tasks.map(task => 
      task.taskId === taskId 
        ? { ...task, status: 'Assigned', assignedAgent: agentId }
        : task
    ));
    addLog(`✅ Task assigned to agent: ${agentId}`);
  };

  const handleCompleteTask = (taskId) => {
    setTasks(tasks.map(task => 
      task.taskId === taskId 
        ? { 
            ...task, 
            status: 'Completed', 
            output: JSON.stringify({ result: 'Task completed successfully' }),
            executionTimeMs: Math.floor(Math.random() * 2000) + 500
          }
        : task
    ));
    addLog(`✅ Task ${taskId} marked as completed!`);
  };

  const handleQueryCapability = (agentId) => {
    setActiveTab('tasks');
    addLog(`📤 Preparing task request for agent: ${agentId}`);
  };

  const clearLogs = () => {
    setLogs([]);
    addLog('📋 Logs cleared');
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>🔐 Polkadot Signal</h1>
        <p className="subtitle">Decentralized End-to-End Encrypted Messenger with AI Agent Support</p>
        
        <div className="tab-navigation">
          <button 
            className={`tab-btn ${activeTab === 'signal' ? 'active' : ''}`}
            onClick={() => setActiveTab('signal')}
          >
            🔐 Signal Protocol
          </button>
          <button 
            className={`tab-btn ${activeTab === 'agents' ? 'active' : ''}`}
            onClick={() => setActiveTab('agents')}
          >
            🤖 Agent Registry
          </button>
          <button 
            className={`tab-btn ${activeTab === 'tasks' ? 'active' : ''}`}
            onClick={() => setActiveTab('tasks')}
          >
            📋 Task Queue
          </button>
          <button 
            className={`tab-btn ${activeTab === 'discovery' ? 'active' : ''}`}
            onClick={() => setActiveTab('discovery')}
          >
            🔍 Discovery
          </button>
        </div>
      </header>

      <main className="main-content">
        {activeTab === 'signal' && (
          <>
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

            <div className="cards-container">
              <div className="card">
                <h2>🔑 Key Generation</h2>
                <p style={{color: '#888', marginBottom: '15px'}}>
                  Generate Signal Protocol identity keys
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
                    <p><strong>🔒 Encrypted:</strong></p>
                    <code>{encryptedMessage.ciphertext.substring(0, 60)}...</code>
                  </div>
                )}
              </div>

              <div className="card">
                <h2>📬 Message Decryption</h2>
                <button onClick={decryptMessage} disabled={!encryptedMessage}>
                  🔓 Decrypt Message
                </button>
                {decryptedMessage && (
                  <div className="decrypted-output">
                    <p><strong>✅ Decrypted:</strong></p>
                    <div className="message-box">{decryptedMessage}</div>
                  </div>
                )}
              </div>
            </div>

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
          </>
        )}

        {activeTab === 'agents' && (
          <div className="agent-panel">
            <div className="panel-header">
              <h2>🤖 Agent Registry</h2>
            </div>
            
            <div className="agents-list">
              <div className="agent-cards">
                {agents.map((agent) => (
                  <div 
                    key={agent.agentId} 
                    className="agent-card"
                    onClick={() => setSelectedAgent(selectedAgent === agent.agentId ? null : agent.agentId)}
                  >
                    <div className="agent-header">
                      <div className="agent-info">
                        <h4>{agent.name}</h4>
                        <span className="agent-id">{agent.agentId}</span>
                      </div>
                      <span className="agent-status" style={{ color: agent.status === 'Active' ? '#4CAF50' : '#FFC107' }}>
                        {agent.status === 'Active' ? '🟢' : '🟡'} {agent.status}
                      </span>
                    </div>
                    <p className="agent-description">{agent.description}</p>
                    <div className="agent-capabilities">
                      {agent.capabilities?.slice(0, 3).map((cap, i) => (
                        <span key={i} className="capability-tag">{cap.name}</span>
                      ))}
                    </div>
                    <div className="agent-stats">
                      <span>📊 {(agent.reliabilityScore / 10).toFixed(1)}%</span>
                      <span>✅ {agent.totalTasksCompleted} tasks</span>
                      <span>⚡ Max: {agent.maxConcurrentTasks}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'tasks' && (
          <div className="task-queue-panel">
            <div className="panel-header">
              <h2>📋 Task Queue</h2>
            </div>
            
            <div className="task-stats">
              <div className="stat-card pending">
                <span className="stat-icon">⏳</span>
                <span className="stat-value">{tasks.filter(t => t.status === 'Pending').length}</span>
                <span className="stat-label">Pending</span>
              </div>
              <div className="stat-card running">
                <span className="stat-icon">▶️</span>
                <span className="stat-value">{tasks.filter(t => t.status === 'Running' || t.status === 'Assigned').length}</span>
                <span className="stat-label">Running</span>
              </div>
              <div className="stat-card completed">
                <span className="stat-icon">✅</span>
                <span className="stat-value">{tasks.filter(t => t.status === 'Completed').length}</span>
                <span className="stat-label">Completed</span>
              </div>
            </div>

            <div className="tasks-list">
              <div className="task-cards">
                {tasks.map((task) => (
                  <div key={task.taskId} className={`task-card priority-${task.priority.toLowerCase()}`}>
                    <div className="task-header">
                      <div className="task-title">
                        <span className="task-type">{task.taskType}</span>
                        <span className="task-priority">{task.priority}</span>
                      </div>
                      <span className="task-status">{task.status}</span>
                    </div>
                    <div className="task-id">ID: {task.taskId}</div>
                    <div className="agent-capabilities">
                      {task.requiredCapabilities.map((cap, i) => (
                        <span key={i} className="capability-tag">{cap}</span>
                      ))}
                    </div>
                    {task.output && (
                      <pre className="code-block output" style={{ marginTop: '10px' }}>
                        {task.output}
                      </pre>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'discovery' && (
          <div className="agent-discovery-panel">
            <div className="panel-header">
              <h2>🔍 Agent Discovery</h2>
            </div>
            
            <div className="search-section">
              <input
                type="text"
                placeholder="Search agents by name, capability, or description..."
                className="search-input"
                style={{ width: '100%', marginBottom: '15px' }}
              />
            </div>

            <div className="discovery-content">
              <div className="agents-grid">
                {agents.map((agent) => (
                  <div key={agent.agentId} className="discovery-card">
                    <div className="card-header">
                      <h3>{agent.name}</h3>
                      <span className="agent-type">{agent.agentType}</span>
                    </div>
                    <p className="card-description">{agent.description}</p>
                    <div className="card-capabilities">
                      {agent.capabilities?.map((cap, i) => (
                        <span key={i} className="capability-badge">{cap.name}</span>
                      ))}
                    </div>
                    <div className="card-meta">
                      <span>📊 {(agent.reliabilityScore / 10).toFixed(1)}%</span>
                      <span>✅ {agent.totalTasksCompleted} tasks</span>
                    </div>
                  </div>
                ))}
              </div>

              <div className="capabilities-sidebar">
                <h3>📊 Statistics</h3>
                <div className="stats-list">
                  <div className="stat-item">
                    <span className="stat-label">Total Agents</span>
                    <span className="stat-value">{agents.length}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Active Agents</span>
                    <span className="stat-value">{agents.filter(a => a.status === 'Active').length}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Total Capabilities</span>
                    <span className="stat-value">
                      {agents.reduce((sum, a) => sum + (a.capabilities?.length || 0), 0)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        <div className="architecture-section">
          <h2>🏗️ Architecture</h2>
          <div className="arch-diagram">
            <div className="arch-layer">
              <span className="layer-title">Frontend (React)</span>
              <span className="layer-desc">User Interface - This Demo</span>
            </div>
            <div className="arch-arrow">↓</div>
            <div className="arch-layer agent-layer">
              <span className="layer-title">AI Agent Communication Layer</span>
              <span className="layer-desc">AgentMessage | TaskQueue | AgentRegistry | Router</span>
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
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
