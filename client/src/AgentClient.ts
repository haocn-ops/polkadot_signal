import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import type { AccountId, Hash } from '@polkadot/types/interfaces';
import type { Option, Bytes } from '@polkadot/types';

export interface AgentProfile {
  agentId: string;
  name: string;
  description: string;
  agentType: AgentType;
  version: string;
  capabilities: Capability[];
  supportedProtocols: string[];
  endpoint: string;
  maxConcurrentTasks: number;
  reliabilityScore: number;
  totalTasksCompleted: number;
  totalTasksFailed: number;
  averageResponseTimeMs: number;
  status: AgentStatus;
  owner: string;
  createdAt: number;
  updatedAt: number;
  lastHeartbeat?: number;
}

export interface Capability {
  name: string;
  category: string;
  description: string;
  version: string;
  tags: string[];
  costUnits: number;
  costAmount: number;
}

export enum AgentType {
  LLM = 'LLM',
  Tool = 'Tool',
  Orchestrator = 'Orchestrator',
  Worker = 'Worker',
  Coordinator = 'Coordinator',
  Custom = 'Custom',
}

export enum AgentStatus {
  Active = 'Active',
  Idle = 'Idle',
  Busy = 'Busy',
  Maintenance = 'Maintenance',
  Offline = 'Offline',
}

export interface Task {
  taskId: string;
  taskType: string;
  priority: TaskPriority;
  input: string;
  output?: string;
  error?: string;
  requester: string;
  assignedAgent?: string;
  requiredCapabilities: string[];
  dependencies: string[];
  status: TaskStatus;
  createdAt: number;
  startedAt?: number;
  completedAt?: number;
  deadline?: number;
  retryCount: number;
  maxRetries: number;
  executionTimeMs?: number;
  parentTaskId?: string;
}

export enum TaskPriority {
  Low = 'Low',
  Normal = 'Normal',
  High = 'High',
  Critical = 'Critical',
}

export enum TaskStatus {
  Pending = 'Pending',
  Queued = 'Queued',
  Assigned = 'Assigned',
  Running = 'Running',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
  Timeout = 'Timeout',
}

export interface TaskResult {
  taskId: string;
  output: string;
  success: boolean;
  executionTimeMs: number;
  completedAt: number;
  agent: string;
}

export class AgentRegistryClient {
  private api: ApiPromise;
  private keyring: Keyring;

  private constructor(api: ApiPromise) {
    this.api = api;
    this.keyring = new Keyring({ type: 'sr25519' });
  }

  static async connect(wsEndpoint: string = 'ws://localhost:9944'): Promise<AgentRegistryClient> {
    const provider = new WsProvider(wsEndpoint);
    const api = await ApiPromise.create({ provider });
    return new AgentRegistryClient(api);
  }

  async disconnect(): Promise<void> {
    await this.api.disconnect();
  }

  async registerAgent(
    account: ReturnType<typeof this.keyring.addFromUri>,
    agentId: string,
    name: string,
    description: string,
    agentType: AgentType,
    version: string,
    capabilities: Array<{
      name: string;
      category: string;
      description: string;
      version: string;
      tags: string[];
      costUnits: number;
      costAmount: number;
    }>,
    supportedProtocols: string[],
    endpoint: string,
    maxConcurrentTasks: number
  ): Promise<Hash> {
    const caps = capabilities.map(cap => [
      Array.from(new TextEncoder().encode(cap.name)),
      Array.from(new TextEncoder().encode(cap.category)),
      Array.from(new TextEncoder().encode(cap.description)),
      Array.from(new TextEncoder().encode(cap.version)),
      cap.tags.map(t => Array.from(new TextEncoder().encode(t))),
      cap.costUnits,
      cap.costAmount,
    ]);

    const protocols = supportedProtocols.map(p => 
      Array.from(new TextEncoder().encode(p))
    );

    const tx = this.api.tx.agentRegistry.registerAgent(
      Array.from(new TextEncoder().encode(agentId)),
      Array.from(new TextEncoder().encode(name)),
      Array.from(new TextEncoder().encode(description)),
      agentType,
      Array.from(new TextEncoder().encode(version)),
      caps,
      protocols,
      Array.from(new TextEncoder().encode(endpoint)),
      maxConcurrentTasks
    );

    return this.signAndSend(account, tx);
  }

