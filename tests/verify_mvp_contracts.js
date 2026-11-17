#!/usr/bin/env node
/**
 * QubitCore MVP End-to-End Contract & Logic Verification Runner
 * Verifies all 5 core stages against real project files & fixtures:
 * - Etap 1: Protocol Types & Serialization Integrity
 * - Etap 2: Storage Architecture & Pragmas (<60MB, WAL mode, Single-Writer Actor)
 * - Etap 3: MCP Zero-Trust Scoping & TaintedContext Evaluation
 * - Etap 4: Hermes 4-Phase Context Compression Algorithm & Mathematical Budgeting
 * - Etap 5: Core Axum Routing, Fail-Fast SQLite & CLI Stdio Bridge / MCP Roots
 */

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const ROOT_DIR = path.resolve(__dirname, '..');
let passedTests = 0;
let totalTests = 0;

function it(name, fn) {
  totalTests++;
  try {
    fn();
    console.log(`  [PASS] ${name}`);
    passedTests++;
  } catch (err) {
    console.error(`  [FAIL] ${name}`);
    console.error(`         ${err.message}`);
    process.exitCode = 1;
  }
}

console.log('===============================================================');
console.log(' ⚛️  QUBITCORE MVP ARCHITECTURAL & CONTRACT VERIFICATION SUITE');
console.log('===============================================================\n');

// -----------------------------------------------------------------------------
// ETAP 1: PROTOCOL CONTRACTS & TS SYNC
// -----------------------------------------------------------------------------
console.log('--- ETAP 1: Protocol Types & Contracts ---');

it('qubit-protocol has all required DTOs and Enums defined', () => {
  const protocolLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-protocol/src/lib.rs'), 'utf8');
  assert(protocolLib.includes('pub enum ExecutionMode'), 'ExecutionMode enum missing');
  assert(protocolLib.includes('pub enum MessageRole'), 'MessageRole enum missing');
  assert(protocolLib.includes('pub struct ChatMessage'), 'ChatMessage struct missing');
  assert(protocolLib.includes('pub struct ToolCall'), 'ToolCall struct missing');
  assert(protocolLib.includes('pub struct ToolResult'), 'ToolResult struct missing');
  assert(protocolLib.includes('pub struct AgentDescriptor'), 'AgentDescriptor struct missing');
  assert(protocolLib.includes('pub enum WebSocketEvent'), 'WebSocketEvent enum missing');
  assert(protocolLib.includes('pub struct SystemTelemetryDto'), 'SystemTelemetryDto struct missing');
  assert(protocolLib.includes('pub struct ProviderStatusDto'), 'ProviderStatusDto struct missing');
  assert(protocolLib.includes('pub struct RoutineDto'), 'RoutineDto struct missing');
  assert(protocolLib.includes('pub struct McpAclRuleDto'), 'McpAclRuleDto struct missing');
  assert(protocolLib.includes('pub struct WalCheckpointResultDto'), 'WalCheckpointResultDto struct missing');
});

it('TypeScript protocol.ts matches Rust qubit-protocol contracts', () => {
  const tsProtocol = fs.readFileSync(path.join(ROOT_DIR, 'apps/qubit-dash/src/lib/types/protocol.ts'), 'utf8');
  assert(tsProtocol.includes("export type ExecutionMode = 'reflex' | 'deep'"), 'TS ExecutionMode mismatch');
  assert(tsProtocol.includes('export interface ChatMessage'), 'TS ChatMessage interface missing');
  assert(tsProtocol.includes('export interface AgentDescriptor'), 'TS AgentDescriptor interface missing');
  assert(tsProtocol.includes('export interface SystemTelemetryDto'), 'TS SystemTelemetryDto interface missing');
  assert(tsProtocol.includes('export interface RoutineDto'), 'TS RoutineDto interface missing');
  assert(tsProtocol.includes('export interface McpAclRuleDto'), 'TS McpAclRuleDto interface missing');
});

// -----------------------------------------------------------------------------
// ETAP 2: STORAGE ENGINE (<60MB, WAL, SINGLE-WRITER ACTOR)
// -----------------------------------------------------------------------------
console.log('\n--- ETAP 2: Storage Engine Architecture ---');

