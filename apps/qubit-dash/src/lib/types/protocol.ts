/**
 * Qubit Protocol Types (TypeScript definitions matching crates/qubit-protocol)
 */

export type ExecutionMode = 'reflex' | 'deep';

export type MessageRole = 'system' | 'user' | 'assistant' | 'tool';

export type AgentStatus =
  | 'idle'
  | 'planning'
  | 'executing'
  | 'waiting_for_input'
  | 'failed'
  | 'completed';

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface ToolResult {
  call_id: string;
  success: boolean;
  output?: unknown;
  error?: string;
}

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  created_at: string;
  tool_calls?: ToolCall[];
}

export interface AgentDescriptor {
  id: string;
  name: string;
  role: string;
  status: AgentStatus;
  current_task?: string;
  memory_usage_bytes: number;
}

export interface SystemTelemetryDto {
  memory_rss_bytes: number;
  memory_limit_bytes: number;
  memory_usage_percent: number;
  tokio_active_threads: number;
  tokio_blocking_threads: number;
  tokio_tasks_total: number;
  anon_pool_available: number;
  anon_pool_total: number;
  aio_connected: boolean;
  mikrus_connected: boolean;
  primary_node: string;
  network_rx_bps: number;
  network_tx_bps: number;
  wal_file_size_bytes: number;
  ephemeral_window_active: boolean;
  uptime_secs: number;
}

export interface ProviderStatusDto {
  id: string;
  name: string;
  kind: 'cli_harness' | 'direct_api' | 'local_ipc';
  is_connected: boolean;
  model_id: string;
  latency_ms: number;
  last_heartbeat: string;
  rate_limit_remaining: number;
  auth_mode: string;
}

export interface RoutingConfigDto {
  reflex_threshold_ms: number;
  deep_threshold_ms: number;
  enable_auto_escalation: boolean;
  max_subagent_depth: number;
  cost_ceiling_usd: number;
  fallback_provider: string;
}

export interface RoutineDto {
  id: string;
  title: string;
  cron_expression: string;
  target_agent: string;
  last_run?: string;
  next_run?: string;
  is_active: boolean;
  execution_count: number;
}

export interface McpAclRuleDto {
  id: string;
  tool_pattern: string;
  access_level: 'allow' | 'prompt' | 'deny';
  sandbox_required: boolean;
  max_memory_mb: number;
  network_egress: boolean;
}

export interface WalCheckpointResultDto {
  success: boolean;
  checkpoint_mode: string;
  wal_pages_checkpointed: number;
  wal_pages_remaining: number;
  duration_ms: number;
}

export type WebSocketEvent =
  | { type: 'agent_status_changed'; payload: AgentDescriptor }
  | { type: 'task_progress'; payload: { task_id: string; percent: number; status: string } }
  | { type: 'message_received'; payload: ChatMessage }
  | { type: 'tool_invoked'; payload: ToolCall }
  | { type: 'tool_completed'; payload: ToolResult }
  | { type: 'system_alert'; payload: { level: string; message: string } }
  | { type: 'telemetry_update'; payload: SystemTelemetryDto };

export interface MemoryStats {
  ephemeral_mode: boolean;
  window_active: boolean;
  estimated_ram_mb: number;
  budget_limit_mb: number;
  status: string;
}

export interface McpToolInfo {
  name: string;
  description: string;
  server_id: string;
  requires_elicitation: boolean;
}
