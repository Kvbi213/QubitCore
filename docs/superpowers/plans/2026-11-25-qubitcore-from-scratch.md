# QubitCore Implementation Plan (From Scratch)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Zbudować od zera kompletny, kompilujący się i przetestowany mikrokernal **QubitCore** (wszystkie bazowe crate'y w Rust: protokół, baza SQLite WAL + FTS5, brama bezpieczeństwa MCP, piaskownica procesów, router agentów, demon HTTP/WS Axum oraz klient CLI) zgodnie ze specyfikacją ekosystemu Qubit.

**Architecture:** Architektura oparta o pojedynczy proces demona w języku Rust (`tokio` + `axum`), w którym zapisy do współdzielonej bazy `qubit_data.db` (SQLite WAL + FTS5) przechodzą przez jednego aktora (`StorageActor` przez `mpsc`), narzędzia podlegają izolacji `MCP Roots` i bramce `ElicitationGate` (`TaintedContext`), a wywołania modeli LLM obsługuje warstwa `ModelProvider` z 4-fazową kompresją kontekstu i trybami `Reflex`/`Deep`.

**Tech Stack:** Rust (edition 2021), Tokio, Axum 0.8, `rusqlite` (bundled WAL + FTS5), `serde` / `serde_json`, `ts-rs` 9.0, `reqwest` (`rustls-tls`), `clap` 4.5, `tower` / `tower-http`, `tracing`.

**Spec:** `/data/projects/Qubit/QubitCore/ARCHITECTURE_SPEC.md`, `/data/projects/Qubit/QubitCore/ROADMAP.md`, `/data/projects/Qubit/QubitCore/EKOSYSTEM_QUBIT_MANIFEST.md`, `/data/projects/Qubit/QubitCore/Cargo.toml`.

## Global Constraints

- Budżet pamięciowy demona `qubit-core` w spoczynku: **< 60 MB RAM** (brak wbudowanego Caddy wewnątrz procesu demona; routing realizuje `axum`).
- Baza danych: pojedynczy plik SQLite w trybie **WAL** (`PRAGMA journal_mode = WAL`, `synchronous = NORMAL`, `cache_size = -4000`, `busy_timeout = 5000`, `foreign_keys = ON`) obsługiwany przez jeden dedykowany wątek piszący.
- Domyślne bindowanie serwera HTTP/WS demona: wyłącznie `127.0.0.1:4000` (brak otwartego portu Admin API `2019`).
- Zero-Trust MCP: twarda izolacja ścieżek (`MCP Roots`), blokada narzędzi mutujących przy fladze `TaintedContext` oraz obowiązkowa kolejka `ElicitationGate` dla operacji ryzykownych.
- Kompresja kontekstu (Hermes pattern): wyzwalana przy przekroczeniu **50%** limitu kontekstu, chroniąca nienaruszone ostatnie **20** wiadomości.
- Zależności crate'ów w workspace muszą korzystać z centralnych definicji w `/data/projects/Qubit/QubitCore/Cargo.toml`.

## Review Focus

- **Path traversal / symlink escape poza `MCP Root`:** Próba odczytu lub zapisu ścieżek typu `../../etc/passwd` albo dowiązania symbolicznego wskazującego poza katalog roboczy musi zostać odrzucona przed jakimkolwiek dostępem do pliku (test w Task 3).
- **Indirect Prompt Injection w `TaintedContext`:** Wywołanie narzędzia mutującego (`run_terminal`, `apply_diff`, `delete_file`) w sesji oznaczonej flagą `tainted = true` musi zostać twardo zablokowane niezależnie od treści promptu (test w Task 3).
- **Współbieżny zapis wielu agentów do SQLite WAL:** Równoległe zapisy z 20 tasków Tokio nie mogą zwracać błędu `SQLITE_BUSY` ani gubić wiadomości (test w Task 2).
- **Zawiecha lub zalanie RAM przez podproces w piaskownicy:** Komenda przekraczająca `timeout` musi zostać ubita, a `stdout`/`stderr` przekraczający limit bajtów musi zostać bezpiecznie obcięty bez alokacji gigabajtów RAM (test w Task 4).
- **Awaria/przeciążenie providera LLM (`408`/`504`/`529`) i brak działającego demona dla CLI:** `ModelProvider` musi przełączyć się na model zapasowy bez utraty sesji (test w Task 5), a `qubit-cli` przy wyłączonym demonie musi zwrócić czytelny błąd zamiast paniki (test w Task 7).

