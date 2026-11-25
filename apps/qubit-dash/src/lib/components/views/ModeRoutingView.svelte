<script lang="ts">
  import { Zap, Brain, Layers, Sliders, ShieldCheck, Check, Sparkles, AlertTriangle } from 'lucide-svelte';
  import { connectionStore } from '../../stores/connection.svelte';

  let currentMode = $state(connectionStore.executionMode);
  let autoOffload = $state(true);
  let requireElicitation = $state(true);
  let compressionThreshold = $state(50);
  let protectedMessages = $state(20);

  let keywords = $state([
    'dekompiluj',
    'reverse engineering',
    'przeanalizuj cały projekt',
    'ghidra_headless',
    'audyt podatności cve',
  ]);
  let newKeyword = $state('');

  let saveSuccess = $state(false);

  function addKeyword() {
    if (newKeyword.trim() && !keywords.includes(newKeyword.trim())) {
      keywords = [...keywords, newKeyword.trim()];
      newKeyword = '';
    }
  }

  function removeKeyword(kw: string) {
    keywords = keywords.filter((k) => k !== kw);
  }

  function saveConfig() {
    connectionStore.setMode(currentMode);
    saveSuccess = true;
    setTimeout(() => {
      saveSuccess = false;
    }, 2500);
  }
</script>

