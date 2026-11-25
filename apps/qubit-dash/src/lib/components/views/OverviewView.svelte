<script lang="ts">
  import { Activity, Cpu, Zap, HardDrive, Network, Sparkles, RefreshCw, Layers, ArrowUpRight } from 'lucide-svelte';
  import { connectionStore } from '../../stores/connection.svelte';

  const executionMode = $derived(connectionStore.executionMode);
  const memoryStats = $derived(connectionStore.memoryStats);

  function setMode(mode: 'reflex' | 'deep') {
    connectionStore.setMode(mode);
  }
</script>

<div class="space-y-4">
  <!-- Top Grid: Throughput, Central Plasma Orb & Memory Cap -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
    <!-- Card 1: Runtime Throughput -->
    <div class="hud-card rounded-2xl p-4 flex flex-col justify-between font-mono">
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="text-[10px] text-zinc-400 font-bold tracking-widest uppercase">RUNTIME THROUGHPUT</span>
          <span class="text-[9px] px-1.5 py-0.5 rounded bg-plasma-cyan/10 text-plasma-cyan border border-plasma-cyan/30">REFLEX</span>
        </div>

        <div class="flex items-baseline gap-2 mb-2">
          <span class="text-3xl font-extrabold text-white tracking-tight">1,284</span>
          <span class="text-xs text-plasma-cyan font-bold">tok/m</span>
          <span class="text-[10px] text-zinc-500 ml-auto">Peak: 2,410 tok/m</span>
        </div>

        <!-- Waveform SVG Graphic -->
        <div class="h-20 w-full overflow-hidden relative my-2 rounded-lg bg-plasma-bg/50 border border-white/[0.04]">
          <svg class="w-full h-full" viewBox="0 0 300 80" preserveAspectRatio="none">
            <defs>
              <linearGradient id="cyanGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#00f2fe" stop-opacity="0.35" />
                <stop offset="100%" stop-color="#00f2fe" stop-opacity="0.0" />
              </linearGradient>
            </defs>
            <path
              d="M0,50 Q25,30 50,45 T100,20 T150,35 T200,15 T250,40 T300,25 L300,80 L0,80 Z"
              fill="url(#cyanGrad)"
            />
            <path
              d="M0,50 Q25,30 50,45 T100,20 T150,35 T200,15 T250,40 T300,25"
              fill="none"
              stroke="#00f2fe"
              stroke-width="2"
            />
          </svg>
        </div>
      </div>

      <div class="pt-2 border-t border-white/[0.06] flex items-center justify-between text-[10px] text-zinc-400">
        <span>Token Stream Rate: Sustained</span>
        <span class="text-plasma-mint">Zero Packet Loss</span>
      </div>
    </div>

    <!-- Card 2: Central Quantum Runtime :: Plasma Orb -->
    <div class="hud-card rounded-2xl p-5 flex flex-col items-center justify-between font-mono relative overflow-hidden">
      <!-- Background Ambient Glow -->
      <div class="absolute inset-0 bg-radial from-plasma-cyan/10 via-plasma-purple/5 to-transparent pointer-events-none"></div>

      <div class="w-full flex items-center justify-between z-10">
        <span class="text-[10px] text-zinc-400 font-bold tracking-widest uppercase">QUANTUM RUNTIME</span>
        <span class="text-[10px] text-plasma-cyan font-bold">COHERENCE 99.98%</span>
      </div>

      <!-- Glowing Plasma Orb Simulation -->
      <div class="relative my-4 flex items-center justify-center">
        <!-- Outer Rotating Rings -->
        <div class="w-32 h-32 rounded-full border border-plasma-cyan/30 border-dashed animate-spin-slow absolute"></div>
        <div class="w-28 h-28 rounded-full border border-plasma-purple/40 border-dotted animate-spin-reverse absolute"></div>

        <!-- Glowing Core Orb -->
        <div class="w-20 h-20 rounded-full bg-gradient-to-tr from-plasma-purple via-plasma-cyan to-plasma-mint shadow-plasma-glow flex items-center justify-center animate-pulse-slow">
          <div class="w-16 h-16 rounded-full bg-[#060913] flex flex-col items-center justify-center">
            <Sparkles class="w-5 h-5 text-plasma-cyan mb-0.5" />
            <span class="text-[9px] font-bold text-white uppercase">{executionMode}</span>
          </div>
        </div>
      </div>

      <!-- Mode Selector Pills -->
      <div class="w-full flex items-center justify-center gap-2 z-10">
        <button
          onclick={() => setMode('reflex')}
          class="px-3 py-1 rounded-md text-[10px] font-bold uppercase transition {
            executionMode === 'reflex'
              ? 'bg-plasma-cyan/20 text-plasma-cyan border border-plasma-cyan/40 shadow-plasma-cyan'
              : 'text-zinc-500 hover:text-zinc-300 bg-white/[0.02]'
          }"
        >
          Reflex Mode (&lt;800ms)
        </button>
        <button
          onclick={() => setMode('deep')}
          class="px-3 py-1 rounded-md text-[10px] font-bold uppercase transition {
            executionMode === 'deep'
              ? 'bg-plasma-purple/20 text-plasma-purple border border-plasma-purple/40 shadow-plasma-purple'
              : 'text-zinc-500 hover:text-zinc-300 bg-white/[0.02]'
          }"
        >
          Deep Mode (Sonnet)
        </button>
      </div>
    </div>

    <!-- Card 3: Memory Cap & Allocation -->
    <div class="hud-card rounded-2xl p-4 flex flex-col justify-between font-mono">
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="text-[10px] text-zinc-400 font-bold tracking-widest uppercase">MEMORY CAP</span>
          <span class="text-[10px] text-plasma-mint font-bold">0.80 / 1.00</span>
        </div>

        <div class="flex items-baseline gap-2 mb-2">
          <span class="text-3xl font-extrabold text-white tracking-tight">12.8</span>
          <span class="text-xs text-zinc-400 font-semibold">/ 16.0 GB</span>
          <span class="text-[10px] text-plasma-amber ml-auto">Pool Reserve 80%</span>
        </div>

        <!-- Gauge Bar -->
        <div class="w-full h-2 rounded-full bg-white/[0.05] overflow-hidden my-3 border border-white/[0.06]">
          <div class="h-full bg-gradient-to-r from-plasma-cyan via-plasma-mint to-plasma-amber rounded-full w-[80%]"></div>
        </div>

        <div class="grid grid-cols-2 gap-2 text-[10px] text-zinc-400">
          <div class="p-2 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <span class="text-zinc-500">Backpressure:</span>
            <p class="text-white font-semibold">Active at 80%</p>
          </div>
          <div class="p-2 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <span class="text-zinc-500">OOM Guard:</span>
            <p class="text-emerald-400 font-semibold">Protected</p>
          </div>
        </div>
      </div>

      <div class="pt-2 border-t border-white/[0.06] flex items-center justify-between text-[10px] text-zinc-400">
        <span>cgroups v2 anon memory</span>
        <span class="text-plasma-cyan">28 MB in Core</span>
      </div>
    </div>
  </div>

  <!-- Middle Grid: Anon Pool & Swarm Mesh + Quick Pair -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <!-- Swarm Mesh -->
    <div class="hud-card rounded-2xl p-4 font-mono">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <Network class="w-4 h-4 text-plasma-cyan" />
          <span class="text-xs font-bold text-white uppercase tracking-wider">SWARM MESH :: 03 ACTIVE NODES</span>
        </div>
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
          ALL HEALTHY
        </span>
      </div>

      <div class="grid grid-cols-3 gap-2 text-xs mb-3">
        <div class="p-2.5 rounded-xl bg-white/[0.02] border border-plasma-cyan/20 space-y-1">
          <div class="flex items-center justify-between">
            <span class="font-bold text-white">N1-PRIMARY</span>
            <span class="w-2 h-2 rounded-full bg-plasma-cyan"></span>
          </div>
          <p class="text-[10px] text-zinc-400">Local Daemon</p>
          <p class="text-[9px] text-plasma-cyan">0.4ms • 3.8 MB</p>
        </div>

        <div class="p-2.5 rounded-xl bg-white/[0.02] border border-white/[0.06] space-y-1">
          <div class="flex items-center justify-between">
            <span class="font-bold text-white">N2-RTX4090</span>
            <span class="w-2 h-2 rounded-full bg-plasma-mint"></span>
          </div>
          <p class="text-[10px] text-zinc-400">Home Compute</p>
          <p class="text-[9px] text-plasma-mint">12ms • Ollama IPC</p>
        </div>

        <div class="p-2.5 rounded-xl bg-white/[0.02] border border-white/[0.06] space-y-1">
          <div class="flex items-center justify-between">
            <span class="font-bold text-white">N3-BRIDGE</span>
            <span class="w-2 h-2 rounded-full bg-plasma-purple"></span>
          </div>
          <p class="text-[10px] text-zinc-400">VS Code Root</p>
          <p class="text-[9px] text-plasma-purple">1.2ms • rmcp stdio</p>
        </div>
      </div>

      <div class="flex items-center justify-between text-[10px] text-zinc-400 pt-2 border-t border-white/[0.05]">
        <span>Mesh Encryption: Noise Protocol XX</span>
        <span class="text-zinc-500">Auto-Discovery ON</span>
      </div>
    </div>

    <!-- Connected Ecosystem & Throughput -->
    <div class="hud-card rounded-2xl p-4 font-mono flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Layers class="w-4 h-4 text-plasma-purple" />
            <span class="text-xs font-bold text-white uppercase tracking-wider">CONNECTED ECOSYSTEM</span>
          </div>
          <button class="flex items-center gap-1 text-[10px] text-plasma-cyan hover:text-white transition">
            <span>+ Pair Node</span>
            <ArrowUpRight class="w-3 h-3" />
          </button>
        </div>

        <div class="space-y-2 text-xs">
          <div class="flex items-center justify-between p-2 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <span class="text-zinc-300">QubitDash Tauri v2</span>
            <span class="text-plasma-cyan font-semibold">Active Window (Ephemeral)</span>
          </div>
          <div class="flex items-center justify-between p-2 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <span class="text-zinc-300">Google Antigravity CLI</span>
            <span class="text-plasma-mint font-semibold">Zero-Cost Token Offload</span>
          </div>
          <div class="flex items-center justify-between p-2 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <span class="text-zinc-300">Automated Background Routines</span>
            <span class="text-white font-semibold">148 Scheduled Tasks</span>
          </div>
        </div>
      </div>

      <div class="pt-3 border-t border-white/[0.05] flex items-center justify-between text-[10px] text-zinc-400">
        <span>Daily Run Rate: <strong class="text-emerald-400">$0.00 / 24h</strong></span>
        <span class="text-plasma-cyan">100% Free Offloading</span>
      </div>
    </div>
  </div>
</div>
