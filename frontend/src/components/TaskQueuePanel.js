import React, { useState } from 'react';

const TASK_PRIORITY = {
  Low: { color: '#4CAF50', icon: '🟢' },
  Normal: { color: '#2196F3', icon: '🔵' },
  High: { color: '#FF9800', icon: '🟠' },
  Critical: { color: '#F44336', icon: '🔴' },
};

const TASK_STATUS = {
  Pending: { color: '#9E9E9E', icon: '⏳' },
  Queued: { color: '#2196F3', icon: '📋' },
  Assigned: { color: '#FF9800', icon: '👤' },
  Running: { color: '#4CAF50', icon: '▶️' },
  Completed: { color: '#4CAF50', icon: '✅' },
  Failed: { color: '#F44336', icon: '❌' },
  Cancelled: { color: '#9E9E9E', icon: '🚫' },
  Timeout: { color: '#FF5722', icon: '⏰' },
};

function TaskQueuePanel({ onCreateTask, tasks, agents, onAssignTask, onCompleteTask }) {
  const [showForm, setShowForm] = useState(false);
  const [selectedTask, setSelectedTask] = useState(null);
  const [formData, setFormData] = useState({
    taskType: '',
    priority: 'Normal',
    input: '',
    requiredCapabilities: [],
    maxRetries: 3,
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onCreateTask(formData);
    setShowForm(false);
    setFormData({
      taskType: '',
      priority: 'Normal',
      input: '',
      requiredCapabilities: [],
      maxRetries: 3,
    });
  };

  const pendingTasks = tasks.filter(t => t.status === 'Pending' || t.status === 'Queued');
  const runningTasks = tasks.filter(t => t.status === 'Running' || t.status === 'Assigned');
  const completedTasks = tasks.filter(t => t.status === 'Completed' || t.status === 'Failed');

  return (
    <div className="task-queue-panel">
      <div className="panel-header">
        <h2>📋 Task Queue</h2>
        <button 
          className="btn-primary"
          onClick={() => setShowForm(!showForm)}
        >
          {showForm ? '✕ Cancel' : '+ Create Task'}
        </button>
      </div>

      {showForm && (
        <form className="task-form" onSubmit={handleSubmit}>
          <div className="form-row">
            <div className="form-group">
              <label>Task Type</label>
              <input
                type="text"
                value={formData.taskType}
                onChange={(e) => setFormData({ ...formData, taskType: e.target.value })}
                placeholder="e.g., text_generation, data_analysis"
                required
              />
            </div>
            <div className="form-group">
              <label>Priority</label>
              <select
                value={formData.priority}
                onChange={(e) => setFormData({ ...formData, priority: e.target.value })}
              >
                {Object.entries(TASK_PRIORITY).map(([key, { icon }]) => (
                  <option key={key} value={key}>
                    {icon} {key}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-group">
            <label>Input Data (JSON)</label>
            <textarea
              value={formData.input}
              onChange={(e) => setFormData({ ...formData, input: e.target.value })}
              placeholder='{"prompt": "Hello, world!"}'
              rows={4}
            />
          </div>

          <div className="form-group">
            <label>Required Capabilities</label>
            <input
              type="text"
              value={formData.requiredCapabilities.join(', ')}
              onChange={(e) => setFormData({ 
                ...formData, 
                requiredCapabilities: e.target.value.split(',').map(s => s.trim()).filter(Boolean)
              })}
              placeholder="text_generation, translation (comma separated)"
            />
          </div>

          <div className="form-group">
            <label>Max Retries</label>
            <input
              type="number"
              value={formData.maxRetries}
              onChange={(e) => setFormData({ ...formData, maxRetries: parseInt(e.target.value) })}
              min={0}
              max={10}
            />
          </div>

          <button type="submit" className="btn-submit">
            🚀 Create Task
          </button>
        </form>
      )}

      <div className="task-stats">
        <div className="stat-card pending">
          <span className="stat-icon">⏳</span>
          <span className="stat-value">{pendingTasks.length}</span>
          <span className="stat-label">Pending</span>
        </div>
        <div className="stat-card running">
          <span className="stat-icon">▶️</span>
          <span className="stat-value">{runningTasks.length}</span>
          <span className="stat-label">Running</span>
        </div>
        <div className="stat-card completed">
          <span className="stat-icon">✅</span>
          <span className="stat-value">{completedTasks.length}</span>
          <span className="stat-label">Completed</span>
        </div>
      </div>

      <div className="tasks-list">
        <h3>Tasks</h3>
        {tasks.length === 0 ? (
          <div className="empty-state">
            <p>No tasks in queue</p>
            <p className="hint">Click "Create Task" to add a new task</p>
          </div>
        ) : (
          <div className="task-cards">
            {tasks.map((task) => (
              <div 
                key={task.taskId} 
                className={`task-card priority-${task.priority.toLowerCase()}`}
                onClick={() => setSelectedTask(selectedTask === task.taskId ? null : task.taskId)}
              >
                <div className="task-header">
                  <div className="task-title">
                    <span className="task-type">{task.taskType}</span>
                    <span 
                      className="task-priority"
                      style={{ color: TASK_PRIORITY[task.priority]?.color }}
                    >
                      {TASK_PRIORITY[task.priority]?.icon} {task.priority}
                    </span>
                  </div>
                  <span 
                    className="task-status"
                    style={{ color: TASK_STATUS[task.status]?.color }}
                  >
                    {TASK_STATUS[task.status]?.icon} {task.status}
                  </span>
                </div>
                
                <div className="task-id">ID: {task.taskId.substring(0, 16)}...</div>
                
                {selectedTask === task.taskId && (
                  <div className="task-details">
                    <div className="detail-row">
                      <span className="label">Requester:</span>
                      <span className="value">{task.requester}</span>
                    </div>
                    {task.assignedAgent && (
                      <div className="detail-row">
                        <span className="label">Assigned Agent:</span>
                        <span className="value">{task.assignedAgent}</span>
                      </div>
                    )}
                    <div className="detail-row">
                      <span className="label">Capabilities:</span>
                      <div className="capability-tags">
                        {task.requiredCapabilities.map((cap, i) => (
                          <span key={i} className="capability-tag">{cap}</span>
                        ))}
                      </div>
                    </div>
                    {task.input && (
                      <div className="detail-row">
                        <span className="label">Input:</span>
                        <pre className="code-block">{task.input}</pre>
                      </div>
                    )}
                    {task.output && (
                      <div className="detail-row">
                        <span className="label">Output:</span>
                        <pre className="code-block output">{task.output}</pre>
                      </div>
                    )}
                    {task.error && (
                      <div className="detail-row error">
                        <span className="label">Error:</span>
                        <span className="error-text">{task.error}</span>
                      </div>
                    )}
                    <div className="task-actions">
                      {task.status === 'Pending' && (
                        <select 
                          onChange={(e) => e.target.value && onAssignTask(task.taskId, e.target.value)}
                          onClick={(e) => e.stopPropagation()}
                          defaultValue=""
                        >
                          <option value="" disabled>Assign to Agent...</option>
                          {agents.map((agent) => (
                            <option key={agent.agentId} value={agent.agentId}>
                              {agent.name}
                            </option>
                          ))}
                        </select>
                      )}
                      {task.status === 'Running' && (
                        <button 
                          className="btn-complete"
                          onClick={(e) => {
                            e.stopPropagation();
                            onCompleteTask(task.taskId);
                          }}
                        >
                          ✅ Mark Complete
                        </button>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default TaskQueuePanel;
