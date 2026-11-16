<script lang="ts">
  import Header from './lib/components/Header.svelte';
  import DaemonStatusCard from './lib/components/DaemonStatusCard.svelte';
  import MemoryBudgetGauge from './lib/components/MemoryBudgetGauge.svelte';
  import AgentListCard from './lib/components/AgentListCard.svelte';
  import McpToolsCard from './lib/components/McpToolsCard.svelte';
  import LiveEventsConsole from './lib/components/LiveEventsConsole.svelte';
  import { onMount } from 'svelte';
  import { connectionStore } from './lib/stores/connection.svelte';

  onMount(() => {
    // Keyboard shortcut handler for Webview inside app
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'q' || e.key === 'Q')) {
        e.preventDefault();
        connectionStore.triggerMinimizeToTray();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<div class="h-screen w-screen flex flex-col bg-[#070a12] text-zinc-100 overflow-hidden">
  <!-- Top Navigation Bar -->
  <Header />

  <!-- Mission Control Dashboard Body -->
  <main class="flex-1 overflow-y-auto p-4 sm:p-5 space-y-4">
    <!-- Top Deck: System Core Status & Memory Gauge -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <DaemonStatusCard />
      <MemoryBudgetGauge />
    </div>

    <!-- Middle Deck: Agent Pool & MCP Capabilities -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <AgentListCard />
      <McpToolsCard />
    </div>

    <!-- Bottom Deck: Real-Time Telemetry Stream -->
    <div>
      <LiveEventsConsole />
    </div>
  </main>
</div>
