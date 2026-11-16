import type { AgentDescriptor, ExecutionMode, McpToolInfo, MemoryStats, WebSocketEvent } from '../types/protocol';

class QubitConnectionStore {
  // Svelte 5 reactive runes
  isConnected = $state(false);
  isConnecting = $state(false);
  daemonUrl = $state('http://localhost:3000');
  wsUrl = $state('ws://localhost:3000/ws');
  latencyMs = $state(0);
  lastPingTime = $state<Date | null>(null);

  // System & Memory Metrics
  memoryStats = $state<MemoryStats>({
    ephemeral_mode: true,
    window_active: true,
    estimated_ram_mb: 48.5,
    budget_limit_mb: 60.0,
    status: 'Active (GUI Window Open)',
  });

  // Agents state
  executionMode = $state<ExecutionMode>('reflex');
  agents = $state<AgentDescriptor[]>([
    {
      id: '00000000-0000-0000-0000-000000000001',
      name: 'Supervisor-01',
      role: 'Core Orchestrator & Session Hub',
      status: 'idle',
      current_task: 'Listening for IPC / MCP client connections',
      memory_usage_bytes: 1_250_000,
    },
    {
      id: '00000000-0000-0000-0000-000000000002',
      name: 'Hermes-Compressor',
      role: 'Proactive Context Memory Pruning',
      status: 'idle',
      current_task: 'Monitoring token budget (50% threshold)',
      memory_usage_bytes: 850_000,
    },
    {
      id: '00000000-0000-0000-0000-000000000003',
      name: 'Reversing-Specialist',
      role: 'Ghidra MCP / Sandbox Orchestrator',
      status: 'idle',
      current_task: 'Standby for binary inspection requests',
      memory_usage_bytes: 2_400_000,
    },
  ]);

  // MCP Tools state
  mcpTools = $state<McpToolInfo[]>([
    {
      name: 'get_program_info',
      description: 'Orientation: extracts binary metadata, arch, and imports',
      server_id: 'reversing-mcp',
      requires_elicitation: false,
    },
    {
      name: 'get_xrefs_to',
      description: 'Find & Xrefs: inspect cross references to symbols/strings',
      server_id: 'reversing-mcp',
      requires_elicitation: false,
    },
    {
      name: 'decompile_function_by_address',
      description: 'Point decompilation: produces C pseudocode for candidate address',
      server_id: 'reversing-mcp',
      requires_elicitation: false,
    },
    {
      name: 'write_back_annotation',
      description: 'Write-Back: renames symbols and updates project comments',
      server_id: 'reversing-mcp',
      requires_elicitation: true,
    },
    {
      name: 'filesystem_workspace_read',
      description: 'Reads local workspace files under authorized root',
      server_id: 'qubit-core',
      requires_elicitation: false,
    },
  ]);

  // Events Log
  events = $state<{ id: string; time: string; type: string; details: string; level: 'info' | 'warn' | 'success' | 'alert' }[]>([
    {
      id: '1',
      time: new Date().toLocaleTimeString(),
      type: 'SYS_INIT',
      details: 'QubitDash Mission Control initialized with Ephemeral WebView life-cycle',
      level: 'info',
    },
    {
      id: '2',
      time: new Date().toLocaleTimeString(),
      type: 'TRAY_CONFIG',
      details: 'System Tray active. Closing window will destroy WebView to reduce RAM to ~3.8 MB',
      level: 'success',
    },
  ]);

  private ws: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingInterval: ReturnType<typeof setInterval> | null = null;

  constructor() {
    this.init();
  }

  private init() {
    this.connectWs();
    this.startPingLoop();
    this.queryTauriStats();
  }