---

### Task 1: Toolchain Bootstrap & `qubit-protocol` (Typy Domenowe i Kontrakty)

**Files:**
- Modify: `/data/projects/Qubit/QubitCore/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-protocol/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-protocol/src/lib.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-protocol/tests/protocol_roundtrip.rs`

**Interfaces:**
- Consumes: Workspace dependencies z `/data/projects/Qubit/QubitCore/Cargo.toml` (`serde`, `serde_json`, `uuid`, `chrono`, `ts-rs`, `thiserror`).
- Produces:
  - `ExecutionMode` (`Reflex`, `Deep`)
  - `MessageRole` (`System`, `User`, `Assistant`, `Tool`)
  - `ChatMessage { id: Uuid, session_id: Uuid, role: MessageRole, content: String, tainted: bool, created_at: DateTime<Utc> }`
  - `HealthResponse { status: String, version: String, rss_mb: f64, wal_ok: bool }`
  - `ExecuteTaskRequest { session_id: Option<Uuid>, prompt: String, mode: Option<ExecutionMode>, tainted: bool }`
  - `ExecuteTaskResponse { session_id: Uuid, task_id: Uuid, mode: ExecutionMode, reply: String }`
  - `ElicitationStatus` (`Pending`, `Approved`, `Denied`)
  - `ElicitationRequest { id: Uuid, session_id: Uuid, tool_name: String, args_preview: String, status: ElicitationStatus }`
  - `WebSocketEvent` (enum: `TaskStarted`, `TokenDelta`, `ElicitationRequired`, `TaskCompleted`, `Error`)

- [ ] **Step 1: Verify or install Rust toolchain (`cargo` / `rustc`) and create `qubit-protocol` manifest**

Sprawdź obecność `cargo` / `rustc`; jeśli brak, zainstaluj minimalny toolchain przez `rustup` (z `rust-toolchain.toml`), a następnie utwórz `crates/qubit-protocol/Cargo.toml` z zależnościami workspace (`serde`, `serde_json`, `uuid`, `chrono`, `ts-rs`, `thiserror`).

- [ ] **Step 2: Write the failing test**

```rust
// crates/qubit-protocol/tests/protocol_roundtrip.rs
use qubit_protocol::{ChatMessage, ExecutionMode, ExecuteTaskRequest, MessageRole};
use uuid::Uuid;

#[test]
fn roundtrip_execute_task_request_and_chat_message() {
    let req = ExecuteTaskRequest {
        session_id: None,
        prompt: "status".to_string(),
        mode: Some(ExecutionMode::Reflex),
        tainted: false,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: ExecuteTaskRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.prompt, "status");
    assert_eq!(decoded.mode, Some(ExecutionMode::Reflex));

    let msg = ChatMessage::new(Uuid::new_v4(), MessageRole::User, "hello", true);
    assert!(msg.tainted);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p qubit-protocol --test protocol_roundtrip`
Expected: FAIL z powodu braku `crates/qubit-protocol/src/lib.rs` i brakujących typów.

- [ ] **Step 4: Implement protocol DTOs in `crates/qubit-protocol/src/lib.rs`**

Zdefiniuj wszystkie typy z bloku `Produces` z atrybutami `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]` (z `#[ts(export)]` dla typów frontendowych) oraz metodą `ChatMessage::new(session_id: Uuid, role: MessageRole, content: impl Into<String>, tainted: bool) -> Self`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p qubit-protocol`
Expected: PASS (wszystkie testy serializacji przechodzą, a `ts-rs` poprawnie generuje typy przy `cargo test`).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/qubit-protocol/
git commit -m "feat(protocol): scaffold qubit-protocol types and ts-rs contracts"
```

---