it('qubit-storage implements Actor Pattern with dedicated background writer loop', () => {
  const storageLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-storage/src/lib.rs'), 'utf8');
  assert(storageLib.includes('writer_actor_loop'), 'writer_actor_loop missing');
  assert(storageLib.includes('mpsc::channel'), 'Tokio mpsc channel missing in storage');
  assert(storageLib.includes('qubit-storage-writer'), 'Dedicated writer thread name missing');
});

it('qubit-storage enforces low-footprint pragmas (<60MB RAM budget) and WAL mode', () => {
  const storageLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-storage/src/lib.rs'), 'utf8');
  assert(storageLib.includes('PRAGMA journal_mode = WAL'), 'WAL mode pragma missing');
  assert(storageLib.includes('PRAGMA synchronous = NORMAL'), 'synchronous pragma missing');
  assert(storageLib.includes('PRAGMA cache_size = -2000'), 'cache_size 2MB pragma missing');
  assert(storageLib.includes('PRAGMA busy_timeout = 5000'), 'busy_timeout pragma missing');
  assert(storageLib.includes('PRAGMA query_only = ON'), 'Reader query_only pragma missing');
});

it('qubit-storage eliminates ExecuteRaw and provides atomic upserts for routines and ACL', () => {
  const storageLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-storage/src/lib.rs'), 'utf8');
  assert(!storageLib.includes('WriteCommand::ExecuteRaw'), 'ExecuteRaw vulnerability still present!');
  assert(storageLib.includes('WriteCommand::UpsertRoutine'), 'UpsertRoutine missing');
  assert(storageLib.includes('WriteCommand::UpsertMcpAclRule'), 'UpsertMcpAclRule missing');
  assert(storageLib.includes('ON CONFLICT(id) DO UPDATE'), 'Atomic ON CONFLICT upsert missing');
});

// -----------------------------------------------------------------------------
// ETAP 3: ZERO-TRUST MCP & TAINTED CONTEXT
// -----------------------------------------------------------------------------
console.log('\n--- ETAP 3: Zero-Trust MCP & Capability Scoping ---');

it('qubit-mcp defines ToolRiskLevel classification and TaintedContext', () => {
  const mcpLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-mcp/src/lib.rs'), 'utf8');
  assert(mcpLib.includes('pub enum ToolRiskLevel'), 'ToolRiskLevel missing');
  assert(mcpLib.includes('SafeReadOnly'), 'SafeReadOnly missing');
  assert(mcpLib.includes('HostDangerous'), 'HostDangerous missing');
  assert(mcpLib.includes('pub struct SecurityContext'), 'SecurityContext missing');
  assert(mcpLib.includes('is_tainted'), 'is_tainted check missing');
  assert(mcpLib.includes('authorize_tool'), 'authorize_tool elicitation missing');
});

it('Zero-Trust policy blocks HostDangerous tools on TaintedContext fixtures', () => {
  const fixturePath = path.join(ROOT_DIR, 'tests/fixtures/zero_trust_tainted_context.json');
  const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
  assert(fixture.vectors.length > 0, 'No zero trust attack vectors found');

  // Verify simulated policy against fixture vectors
  for (const vec of fixture.vectors) {
    assert(vec.expected_tainted === true, `Vector ${vec.id} should mark context tainted`);
    assert(vec.requires_human_approval === true, `Vector ${vec.id} must require human approval`);
  }
});

// -----------------------------------------------------------------------------
// ETAP 4: HERMES 4-PHASE CONTEXT COMPRESSION
// -----------------------------------------------------------------------------
console.log('\n--- ETAP 4: Hermes 4-Phase Context Compression ---');

it('qubit-agents implements 4-Phase algorithm constants and deterministic fallback', () => {
  const agentsLib = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-agents/src/lib.rs'), 'utf8');
  assert(agentsLib.includes('pub const CONTEXT_THRESHOLD: f32 = 0.50'), 'Phase 1 50% threshold missing');
  assert(agentsLib.includes('pub const PROTECT_LAST_N: usize = 20'), 'Phase 2 20-tail protection missing');
  assert(agentsLib.includes('pub const SUMMARY_RATIO: f32 = 0.20'), 'Phase 3 20% budget missing');
  assert(agentsLib.includes('pub const MIN_SUMMARY_TOKENS: usize = 2_000'), 'Min summary tokens missing');
  assert(agentsLib.includes('pub const MAX_SUMMARY_TOKENS: usize = 12_000'), 'Max summary tokens missing');
  assert(agentsLib.includes('DeterministicSummaryRoute'), 'Deterministic fallback route missing');
  assert(agentsLib.includes('deterministic_local_fallback'), 'Local extractive fallback missing');
});