  async updateStatus(
    account: ReturnType<typeof this.keyring.addFromUri>,
    status: AgentStatus
  ): Promise<Hash> {
    const tx = this.api.tx.agentRegistry.updateStatus(status);
    return this.signAndSend(account, tx);
  }

  async heartbeat(
    account: ReturnType<typeof this.keyring.addFromUri>
  ): Promise<Hash> {
    const tx = this.api.tx.agentRegistry.heartbeat();
    return this.signAndSend(account, tx);
  }

  async updateCapabilities(
    account: ReturnType<typeof this.keyring.addFromUri>,
    capabilities: Array<{
      name: string;
      category: string;
      description: string;
      version: string;
      tags: string[];
      costUnits: number;
      costAmount: number;
    }>
  ): Promise<Hash> {
    const caps = capabilities.map(cap => [
      Array.from(new TextEncoder().encode(cap.name)),
      Array.from(new TextEncoder().encode(cap.category)),
      Array.from(new TextEncoder().encode(cap.description)),
      Array.from(new TextEncoder().encode(cap.version)),
      cap.tags.map(t => Array.from(new TextEncoder().encode(t))),
      cap.costUnits,
      cap.costAmount,
    ]);

    const tx = this.api.tx.agentRegistry.updateCapabilities(caps);
    return this.signAndSend(account, tx);
  }

  async updateStats(
    account: ReturnType<typeof this.keyring.addFromUri>,
    tasksCompleted?: number,
    tasksFailed?: number,
    responseTimeMs?: number
  ): Promise<Hash> {
    const tx = this.api.tx.agentRegistry.updateStats(
      tasksCompleted ? [tasksCompleted] : [],
      tasksFailed ? [tasksFailed] : [],
      responseTimeMs ? [responseTimeMs] : []
    );
    return this.signAndSend(account, tx);
  }

  async deregisterAgent(
    account: ReturnType<typeof this.keyring.addFromUri>
  ): Promise<Hash> {
    const tx = this.api.tx.agentRegistry.deregisterAgent();
    return this.signAndSend(account, tx);
  }

  async getAgent(accountId: string | AccountId): Promise<AgentProfile | null> {
    const result = await this.api.query.agentRegistry.agents(accountId);
    
    if (result.isEmpty || result.toPrimitive() === null) {
      return null;
    }

    const profile: any = result.toJSON();
    return {
      agentId: new TextDecoder().decode(new Uint8Array(profile.agentId)),
      name: new TextDecoder().decode(new Uint8Array(profile.name)),
      description: new TextDecoder().decode(new Uint8Array(profile.description)),
      agentType: profile.agentType as AgentType,
      version: new TextDecoder().decode(new Uint8Array(profile.version)),
      capabilities: profile.capabilities.map((cap: any) => ({
        name: new TextDecoder().decode(new Uint8Array(cap.name)),
        category: new TextDecoder().decode(new Uint8Array(cap.category)),
        description: new TextDecoder().decode(new Uint8Array(cap.description)),
        version: new TextDecoder().decode(new Uint8Array(cap.version)),
        tags: cap.tags.map((t: any) => new TextDecoder().decode(new Uint8Array(t))),
        costUnits: cap.costUnits,
        costAmount: cap.costAmount,
      })),
      supportedProtocols: profile.supportedProtocols.map((p: any) => 
        new TextDecoder().decode(new Uint8Array(p))
      ),
      endpoint: new TextDecoder().decode(new Uint8Array(profile.endpoint)),
      maxConcurrentTasks: profile.maxConcurrentTasks,
      reliabilityScore: profile.reliabilityScore,
      totalTasksCompleted: profile.totalTasksCompleted,
      totalTasksFailed: profile.totalTasksFailed,
      averageResponseTimeMs: profile.averageResponseTimeMs,
      status: profile.status as AgentStatus,
      owner: profile.owner,
      createdAt: profile.createdAt,
      updatedAt: profile.updatedAt,
      lastHeartbeat: profile.lastHeartbeat,
    };
  }

