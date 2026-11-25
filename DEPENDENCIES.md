# Macierz Zależności i Specyfikacja Narzędziowa Qubit Core

> Dokumentacja komponentów, bibliotek i wersji środowiskowych ekosystemu Qubit.

---

## 1. Wymagania Środowiskowe (Toolchain)

| Narzędzie | Wersja | Plik konfiguracyjny | Uwagi |
| :--- | :--- | :--- | :--- |
| **Rust Toolchain** | `stable` (rekomendowany Rust 1.85+) | `rust-toolchain.toml` | Komponenty: `rustfmt`, `clippy` |
| **Cargo Edition** | `2021` | `Cargo.toml` | Workspace resolver `2` |
| **Node.js / pnpm** | Node 20+ LTS / pnpm 9+ | `pnpm-workspace.yaml` | Dla GUI / QubitDash |
| **Tauri CLI** | `v2.2+` | `apps/qubit-dash/src-tauri` | Desktop shell |
| **Svelte** | `v5.16+` | `apps/qubit-dash/package.json` | Runes reactive engine |

### `rust-toolchain.toml`
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

---

## 2. Podział na Crates (Cargo Workspace)

```
crates/
├── qubit-protocol/   # Wspólne typy DTO, definicje wiadomości JSON-RPC/IPC, eksport do TypeScript (ts-rs)
├── qubit-storage/    # Warstwa persystencji SQLite WAL (rusqlite 0.42), wektory (sqlite-vec), migracje
├── qubit-mcp/        # Model Context Protocol: klient/serwer SSE & Stdio dla narzędzi agentów
├── qubit-sandbox/    # Bezpieczna egzekucja kodu (Linux cgroups v2, Docker/bollard, namespace)
├── qubit-agents/     # Silnik orkiestracji agentów AI, pętla planowania, delegowanie zadań
├── qubit-core/       # Główny demon i serwer HTTP/WebSocket (Axum 0.8, Tokio 1.54)
└── qubit-cli/        # Narzędzie linii poleceń CLI (clap 4.5)
```

---

## 3. Kluczowe Zależności Rust (`Cargo.toml`)

### Runtime Asynchroniczny i Sieć
- **`tokio`** (`1.54`): Pełny runtime asynchroniczny (`full`), pule wątków, timery, operacje I/O.
- **`tokio-stream`** (`0.1`): Obsługa strumieniowania danych i eventów SSE.
- **`axum`** (`0.8`): Wysokowydajny serwer HTTP/WebSocket z obsługą makr i routingu.
- **`tower`** (`0.5`) / **`tower-http`** (`0.6`): Warstwy middleware (CORS, Trace, obsługa plików statycznych).

### Serializacja i Interfejsy
- **`serde`** (`1.0`) / **`serde_json`** (`1.0`): Uniwersalna serializacja i deserializacja struktur.
- **`ts-rs`** (`9.0`): Automatyczne generowanie typów TypeScript ze struktur Rustowych dla frontendu Tauri/Svelte.

### Pamięć i Bazy Danych
- **`rusqlite`** (`0.42` z flagą `bundled`): Wbudowana baza SQLite w trybie WAL zoptymalizowana pod minimalne zużycie RAM (<60 MB).
- **`sqlite-vec`**: Wektory embeddingów wewnątrz SQLite dla wyszukiwania semantycznego RAG.

### Narzędzia i Diagnostyka
- **`clap`** (`4.5`): Parser argumentów linii poleceń z obsługą zmiennych środowiskowych.
- **`tracing`** (`0.1`) / **`tracing-subscriber`** (`0.3`): Ustrukturyzowane logowanie telemetryczne (`env-filter`).
- **`anyhow`** (`1.0`) / **`thiserror`** (`2.0`): Zarządzanie błędami i typowane domeny błędów.
- **`uuid`** (`1.16` v4): Unikalne identyfikatory zadań, sesji i agentów.
- **`chrono`** (`0.4`): Operacje czasowe ze wsparciem serde.

---

## 4. Flagi Kompilacji Wydaniowej (`profile.release`)

Zoptymalizowane pod minimalny rozmiar binarki i maksymalną wydajność:

```toml
[profile.release]
opt-level = 3          # Pełna optymalizacja kodu maszynowego
lto = "fat"            # Agresywna optymalizacja Link-Time across crates
codegen-units = 1      # Maksymalna redukcja wielkości i optymalizacja globalna
panic = "abort"        # Usunięcie tablic unwindingu dla mniejszego narzutu binarnego
strip = true           # Automatyczne usuwanie symboli debugowania
```

---

## 5. Powiązane Dokumenty

- [Spis Treści i README](../README.md)
- [01. Wizja Ekosystemu Qubit](01_ECOSYSTEM_VISION.md)
- [02. Specyfikacja Architektury](02_ARCHITECTURE_SPEC.md)
- [03. Harmonogram Wdrożenia](03_IMPLEMENTATION_ROADMAP.md)
