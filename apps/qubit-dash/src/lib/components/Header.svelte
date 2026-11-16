<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Shield, Radio, Minimize2, Cpu, EyeOff, Sparkles } from 'lucide-svelte';

  const isConnected = $derived(connectionStore.isConnected);
  const memoryStats = $derived(connectionStore.memoryStats);
  const latency = $derived(connectionStore.latencyMs);

  function handleMinimize() {
    connectionStore.triggerMinimizeToTray();
  }
</script>

<header class="h-14 border-b border-white/[0.08] bg-[#070a12]/90 backdrop-blur-md px-5 flex items-center justify-between shrink-0">
  <!-- Brand & Identity -->
  <div class="flex items-center gap-3">
    <div class="relative w-8 h-8 rounded-xl bg-gradient-to-br from-cyan-500/20 to-purple-600/30 border border-cyan-400/30 flex items-center justify-center shadow-lg shadow-cyan-500/10">
      <span class="font-mono font-bold text-sm text-cyan-300">Q</span>
      <div class="absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full {isConnected ? 'bg-emerald-400 ring-2 ring-emerald-950' : 'bg-rose-500'}"></div>
    </div>
    <div>
      <div class="flex items-center gap-2">
        <h1 class="text-xs font-bold tracking-wider font-mono text-white uppercase">QubitDash</h1>
        <span class="text-[9px] font-mono font-semibold px-1.5 py-0.5 rounded bg-white/[0.06] border border-white/10 text-cyan-300">
          EPHEMERAL v2.2
        </span>
      </div>
      <p class="text-[10px] text-zinc-400 font-medium font-mono">Mission Control for QubitCore Daemon</p>
    </div>
  </div>

  <!-- Center Telemetry Badges -->
  <div class="hidden md:flex items-center gap-4 text-xs font-mono">
    <!-- Connection Status -->
    <div class="flex items-center gap-2 px-3 py-1 rounded-xl bg-white/[0.03] border border-white/[0.06]">
      <Radio class="w-3.5 h-3.5 {isConnected ? 'text-emerald-400 animate-pulse' : 'text-zinc-500'}" />
      <span class="text-[11px] {isConnected ? 'text-emerald-300 font-semibold' : 'text-zinc-400'}">
        {isConnected ? `DAEMON ONLINE (${latency}ms)` : 'DISCONNECTED'}
      </span>
    </div>

    <!-- RAM Footprint Status -->
    <div class="flex items-center gap-2 px-3 py-1 rounded-xl bg-white/[0.03] border border-white/[0.06]">
      <Cpu class="w-3.5 h-3.5 text-cyan-400" />
      <span class="text-[11px] text-zinc-300">
        RAM: <strong class="text-cyan-300">{memoryStats.estimated_ram_mb} MB</strong> / {memoryStats.budget_limit_mb} MB
      </span>
      <span class="text-[9px] px-1 py-0.5 rounded bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
        &lt;60MB BUDGET
      </span>
    </div>
  </div>

  <!-- Controls & Ephemeral Tray Action -->
  <div class="flex items-center gap-2">
    <div class="hidden sm:flex items-center text-[10px] font-mono text-zinc-500 bg-white/[0.02] px-2 py-1 rounded-lg border border-white/[0.05]">
      <span>Skrót: </span>
      <kbd class="ml-1 px-1.5 py-0.5 rounded bg-white/[0.06] text-zinc-300 font-bold border border-white/10">Ctrl+Shift+Q</kbd>
    </div>

    <button
      onclick={handleMinimize}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/[0.05] hover:bg-white/[0.1] active:scale-95 border border-white/10 text-zinc-200 hover:text-white text-xs font-semibold font-mono transition shadow-sm"
      title="Zniszcz okno WebView i zminimalizuj do zasobnika systemowego, zwalniając pamięć RAM"
    >
      <EyeOff class="w-3.5 h-3.5 text-cyan-400" />
      <span>Tray (~3.8 MB)</span>
    </button>
  </div>
</header>
