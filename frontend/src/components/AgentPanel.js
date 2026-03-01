import React, { useState, useEffect } from 'react';

const AGENT_TYPES = [
  { value: 'LLM', label: 'LLM Agent', icon: '🤖' },
  { value: 'Tool', label: 'Tool Agent', icon: '🔧' },
  { value: 'Orchestrator', label: 'Orchestrator', icon: '🎛️' },
  { value: 'Worker', label: 'Worker Agent', icon: '⚙️' },
  { value: 'Coordinator', label: 'Coordinator', icon: '📊' },
];

const AGENT_STATUS = {
  Active: { color: '#4CAF50', icon: '🟢' },
  Idle: { color: '#FFC107', icon: '🟡' },
  Busy: { color: '#FF9800', icon: '🟠' },
  Maintenance: { color: '#9E9E9E', icon: '⚪' },
  Offline: { color: '#F44336', icon: '🔴' },
};

function AgentPanel({ onRegisterAgent, agents, onSelectAgent }) {
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    agentId: '',
    name: '',
    description: '',
    agentType: 'LLM',
    version: '1.0.0',
    capabilities: [{ name: '', category: '', description: '' }],
    endpoint: '',
    maxConcurrentTasks: 5,
  });

  const handleCapabilityChange = (index, field, value) => {
    const newCapabilities = [...formData.capabilities];
    newCapabilities[index][field] = value;
    setFormData({ ...formData, capabilities: newCapabilities });
  };

  const addCapability = () => {
    setFormData({
      ...formData,
      capabilities: [...formData.capabilities, { name: '', category: '', description: '' }],
    });
  };

  const removeCapability = (index) => {
    const newCapabilities = formData.capabilities.filter((_, i) => i !== index);
    setFormData({ ...formData, capabilities: newCapabilities });
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onRegisterAgent(formData);
    setShowForm(false);
    setFormData({
      agentId: '',
      name: '',
      description: '',
      agentType: 'LLM',
      version: '1.0.0',
      capabilities: [{ name: '', category: '', description: '' }],
      endpoint: '',
      maxConcurrentTasks: 5,
    });
  };

  return (
    <div className="agent-panel">
      <div className="panel-header">
        <h2>🤖 Agent Registry</h2>
        <button 
          className="btn-primary"
          onClick={() => setShowForm(!showForm)}
        >
          {showForm ? '✕ Cancel' : '+ Register Agent'}
        </button>
      </div>

      {showForm && (
        <form className="agent-form" onSubmit={handleSubmit}>
          <div className="form-row">
            <div className="form-group">
              <label>Agent ID</label>
              <input
                type="text"
                value={formData.agentId}
                onChange={(e) => setFormData({ ...formData, agentId: e.target.value })}
                placeholder="agent-001"
                required
              />
            </div>
            <div className="form-group">
              <label>Name</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="My AI Agent"
                required
              />
            </div>
          </div>

          <div className="form-group">
            <label>Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Describe what this agent does..."
              rows={2}
            />
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Agent Type</label>
              <select
                value={formData.agentType}
                onChange={(e) => setFormData({ ...formData, agentType: e.target.value })}
              >
                {AGENT_TYPES.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.icon} {type.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>Version</label>
              <input
                type="text"
                value={formData.version}
                onChange={(e) => setFormData({ ...formData, version: e.target.value })}
                placeholder="1.0.0"
              />
            </div>
          </div>

          <div className="form-group">
            <label>Endpoint</label>
            <input
              type="text"
              value={formData.endpoint}
              onChange={(e) => setFormData({ ...formData, endpoint: e.target.value })}
              placeholder="/ip4/127.0.0.1/tcp/4001"
            />
          </div>

          <div className="form-group">
            <label>Max Concurrent Tasks</label>
            <input
              type="number"
              value={formData.maxConcurrentTasks}
              onChange={(e) => setFormData({ ...formData, maxConcurrentTasks: parseInt(e.target.value) })}
              min={1}
              max={100}
            />
          </div>

          <div className="capabilities-section">
            <div className="section-header">
              <label>Capabilities</label>
              <button type="button" className="btn-small" onClick={addCapability}>
                + Add Capability
              </button>
            </div>
            {formData.capabilities.map((cap, index) => (
              <div key={index} className="capability-item">
                <input
                  type="text"
                  value={cap.name}
                  onChange={(e) => handleCapabilityChange(index, 'name', e.target.value)}
                  placeholder="Capability name (e.g., text_generation)"
                />
                <input
                  type="text"
                  value={cap.category}
                  onChange={(e) => handleCapabilityChange(index, 'category', e.target.value)}
                  placeholder="Category (e.g., nlp)"
                />
                <input
                  type="text"
                  value={cap.description}
                  onChange={(e) => handleCapabilityChange(index, 'description', e.target.value)}
                  placeholder="Description"
                />
                {formData.capabilities.length > 1 && (
                  <button 
                    type="button" 
                    className="btn-remove"
                    onClick={() => removeCapability(index)}
                  >
                    ✕
                  </button>
                )}
              </div>
            ))}
          </div>

          <button type="submit" className="btn-submit">
            🚀 Register Agent
          </button>
        </form>
      )}

      <div className="agents-list">
        <h3>Registered Agents ({agents.length})</h3>
        {agents.length === 0 ? (
          <div className="empty-state">
            <p>No agents registered yet</p>
            <p className="hint">Click "Register Agent" to add your first AI agent</p>
          </div>
        ) : (
          <div className="agent-cards">
            {agents.map((agent) => (
              <div 
                key={agent.agentId} 
                className="agent-card"
                onClick={() => onSelectAgent(agent)}
              >
                <div className="agent-header">
                  <span className="agent-type-icon">
                    {AGENT_TYPES.find(t => t.value === agent.agentType)?.icon || '🤖'}
                  </span>
                  <div className="agent-info">
                    <h4>{agent.name}</h4>
                    <span className="agent-id">{agent.agentId}</span>
                  </div>
                  <span 
                    className="agent-status"
                    style={{ color: AGENT_STATUS[agent.status]?.color }}
                  >
                    {AGENT_STATUS[agent.status]?.icon || '⚪'} {agent.status}
                  </span>
                </div>
                <p className="agent-description">{agent.description}</p>
                <div className="agent-capabilities">
                  {agent.capabilities?.slice(0, 3).map((cap, i) => (
                    <span key={i} className="capability-tag">{cap.name}</span>
                  ))}
                  {agent.capabilities?.length > 3 && (
                    <span className="capability-tag">+{agent.capabilities.length - 3} more</span>
                  )}
                </div>
                <div className="agent-stats">
                  <span>📊 Reliability: {(agent.reliabilityScore / 10).toFixed(1)}%</span>
                  <span>✅ Tasks: {agent.totalTasksCompleted}</span>
                  <span>⚡ Max: {agent.maxConcurrentTasks}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default AgentPanel;
