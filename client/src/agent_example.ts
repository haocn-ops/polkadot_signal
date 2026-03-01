import { AgentRegistryClient, TaskQueueClient, AgentType, AgentStatus, TaskPriority } from './AgentClient';

async function main() {
  console.log('=== AI Agent Communication Demo ===\n');

  const registryClient = await AgentRegistryClient.connect('ws://localhost:9944');
  const taskClient = await TaskQueueClient.connect('ws://localhost:9944');

  try {
    const orchestrator = registryClient.createAccountFromUri('//Alice');
    const llmAgent = registryClient.createAccountFromUri('//Bob');
    const toolAgent = registryClient.createAccountFromUri('//Charlie');

    console.log('1. Registering LLM Agent...');
    await registryClient.registerAgent(
      llmAgent,
      'llm-agent-001',
      'GPT-4 Agent',
      'A powerful language model agent for text generation and analysis',
      AgentType.LLM,
      '1.0.0',
      [
        {
          name: 'text_generation',
          category: 'nlp',
          description: 'Generate text based on prompts',
          version: '1.0.0',
          tags: ['llm', 'generation', 'gpt'],
          costUnits: 1000,
          costAmount: 100,
        },
        {
          name: 'text_analysis',
          category: 'nlp',
          description: 'Analyze text for sentiment, entities, etc.',
          version: '1.0.0',
          tags: ['llm', 'analysis', 'nlp'],
          costUnits: 500,
          costAmount: 50,
        },
      ],
      ['agent-message/1.0', 'json-rpc/2.0'],
      '/ip4/127.0.0.1/tcp/4001',
      5
    );
    console.log('   LLM Agent registered!\n');

    console.log('2. Registering Tool Agent...');
    await registryClient.registerAgent(
      toolAgent,
      'tool-agent-001',
      'Search Tool Agent',
      'An agent that provides web search capabilities',
      AgentType.Tool,
      '1.0.0',
      [
        {
          name: 'web_search',
          category: 'search',
          description: 'Search the web for information',
          version: '1.0.0',
          tags: ['search', 'web', 'tool'],
          costUnits: 100,
          costAmount: 10,
        },
        {
          name: 'code_execution',
          category: 'compute',
          description: 'Execute code in a sandboxed environment',
          version: '1.0.0',
          tags: ['code', 'execution', 'sandbox'],
          costUnits: 2000,
          costAmount: 200,
        },
      ],
      ['agent-message/1.0', 'json-rpc/2.0'],
      '/ip4/127.0.0.1/tcp/4002',
      10
    );
    console.log('   Tool Agent registered!\n');

    console.log('3. Querying registered agents...');
    const llmProfile = await registryClient.getAgent(llmAgent.address);
    if (llmProfile) {
      console.log(`   LLM Agent: ${llmProfile.name}`);
      console.log(`   - Capabilities: ${llmProfile.capabilities.map(c => c.name).join(', ')}`);
      console.log(`   - Status: ${llmProfile.status}`);
    }

    const toolProfile = await registryClient.getAgent(toolAgent.address);
    if (toolProfile) {
      console.log(`   Tool Agent: ${toolProfile.name}`);
      console.log(`   - Capabilities: ${toolProfile.capabilities.map(c => c.name).join(', ')}`);
      console.log(`   - Status: ${toolProfile.status}`);
    }
    console.log();

    console.log('4. Creating a task...');
    const taskInput = JSON.stringify({
      prompt: 'Explain the concept of decentralized AI agents',
      max_tokens: 500,
    });
    
    await taskClient.createTask(
      orchestrator,
      'text_generation',
      TaskPriority.Normal,
      taskInput,
      ['text_generation'],
      [],
      undefined,
      3
    );
    console.log('   Task created!\n');

    console.log('5. Getting pending tasks...');
    const pendingTasks = await taskClient.getPendingTasks();
    console.log(`   Pending tasks: ${pendingTasks.length}`);
    console.log();

    console.log('6. Agent heartbeat...');
    await registryClient.heartbeat(llmAgent);
    console.log('   Heartbeat sent!\n');

    console.log('7. Updating agent status...');
    await registryClient.updateStatus(llmAgent, AgentStatus.Busy);
    console.log('   Status updated to Busy!\n');

    console.log('8. Getting agents by type...');
    const llmAgents = await registryClient.getAgentsByType(AgentType.LLM);
    console.log(`   LLM Agents: ${llmAgents.length}`);
    console.log();

    console.log('9. Getting total agent count...');
    const totalAgents = await registryClient.getTotalAgents();
    console.log(`   Total agents: ${totalAgents}`);
    console.log();

    console.log('=== Demo Complete ===');

  } catch (error) {
    console.error('Error:', error);
  } finally {
    await registryClient.disconnect();
    await taskClient.disconnect();
  }
}

main().catch(console.error);
