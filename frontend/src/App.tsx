import { useState, useEffect, useRef } from 'react';
import {
  Terminal,
  Cpu,
  Settings,
  Send,
  Zap,
  RefreshCw,
  Play,
  HardDrive,
  Activity,
  Wrench,
  Bot
} from 'lucide-react';

interface ChatMessage {
  role: string;
  content: string;
  tool_calls?: any[];
  tool_call_id?: string;
  name?: string;
}

interface ToolResult {
  call_id: string;
  tool_name: string;
  success: boolean;
  output: string;
}

interface ChatResponse {
  assistant_reply: string;
  updated_history: ChatMessage[];
  executed_tools: ToolResult[];
}

interface Config {
  server_host: string;
  server_port: number;
  default_provider: string;
  default_model: string;
  openai_api_key?: string;
  openai_base_url: string;
  anthropic_api_key?: string;
  openrouter_api_key?: string;
  ollama_base_url: string;
  system_prompt: string;
}

interface ModelInfo {
  provider: string;
  id: string;
  name: string;
}

interface ToolDef {
  name: String;
  description: String;
  parameters: any;
}

export default function App() {
  const [activeTab, setActiveTab] = useState<'chat' | 'system' | 'tools' | 'settings'>('chat');
  const [inputMessage, setInputMessage] = useState('');
  const [history, setHistory] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);

  const [provider, setProvider] = useState('openai');
  const [model, setModel] = useState('gpt-4o');
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [availableTools, setAvailableTools] = useState<ToolDef[]>([]);
  const [config, setConfig] = useState<Config | null>(null);

  const [sysInfoOutput, setSysInfoOutput] = useState<string>('');
  const [processOutput, setProcessOutput] = useState<string>('');
  const [quickCmd, setQuickCmd] = useState<string>('');
  const [cmdOutput, setCmdOutput] = useState<string>('');

  const chatBottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchConfig();
    fetchModels();
    fetchTools();
  }, []);

  useEffect(() => {
    chatBottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [history, loading]);

  const fetchConfig = async () => {
    try {
      const res = await fetch('/api/config');
      if (res.ok) {
        const data: Config = await res.json();
        setConfig(data);
        if (data.default_provider) setProvider(data.default_provider);
        if (data.default_model) setModel(data.default_model);
      }
    } catch (err) {
      console.error('Failed to fetch config', err);
    }
  };

  const fetchModels = async () => {
    try {
      const res = await fetch('/api/models');
      if (res.ok) {
        const data = await res.json();
        setAvailableModels(data);
      }
    } catch (err) {
      console.error('Failed to fetch models', err);
    }
  };

  const fetchTools = async () => {
    try {
      const res = await fetch('/api/tools');
      if (res.ok) {
        const data = await res.json();
        setAvailableTools(data);
      }
    } catch (err) {
      console.error('Failed to fetch tools', err);
    }
  };

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || loading) return;

    const userText = inputMessage.trim();
    setInputMessage('');
    setLoading(true);

    const newHistory = [
      ...history,
      { role: 'user', content: userText }
    ];
    setHistory(newHistory);

    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          model,
          user_message: userText,
          history,
          max_tool_iterations: 5,
        }),
      });

      if (res.ok) {
        const data: ChatResponse = await res.json();
        setHistory(data.updated_history);
      } else {
        const errText = await res.text();
        setHistory([
          ...newHistory,
          { role: 'assistant', content: `⚠️ Error executing request: ${errText}` }
        ]);
      }
    } catch (err: any) {
      setHistory([
        ...newHistory,
        { role: 'assistant', content: `⚠️ Network error: ${err.message}` }
      ]);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    if (!config) return;
    try {
      const res = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      if (res.ok) {
        alert('Configuration saved successfully!');
        fetchModels();
      }
    } catch (err) {
      alert('Failed to save configuration');
    }
  };

  const runSystemInfo = async () => {
    setSysInfoOutput('Loading system metrics...');
    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          model,
          user_message: 'Please inspect the system hardware and status using system_info tool.',
          history: [],
        }),
      });
      const data: ChatResponse = await res.json();
      setSysInfoOutput(data.assistant_reply);
    } catch (err: any) {
      setSysInfoOutput(`Failed: ${err.message}`);
    }
  };

  const runProcessList = async () => {
    setProcessOutput('Fetching running processes...');
    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          model,
          user_message: 'List running processes using list_processes tool.',
          history: [],
        }),
      });
      const data: ChatResponse = await res.json();
      setProcessOutput(data.assistant_reply);
    } catch (err: any) {
      setProcessOutput(`Failed: ${err.message}`);
    }
  };

  const runQuickCommand = async () => {
    if (!quickCmd.trim()) return;
    setCmdOutput('Executing shell command directly...');
    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          model,
          user_message: `Execute shell command: ${quickCmd}`,
          history: [],
        }),
      });
      const data: ChatResponse = await res.json();
      setCmdOutput(data.assistant_reply);
    } catch (err: any) {
      setCmdOutput(`Failed: ${err.message}`);
    }
  };

  return (
    <div className="flex h-screen bg-[#0b0f19] text-gray-100 font-sans">
      {/* Sidebar */}
      <div className="w-64 bg-[#111827] border-r border-[#1f293d] flex flex-col">
        <div className="p-4 border-b border-[#1f293d] flex items-center gap-3">
          <div className="p-2 bg-indigo-600 rounded-lg shadow-lg shadow-indigo-600/30">
            <Zap className="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 className="font-bold text-lg tracking-wide bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
              HERMES-CLAW
            </h1>
            <p className="text-xs text-gray-400">Unrestricted Agent & Bridge</p>
          </div>
        </div>

        <nav className="p-3 flex-1 space-y-1">
          <button
            onClick={() => setActiveTab('chat')}
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
              activeTab === 'chat'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                : 'text-gray-400 hover:bg-[#1a2333] hover:text-gray-200'
            }`}
          >
            <Bot className="w-4 h-4" />
            Agent Chat
          </button>

          <button
            onClick={() => setActiveTab('system')}
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
              activeTab === 'system'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                : 'text-gray-400 hover:bg-[#1a2333] hover:text-gray-200'
            }`}
          >
            <Cpu className="w-4 h-4" />
            Computer Controller
          </button>

          <button
            onClick={() => setActiveTab('tools')}
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
              activeTab === 'tools'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                : 'text-gray-400 hover:bg-[#1a2333] hover:text-gray-200'
            }`}
          >
            <Wrench className="w-4 h-4" />
            Registered Tools
          </button>

          <button
            onClick={() => setActiveTab('settings')}
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
              activeTab === 'settings'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                : 'text-gray-400 hover:bg-[#1a2333] hover:text-gray-200'
            }`}
          >
            <Settings className="w-4 h-4" />
            Connections & Config
          </button>
        </nav>

        <div className="p-4 border-t border-[#1f293d] bg-[#0e1422] text-xs text-gray-400">
          <div className="flex items-center justify-between mb-1">
            <span>Status</span>
            <span className="flex items-center gap-1.5 text-emerald-400 font-semibold">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
              Bridge Active
            </span>
          </div>
          <div className="text-[11px] text-gray-500">v0.1.0 • Rust Engine</div>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col bg-[#0b0f19]">
        {/* Top Header / Model Selector Bar */}
        <header className="h-16 border-b border-[#1f293d] px-6 flex items-center justify-between bg-[#111827]/50 backdrop-blur-md">
          <div className="flex items-center gap-4">
            <span className="text-xs font-semibold uppercase tracking-wider text-gray-400">Model Engine:</span>
            <select
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
              className="bg-[#1a2333] text-sm text-gray-200 border border-[#2d3a54] rounded-md px-3 py-1.5 focus:outline-none focus:border-indigo-500"
            >
              <option value="openai">OpenAI Endpoint</option>
              <option value="ollama">Local Ollama</option>
            </select>

            <select
              value={model}
              onChange={(e) => setModel(e.target.value)}
              className="bg-[#1a2333] text-sm text-gray-200 border border-[#2d3a54] rounded-md px-3 py-1.5 focus:outline-none focus:border-indigo-500"
            >
              {availableModels.length > 0 ? (
                availableModels.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}
                  </option>
                ))
              ) : (
                <option value="gpt-4o">GPT-4o (Default)</option>
              )}
            </select>
          </div>

          <button
            onClick={() => {
              fetchModels();
              fetchTools();
            }}
            className="p-2 text-gray-400 hover:text-white rounded-lg hover:bg-[#1f293d] transition-colors"
            title="Refresh Services"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </header>

        {/* Tab 1: Agent Chat */}
        {activeTab === 'chat' && (
          <div className="flex-1 flex flex-col overflow-hidden">
            <div className="flex-1 p-6 overflow-y-auto space-y-4">
              {history.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-center max-w-md mx-auto space-y-4">
                  <div className="p-4 bg-indigo-600/10 border border-indigo-500/20 rounded-2xl">
                    <Terminal className="w-10 h-10 text-indigo-400" />
                  </div>
                  <div>
                    <h2 className="text-xl font-bold text-gray-100">Hermes-Claw Active</h2>
                    <p className="text-sm text-gray-400 mt-1">
                      Direct computer access with shell tool execution, process control, and system automation enabled.
                    </p>
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-xs w-full text-left">
                    <button
                      onClick={() => setInputMessage('Check my computer specs and memory usage.')}
                      className="p-3 bg-[#111827] border border-[#1f293d] rounded-xl hover:border-indigo-500/50 transition-colors text-gray-300"
                    >
                      💻 Check Computer Specs
                    </button>
                    <button
                      onClick={() => setInputMessage('List top 10 CPU-consuming processes.')}
                      className="p-3 bg-[#111827] border border-[#1f293d] rounded-xl hover:border-indigo-500/50 transition-colors text-gray-300"
                    >
                      📊 List Processes
                    </button>
                    <button
                      onClick={() => setInputMessage('List files in the current working directory.')}
                      className="p-3 bg-[#111827] border border-[#1f293d] rounded-xl hover:border-indigo-500/50 transition-colors text-gray-300"
                    >
                      📁 List Current Directory
                    </button>
                    <button
                      onClick={() => setInputMessage('Run a shell command `uname -a` to check system kernel.')}
                      className="p-3 bg-[#111827] border border-[#1f293d] rounded-xl hover:border-indigo-500/50 transition-colors text-gray-300"
                    >
                      ⚡ Execute Shell Command
                    </button>
                  </div>
                </div>
              ) : (
                history.map((msg, idx) => {
                  if (msg.role === 'system') return null;
                  const isUser = msg.role === 'user';
                  const isTool = msg.role === 'tool';

                  return (
                    <div
                      key={idx}
                      className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}
                    >
                      <div
                        className={`max-w-3xl rounded-2xl p-4 shadow-lg ${
                          isUser
                            ? 'bg-indigo-600 text-white rounded-br-none'
                            : isTool
                            ? 'bg-[#111827] border border-amber-500/30 text-amber-300 rounded-bl-none font-mono text-xs'
                            : 'bg-[#111827] border border-[#1f293d] text-gray-200 rounded-bl-none'
                        }`}
                      >
                        <div className="flex items-center gap-2 mb-1 text-xs opacity-75 font-mono">
                          {isUser ? 'You' : isTool ? `Tool Output [${msg.name || 'system'}]` : 'Hermes-Claw'}
                        </div>
                        <div className="whitespace-pre-wrap leading-relaxed text-sm">
                          {msg.content}
                        </div>
                      </div>
                    </div>
                  );
                })
              )}

              {loading && (
                <div className="flex justify-start">
                  <div className="bg-[#111827] border border-indigo-500/30 p-4 rounded-2xl text-indigo-400 text-sm flex items-center gap-3">
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    Hermes-Claw reasoning and executing tools...
                  </div>
                </div>
              )}
              <div ref={chatBottomRef} />
            </div>

            {/* Input Bar */}
            <div className="p-4 border-t border-[#1f293d] bg-[#111827]">
              <div className="flex items-center gap-3 max-w-5xl mx-auto">
                <input
                  type="text"
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSendMessage()}
                  placeholder="Ask Hermes-Claw or instruct it to control your computer..."
                  className="flex-1 bg-[#1a2333] border border-[#2d3a54] rounded-xl px-4 py-3 text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onClick={handleSendMessage}
                  disabled={loading || !inputMessage.trim()}
                  className="bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white p-3 rounded-xl transition-colors shadow-lg shadow-indigo-600/30"
                >
                  <Send className="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Tab 2: Computer Controller */}
        {activeTab === 'system' && (
          <div className="flex-1 p-6 overflow-y-auto space-y-6">
            <div>
              <h2 className="text-xl font-bold">Computer Controller Dashboard</h2>
              <p className="text-sm text-gray-400">Direct execution, hardware inspection, and shell control.</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Card 1: System Info */}
              <div className="bg-[#111827] border border-[#1f293d] rounded-2xl p-5 flex flex-col">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2 text-indigo-400 font-semibold">
                    <HardDrive className="w-5 h-5" />
                    System Metrics
                  </div>
                  <button
                    onClick={runSystemInfo}
                    className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold rounded-lg transition-colors flex items-center gap-1.5"
                  >
                    <Play className="w-3.5 h-3.5" /> Run Tool
                  </button>
                </div>
                <pre className="flex-1 bg-[#0b0f19] p-4 rounded-xl font-mono text-xs text-gray-300 border border-[#1f293d] overflow-x-auto min-h-[160px]">
                  {sysInfoOutput || 'Click "Run Tool" to inspect CPU, Memory, and Host metrics.'}
                </pre>
              </div>

              {/* Card 2: Process Monitor */}
              <div className="bg-[#111827] border border-[#1f293d] rounded-2xl p-5 flex flex-col">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2 text-purple-400 font-semibold">
                    <Activity className="w-5 h-5" />
                    Process Inspector
                  </div>
                  <button
                    onClick={runProcessList}
                    className="px-3 py-1.5 bg-purple-600 hover:bg-purple-500 text-xs font-semibold rounded-lg transition-colors flex items-center gap-1.5"
                  >
                    <Play className="w-3.5 h-3.5" /> Scan Processes
                  </button>
                </div>
                <pre className="flex-1 bg-[#0b0f19] p-4 rounded-xl font-mono text-xs text-gray-300 border border-[#1f293d] overflow-x-auto min-h-[160px]">
                  {processOutput || 'Click "Scan Processes" to list active computer tasks.'}
                </pre>
              </div>
            </div>

            {/* Quick Shell Command Tool */}
            <div className="bg-[#111827] border border-[#1f293d] rounded-2xl p-5">
              <h3 className="font-semibold text-gray-200 mb-2 flex items-center gap-2">
                <Terminal className="w-5 h-5 text-emerald-400" />
                Direct Shell Execution Tool
              </h3>
              <div className="flex gap-3 mb-4">
                <input
                  type="text"
                  value={quickCmd}
                  onChange={(e) => setQuickCmd(e.target.value)}
                  placeholder="Enter shell command (e.g. ls -la /app or uptime)"
                  className="flex-1 bg-[#0b0f19] border border-[#1f293d] rounded-xl px-4 py-2.5 text-sm font-mono text-emerald-400 focus:outline-none focus:border-emerald-500"
                />
                <button
                  onClick={runQuickCommand}
                  className="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-sm rounded-xl transition-colors"
                >
                  Execute Command
                </button>
              </div>
              {cmdOutput && (
                <pre className="bg-[#0b0f19] p-4 rounded-xl font-mono text-xs text-gray-300 border border-[#1f293d] overflow-x-auto">
                  {cmdOutput}
                </pre>
              )}
            </div>
          </div>
        )}

        {/* Tab 3: Registered Tools */}
        {activeTab === 'tools' && (
          <div className="flex-1 p-6 overflow-y-auto space-y-6">
            <div>
              <h2 className="text-xl font-bold">Registered Hermes Tools</h2>
              <p className="text-sm text-gray-400">Capabilities available to the AI agent during turn cycles.</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {availableTools.map((t, idx) => (
                <div key={idx} className="bg-[#111827] border border-[#1f293d] rounded-2xl p-5">
                  <div className="flex items-center gap-2 text-indigo-400 font-mono font-semibold mb-2">
                    <Wrench className="w-4 h-4" />
                    {t.name}
                  </div>
                  <p className="text-sm text-gray-300 mb-3">{t.description}</p>
                  <details className="text-xs text-gray-400">
                    <summary className="cursor-pointer hover:text-indigo-400">View Parameter Schema</summary>
                    <pre className="mt-2 p-3 bg-[#0b0f19] rounded-lg font-mono text-[11px] text-gray-400 border border-[#1f293d]">
                      {JSON.stringify(t.parameters, null, 2)}
                    </pre>
                  </details>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Tab 4: Connections & Config */}
        {activeTab === 'settings' && config && (
          <div className="flex-1 p-6 overflow-y-auto space-y-6 max-w-3xl">
            <div>
              <h2 className="text-xl font-bold">API Connections & Agent Settings</h2>
              <p className="text-sm text-gray-400">Configure LLM providers, endpoints, and system prompts.</p>
            </div>

            <div className="bg-[#111827] border border-[#1f293d] rounded-2xl p-6 space-y-4">
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-gray-400 mb-2">
                  OpenAI API Key
                </label>
                <input
                  type="password"
                  value={config.openai_api_key || ''}
                  onChange={(e) => setConfig({ ...config, openai_api_key: e.target.value })}
                  placeholder="sk-proj-..."
                  className="w-full bg-[#0b0f19] border border-[#1f293d] rounded-xl px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-gray-400 mb-2">
                  Ollama Base URL
                </label>
                <input
                  type="text"
                  value={config.ollama_base_url}
                  onChange={(e) => setConfig({ ...config, ollama_base_url: e.target.value })}
                  className="w-full bg-[#0b0f19] border border-[#1f293d] rounded-xl px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-gray-400 mb-2">
                  System Prompt
                </label>
                <textarea
                  rows={4}
                  value={config.system_prompt}
                  onChange={(e) => setConfig({ ...config, system_prompt: e.target.value })}
                  className="w-full bg-[#0b0f19] border border-[#1f293d] rounded-xl px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-indigo-500"
                />
              </div>

              <button
                onClick={saveConfig}
                className="px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm rounded-xl transition-colors shadow-lg shadow-indigo-600/30"
              >
                Save Configuration
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