### Task 2: `qubit-storage` — Single-Writer Actor SQLite WAL + FTS5 (BM25)

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-storage/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-storage/src/lib.rs`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-storage/src/schema.sql`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-storage/tests/storage_actor.rs`

**Interfaces:**
- Consumes: `qubit_protocol::{ChatMessage, MessageRole, ElicitationRequest, ElicitationStatus}`
- Produces:
  - `StorageError`
  - `DocRecord { id: String, title: String, body: String, tags: Vec<String> }`
  - `StorageHandle` (clonable handle z kanałem `tokio::sync::mpsc::Sender<StorageCmd>`):
    - `StorageHandle::open(path: impl AsRef<Path>) -> Result<Self, StorageError>`
    - `StorageHandle::create_session(&self, title: &str) -> Result<Uuid, StorageError>`
    - `StorageHandle::append_message(&self, msg: ChatMessage) -> Result<(), StorageError>`
    - `StorageHandle::list_messages(&self, session_id: Uuid) -> Result<Vec<ChatMessage>, StorageError>`
    - `StorageHandle::upsert_doc(&self, doc: DocRecord) -> Result<(), StorageError>`
    - `StorageHandle::search_docs(&self, query: &str, limit: usize) -> Result<Vec<DocRecord>, StorageError>`
    - `StorageHandle::wal_checkpoint(&self) -> Result<(), StorageError>`

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-storage/tests/storage_actor.rs
use qubit_protocol::{ChatMessage, MessageRole};
use qubit_storage::{DocRecord, StorageHandle};
use tempfile::tempdir;

#[tokio::test]
async fn concurrent_writes_never_hit_sqlite_busy_and_fts5_finds_docs() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("qubit_data.db");
    let storage = StorageHandle::open(&db_path).expect("open WAL storage");
    let sid = storage.create_session("test-session").await.unwrap();

    let mut handles = Vec::new();
    for i in 0..20 {
        let st = storage.clone();
        handles.push(tokio::spawn(async move {
            st.append_message(ChatMessage::new(sid, MessageRole::User, format!("msg-{i}"), false))
                .await
                .unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let msgs = storage.list_messages(sid).await.unwrap();
    assert_eq!(msgs.len(), 20);

    storage
        .upsert_doc(DocRecord {
            id: "adr-01".into(),
            title: "Tokio Single Writer".into(),
            body: "SQLite WAL eliminates lock contention via mpsc actor".into(),
            tags: vec!["rust".into(), "sqlite".into()],
        })
        .await
        .unwrap();

    let hits = storage.search_docs("contention", 5).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "adr-01");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-storage --test storage_actor`