  async getAgentsByType(agentType: AgentType): Promise<string[]> {
    const result = await this.api.query.agentRegistry.agentsByType(agentType);
    return result.toJSON() as string[];
  }

  async getTotalAgents(): Promise<number> {
    const result = await this.api.query.agentRegistry.agentCounter();
    return Number(result.toString());
  }

  createAccountFromUri(uri: string): ReturnType<typeof this.keyring.addFromUri> {
    return this.keyring.addFromUri(uri);
  }

  getApi(): ApiPromise {
    return this.api;
  }

  private async signAndSend(
    account: ReturnType<typeof this.keyring.addFromUri>,
    tx: any
  ): Promise<Hash> {
    return new Promise((resolve, reject) => {
      tx.signAndSend(account, ({ status, dispatchError }) => {
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = this.api.registry.findMetaError(dispatchError.asModule);
            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`));
          } else {
            reject(new Error(dispatchError.toString()));
          }
        } else if (status.isInBlock) {
          resolve(status.asInBlock);
        }
      });
    });
  }
}

export class TaskQueueClient {
  private api: ApiPromise;
  private keyring: Keyring;

  private constructor(api: ApiPromise) {
    this.api = api;
    this.keyring = new Keyring({ type: 'sr25519' });
  }

  static async connect(wsEndpoint: string = 'ws://localhost:9944'): Promise<TaskQueueClient> {
    const provider = new WsProvider(wsEndpoint);
    const api = await ApiPromise.create({ provider });
    return new TaskQueueClient(api);
  }

  async disconnect(): Promise<void> {
    await this.api.disconnect();
  }

  async createTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskType: string,
    priority: TaskPriority,
    input: string,
    requiredCapabilities: string[] = [],
    dependencies: string[] = [],
    deadline?: number,
    maxRetries: number = 3
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.createTask(
      Array.from(new TextEncoder().encode(taskType)),
      priority,
      Array.from(new TextEncoder().encode(input)),
      requiredCapabilities.map(c => Array.from(new TextEncoder().encode(c))),
      dependencies.map(d => Array.from(new TextEncoder().encode(d))),
      deadline ? [deadline] : [],
      maxRetries
    );

    return this.signAndSend(account, tx);
  }

  async assignTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskId: string,
    agent: string
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.assignTask(
      Array.from(new TextEncoder().encode(taskId)),
      agent
    );
    return this.signAndSend(account, tx);
  }

  async startTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskId: string
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.startTask(
      Array.from(new TextEncoder().encode(taskId))
    );
    return this.signAndSend(account, tx);
  }

  async completeTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskId: string,
    output: string,
    executionTimeMs: number
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.completeTask(
      Array.from(new TextEncoder().encode(taskId)),
      Array.from(new TextEncoder().encode(output)),
      executionTimeMs
    );
    return this.signAndSend(account, tx);
  }

  async failTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskId: string,
    error: string
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.failTask(
      Array.from(new TextEncoder().encode(taskId)),
      Array.from(new TextEncoder().encode(error))
    );
    return this.signAndSend(account, tx);
  }

  async cancelTask(
    account: ReturnType<typeof this.keyring.addFromUri>,
    taskId: string
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.cancelTask(
      Array.from(new TextEncoder().encode(taskId))
    );
    return this.signAndSend(account, tx);
  }

  async processQueue(
    account: ReturnType<typeof this.keyring.addFromUri>,
    maxTasks: number
  ): Promise<Hash> {
    const tx = this.api.tx.taskQueue.processQueue(maxTasks);
    return this.signAndSend(account, tx);
  }

  async getTask(taskId: string): Promise<Task | null> {
    const result = await this.api.query.taskQueue.tasks(
      Array.from(new TextEncoder().encode(taskId))
    );
    
    if (result.isEmpty || result.toPrimitive() === null) {
      return null;
    }

    const task: any = result.toJSON();
    return {
      taskId: new TextDecoder().decode(new Uint8Array(task.taskId)),
      taskType: new TextDecoder().decode(new Uint8Array(task.taskType)),
      priority: task.priority as TaskPriority,
      input: new TextDecoder().decode(new Uint8Array(task.input)),
      output: task.output ? new TextDecoder().decode(new Uint8Array(task.output)) : undefined,
      error: task.error ? new TextDecoder().decode(new Uint8Array(task.error)) : undefined,
      requester: task.requester,
      assignedAgent: task.assignedAgent,
      requiredCapabilities: task.requiredCapabilities.map((c: any) => 
        new TextDecoder().decode(new Uint8Array(c))
      ),
      dependencies: task.dependencies.map((d: any) => 
        new TextDecoder().decode(new Uint8Array(d))
      ),
      status: task.status as TaskStatus,
      createdAt: task.createdAt,
      startedAt: task.startedAt,
      completedAt: task.completedAt,
      deadline: task.deadline,
      retryCount: task.retryCount,
      maxRetries: task.maxRetries,
      executionTimeMs: task.executionTimeMs,
      parentTaskId: task.parentTaskId,
    };
  }

  async getTasksByRequester(requester: string): Promise<string[]> {
    const entries = await this.api.query.taskQueue.tasksByRequester.entries(requester);
    return entries
      .filter(([_, value]) => value.toPrimitive() === true)
      .map(([key, _]) => new TextDecoder().decode(new Uint8Array(key.args[1].toJSON())));
  }

  async getTasksByAgent(agent: string): Promise<string[]> {
    const entries = await this.api.query.taskQueue.tasksByAgent.entries(agent);
    return entries
      .filter(([_, value]) => value.toPrimitive() === true)
      .map(([key, _]) => new TextDecoder().decode(new Uint8Array(key.args[1].toJSON())));
  }

  async getPendingTasks(): Promise<string[]> {
    const result = await this.api.query.taskQueue.pendingQueue();
    return result.toJSON().map((id: any) => 
      new TextDecoder().decode(new Uint8Array(id))
    );
  }

  async getTaskResult(taskId: string): Promise<TaskResult | null> {
    const result = await this.api.query.taskQueue.taskResults(
      Array.from(new TextEncoder().encode(taskId))
    );
    
    if (result.isEmpty || result.toPrimitive() === null) {
      return null;
    }

    const taskResult: any = result.toJSON();
    return {
      taskId: new TextDecoder().decode(new Uint8Array(taskResult.taskId)),
      output: new TextDecoder().decode(new Uint8Array(taskResult.output)),
      success: taskResult.success,
      executionTimeMs: taskResult.executionTimeMs,
      completedAt: taskResult.completedAt,
      agent: taskResult.agent,
    };
  }

  async getQueueSize(): Promise<number> {
    const result = await this.api.query.taskQueue.pendingQueue();
    return result.toJSON().length;
  }

  async getActiveTaskCount(agent: string): Promise<number> {
    const result = await this.api.query.taskQueue.agentActiveTasks(agent);
    return Number(result.toString());
  }

  createAccountFromUri(uri: string): ReturnType<typeof this.keyring.addFromUri> {
    return this.keyring.addFromUri(uri);
  }

  getApi(): ApiPromise {
    return this.api;
  }

  private async signAndSend(
    account: ReturnType<typeof this.keyring.addFromUri>,
    tx: any
  ): Promise<Hash> {
    return new Promise((resolve, reject) => {
      tx.signAndSend(account, ({ status, dispatchError }) => {
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = this.api.registry.findMetaError(dispatchError.asModule);
            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`));
          } else {
            reject(new Error(dispatchError.toString()));
          }
        } else if (status.isInBlock) {
          resolve(status.asInBlock);
        }
      });
    });
  }
}

export default { AgentRegistryClient, TaskQueueClient };
