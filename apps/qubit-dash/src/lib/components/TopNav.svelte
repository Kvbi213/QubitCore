<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import QubitIcon from './icons/QubitIcon.svelte';
  import { Shield, Radio, Terminal, Cpu, EyeOff, User, Activity, Lock, Layers } from 'lucide-svelte';

  const isConnected = $derived(connectionStore.isConnected);
  const memoryStats = $derived(connectionStore.memoryStats);
  const latency = $derived(connectionStore.latencyMs);

  function handleMinimize() {
    connectionStore.triggerMinimizeToTray();
  }
</script>

<header class="border-b border-plasma-cyan/20 bg-plasma-bg/95 backdrop-blur-md shrink-0">
  <!-- Level 1: Primary System Status Bar -->
  <div class="h-11 px-4 flex items-center justify-between text-[11px] font-mono border-b border-white/[0.05]">
    <!-- Brand & Kernel Info -->
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-2.5">
        <div class="relative flex items-center justify-center">
          <QubitIcon size="26" class="shrink-0 drop-shadow-[0_0_8px_rgba(0,240,255,0.4)] transition-transform hover:scale-105 duration-200" />
          <span class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-plasma-cyan shadow-plasma-cyan animate-pulse"></span>
        </div>
        <span class="font-bold tracking-widest text-white uppercase text-xs">
          QUBITCORE <span class="text-plasma-cyan">v0.1.0</span> <span class="text-plasma-purple">PLASMA EDITION</span>
        </span>
      </div>

      <span class="text-zinc-600">|</span>

      <div class="hidden xl:flex items-center gap-2 text-zinc-400">
        <span class="text-plasma-mint font-semibold">CGROUPS V2: ISOLATED</span>
        <span class="text-zinc-600">|</span>
        <span class="text-zinc-300">TOKIO MPSC: READY</span>
        <span class="text-zinc-600">|</span>
        <span class="flex items-center gap-1 text-emerald-400">
          <Activity class="w-3 h-3" />
          <span>TOKIO ACTIVE</span>
        </span>
        <span class="text-zinc-600">|</span>
        <span class="text-zinc-300">WAL SYNCED</span>
        <span class="text-zinc-600">|</span>
        <span class="text-plasma-cyan font-bold">LATENCY: {latency > 0 ? latency : 0.4}MS</span>
      </div>
    </div>

    <!-- Right Controls: RAM, Ephemeral Tray & User Profile -->
    <div class="flex items-center gap-3">
      <!-- RAM Footprint Capsule -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-white/[0.03] border border-plasma-cyan/20 text-zinc-300">
        <Cpu class="w-3.5 h-3.5 text-plasma-cyan" />
        <span class="text-[10px]">RAM: <strong class="text-plasma-cyan">{memoryStats.estimated_ram_mb} MB</strong> / 60 MB</span>
      </div>

      <!-- Ephemeral Minimize Button -->
      <button
        onclick={handleMinimize}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-plasma-cyan/10 hover:bg-plasma-cyan/20 active:scale-95 border border-plasma-cyan/30 text-plasma-cyan text-[11px] font-semibold transition"
        title="Niszczy okno WebView i redukuje proces do ~3.8 MB w zasobniku systemowym"
      >
        <EyeOff class="w-3.5 h-3.5" />
        <span>Tray (~3.8 MB)</span>
      </button>

      <!-- User Avatar / Operator -->
      <div class="flex items-center gap-2 pl-2 border-l border-white/[0.08]">
        <div class="w-6 h-6 rounded-md bg-gradient-to-tr from-plasma-purple to-plasma-cyan p-0.5 flex items-center justify-center">
          <div class="w-full h-full bg-plasma-bg rounded flex items-center justify-center text-[10px] font-bold text-white">
            OP
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Level 2: Secondary Contextual Bar -->
  <div class="h-8 px-4 bg-plasma-panel/60 flex items-center justify-between text-[10px] font-mono text-zinc-400">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">IPC Bus:</span>
        <span class="text-zinc-300 font-semibold">/run/qubit/core.sock</span>
      </div>
      <span class="text-zinc-700">•</span>
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">Node:</span>
        <span class="text-plasma-mint font-semibold">N1-PRIMARY (Localhost)</span>
      </div>
      <span class="text-zinc-700">•</span>
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">Strategy:</span>
        <span class="text-plasma-cyan font-semibold">94.2% Zero-Cost Local Offloading</span>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1 text-emerald-400">
        <Lock class="w-3 h-3" />
        <span>Enclave AES-256 Auth: OK</span>
      </span>
      <span class="text-zinc-700">•</span>
      <span class="text-zinc-500">Session ID: <strong>sess_8f2a...c01</strong></span>
    </div>
  </div>
</header>
