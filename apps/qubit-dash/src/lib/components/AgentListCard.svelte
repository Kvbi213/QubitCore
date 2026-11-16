<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Users, Bot, Zap, Brain, Layers, Clock } from 'lucide-svelte';
  import type { ExecutionMode } from '../types/protocol';

  const agents = $derived(connectionStore.agents);
  const currentMode = $derived(connectionStore.executionMode);

  function handleModeChange(mode: ExecutionMode) {
    connectionStore.setMode(mode);
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(0) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }
</script>

<div class="p-4 rounded-2xl bg-[#0b101d] border border-white/[0.08] shadow-lg flex flex-col justify-between">
  <div>
    <!-- Header with Execution Mode Pills -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <div class="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
          <Bot class="w-4 h-4" />
        </div>
        <div>
          <h2 class="text-xs font-bold text-white uppercase tracking-wider font-mono">Qubit Agents Pool</h2>
          <p class="text-[10px] text-zinc-400 font-mono">Reflex vs Deep Execution Router</p>
        </div>
      </div>

      <!-- Mode Toggle -->
      <div class="flex items-center p-0.5 rounded-xl bg-white/[0.04] border border-white/[0.08]">
        <button
          onclick={() => handleModeChange('reflex')}
          class="px-2.5 py-1 rounded-lg text-[10px] font-mono font-bold transition flex items-center gap-1 {currentMode === 'reflex' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30' : 'text-zinc-400 hover:text-white'}"
        >
          <Zap class="w-3 h-3" />
          <span>Reflex</span>
        </button>
        <button
          onclick={() => handleModeChange('deep')}
          class="px-2.5 py-1 rounded-lg text-[10px] font-mono font-bold transition flex items-center gap-1 {currentMode === 'deep' ? 'bg-purple-500/20 text-purple-300 border border-purple-500/30' : 'text-zinc-400 hover:text-white'}"
        >
          <Brain class="w-3 h-3" />
          <span>Deep</span>
        </button>
      </div>
    </div>

    <!-- Agent Items -->
    <div class="space-y-2">
      {#each agents as agent (agent.id)}
        <div class="p-3 rounded-xl bg-white/[0.02] border border-white/[0.05] hover:border-white/10 transition">
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full {agent.status === 'executing' ? 'bg-cyan-400 animate-ping' : agent.status === 'completed' ? 'bg-emerald-400' : 'bg-emerald-500'}"></span>
              <span class="text-xs font-mono font-bold text-white">{agent.name}</span>
            </div>
            <div class="flex items-center gap-1.5 text-[10px] font-mono">
              <span class="px-1.5 py-0.5 rounded bg-white/[0.04] text-zinc-400 border border-white/[0.05]">
                {formatBytes(agent.memory_usage_bytes)}
              </span>
              <span class="px-1.5 py-0.5 rounded uppercase font-semibold text-cyan-400 bg-cyan-500/10 border border-cyan-500/20">
                {agent.status}
              </span>
            </div>
          </div>

          <p class="text-[11px] text-zinc-400 font-medium font-sans mb-1">{agent.role}</p>

          {#if agent.current_task}
            <div class="text-[10px] font-mono text-zinc-400 bg-white/[0.02] p-1.5 rounded-lg border border-white/[0.04] flex items-center gap-1.5">
              <Clock class="w-3 h-3 text-cyan-400 shrink-0" />
              <span class="truncate">{agent.current_task}</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <div class="mt-3 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[10px] font-mono text-zinc-500">
    <span>Hermes 4-Phase Pruning: ACTIVE</span>
    <span>3 Agents Registered</span>
  </div>
</div>
