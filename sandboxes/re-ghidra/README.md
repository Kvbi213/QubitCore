# Qubit Reverse Engineering Sandbox (`re-ghidra`)

Isolated Docker execution sandbox implementing the **5-Step Progressive Disclosure Loop** for safe, memory-efficient reverse engineering of untrusted binaries.

## Architecture

```
[Qubit Host: qubit-sandbox]
             │ stdio / MCP JSON-RPC
             ▼
[Docker Container: re-ghidra]
  ├── PyGhidra / Headless Analyzer
  ├── Progressive Query Engine (File -> Symbols -> CFG -> Decompile)
  └── Ephemeral Execution Guard
```

## Security Guarantees
- Network egress disabled (`--network none`) by default.
- Read-only root filesystem with ephemeral memory-backed `/workspace` scratchpad.
- CPU & RAM cgroup limits enforced by `qubit-sandbox`.
