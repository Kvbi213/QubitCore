<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Terminal, Shield, Activity, Trash2 } from 'lucide-svelte';

  const events = $derived(connectionStore.events);

  function clearLogs() {
    connectionStore.events = [];
  }
</script>

<div class="p-4 rounded-2xl bg-[#0b101d] border border-white/[0.08] shadow-lg flex flex-col justify-between h-full">
  <div class="flex items-center justify-between mb-3 shrink-0">
    <div class="flex items-center gap-2">
      <div class="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
        <Terminal class="w-4 h-4" />
      </div>
      <div>
        <h2 class="text-xs font-bold text-white uppercase tracking-wider font-mono">Live IPC & WebSocket Stream</h2>
        <p class="text-[10px] text-zinc-400 font-mono">Deterministic Event Telemetry</p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <button
        onclick={clearLogs}
        class="p-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] text-zinc-400 hover:text-white transition"
        title="Wyczyść logi"
      >
        <Trash2 class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>

  <!-- Terminal Output -->
  <div class="flex-1 overflow-y-auto space-y-1.5 font-mono text-[11px] pr-1 min-h-[160px] max-h-[220px]">
    {#if events.length === 0}
      <div class="p-4 text-center text-zinc-600 text-xs font-mono">
        Brak zdarzeń w strumieniu.
      </div>
    {:else}
      {#each events as ev (ev.id)}
        <div class="p-2 rounded-lg bg-white/[0.02] border border-white/[0.04] flex items-start gap-2.5">
          <span class="text-zinc-500 shrink-0 text-[10px]">{ev.time}</span>
          <span
            class="px-1.5 py-0.5 rounded text-[9px] font-bold shrink-0 uppercase {
              ev.level === 'success'
                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                : ev.level === 'warn'
                ? 'bg-amber-500/10 text-amber-300 border border-amber-500/20'
                : ev.level === 'alert'
                ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                : 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20'
            }"
          >
            {ev.type}
          </span>
          <span class="text-zinc-300 break-all leading-relaxed">{ev.details}</span>
        </div>
      {/each}
    {/if}
  </div>

  <div class="mt-3 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[10px] font-mono text-zinc-500 shrink-0">
    <span>Zero-Drift ts-rs Schema Validation</span>
    <span>Auto-Scroll: LOCKED</span>
  </div>
</div>