  public connectWs() {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.isConnecting = true;
    try {
      this.ws = new WebSocket(this.wsUrl);

      this.ws.onopen = () => {
        this.isConnected = true;
        this.isConnecting = false;
        this.addEvent('WS_OPEN', 'Connected to QubitCore daemon at ' + this.wsUrl, 'success');
      };

      this.ws.onmessage = (event) => {
        try {
          const parsed = JSON.parse(event.data) as WebSocketEvent;
          this.handleServerEvent(parsed);
        } catch {
          // Plain message fallback
          this.addEvent('RAW_MSG', String(event.data), 'info');
        }
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.isConnecting = false;
        this.scheduleReconnect();
      };

      this.ws.onerror = () => {
        this.isConnected = false;
        this.isConnecting = false;
      };
    } catch {
      this.isConnected = false;
      this.isConnecting = false;
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(() => {
      this.connectWs();
    }, 4000);
  }

  private startPingLoop() {
    if (this.pingInterval) clearInterval(this.pingInterval);
    this.pingInterval = setInterval(async () => {
      const start = Date.now();
      try {
        const res = await fetch(`${this.daemonUrl}/api/health`, { method: 'GET', signal: AbortSignal.timeout(1500) });
        if (res.ok) {
          this.latencyMs = Date.now() - start;
          this.lastPingTime = new Date();
          if (!this.isConnected && !this.isConnecting) {
            this.connectWs();
          }
        }
      } catch {
        // Daemon might not be running on localhost:3000 yet
        this.latencyMs = 0;
      }
      this.queryTauriStats();
    }, 3000);
  }

  private async queryTauriStats() {
    // If running in Tauri environment, invoke IPC
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const stats = await invoke<MemoryStats>('get_memory_stats');
        if (stats) {
          this.memoryStats = stats;
        }
      } catch {
        // Ignore outside Tauri
      }
    }
  }

  public async triggerMinimizeToTray() {
    this.addEvent('EPHEMERAL_REQ', 'User requested minimize to System Tray. Destroying WebKit engine...', 'warn');
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('minimize_to_tray');
      } catch (err) {
        this.addEvent('ERROR', 'Failed to invoke minimize_to_tray: ' + String(err), 'alert');
      }
    } else {
      this.addEvent('SIMULATION', 'Web browser demo mode: simulated window destroy to ~3.8 MB RAM', 'info');
      this.memoryStats.window_active = false;
      this.memoryStats.estimated_ram_mb = 3.8;
      this.memoryStats.status = 'Simulated Tray Sleep (~3.8 MB RAM)';
    }
  }

  public async restoreSimulation() {
    this.memoryStats.window_active = true;
    this.memoryStats.estimated_ram_mb = 48.5;
    this.memoryStats.status = 'Active (GUI Window Open)';
    this.addEvent('RESTORE', 'Restored GUI Window in <200ms', 'success');
  }

  public setMode(mode: ExecutionMode) {
    this.executionMode = mode;
    this.addEvent('MODE_CHANGE', `Execution mode switched to: ${mode.toUpperCase()}`, 'info');
  }

  public addEvent(type: string, details: string, level: 'info' | 'warn' | 'success' | 'alert' = 'info') {
    this.events = [
      {
        id: Math.random().toString(36).substring(2, 9),
        time: new Date().toLocaleTimeString(),
        type,
        details,
        level,
      },
      ...this.events.slice(0, 49),
    ];
  }

  private handleServerEvent(event: WebSocketEvent) {
    if (event.type === 'agent_status_changed') {
      const idx = this.agents.findIndex((a) => a.id === event.payload.id);
      if (idx !== -1) {
        this.agents[idx] = event.payload;
      } else {
        this.agents.push(event.payload);
      }
      this.addEvent('AGENT_UPDATE', `${event.payload.name} -> ${event.payload.status}`, 'info');
    } else if (event.type === 'system_alert') {
      this.addEvent('ALERT', event.payload.message, 'alert');
    } else {
      this.addEvent(event.type.toUpperCase(), JSON.stringify(event.payload), 'info');
    }
  }
}

export const connectionStore = new QubitConnectionStore();
