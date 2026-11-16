<script lang="ts">
  import { connectionStore } from '../stores/connection.svelte';
  import { Wrench, ShieldAlert, Sparkles, Terminal } from 'lucide-svelte';

  const mcpTools = $derived(connectionStore.mcpTools);
</script>

<div class="p-4 rounded-2xl bg-[#0b101d] border border-white/[0.08] shadow-lg flex flex-col justify-between">
  <div>
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <div class="p-2 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400">
          <Wrench class="w-4 h-4" />
        </div>
        <div>
          <h2 class="text-xs font-bold text-white uppercase tracking-wider font-mono">MCP Capabilities</h2>
          <p class="text-[10px] text-zinc-400 font-mono">Tools, Resources & Elicitation Gates</p>
        </div>
      </div>
      <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-white/[0.04] text-cyan-300 border border-white/10">
        SDK rmcp v1.3
      </span>
    </div>

    <!-- Tools list -->
    <div class="space-y-2">
      {#each mcpTools as tool (tool.name)}
        <div class="p-2.5 rounded-xl bg-white/[0.02] border border-white/[0.05] hover:border-white/10 transition">
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-1.5 font-mono text-xs">
              <Terminal class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
              <span class="font-bold text-white">{tool.name}</span>
            </div>
            <div class="flex items-center gap-1">
              <span class="text-[9px] font-mono px-1.5 py-0.5 rounded bg-white/[0.04] text-zinc-400 border border-white/10">
                {tool.server_id}
              </span>
              {#if tool.requires_elicitation}
                <span class="text-[9px] font-mono px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/30 flex items-center gap-0.5">
                  <ShieldAlert class="w-2.5 h-2.5" />
                  <span>ELICIT</span>
                </span>
              {/if}
            </div>
          </div>
          <p class="text-[11px] text-zinc-400 font-medium font-sans leading-relaxed">{tool.description}</p>
        </div>
      {/each}
    </div>
  </div>

  <div class="mt-3 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[10px] font-mono text-zinc-500">
    <span>Roots: 1 Active Workspace</span>
    <span>Capability ACL Enforced</span>
  </div>
</div>
