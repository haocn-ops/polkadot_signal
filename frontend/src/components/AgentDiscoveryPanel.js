import React, { useState } from 'react';

const CAPABILITY_CATEGORIES = [
  'nlp', 'search', 'compute', 'data', 'vision', 'audio', 'tool', 'other'
];

function AgentDiscoveryPanel({ agents, onQueryCapability }) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('');
  const [selectedAgent, setSelectedAgent] = useState(null);

  const allCapabilities = agents.reduce((acc, agent) => {
    agent.capabilities?.forEach(cap => {
      if (!acc.find(c => c.name === cap.name)) {
        acc.push(cap);
      }
    });
    return acc;
  }, []);

  const filteredAgents = agents.filter(agent => {
    const matchesSearch = !searchQuery || 
      agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      agent.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      agent.capabilities?.some(cap => 
        cap.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        cap.category.toLowerCase().includes(searchQuery.toLowerCase())
      );
    
    const matchesCategory = !selectedCategory ||
      agent.capabilities?.some(cap => cap.category === selectedCategory);
    
    return matchesSearch && matchesCategory;
  });

  const getAgentMatchScore = (agent, query) => {
    if (!query) return 0;
    let score = 0;
    if (agent.name.toLowerCase().includes(query.toLowerCase())) score += 3;
    if (agent.description.toLowerCase().includes(query.toLowerCase())) score += 2;
    agent.capabilities?.forEach(cap => {
      if (cap.name.toLowerCase().includes(query.toLowerCase())) score += 2;
      if (cap.category.toLowerCase().includes(query.toLowerCase())) score += 1;
    });
    return score;
  };

  const sortedAgents = searchQuery 
    ? [...filteredAgents].sort((a, b) => 
        getAgentMatchScore(b, searchQuery) - getAgentMatchScore(a, searchQuery)
      )
    : filteredAgents;

  return (
    <div className="agent-discovery-panel">
      <div className="panel-header">
        <h2>🔍 Agent Discovery</h2>
      </div>

      <div className="search-section">
        <div className="search-bar">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search agents by name, capability, or description..."
            className="search-input"
          />
          {searchQuery && (
            <button 
              className="btn-clear"
              onClick={() => setSearchQuery('')}
            >
              ✕
            </button>
          )}
        </div>

        <div className="category-filters">
          <button 
            className={`category-btn ${!selectedCategory ? 'active' : ''}`}
            onClick={() => setSelectedCategory('')}
          >
            All
          </button>
          {CAPABILITY_CATEGORIES.map(cat => (
            <button
              key={cat}
              className={`category-btn ${selectedCategory === cat ? 'active' : ''}`}
              onClick={() => setSelectedCategory(cat)}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      <div className="discovery-content">
        <div className="agents-grid">
          {sortedAgents.length === 0 ? (
            <div className="empty-state">
              <p>No agents found</p>
              <p className="hint">Try adjusting your search or filters</p>
            </div>
          ) : (
            sortedAgents.map((agent) => (
              <div 
                key={agent.agentId}
                className={`discovery-card ${selectedAgent === agent.agentId ? 'selected' : ''}`}
                onClick={() => setSelectedAgent(selectedAgent === agent.agentId ? null : agent.agentId)}
              >
                <div className="card-header">
                  <h3>{agent.name}</h3>
                  <span className="agent-type">{agent.agentType}</span>
                </div>
                
                <p className="card-description">{agent.description}</p>
                
                <div className="card-capabilities">
                  {agent.capabilities?.slice(0, 4).map((cap, i) => (
                    <span key={i} className="capability-badge" title={cap.description}>
                      {cap.name}
                    </span>
                  ))}
                  {agent.capabilities?.length > 4 && (
                    <span className="capability-badge more">
                      +{agent.capabilities.length - 4}
                    </span>
                  )}
                </div>

                <div className="card-meta">
                  <span className="reliability">
                    📊 {(agent.reliabilityScore / 10).toFixed(1)}%
                  </span>
                  <span className="tasks">
                    ✅ {agent.totalTasksCompleted} tasks
                  </span>
                  <span className="response-time">
                    ⚡ {agent.averageResponseTimeMs}ms
                  </span>
                </div>

                {selectedAgent === agent.agentId && (
                  <div className="card-expanded">
                    <h4>Capabilities</h4>
                    <div className="capabilities-list">
                      {agent.capabilities?.map((cap, i) => (
                        <div key={i} className="capability-detail">
                          <div className="cap-header">
                            <span className="cap-name">{cap.name}</span>
                            <span className="cap-category">{cap.category}</span>
                          </div>
                          <p className="cap-description">{cap.description}</p>
                          {cap.tags?.length > 0 && (
                            <div className="cap-tags">
                              {cap.tags.map((tag, j) => (
                                <span key={j} className="tag">{tag}</span>
                              ))}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>

                    <div className="agent-actions">
                      <button 
                        className="btn-action"
                        onClick={(e) => {
                          e.stopPropagation();
                          onQueryCapability(agent.agentId);
                        }}
                      >
                        📤 Send Task Request
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        <div className="capabilities-sidebar">
          <h3>🏷️ Available Capabilities</h3>
          <div className="capability-cloud">
            {allCapabilities.map((cap, i) => (
              <button
                key={i}
                className="cloud-tag"
                onClick={() => setSearchQuery(cap.name)}
              >
                {cap.name}
              </button>
            ))}
          </div>

          <h3>📊 Statistics</h3>
          <div className="stats-list">
            <div className="stat-item">
              <span className="stat-label">Total Agents</span>
              <span className="stat-value">{agents.length}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Active Agents</span>
              <span className="stat-value">
                {agents.filter(a => a.status === 'Active' || a.status === 'Idle').length}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Total Capabilities</span>
              <span className="stat-value">{allCapabilities.length}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Avg Reliability</span>
              <span className="stat-value">
                {agents.length > 0 
                  ? (agents.reduce((sum, a) => sum + a.reliabilityScore, 0) / agents.length / 10).toFixed(1)
                  : 0
                }%
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default AgentDiscoveryPanel;