<div class="space-y-4 font-mono">
  <!-- Section Title -->
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
        <Sliders class="w-4 h-4 text-plasma-cyan" />
        <span>INTELLIGENT MODE ROUTING & AUTO-ESCALATION</span>
      </h2>
      <p class="text-[11px] text-zinc-400 font-sans">
        Deterministyczny podział zadań na pętle Reflex, Deep oraz 4-fazową kompresję kontekstu Hermes.
      </p>
    </div>
    {#if saveSuccess}
      <span class="text-[10px] px-2.5 py-1 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 font-bold flex items-center gap-1">
        <Check class="w-3 h-3" />
        <span>SAVED TO QUBIT_CORE</span>
      </span>
    {/if}
  </div>

  <!-- 3 Main Mode Selection Cards -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <!-- Reflex Mode -->
    <button
      onclick={() => (currentMode = 'reflex')}
      class="hud-card rounded-2xl p-4 text-left transition relative {
        currentMode === 'reflex'
          ? 'border-plasma-cyan/60 shadow-plasma-cyan bg-plasma-cyan/5'
          : 'border-white/[0.08] hover:border-white/20'
      }"
    >
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-plasma-cyan/10 text-plasma-cyan">
            <Zap class="w-4 h-4" />
          </div>
          <span class="text-xs font-bold text-white uppercase">Tryb Lekki (Reflex)</span>
        </div>
        {#if currentMode === 'reflex'}
          <span class="text-[9px] font-bold text-plasma-cyan bg-plasma-cyan/10 px-1.5 py-0.5 rounded border border-plasma-cyan/30">
            ACTIVE
          </span>
        {/if}
      </div>

      <p class="text-[11px] text-zinc-400 font-sans leading-relaxed mb-3">
        Bezpośrednia realizacja pytań, szybki kod i diagnostyka w czasie &lt;800ms. 100% zero-cost przez Antigravity CLI.
      </p>

      <div class="space-y-1 text-[10px] text-zinc-500 pt-2 border-t border-white/[0.04]">
        <div class="flex justify-between"><span>Latency:</span><strong class="text-plasma-cyan">&lt;800 ms</strong></div>
        <div class="flex justify-between"><span>Model:</span><span class="text-zinc-300">gemini-2.5-flash</span></div>
        <div class="flex justify-between"><span>Sub-agenci:</span><span class="text-zinc-400">Wyłączeni</span></div>
      </div>
    </button>

    <!-- Deep Mode -->
    <button
      onclick={() => (currentMode = 'deep')}
      class="hud-card rounded-2xl p-4 text-left transition relative {
        currentMode === 'deep'
          ? 'border-plasma-purple/60 shadow-plasma-purple bg-plasma-purple/5'
          : 'border-white/[0.08] hover:border-white/20'
      }"
    >
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-plasma-purple/10 text-plasma-purple">
            <Brain class="w-4 h-4" />
          </div>
          <span class="text-xs font-bold text-white uppercase">Tryb Ciężki (Deep)</span>
        </div>
        {#if currentMode === 'deep'}
          <span class="text-[9px] font-bold text-plasma-purple bg-plasma-purple/10 px-1.5 py-0.5 rounded border border-plasma-purple/30">
            ACTIVE
          </span>
        {/if}
      </div>

      <p class="text-[11px] text-zinc-400 font-sans leading-relaxed mb-3">
        Wieloetapowe wnioskowanie, orkiestracja sub-agentów, inżynieria wsteczna Ghidra i analiza całych repozytoriów.
      </p>

      <div class="space-y-1 text-[10px] text-zinc-500 pt-2 border-t border-white/[0.04]">
        <div class="flex justify-between"><span>Routing:</span><strong class="text-plasma-purple">Claude 3.7 Sonnet</strong></div>
        <div class="flex justify-between"><span>Bramka Elicitation:</span><span class="text-emerald-400">Wymagana</span></div>
        <div class="flex justify-between"><span>Piaskownica:</span><span class="text-zinc-300">cgroups v2 isol</span></div>
      </div>
    </button>

    <!-- Context Compressor -->
    <div class="hud-card rounded-2xl p-4 text-left border-white/[0.08]">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-plasma-mint/10 text-plasma-mint">
            <Layers class="w-4 h-4" />
          </div>
          <span class="text-xs font-bold text-white uppercase">Hermes Compressor</span>
        </div>
        <span class="text-[9px] font-bold text-plasma-mint bg-plasma-mint/10 px-1.5 py-0.5 rounded border border-plasma-mint/30">
          DAEMON 24/7
        </span>
      </div>

      <p class="text-[11px] text-zinc-400 font-sans leading-relaxed mb-3">
        4-fazowy algorytm selektywnej redukcji tokenów z zabezpieczeniem ostatnich 20 wiadomości i deterministycznym fallbackiem.
      </p>

      <div class="space-y-1 text-[10px] text-zinc-500 pt-2 border-t border-white/[0.04]">
        <div class="flex justify-between"><span>Próg aktywacji:</span><strong class="text-plasma-mint">{compressionThreshold}%</strong></div>
        <div class="flex justify-between"><span>Silnik streszczeń:</span><span class="text-zinc-300">Ollama IPC Local</span></div>
        <div class="flex justify-between"><span>Ochrona:</span><span class="text-zinc-300">Last 20 tails</span></div>
      </div>
    </div>
  </div>

  <!-- Auto-Escalation Rules & Keyword Triggers -->
  <div class="hud-card rounded-2xl p-4 space-y-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-plasma-cyan"></span>
        <span class="text-xs font-bold text-white uppercase tracking-wider">AUTO-ESCALATION TRIGGERS (REGEX & KEYWORDS)</span>
      </div>
      <label class="flex items-center gap-2 text-xs text-zinc-300 cursor-pointer">
        <input type="checkbox" bind:checked={autoOffload} class="accent-plasma-cyan" />
        <span>Auto-Offload na Heavy Compute Node</span>
      </label>
    </div>

    <div class="flex flex-wrap gap-2 items-center">
      {#each keywords as kw}
        <span class="px-2.5 py-1 rounded-lg bg-plasma-cyan/10 border border-plasma-cyan/30 text-plasma-cyan text-xs flex items-center gap-1.5 font-mono">
          <span>{kw}</span>
          <button onclick={() => removeKeyword(kw)} class="hover:text-white transition">×</button>
        </span>
      {/each}

      <div class="flex items-center gap-1">
        <input
          type="text"
          placeholder="+ Dodaj słowo kluczowe..."
          bind:value={newKeyword}
          onkeydown={(e) => e.key === 'Enter' && addKeyword()}
          class="px-2.5 py-1 rounded-lg bg-plasma-bg border border-white/[0.1] text-xs text-zinc-200 outline-none focus:border-plasma-cyan w-48"
        />
        <button onclick={addKeyword} class="px-2 py-1 rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-xs text-zinc-300 transition">
          +
        </button>
      </div>
    </div>
  </div>

  <!-- Compression Tuning Sliders & Save Button -->
  <div class="hud-card rounded-2xl p-4 space-y-4">
    <div class="flex items-center justify-between">
      <span class="text-xs font-bold text-white uppercase tracking-wider">PARAMETRY KOMPRESJI I NIEZAWODNOŚCI</span>
      <span class="text-[10px] text-zinc-400">Hermes Protocol Spec 1.1</span>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="space-y-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-zinc-400">Próg kompresji okna kontekstu:</span>
          <span class="text-plasma-cyan font-bold">{compressionThreshold}% budżetu</span>
        </div>
        <input
          type="range"
          min="30"
          max="80"
          bind:value={compressionThreshold}
          class="w-full accent-plasma-cyan"
        />
        <p class="text-[10px] text-zinc-500">
          Uruchamia procedurę przyrostowego czyszczenia po zapełnieniu połowy dozwolonego okna tokenów.
        </p>
      </div>

      <div class="space-y-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-zinc-400">Chronione ogony konwersacji (Tails):</span>
          <span class="text-plasma-purple font-bold">{protectedMessages} wiadomości</span>
        </div>
        <input
          type="range"
          min="10"
          max="50"
          bind:value={protectedMessages}
          class="w-full accent-plasma-purple"
        />
        <p class="text-[10px] text-zinc-500">
          Ostatnie wiadomości nigdy nie podlegają kompresji w celu zachowania natychmiastowego kontekstu.
        </p>
      </div>
    </div>

    <div class="pt-3 border-t border-white/[0.06] flex items-center justify-between">
      <div class="flex items-center gap-2 text-[10px] text-zinc-400">
        <ShieldCheck class="w-3.5 h-3.5 text-plasma-mint" />
        <span>Deterministic Fallback Ladder: Ollama IPC &rarr; Antigravity CLI &rarr; Direct API</span>
      </div>

      <button
        onclick={saveConfig}
        class="px-4 py-2 rounded-xl bg-gradient-to-r from-plasma-cyan to-plasma-purple text-[#060913] hover:opacity-90 active:scale-95 text-xs font-bold transition shadow-plasma-cyan"
      >
        Zapisz Konfigurację Routingu
      </button>
    </div>
  </div>
</div>