Expected: FAIL (brak crate'a `qubit-storage` / `StorageHandle`).

- [ ] **Step 3: Create `schema.sql` and implement `StorageHandle` in `crates/qubit-storage/src/lib.rs`**

Użyj dedykowanego wątku `std::thread::spawn` obsługującego polecenia z `tokio::sync::mpsc::channel(256)` oraz `oneshot::Sender` do zwrotów. Przy otwarciu połączenia wykonaj pragmy: `journal_mode=WAL`, `synchronous=NORMAL`, `cache_size=-4000`, `busy_timeout=5000`, `foreign_keys=ON`. Utwórz tabele `sessions`, `messages`, `docs` oraz wirtualną tabelę `docs_fts USING fts5(id UNINDEXED, title, body)` z zapytaniem `bm25(docs_fts)` i bindowanymi parametrami (`params![]` — zero konkatenacji SQL).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qubit-storage`
Expected: PASS (20 współbieżnych zapisów bez `SQLITE_BUSY` + wyszukiwanie FTS5 BM25 przechodzi).

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-storage/
git commit -m "feat(storage): implement single-writer SQLite WAL actor and FTS5 docs index"
```

---

### Task 3: `qubit-mcp` — Zero-Trust `RootGuard`, `TaintedContext` & `ElicitationGate`

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-mcp/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-mcp/src/lib.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-mcp/tests/zero_trust.rs`

**Interfaces:**
- Consumes: `qubit_protocol::{ElicitationRequest, ElicitationStatus}`
- Produces:
  - `McpError` (`OutsideRoot`, `TaintedBlocked`, `ElicitationDenied`, `ElicitationPending(Uuid)`)
  - `ToolRisk` (`ReadOnly`, `Mutating`)
  - `RootGuard::new(roots: Vec<PathBuf>) -> Result<Self, McpError>`
  - `RootGuard::resolve_safe(&self, candidate: &Path) -> Result<PathBuf, McpError>`
  - `SecurityContext { pub tainted: bool }`
  - `SecurityContext::authorize_tool(&self, tool_name: &str, risk: ToolRisk) -> Result<(), McpError>`
  - `ElicitationGate` (`request_approval`, `resolve`, `status`)

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-mcp/tests/zero_trust.rs
use qubit_mcp::{ElicitationGate, McpError, RootGuard, SecurityContext, ToolRisk};
use qubit_protocol::ElicitationStatus;
use std::path::Path;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn root_guard_blocks_traversal_and_symlink_escape() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    std::fs::write(root.path().join("ok.txt"), "ok").unwrap();
    std::fs::write(outside.path().join("secret.txt"), "secret").unwrap();

    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path().join("secret.txt"), root.path().join("link.txt")).unwrap();

    let guard = RootGuard::new(vec![root.path().to_path_buf()]).unwrap();
    assert!(guard.resolve_safe(&root.path().join("ok.txt")).is_ok());
    assert!(matches!(guard.resolve_safe(Path::new("../../etc/passwd")), Err(McpError::OutsideRoot)));

    #[cfg(unix)]
    assert!(matches!(guard.resolve_safe(&root.path().join("link.txt")), Err(McpError::OutsideRoot)));
}

#[test]
fn tainted_context_hard_blocks_mutating_tools() {
    let ctx = SecurityContext { tainted: true };
    assert!(ctx.authorize_tool("read_file", ToolRisk::ReadOnly).is_ok());
    assert!(matches!(
        ctx.authorize_tool("run_terminal", ToolRisk::Mutating),
        Err(McpError::TaintedBlocked)
    ));
}

#[tokio::test]
async fn elicitation_gate_waits_and_resolves() {
    let gate = ElicitationGate::new();
    let id = gate.submit(Uuid::new_v4(), "run_terminal", "rm -rf /tmp/x").await;
    assert_eq!(gate.status(id).await, Some(ElicitationStatus::Pending));
    gate.resolve(id, ElicitationStatus::Approved).await.unwrap();
    assert_eq!(gate.status(id).await, Some(ElicitationStatus::Approved));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-mcp --test zero_trust`
Expected: FAIL (brak crate'a `qubit-mcp`).

- [ ] **Step 3: Implement `RootGuard`, `SecurityContext` and `ElicitationGate` in `crates/qubit-mcp/src/lib.rs`**

Kanonikalizuj korzenie przez `std::fs::canonicalize`; dla nieistniejących jeszcze plików docelowych kanonikalizuj najbliższego istniejącego rodzica i sprawdź `starts_with(&canonical_root)`. Dla `SecurityContext::authorize_tool` przy `self.tainted && risk == ToolRisk::Mutating` zwróć natychmiast `Err(McpError::TaintedBlocked)`. `ElicitationGate` przechowuje mapę `Arc<tokio::sync::RwLock<HashMap<Uuid, ElicitationRequest>>>`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qubit-mcp`
Expected: PASS (traversal, symlink escape, `TaintedContext` oraz `ElicitationGate` przechodzą).

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-mcp/
git commit -m "feat(mcp): implement Zero-Trust RootGuard, TaintedContext and ElicitationGate"
```

---

### Task 4: `qubit-sandbox` — Izolacja Procesów, Timeouty, Limit Wyjścia i Memory Backpressure

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-sandbox/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-sandbox/src/lib.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-sandbox/tests/sandbox_limits.rs`

**Interfaces:**
- Consumes: `qubit_mcp::RootGuard`
- Produces:
  - `SandboxConfig { pub timeout_ms: u64, pub max_output_bytes: usize }`
  - `SandboxOutput { pub exit_code: i32, pub stdout: String, pub stderr: String, pub timed_out: bool, pub truncated: bool }`
  - `BackpressureLevel` (`Normal`, `Throttle` /* >= 80% */, `Critical` /* >= 85% */)
  - `SandboxRunner::run(cmd: &str, args: &[&str], cwd: &Path, cfg: &SandboxConfig) -> Result<SandboxOutput, SandboxError>`
  - `classify_memory_pressure(anon_bytes: u64, limit_bytes: u64) -> BackpressureLevel`
  - `current_process_rss_mb() -> f64`

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-sandbox/tests/sandbox_limits.rs
use qubit_sandbox::{classify_memory_pressure, BackpressureLevel, SandboxConfig, SandboxRunner};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn kills_timed_out_process_and_truncates_large_output() {
    let dir = tempdir().unwrap();
    let cfg = SandboxConfig { timeout_ms: 150, max_output_bytes: 256 };
    let out = SandboxRunner::run("sh", &["-c", "sleep 2"], dir.path(), &cfg).await.unwrap();
    assert!(out.timed_out);

    let flood_cfg = SandboxConfig { timeout_ms: 2000, max_output_bytes: 128 };
    let flood = SandboxRunner::run("sh", &["-c", "yes qubit | head -n 500"], dir.path(), &flood_cfg)
        .await
        .unwrap();
    assert!(!flood.timed_out);
    assert!(flood.truncated);
    assert!(flood.stdout.len() <= 128);
}

#[test]
fn memory_backpressure_thresholds_match_spec() {
    assert_eq!(classify_memory_pressure(50, 100), BackpressureLevel::Normal);
    assert_eq!(classify_memory_pressure(80, 100), BackpressureLevel::Throttle);
    assert_eq!(classify_memory_pressure(85, 100), BackpressureLevel::Critical);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-sandbox --test sandbox_limits`
Expected: FAIL (brak crate'a `qubit-sandbox`).

- [ ] **Step 3: Implement `SandboxRunner` and `classify_memory_pressure` in `crates/qubit-sandbox/src/lib.rs`**

Użyj `tokio::process::Command` z `kill_on_drop(true)`, podpiętym `tokio::time::timeout`, oraz ograniczonym buforem odczytu `AsyncReadExt::take((cfg.max_output_bytes + 1) as u64)` na `stdout`/`stderr` (jeśli przekroczy `max_output_bytes`, oznacz `truncated = true` i utnij bufor). Odczyt `current_process_rss_mb()` parsuje `/proc/self/statm` na Linuxie (lub zwraca `0.0` na innych platformach).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qubit-sandbox`
Expected: PASS (ubicie po timeoutcie, ucięcie wyjścia oraz klasyfikacja 80%/85% przechodzą).

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-sandbox/
git commit -m "feat(sandbox): implement bounded process execution, output cap and memory backpressure"
```

---

### Task 5: `qubit-agents` — Router `Reflex`/`Deep`, `ModelProvider` z Fallbackiem i Kompresja Kontekstu

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-agents/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-agents/src/lib.rs`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-agents/src/compressor.rs`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-agents/src/provider.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-agents/tests/agent_engine.rs`

**Interfaces:**
- Consumes: `qubit_protocol::{ChatMessage, ExecutionMode, MessageRole}`, `qubit_storage::StorageHandle`, `qubit_mcp::SecurityContext`
- Produces:
  - `ProviderError` (`RetryableHttp(u16)`, `Fatal(String)`)
  - `#[async_trait::async_trait] pub trait ModelProvider: Send + Sync { async fn complete(&self, messages: &[ChatMessage], mode: ExecutionMode) -> Result<String, ProviderError>; fn name(&self) -> &str; }`
  - `FallbackProviderChain` (`new(providers: Vec<Arc<dyn ModelProvider>>)`, `complete(...)`)
  - `HttpApiProvider` (OpenAI-compatible endpoint + Bearer key, wykrywa HTTP `408`/`504`/`529` jako `RetryableHttp`)
  - `CliBridgeProvider` (uruchamia lokalną komendę CLI przez `stdin`/`stdout`)
  - `ContextCompressor::compress(messages: Vec<ChatMessage>, max_chars: usize) -> Vec<ChatMessage>` (próg 50%, chroni ostatnie 20 wiadomości)
  - `AgentEngine::execute(&self, session_id: Uuid, prompt: &str, mode: Option<ExecutionMode>, tainted: bool) -> Result<(ExecutionMode, String), AgentError>`

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-agents/tests/agent_engine.rs
use qubit_agents::{ContextCompressor, FallbackProviderChain, ModelProvider, ProviderError};
use qubit_protocol::{ChatMessage, ExecutionMode, MessageRole};
use std::sync::Arc;
use uuid::Uuid;

struct FailingProvider(u16);
#[async_trait::async_trait]
impl ModelProvider for FailingProvider {
    async fn complete(&self, _: &[ChatMessage], _: ExecutionMode) -> Result<String, ProviderError> {
        Err(ProviderError::RetryableHttp(self.0))
    }
    fn name(&self) -> &str { "failing" }
}

struct OkProvider;
#[async_trait::async_trait]
impl ModelProvider for OkProvider {
    async fn complete(&self, _: &[ChatMessage], mode: ExecutionMode) -> Result<String, ProviderError> {
        Ok(format!("ok:{mode:?}"))
    }
    fn name(&self) -> &str { "ok" }
}

#[tokio::test]
async fn fallback_chain_switches_on_529_and_compressor_protects_last_20() {
    let chain = FallbackProviderChain::new(vec![Arc::new(FailingProvider(529)), Arc::new(OkProvider)]);
    let reply = chain.complete(&[], ExecutionMode::Reflex).await.unwrap();
    assert_eq!(reply, "ok:Reflex");

    let sid = Uuid::new_v4();
    let msgs: Vec<ChatMessage> = (0..35)
        .map(|i| ChatMessage::new(sid, MessageRole::User, format!("message-{i}-{}", "x".repeat(50)), false))
        .collect();
    let compressed = ContextCompressor::compress(msgs, 2000);
    assert!(compressed.len() <= 21); // 1 summary + 20 chronionych ostatnich wiadomości
    assert_eq!(compressed.last().unwrap().content.starts_with("message-34-"), true);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-agents --test agent_engine`
Expected: FAIL (brak crate'a `qubit-agents`).

- [ ] **Step 3: Implement `compressor.rs`, `provider.rs`, and `AgentEngine` in `crates/qubit-agents/src/lib.rs`**

`ContextCompressor::compress`: jeśli łączna liczba znaków <= `max_chars / 2` lub `messages.len() <= 20`, zwróć bez zmian; w przeciwnym razie utnij środek (przed ostatnimi 20 wiadomościami) do skondensowanej wiadomości `MessageRole::System` o długości <= 20% `max_chars` i doklej nienaruszone ostatnie 20 wiadomości. `FallbackProviderChain` przechodzi kolejno po dostawcach wyłącznie przy `ProviderError::RetryableHttp(408 | 504 | 529 | 429)`. `AgentEngine::execute` zapisuje prompt w `StorageHandle`, dobiera `ExecutionMode::Reflex` (krótkie zapytania < 240 znaków bez słów kluczowych planu) lub `Deep`, wykonuje kompresję, woła providera i zapisuje odpowiedź.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qubit-agents`
Expected: PASS (fallback na 529 oraz ochrona ostatnich 20 wiadomości przechodzą).

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-agents/
git commit -m "feat(agents): implement Reflex/Deep router, ModelProvider fallback chain and context compressor"
```

---

### Task 6: `qubit-core` — Demon Axum (`127.0.0.1:4000`), WebSocket Event Hub i Shutdown

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-core/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-core/src/lib.rs`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-core/src/main.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-core/tests/http_api.rs`

**Interfaces:**
- Consumes: `qubit_protocol::*`, `qubit_storage::StorageHandle`, `qubit_mcp::{RootGuard, ElicitationGate}`, `qubit_sandbox::current_process_rss_mb`, `qubit_agents::AgentEngine`
- Produces:
  - `AppState { pub storage: StorageHandle, pub engine: Arc<AgentEngine>, pub roots: Arc<RwLock<RootGuard>>, pub elicitation: Arc<ElicitationGate>, pub events: broadcast::Sender<WebSocketEvent> }`
  - `build_router(state: AppState) -> axum::Router`
  - Endpointy:
    - `GET /health` -> `200 JSON HealthResponse`
    - `POST /api/sessions` -> `201 JSON { id: Uuid }`
    - `POST /api/roots` -> `200 JSON` (rejestracja katalogu roboczego w `RootGuard`)
    - `POST /api/tasks` -> `200 JSON ExecuteTaskResponse` (lub `400` dla pustego promptu)
    - `POST /api/elicitation/:id` -> `200 JSON`
    - `GET /ws` -> WebSocket upgrade emitujący `WebSocketEvent`

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-core/tests/http_api.rs
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn health_and_empty_prompt_validation_work() {
    let app = qubit_core::test_router().await;

    let res = app.clone().oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bad_task = app
        .oneshot(
            Request::post("/api/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"prompt":"   ","tainted":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_task.status(), StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-core --test http_api`
Expected: FAIL (brak `qubit_core::test_router` / handlerów Axum).

- [ ] **Step 3: Implement `build_router`, `test_router` and `main.rs` in `crates/qubit-core/`**

`main.rs` inicjalizuje `tracing_subscriber`, otwiera `StorageHandle` na `./data/qubit_data.db` (tworząc katalog `data/`), konfiguruje providera z `QUBIT_API_URL` / `QUBIT_API_KEY` (z bezpiecznym fallbackiem na tryb echo/offline gdy brak klucza), binduje `TcpListener` wyłącznie na `127.0.0.1:4000`, i obsługuje `tokio::signal::ctrl_c()` wykonując `storage.wal_checkpoint().await` przed wyjściem.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qubit-core`
Expected: PASS (`GET /health` zwraca 200, pusty prompt w `POST /api/tasks` zwraca 400).

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-core/
git commit -m "feat(core): implement Axum daemon server, WS event broadcast and graceful WAL shutdown"
```

---

### Task 7: `qubit-cli` — Klient Terminalowy (`status`, `.`, `chat`)

**Files:**
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-cli/Cargo.toml`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-cli/src/lib.rs`
- Create: `/data/projects/Qubit/QubitCore/crates/qubit-cli/src/main.rs`
- Test: `/data/projects/Qubit/QubitCore/crates/qubit-cli/tests/cli_client.rs`

**Interfaces:**
- Consumes: `qubit_protocol::{HealthResponse, ExecuteTaskRequest, ExecuteTaskResponse}`, `clap` 4.5, `reqwest`
- Produces:
  - `CliCommand` (`Status`, `MountCurrentDir` /* komenda `.` */, `Chat { prompt: String, deep: bool }`)
  - `run_cli(cmd: CliCommand, base_url: &str) -> Result<String, CliError>`

- [ ] **Step 1: Write the failing test**

```rust
// crates/qubit-cli/tests/cli_client.rs
use qubit_cli::{run_cli, CliCommand, CliError};

#[tokio::test]
async fn returns_clear_error_when_daemon_unreachable() {
    let err = run_cli(CliCommand::Status, "http://127.0.0.1:49999").await.unwrap_err();
    assert!(matches!(err, CliError::DaemonUnavailable(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qubit-cli --test cli_client`
Expected: FAIL (brak crate'a `qubit-cli`).

- [ ] **Step 3: Implement `clap` CLI and HTTP client in `crates/qubit-cli/src/`**

Zmapuj podkomendy `qubit status`, `qubit .` (rejestracja `std::env::current_dir()` w `POST /api/roots`), `qubit chat <prompt> [--deep]`. Błędy połączenia `reqwest` mapuj na `CliError::DaemonUnavailable` z czytelnym komunikatem podpowiadającym uruchomienie `qubit-core`.

- [ ] **Step 4: Run full workspace check & tests**

Run: `cargo test --workspace && cargo build --release -p qubit-core -p qubit-cli`
Expected: Wszystkie testy przechodzą (`PASS`), a binarki release budują się bez błędów.

- [ ] **Step 5: Commit**

```bash
git add crates/qubit-cli/
git commit -m "feat(cli): implement qubit-cli client for status, root mounting and chat"
```
