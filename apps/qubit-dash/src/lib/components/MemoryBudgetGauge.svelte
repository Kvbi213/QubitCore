<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Cpu, Zap, HardDrive, EyeOff, RotateCcw } from 'lucide-svelte';

  const memoryStats = $derived(connectionStore.memoryStats);
  const percentage = $derived(
    Math.min(100, Math.round((memoryStats.estimated_ram_mb / memoryStats.budget_limit_mb) * 100))
  );

  function testMinimize() {
    connectionStore.triggerMinimizeToTray();
  }

  function testRestore() {
    connectionStore.restoreSimulation();
  }
</script>

<div class="p-4 rounded-2xl bg-[#0b101d] border border-white/[0.08] shadow-lg flex flex-col justify-between">
  <div>
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <div class="p-2 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400">
          <Cpu class="w-4 h-4" />
        </div>
        <div>
          <h2 class="text-xs font-bold text-white uppercase tracking-wider font-mono">RAM Budget & Ephemeral Engine</h2>
          <p class="text-[10px] text-zinc-400 font-mono">WebKit vs Rust Footprint Decomposition</p>
        </div>
      </div>
      <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-white/[0.04] text-zinc-400 border border-white/10">
        cgroups v2
      </span>
    </div>

    <!-- Progress Gauge -->
    <div class="space-y-1.5 mb-3">
      <div class="flex items-center justify-between text-xs font-mono">
        <span class="text-zinc-400">Zużycie RAM:</span>
        <span class="font-bold {percentage > 80 ? 'text-amber-400' : 'text-cyan-300'}">
          {memoryStats.estimated_ram_mb} MB / {memoryStats.budget_limit_mb} MB ({percentage}%)
        </span>
      </div>

      <div class="w-full h-2 rounded-full bg-white/[0.04] overflow-hidden p-0.5 border border-white/[0.05]">
        <div
          class="h-full rounded-full transition-all duration-500 bg-gradient-to-r from-cyan-400 to-purple-500"
          style="width: {percentage}%"
        ></div>
      </div>
    </div>

    <!-- Architecture Stats Grid -->
    <div class="grid grid-cols-2 gap-2 text-xs font-mono">
      <div class="p-2 rounded-xl bg-white/[0.02] border border-white/[0.04] space-y-0.5">
        <span class="text-[10px] text-zinc-500 uppercase">Rust Backend IPC</span>
        <p class="text-white font-semibold">~3.8 MB</p>
        <span class="text-[9px] text-emerald-400 font-mono">Zero GC Overhead</span>
      </div>

      <div class="p-2 rounded-xl bg-white/[0.02] border border-white/[0.04] space-y-0.5">
        <span class="text-[10px] text-zinc-500 uppercase">WebView GUI Engine</span>
        <p class="text-white font-semibold">{memoryStats.window_active ? '~44.7 MB' : '0 MB (Destroyed)'}</p>
        <span class="text-[9px] text-cyan-400 font-mono">{memoryStats.window_active ? 'Allocated' : 'Freed to OS'}</span>
      </div>
    </div>
  </div>

  <!-- Ephemeral Toggle Buttons -->
  <div class="mt-3 pt-3 border-t border-white/[0.06] flex items-center justify-between gap-2">
    {#if memoryStats.window_active}
      <button
        onclick={testMinimize}
        class="w-full py-1.5 px-2.5 rounded-xl bg-white/[0.05] hover:bg-white/[0.1] active:scale-95 border border-white/10 text-cyan-300 hover:text-white text-[11px] font-mono font-semibold transition flex items-center justify-center gap-1.5"
      >
        <EyeOff class="w-3.5 h-3.5" />
        <span>Zwolnij RAM (Zniszcz do Tray)</span>
      </button>
    {:else}
      <button
        onclick={testRestore}
        class="w-full py-1.5 px-2.5 rounded-xl bg-cyan-500/20 hover:bg-cyan-500/30 active:scale-95 border border-cyan-400/30 text-cyan-200 text-[11px] font-mono font-semibold transition flex items-center justify-center gap-1.5"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>Przywróć WebView (&lt;200ms)</span>
      </button>
    {/if}
  </div>
</div>
