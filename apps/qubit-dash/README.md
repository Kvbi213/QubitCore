# QubitDash (`apps/qubit-dash`)

Lightweight, modern dashboard and mission control interface for QubitCore.

## Features & Architecture
- **Ephemeral WebView Pattern (Tauri v2)**:
  - Destroys the WebKit/WebView2 window on close/minimize to system tray (`WindowEvent::CloseRequested` interception via `destroy()`).
  - Reduces idle background memory footprint to ~3–5 MB RAM (pure Rust Tokio process).
  - Sub-second instant restoration upon left-clicking the System Tray icon or pressing the global shortcut.
- **System Tray Integration**:
  - Context menu with options: "Pokaż / Przywróć Pulpit", "Zminimalizuj do Tray (Zwolnij RAM)", telemetry memory status indicator, and clean exit.
  - Custom brand icon (cyan/purple quantum atom).
- **Global Shortcut**:
  - Registered shortcut: `CommandOrControl+Shift+Q` (with in-app webview fallback) for quick toggle of Mission Control.
- **Svelte 5 + Runes**:
  - Reactive runes state management (`$state`, `$derived`, `$effect`).
  - Svelte 5 + TailwindCSS dark-tech aesthetic (`#070a12` dark porcelain velvet, neon cyan `#00f0ff`, purple `#8a2be2`).
- **Mission Control Panels**:
  - **Daemon Status Card**: Real-time status for QubitCore Axum daemon on `localhost:3000` (IPC latency, WS streaming).
  - **Memory Budget Gauge**: Strict monitoring against the <60 MB RAM budget with breakdown of Rust backend vs WebView engine.
  - **Qubit Agents Pool**: State tracking for `Supervisor-01`, `Hermes-Compressor`, and `Reversing-Specialist` with Reflex vs Deep router pills.
  - **MCP Capabilities Card**: Tools, Resources, Roots, and Human Elicitation gate flags (`rmcp` SDK).
  - **Live IPC & WebSocket Stream**: Real-time event telemetry log.
