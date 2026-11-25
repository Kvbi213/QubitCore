<script lang="ts">
  import TopNav from './lib/components/TopNav.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import OverviewView from './lib/components/views/OverviewView.svelte';
  import ProvidersAuthView from './lib/components/views/ProvidersAuthView.svelte';
  import ModeRoutingView from './lib/components/views/ModeRoutingView.svelte';
  import RoutinesView from './lib/components/views/RoutinesView.svelte';
  import SecurityStorageView from './lib/components/views/SecurityStorageView.svelte';
  import type { ActiveTab } from './lib/types/navigation';
  import { onMount } from 'svelte';
  import { connectionStore } from './lib/stores/connection.svelte';

  let activeTab = $state<ActiveTab>('overview');

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

<div class="h-screen w-screen flex flex-col bg-plasma-bg text-zinc-100 overflow-hidden font-sans">
  <!-- Top Navigation Bar -->
  <TopNav />

  <!-- Main Work Area with Sidebar -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Left Sidebar: Control Hierarchy -->
    <Sidebar {activeTab} onSelect={(tab) => (activeTab = tab)} />

    <!-- Active Viewport -->
    <main class="flex-1 overflow-y-auto p-4 sm:p-5">
      {#if activeTab === 'overview'}
        <OverviewView />
      {:else if activeTab === 'providers'}
        <ProvidersAuthView />
      {:else if activeTab === 'routing'}
        <ModeRoutingView />
      {:else if activeTab === 'routines'}
        <RoutinesView />
      {:else if activeTab === 'security'}
        <SecurityStorageView />
      {/if}
    </main>
  </div>
</div>
