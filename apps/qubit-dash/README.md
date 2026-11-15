# QubitDash (`apps/qubit-dash`)

Lightweight, modern dashboard and mission control interface for QubitCore.

## Features
- **Svelte 5 + SvelteKit + TailwindCSS**: Runes reactivity, ultra-lean bundle footprint, sub-second hydration.
- **Tauri v2 / Web Hybrid**: Designed to run as an embedded cross-platform Tauri v2 desktop app or directly served by the `qubit-core` Axum daemon.
- **Real-Time WebSocket Protocol**: Live streaming of agent status, progressive decompilation progress, and resource utilization (<60 MB RAM budget monitoring).
- **Zero-Drift Type Sharing**: TypeScript types generated automatically from `crates/qubit-protocol` using `ts-rs`.