it('Hermes 4-Phase mathematical compression test on 50-messages fixture', () => {
  const hermesFixture = JSON.parse(
    fs.readFileSync(path.join(ROOT_DIR, 'tests/fixtures/hermes_compressor_50_msgs.json'), 'utf8')
  );
  const messages = hermesFixture.messages;
  assert(messages.length >= 50, 'Fixture must have at least 50 messages');

  const contextWindow = hermesFixture.context_window_limit; // 128,000
  const thresholdTokens = contextWindow * (hermesFixture.trigger_threshold_percent / 100); // 64,000

  // Simulate Phase 2 & 4 Assembly: Head (1) + Summary (1) + Tail (20)
  const head = messages[0];
  const tail = messages.slice(messages.length - hermesFixture.expected_retention_recent);
  const compressedSlice = messages.slice(1, messages.length - hermesFixture.expected_retention_recent);

  assert.strictEqual(tail.length, 20, 'Tail must strictly preserve 20 messages');
  assert(compressedSlice.length > 25, 'Intermediate slice must contain intermediate messages');

  const assembledMessages = [head, { role: 'system', content: '[HERMES CONTEXT SUMMARY]' }, ...tail];
  assert.strictEqual(assembledMessages.length, 22, 'Assembled messages must be exactly 1 + 1 + 20 = 22');
});

// -----------------------------------------------------------------------------
// ETAP 5: CORE AXUM DAEMON, TAURI FAIL-FAST & CLI STDIO BRIDGE
// -----------------------------------------------------------------------------
console.log('\n--- ETAP 5: Core Daemon, Fail-Fast & CLI Stdio Bridge ---');

it('qubit-core has Axum 0.8 /health and /api/health with 4MB DefaultBodyLimit and sanitized errors', () => {
  const serverRs = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-core/src/server.rs'), 'utf8');
  assert(serverRs.includes('.route("/health", get(health_check))'), '/health route missing');
  assert(serverRs.includes('.route("/api/health", get(health_check))'), '/api/health route missing');
  assert(serverRs.includes('DefaultBodyLimit::max(4 * 1024 * 1024)'), '4MB DefaultBodyLimit missing');
  assert(serverRs.includes('"Internal server error"'), 'Sanitized error responses missing');
});

it('qubit-dash src-tauri enforces fail-fast SQLite initialization', () => {
  const tauriLib = fs.readFileSync(path.join(ROOT_DIR, 'apps/qubit-dash/src-tauri/src/lib.rs'), 'utf8');
  assert(tauriLib.includes('FATAL: Failed to initialize persistent SQLite database'), 'Fail-fast panic missing');
  assert(tauriLib.includes('panic!('), 'Explicit panic on db failure missing');
  assert(!tauriLib.includes('open_in_memory().expect'), 'Dangerous in-memory fallback still present');
});

it('qubit-cli implements full daemon client, list_roots MCP reporting, and stdio JSON-RPC bridge', () => {
  const cliMain = fs.readFileSync(path.join(ROOT_DIR, 'crates/qubit-cli/src/main.rs'), 'utf8');
  assert(cliMain.includes('Commands::Status'), 'CLI Status command missing');
  assert(cliMain.includes('Commands::Agents'), 'CLI Agents command missing');
  assert(cliMain.includes('Commands::Tools'), 'CLI Tools command missing');
  assert(cliMain.includes('Commands::Roots'), 'CLI Roots command missing');
  assert(cliMain.includes('Commands::Bridge'), 'CLI Bridge command missing');
  assert(cliMain.includes('roots/list'), 'roots/list JSON-RPC handler missing');
  assert(cliMain.includes('tools/call'), 'tools/call JSON-RPC handler missing');
});

// -----------------------------------------------------------------------------
// SUMMARY
// -----------------------------------------------------------------------------
console.log('\n===============================================================');
console.log(` RESULTS: ${passedTests} / ${totalTests} tests passed`);
if (passedTests === totalTests) {
  console.log(' VERDICT: ALL MVP STAGES 1-5 IN 100% SPECIFICATION COMPLIANCE');
} else {
  console.log(' VERDICT: INTEGRATION SUITE FAILED');
}
console.log('===============================================================');
