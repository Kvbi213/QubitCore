<script lang="ts">
  import { NAV_ITEMS, type ActiveTab } from '../types/navigation';
  import { Cpu, ShieldCheck, Zap, Activity, HardDrive, Lock } from 'lucide-svelte';

  interface Props {
    activeTab: ActiveTab;
    onSelect: (tab: ActiveTab) => void;
  }

  let { activeTab, onSelect }: Props = $props();
</script>

<aside class="w-64 border-r border-plasma-cyan/15 bg-plasma-panel/95 flex flex-col justify-between shrink-0 font-mono select-none">
  <!-- Nav Items (Control Hierarchy) -->
  <div class="p-3 space-y-1">
    <div class="px-3 py-2 text-[10px] font-bold tracking-widest text-zinc-500 uppercase flex items-center justify-between">
      <span>CONTROL HIERARCHY</span>
      <span class="text-plasma-cyan text-[9px]">LIVE</span>
    </div>

    {#each NAV_ITEMS as item (item.id)}
      <button
        onclick={() => onSelect(item.id)}
        class="w-full px-3 py-2.5 rounded-lg text-xs font-semibold text-left transition flex items-center justify-between {
          activeTab === item.id
            ? 'bg-plasma-cyan/10 text-white border border-plasma-cyan/40 shadow-plasma-cyan'
            : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.03] border border-transparent'
        }"
      >
        <div class="flex items-center gap-2.5">
          <span class="text-[10px] {activeTab === item.id ? 'text-plasma-cyan font-bold' : 'text-zinc-600'}">
            {item.number}
          </span>
          <span class="tracking-wide">{item.label}</span>
        </div>

        {#if item.badge}
          <span
            class="text-[9px] px-1.5 py-0.5 rounded font-bold uppercase {
              item.badgeColor === 'mint'
                ? 'bg-plasma-mint/10 text-plasma-mint border border-plasma-mint/30'
                : 'bg-plasma-cyan/10 text-plasma-cyan border border-plasma-cyan/30'
            }"
          >
            {item.badge}
          </span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Bottom Hardware & Telemetry Widgets -->
  <div class="p-3 space-y-3 border-t border-white/[0.08] text-[10px]">
    <!-- TOKIO TASK LOAD -->
    <div class="p-2.5 rounded-lg bg-plasma-bg border border-white/[0.06] space-y-2">
      <div class="flex items-center justify-between text-zinc-400 font-bold">
        <span class="flex items-center gap-1.5 text-zinc-300">
          <Activity class="w-3.5 h-3.5 text-plasma-cyan" />
          <span>TOKIO TASK LOAD</span>
        </span>
        <span class="text-plasma-cyan">28.4%</span>
      </div>

      <!-- Mini Load Bar -->
      <div class="w-full h-1.5 rounded-full bg-white/[0.05] overflow-hidden">
        <div class="h-full bg-gradient-to-r from-plasma-cyan to-plasma-mint rounded-full w-[28.4%]"></div>
      </div>

      <div class="flex items-center justify-between text-zinc-500 text-[9px]">
        <span>Pool: 16 Workers</span>
        <span>IPC: 142.8 MB/s</span>
      </div>
    </div>

    <!-- SECURITY VAULT -->
    <div class="p-2.5 rounded-lg bg-plasma-bg border border-white/[0.06] space-y-1.5">
      <div class="flex items-center justify-between text-zinc-400 font-bold">
        <span class="flex items-center gap-1.5 text-zinc-300">
          <Lock class="w-3.5 h-3.5 text-plasma-purple" />
          <span>SECURITY VAULT</span>
        </span>
        <span class="text-plasma-mint text-[9px]">ISOLATED</span>
      </div>

      <p class="text-[9px] text-zinc-500 font-mono leading-tight">
        Enclave: AES-256-GCM in memory (/dev/shm)
      </p>

      <div class="flex items-center justify-between text-[9px] text-zinc-400 pt-1 border-t border-white/[0.04]">
        <span>Zero Leaks Guard</span>
        <span class="text-emerald-400 font-bold">PASS</span>
      </div>
    </div>
  </div>
</aside>
