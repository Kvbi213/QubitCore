<script lang="ts">
  import { Shield, Key, Terminal, Cpu, CheckCircle2, Zap, AlertCircle, RefreshCw, Layers, DollarSign } from 'lucide-svelte';

  let agyActive = $state(true);
  let claudeActive = $state(true);

  let geminiKey = $state('AIzaSyD-••••••••••••••••••••••••••••');
  let anthropicKey = $state('sk-ant-••••••••••••••••••••••••••••');
  let openrouterKey = $state('sk-or-••••••••••••••••••••••••••••');
</script>

<div class="space-y-4 font-mono">
  <!-- Section Header -->
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
        <Key class="w-4 h-4 text-plasma-cyan" />
        <span>ZERO-API-COST OFFLOADING FABRIC</span>
      </h2>
      <p class="text-[11px] text-zinc-400 font-sans">
        Subskrypcyjne mostki CLI oraz zabezpieczone enklawy zapasowe z bezpośrednimi kluczami API.
      </p>
    </div>
    <span class="text-[10px] px-2.5 py-1 rounded-md bg-plasma-mint/10 text-plasma-mint border border-plasma-mint/30 font-bold">
      OFFLOAD RATIO: 94.2%
    </span>
  </div>

  <!-- STAGE 01: Local CLI Harnesses & Subscription Bridges -->
  <div class="space-y-3">
    <div class="flex items-center gap-2 text-xs font-bold text-zinc-400 uppercase tracking-widest">
      <span class="w-2 h-2 rounded-full bg-plasma-cyan"></span>
      <span>STAGE 01 :: LOCAL CLI HARNESSES & SUBSCRIPTION BRIDGES</span>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Google Antigravity CLI (agy) -->
      <div class="hud-card rounded-2xl p-4 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 rounded-xl bg-plasma-cyan/10 border border-plasma-cyan/30 flex items-center justify-center font-bold text-plasma-cyan">
                AGY
              </div>
              <div>
                <h3 class="text-xs font-bold text-white uppercase">Google Antigravity CLI</h3>
                <span class="text-[10px] text-zinc-500">Binary: /usr/local/bin/agy</span>
              </div>
            </div>

            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" bind:checked={agyActive} class="sr-only peer" />
              <div class="w-9 h-5 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-plasma-cyan"></div>
            </label>
          </div>

          <!-- Specs Table -->
          <div class="space-y-1.5 text-xs">
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Tożsamość sesyjna:</span>
              <span class="text-zinc-300">jakub@kvbi.pl (OAuth PKCE)</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Czas wygaśnięcia (TTL):</span>
              <span class="text-plasma-mint font-semibold">14h 22m (Auto-Refresh)</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Opóźnienie IPC:</span>
              <span class="text-plasma-cyan font-bold">98 ms</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Oszczędności (24h):</span>
              <span class="text-emerald-400 font-bold">+$12.80 USD saved</span>
            </div>
          </div>

          <!-- Mini Bar Chart SVG -->
          <div class="my-3 p-2 rounded-lg bg-plasma-bg border border-white/[0.04]">
            <div class="flex items-center justify-between text-[9px] text-zinc-500 mb-1">
              <span>Token Volume Offloaded (Hourly)</span>
              <span class="text-plasma-cyan font-bold">428k tokens</span>
            </div>
            <div class="flex items-end gap-1.5 h-10 w-full pt-1">
              <div class="w-full bg-plasma-cyan/30 rounded-t h-[35%]"></div>
              <div class="w-full bg-plasma-cyan/40 rounded-t h-[60%]"></div>
              <div class="w-full bg-plasma-cyan/50 rounded-t h-[45%]"></div>
              <div class="w-full bg-plasma-cyan/70 rounded-t h-[80%]"></div>
              <div class="w-full bg-plasma-cyan/90 rounded-t h-[100%] shadow-plasma-cyan"></div>
              <div class="w-full bg-plasma-cyan/60 rounded-t h-[55%]"></div>
            </div>
          </div>
        </div>

        <div class="pt-2 border-t border-white/[0.06] flex items-center justify-between text-[10px]">
          <span class="text-zinc-400">IPC Socket: <strong class="text-zinc-300">READY</strong></span>
          <button class="px-2 py-1 rounded bg-white/[0.04] hover:bg-white/[0.08] text-plasma-cyan transition">
            Test Sync
          </button>
        </div>
      </div>

      <!-- Claude Code CLI (claude) -->
      <div class="hud-card rounded-2xl p-4 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 rounded-xl bg-plasma-purple/10 border border-plasma-purple/30 flex items-center justify-center font-bold text-plasma-purple">
                CLD
              </div>
              <div>
                <h3 class="text-xs font-bold text-white uppercase">Claude Code CLI Bridge</h3>
                <span class="text-[10px] text-zinc-500">Binary: ~/.local/bin/claude</span>
              </div>
            </div>

            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" bind:checked={claudeActive} class="sr-only peer" />
              <div class="w-9 h-5 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-plasma-purple"></div>
            </label>
          </div>

          <div class="space-y-1.5 text-xs">
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Metoda autoryzacji:</span>
              <span class="text-zinc-300">Claude Max Subscription</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Runtime bridge:</span>
              <span class="text-plasma-mint font-semibold">Native Stdio Stream</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Opóźnienie IPC:</span>
              <span class="text-plasma-purple font-bold">184 ms</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded bg-white/[0.02]">
              <span class="text-zinc-500">Keychain status:</span>
              <span class="text-emerald-400 font-bold">ENCLAVE SECURED</span>
            </div>
          </div>

          <!-- Status Box -->
          <div class="my-3 p-3 rounded-xl bg-plasma-bg border border-white/[0.04] text-[11px] text-zinc-400 space-y-1">
            <div class="flex items-center justify-between text-zinc-300 font-bold">
              <span>Claude 3.7 Sonnet Offload</span>
              <span class="text-emerald-400">UNLIMITED TIER</span>
            </div>
            <p class="text-[10px] text-zinc-500">
              Zapytania w trybie Deep Reasoning przekierowywane są do subskrypcji użytkownika z pominięciem opłat za tokeny API.
            </p>
          </div>
        </div>

        <div class="pt-2 border-t border-white/[0.06] flex items-center justify-between text-[10px]">
          <span class="text-zinc-400">Active Pipe: <strong class="text-zinc-300">claude.ipc</strong></span>
          <button class="px-2 py-1 rounded bg-white/[0.04] hover:bg-white/[0.08] text-plasma-purple transition">
            Ping Bridge
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- STAGE 02: Direct API Fallback Enclaves -->
  <div class="space-y-3 pt-2">
    <div class="flex items-center gap-2 text-xs font-bold text-zinc-400 uppercase tracking-widest">
      <span class="w-2 h-2 rounded-full bg-plasma-purple"></span>
      <span>STAGE 02 :: DIRECT API FALLBACK ENCLAVES</span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
      <!-- Gemini API -->
      <div class="hud-card rounded-xl p-3 space-y-2">
        <div class="flex items-center justify-between">
          <span class="font-bold text-white text-xs">Gemini API</span>
          <span class="text-[9px] px-1 py-0.5 rounded bg-plasma-cyan/10 text-plasma-cyan border border-plasma-cyan/20">gemini-2.5-flash</span>
        </div>
        <input
          type="password"
          bind:value={geminiKey}
          class="w-full px-2 py-1 rounded bg-plasma-bg border border-white/[0.08] text-[10px] text-zinc-300 outline-none focus:border-plasma-cyan"
        />
        <div class="flex items-center justify-between text-[9px] text-zinc-400 pt-1 border-t border-white/[0.04]">
          <span>Ping: 112ms</span>
          <span class="text-emerald-400 font-bold">STANDBY</span>
        </div>
      </div>

      <!-- Anthropic API -->
      <div class="hud-card rounded-xl p-3 space-y-2">
        <div class="flex items-center justify-between">
          <span class="font-bold text-white text-xs">Anthropic API</span>
          <span class="text-[9px] px-1 py-0.5 rounded bg-plasma-purple/10 text-plasma-purple border border-plasma-purple/20">claude-3-5-haiku</span>
        </div>
        <input
          type="password"
          bind:value={anthropicKey}
          class="w-full px-2 py-1 rounded bg-plasma-bg border border-white/[0.08] text-[10px] text-zinc-300 outline-none focus:border-plasma-purple"
        />
        <div class="flex items-center justify-between text-[9px] text-zinc-400 pt-1 border-t border-white/[0.04]">
          <span>Ping: 195ms</span>
          <span class="text-emerald-400 font-bold">STANDBY</span>
        </div>
      </div>

      <!-- OpenRouter -->
      <div class="hud-card rounded-xl p-3 space-y-2">
        <div class="flex items-center justify-between">
          <span class="font-bold text-white text-xs">OpenRouter</span>
          <span class="text-[9px] px-1 py-0.5 rounded bg-plasma-mint/10 text-plasma-mint border border-plasma-mint/20">llama-3.3-70b</span>
        </div>
        <input
          type="password"
          bind:value={openrouterKey}
          class="w-full px-2 py-1 rounded bg-plasma-bg border border-white/[0.08] text-[10px] text-zinc-300 outline-none focus:border-plasma-mint"
        />
        <div class="flex items-center justify-between text-[9px] text-zinc-400 pt-1 border-t border-white/[0.04]">
          <span>Ping: 240ms</span>
          <span class="text-zinc-400 font-bold">READY</span>
        </div>
      </div>

      <!-- Local Ollama IPC -->
      <div class="hud-card rounded-xl p-3 space-y-2">
        <div class="flex items-center justify-between">
          <span class="font-bold text-white text-xs">Local Ollama</span>
          <span class="text-[9px] px-1 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">qwen2.5-coder:7b</span>
        </div>
        <div class="px-2 py-1 rounded bg-plasma-bg border border-white/[0.08] text-[10px] text-zinc-400">
          Endpoint: http://127.0.0.1:11434
        </div>
        <div class="flex items-center justify-between text-[9px] text-zinc-400 pt-1 border-t border-white/[0.04]">
          <span>VRAM: 8.2 / 16 GB</span>
          <span class="text-plasma-cyan font-bold">LOCAL ZERO-COST</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Bottom Run Rate & Pipeline Health Bar -->
  <div class="hud-card rounded-xl p-3 flex flex-col sm:flex-row items-center justify-between gap-3 text-xs">
    <div class="flex items-center gap-2">
      <div class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></div>
      <span class="text-zinc-300 font-bold">Pipeline Health: 94.2% Routed to Zero-Cost Bridges</span>
    </div>
    <div class="flex items-center gap-3">
      <span class="text-zinc-400">Current Run Rate: <strong class="text-emerald-400">$0.00 / 24h incurred</strong></span>
      <span class="text-zinc-500 text-[10px]">Vault Guard Active</span>
    </div>
  </div>
</div>
