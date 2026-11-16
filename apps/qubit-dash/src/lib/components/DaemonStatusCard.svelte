<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Server, Activity, RefreshCw, CheckCircle2, AlertTriangle, ShieldCheck } from 'lucide-svelte';

  const isConnected = $derived(connectionStore.isConnected);
  const isConnecting = $derived(connectionStore.isConnecting);
  const daemonUrl = $derived(connectionStore.daemonUrl);
  const wsUrl = $derived(connectionStore.wsUrl);
  const latency = $derived(connectionStore.latencyMs);

  function reconnect() {
    connectionStore.connectWs();
  }
</script>

<div class="p-4 rounded-2xl bg-[#0b101d] border border-white/[0.08] shadow-lg flex flex-col justify-between">
  <div>
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <div class="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
          <Server class="w-4 h-4" />
        </div>
        <div>
          <h2 class="text-xs font-bold text-white uppercase tracking-wider font-mono">QubitCore Daemon</h2>
          <p class="text-[10px] text-zinc-400 font-mono">Axum Tokio IPC Endpoint</p>
        </div>
      </div>
      <button
        onclick={reconnect}
        disabled={isConnecting}
        class="p-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] text-zinc-400 hover:text-white transition disabled:opacity-50"
        title="Ponów połączenie"
      >
        <RefreshCw class="w-3.5 h-3.5 {isConnecting ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>

    <!-- Status rows -->
    <div class="space-y-2 text-xs font-mono">
      <div class="flex items-center justify-between p-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
        <span class="text-zinc-400">Status Serwera:</span>
        <span class="flex items-center gap-1.5 font-semibold {isConnected ? 'text-emerald-400' : 'text-amber-400'}">
          {#if isConnected}
            <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
            <span>ONLINE</span>
          {:else}
            <AlertTriangle class="w-3.5 h-3.5 text-amber-400" />
            <span>STANDBY (MOCK)</span>
          {/if}
        </span>
      </div>

      <div class="flex items-center justify-between p-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
        <span class="text-zinc-400">Adres HTTP:</span>
        <span class="text-zinc-200">{daemonUrl}</span>
      </div>

      <div class="flex items-center justify-between p-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
        <span class="text-zinc-400">WebSocket:</span>
        <span class="text-zinc-200">{wsUrl}</span>
      </div>

      <div class="flex items-center justify-between p-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
        <span class="text-zinc-400">Opóźnienie IPC:</span>
        <span class="text-cyan-300 font-semibold">{latency} ms</span>
      </div>
    </div>
  </div>

  <div class="mt-3 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[10px] font-mono text-zinc-400">
    <div class="flex items-center gap-1">
      <ShieldCheck class="w-3.5 h-3.5 text-emerald-400" />
      <span>Zero-Trust Sandbox ACL</span>
    </div>
    <span class="text-zinc-500">v0.1.0</span>
  </div>
</div>
