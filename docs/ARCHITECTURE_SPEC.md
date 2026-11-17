QubitCore: Kompletna Specyfikacja Inżynieryjna i Dokumentacja Implementacji Silnika

Dokumentacja ta stanowi szczegółowy, inżynieryjny przewodnik implementacji centralnego demona QubitCore przeznaczony dla oficjalnego repozytorium https://github.com/Kvbi213/QubitCore.git. Architektura systemu realizuje paradygmat „Jeden Mózg, Wiele Oczu i Rąk” — tworzy lekki, rozproszony system operacyjny dla agentów sztucznej inteligencji, który od pierwszego etapu prac łączy ultra-niskie zużycie zasobów sprzętowych (poniżej 60 MB RAM w spoczynku na serwerach 1–2 GB RAM) z natywnym bezpieczeństwem wykonawczym i architekturą wielodostępną (multi-tenancy) przygotowaną pod chmurowy model SaaS.

1. Podwaliny Architektury i Standardy Monorepo Cargo Workspace

Budowa QubitCore wymaga bezwzględnej dyscypliny w zarządzaniu czasem kompilacji, alokacjami sterty oraz cyklem życia pamięci podręcznych. Założenie utrzymania procesu w spoczynku poniżej progu 60 MB RAM wyklucza stosowanie środowisk uruchomieniowych opartych na językach Python lub Node.js w warstwie centralnej. Wyklucza także wbudowanie serwera proxy Caddy bezpośrednio do obrazu bazowego MVP, gdyż środowisko Go ze swoim systemem odśmiecania pamięci oraz narzutem buforów sieciowych generuje zużycie rzędu 85 MB do ponad 1,2 GB RAM przy intensywnym ruchu. Silnik opiera się w całości na ekosystemie języka Rust i asynchronicznym środowisku Tokio, co gwarantuje czasy zimnego startu poniżej 5 milisekund oraz walidację schematów danych w czasie kompilacji.

Współdzielenie sesji między aplikacją desktopową QubitDash a silnikiem QubitCore opiera się na wzorcu Ephemeral WebView. Kiedy interfejs graficzny jest minimalizowany do zasobnika systemowego (System Tray), okno silnika przeglądarki (WebView2 / WebKit) jest niszczone, co redukuje zużycie pamięci frontendu do zaledwie 3–5 MB RAM procesu Rusta. W tym samym czasie demon QubitCore nieprzerwanie zarządza stanem aktywnej sesji (session_id), obsługując zapytania z terminalowego CLI oraz edytorów kodu.

1.1. Układ Monorepo i Separacja Odpowiedzialności

Aby zapobiec powstawaniu cyklicznych zależności oraz skrócić czas kompilacji przyrostowej, kod silnika został zorganizowany w architekturze wielopakietowej Cargo Workspace, połączonej z narzędziem pnpm Workspace dla warstwy frontendu TypeScript:

* /Cargo.toml definiuje centralne wersje bibliotek zewnętrznych oraz flagi optymalizacyjne kompilatora.
* /pnpm-workspace.yaml zarządza zależnościami pakietów webowych interfejsu QubitDash.
* /crates/qubit-protocol stanowi pojedyncze źródło prawdy (Single Source of Truth) dla typów domenowych, obiektów DTO i kontraktów zdarzeń, generując automatyczne definicje TypeScript (bindings.ts) oraz schematy JSON.
* /crates/qubit-storage odpowiada za warstwę trwałości danych, konfigurację SQLite WAL z pojedynczym wątkiem piszącym, rejestrację rozszerzenia wektorowego sqlite-vec przez C FFI, migracje bazy i relacyjne grafy wiedzy.
* /crates/qubit-mcp implementuje integrację z oficjalnym SDK rmcp, dynamiczną rejestrację możliwości, wykrywanie katalogów roboczych (Roots), bramki autoryzacji człowieka (Elicitation) oraz polityki bezpieczeństwa Capability ACL.
* /crates/qubit-sandbox realizuje izolację systemową podprocesów, badanie grup kontrolnych Linuksa cgroups v2 (z dekompozycją pamięci anonimowej), zarządzanie kolejką Memory Backpressure oraz orkiestrację kontenerów przez Docker API (bollard) i Kubernetes API (kube-rs).
* /crates/qubit-agents zawiera pętlę decyzyjną agentów, router trybów (Reflex vs. Deep), port 4-fazowego kompresora kontekstu z frameworka Hermes Agent oraz mechanizmy proaktywnej pamięci w tle z bezpiecznikiem Circuit Breaker.
* /crates/qubit-core stanowi centralny punkt wejścia silnika — łączy serwer axum (HTTP, WebSocket, transport SSE dla MCP), lokalny transport gniazd uniksowych IPC, Tokio Supervisor oraz instancję SessionHub.
* /crates/qubit-cli jest lekkim klientem wiersza poleceń wpinającym bieżący katalog roboczy terminala do aktywnej sesji jako MCP Root.
* /apps/qubit-dash to hybrydowy interfejs Mission Control (Tauri v2 dla desktopu oraz statyczny build HTML serwowany przez sieć).
* /sandboxes/re-ghidra zawiera definicje kontenerów piaskownicy ze środowiskiem JDK 21, dekompilatorem Ghidra 12.1.4/12.2, wtyczką ReVa / GhidraMCP oraz serwerem reversing-mcp.

1.2. Konfiguracja Globalnego Manifestu Cargo.toml

W głównym pliku Cargo.toml zdefiniowano scentralizowany zestaw zależności oraz agresywne optymalizacje profilu produkcyjnego. Zastosowanie optymalizacji czasu łączenia (lto = "fat"), redukcji jednostek generowania kodu (codegen-units = 1) oraz usuwania symboli debugowania (strip = true) pozwala zminimalizować rozmiar wynikowej binarki i przyspieszyć jej wykonanie.

[workspace]
resolver = "2"
members = [
    "crates/qubit-protocol",
    "crates/qubit-storage",
    "crates/qubit-mcp",
    "crates/qubit-sandbox",
    "crates/qubit-agents",
    "crates/qubit-core",
    "crates/qubit-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Qubit Ecosystem Core Team"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Runtime asynchroniczny i wielozadaniowość
tokio = { version = "1.54", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
futures = "0.3"

# Sieć, HTTP, WebSocket i Serializacja
axum = { version = "0.8", features = ["ws", "macros"] }
tower = { version = "0.5", features = ["util", "timeout"] }
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Protokół MCP i Generatory Schematów
rmcp = { version = "1.3", features = ["server", "transport-io", "transport-sse-server"] }
schemars = { version = "0.8", features = ["derive"] }
specta = { version = "=2.0.0-rc.20", features = ["derive"] }

# Baza danych, wektory i grafy
rusqlite = { version = "0.42", features = ["bundled"] }
tokio-rusqlite = "0.6"
refinery = { version = "0.8", features = ["rusqlite"] }
petgraph = "0.6"

# Izolacja systemowa i kontenery
cgroups-rs = "0.5"
bollard = "0.18"
kube = { version = "0.98", features = ["runtime", "client"] }

# Obserwowalność, telemetria i błędy
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
thiserror = "2.0"
anyhow = "1.0"
uuid = { version = "1.16", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true


2. Etap 1: Warstwa Kontraktów Domenowych i Generator Typów (qubit-protocol)

Crate qubit-protocol definiuje wspólny język komunikacji w całym ekosystemie Qubit. Odpowiada za dostarczenie silnie typowanych struktur danych transferowych (DTO), identyfikatorów sesji, zdarzeń pętli wykonawczej oraz manifestów możliwości, eliminując ryzyko niezgodności formatów między backendem a interfejsami użytkownika.

2.1. Cele Inżynieryjne Etapu 1

1. Zdefiniowanie struktur domenowych w czystym kodzie Rusta bez powiązań z konkretną implementacją transportu sieciowego.
2. Zaimplementowanie binarnego generatora gen_bindings.rs, który eksportuje definicje interfejsów TypeScript oraz schematy walidacji JSON Schema na podstawie atrybutów #[derive(Type, JsonSchema)].
3. Zagwarantowanie idempotentności generatora — plik wynikowy w pakiecie frontendowym jest nadpisywany wyłącznie w przypadku wykrycia faktycznych zmian w strukturach.

2.2. Struktury Domenowe i Model Sesji

Kluczowym elementem protokołu jest struktura SessionId, opakowująca identyfikator UUID v4. Wszystkie podłączane aplikacje (klient IDE, terminal CLI, aplikacja desktopowa) rejestrują się w kontekście tego samego identyfikatora sesji.

// crates/qubit-protocol/src/session.rs
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Reflex,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "role", content = "content")]
pub enum MessageRole {
    System(String),
    User(String),
    Assistant {
        text: Option<String>,
        tool_calls: Vec<ToolCallRequest>,
    },
    ToolResult {
        tool_call_id: String,
        output: String,
        is_error: bool,
    },
}


2.3. Manifesty Możliwości i Strumień Zdarzeń Silnika

Przy nawiązaniu połączenia każdy klient przesyła manifest definiujący narzędzia, które wnosi do wspólnej puli (np. operacje na plikach w edytorze kodu), oraz flagi określające gotowość do obsługi zapytań autoryzacyjnych.

// crates/qubit-protocol/src/manifest.rs
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::session::{ExecutionMode, SessionId};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ToolDefinitionDto {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub requires_tainted_isolation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClientCapabilityManifest {
    pub client_name: String,
    pub client_version: String,
    pub supported_tools: Vec<ToolDefinitionDto>,
    pub supports_elicitation: bool,
    pub workspace_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "event_type", content = "payload")]
pub enum EngineEvent {
    SessionCreated { session_id: SessionId },
    ClientConnected { client_id: String, manifest: ClientCapabilityManifest },
    ClientDisconnected { client_id: String },
    StepStarted { step_index: u32, mode: ExecutionMode },
    ToolExecuted { tool_call_id: String, success: bool },
    MemoryCompressionTriggered { preserved_tokens: usize },
    ElicitationRequested { request_id: String, message: String, schema: serde_json::Value },
    ErrorOccurred { code: String, message: String },
}


2.4. Implementacja Generatora Wiązań TypeScript (gen_bindings.rs)

Generator weryfikuje zawartość pliku docelowego przed wykonaniem operacji wejścia/wyjścia, chroniąc potoki watchera Vite przed niepotrzebnymi restartami.

// crates/qubit-protocol/src/bin/gen_bindings.rs
use specta::ts::{export, BigIntExportBehavior, ExportConfiguration};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new("../../packages/protocol/src/bindings.ts");
    
    let config = ExportConfiguration::default()
        .bigint(BigIntExportBehavior::Number);

    let mut declarations = Vec::new();
    declarations.push(export::<qubit_protocol::session::SessionId>(&config)?);
    declarations.push(export::<qubit_protocol::session::ExecutionMode>(&config)?);
    declarations.push(export::<qubit_protocol::session::ToolCallRequest>(&config)?);
    declarations.push(export::<qubit_protocol::session::MessageRole>(&config)?);
    declarations.push(export::<qubit_protocol::manifest::ToolDefinitionDto>(&config)?);
    declarations.push(export::<qubit_protocol::manifest::ClientCapabilityManifest>(&config)?);
    declarations.push(export::<qubit_protocol::manifest::EngineEvent>(&config)?);

    let ts_file_content = format!(
        "// Auto-generated by qubit-protocol (gen_bindings). DO NOT MODIFY DIRECTLY.\n\n{}\n",
        declarations.join("\n\n")
    );

    if let Ok(existing) = fs::read_to_string(output_path) {
        if existing == ts_file_content {
            println!("Wiązania TypeScript są aktualne. Pomijanie zapisu.");
            return Ok(());
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, ts_file_content)?;
    println!("Pomyślnie wygenerowano wiązania TypeScript: {:?}", output_path);
    Ok(())
}


3. Etap 2: Baza Danych SQLite WAL, Tabele Wektorowe i Stan Sesji (qubit-storage)

Warstwa trwałości danych musi łączyć zerowy narzut pamięciowy z możliwością wykonywania zapytań wektorowych oraz bezpiecznym utrwalaniem stanu po każdym kroku (checkpointing). Użycie zewnętrznego kontenera bazy danych (PostgreSQL, Qdrant) na serwerze z 1 GB RAM doprowadziłoby do natychmiastowej awarii z powodu braku pamięci, dlatego QubitCore wykorzystuje wbudowany proces SQLite.

3.1. Architektura Współbieżności SQLite WAL

Tryb WAL (Write-Ahead Logging) pozwala na nieblokujące odczyty przez wiele zadań asynchronicznych Tokio, lecz operacje zapisu w SQLite pozostają sekwencyjne. Próba równoczesnego zapisu z wielu zadań na współdzielonym obiekcie połączenia wywołuje błędy blokady SQLITE_BUSY oraz błędy wielowątkowości biblioteki C.

Architektura qubit-storage wprowadza ścisłą separację:

1. Dedykowany wątek piszący: Wszystkie operacje modyfikujące bazę (inserty wektorów, aktualizacje historii, zapis punktów kontrolnych) kierowane są przez asynchroniczny wrapper tokio-rusqlite do pojedynczego wątku posiadającego wyłączne prawo zapisu.
2. Pula połączeń odczytowych: Zadania odczytujące tworzą niezależne, lekkie połączenia w trybie wielowątkowym z ustawioną pragmą query_only = ON.

3.2. Dynamiczna Rejestracja sqlite-vec przez C FFI

Rozszerzenie wektorowe sqlite-vec musi zostać zarejestrowane w pamięci procesu przed otwarciem pierwszego połączenia do pliku bazodanowego. Wykorzystywana jest do tego natywna funkcja C FFI sqlite3_auto_extension().

// crates/qubit-storage/src/db.rs
use rusqlite::ffi::sqlite3_auto_extension;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};
use tokio_rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Błąd bazy danych SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Błąd asynchronicznego połączenia tokio-rusqlite: {0}")]
    AsyncSqlite(#[from] tokio_rusqlite::Error),
    #[error("Błąd operacji I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Błąd migracji refinery: {0}")]
    Migration(String),
}

extern "C" {
    fn sqlite3_vec_init(
        db: *mut rusqlite::ffi::sqlite3,
        pzErrMsg: *mut *mut std::os::raw::c_char,
        pApi: *const c_void,
    ) -> std::os::raw::c_int;
}

pub struct StorageEngine {
    write_conn: Connection,
    db_path: PathBuf,
}

impl StorageEngine {
    pub async fn initialize<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Globalna rejestracja rozszerzenia wektorowego sqlite-vec
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }

        let write_conn = Connection::open(&db_path).await?;

        // Konfiguracja parametrów wydajnościowych trybu WAL
        write_conn.call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "busy_timeout", 5000)?;
            Ok(())
        }).await?;

        let engine = Self { write_conn, db_path };
        engine.run_migrations().await?;
        Ok(engine)
    }

    pub async fn open_read_connection(&self) -> Result<Connection, StorageError> {
        let conn = Connection::open(&self.db_path).await?;
        conn.call(|c| {
            c.pragma_update(None, "query_only", "ON")?;
            c.pragma_update(None, "busy_timeout", 5000)?;
            Ok(())
        }).await?;
        Ok(conn)
    }

    async fn run_migrations(&self) -> Result<(), StorageError> {
        self.write_conn.call(|conn| {
            crate::migrations::runner().run(conn)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(())
        }).await.map_err(|e| StorageError::Migration(e.to_string()))
    }
}


3.3. Schemat Bazy Danych i Migracje SQL

Schemat łączy relacyjne tabele sesji, pełnotekstowy indeks FTS5, wirtualne tabele wektorowe vec0 oraz relacje krawędzi grafu wiedzy przetwarzanego w pamięci przez bibliotekę petgraph.

-- crates/qubit-storage/migrations/V1__initial_schema.sql

-- Główny rejestr aktywnych sesji
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    current_mode TEXT NOT NULL DEFAULT 'reflex',
    metadata TEXT
);

-- Historia wiadomości transkryptu
CREATE TABLE IF NOT EXISTS session_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Punkty kontrolne stanu wykonania (Checkpointing)
CREATE TABLE IF NOT EXISTS execution_checkpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL,
    state_blob BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(session_id, step_index)
);

-- Indeks pełnotekstowy FTS5 dla wyszukiwania leksykalnego
CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    document_id UNINDEXED,
    content,
    tokenize = 'porter unicode61'
);

-- Tabela wektorowa sqlite-vec (1536 wymiarów dla embeddingów)
CREATE VIRTUAL TABLE IF NOT EXISTS memory_vectors USING vec0(
    document_id TEXT PRIMARY KEY,
    embedding float[1536]
);

-- Relacje grafu wiedzy (konsolidowane w pamięci RAM przez petgraph)
CREATE TABLE IF NOT EXISTS knowledge_graph_edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    PRIMARY KEY(source_id, target_id, relation_type)
);


3.4. Implementacja Mechanizmu Checkpointingu

Po zakończeniu każdego kroku pętli decyzyjnej zrzut binarny stanu agenta jest utrwalany w tabeli execution_checkpoints. W razie awarii zasilania lub błędu panic! silnik wznawia przerwane zadanie w kilkanaście milisekund.

// crates/qubit-storage/src/checkpoint.rs
use crate::db::{StorageEngine, StorageError};
use qubit_protocol::session::SessionId;

impl StorageEngine {
    pub async fn save_checkpoint(
        &self,
        session_id: SessionId,
        step_index: u32,
        state_blob: &[u8],
    ) -> Result<(), StorageError> {
        let session_str = session_id.0.to_string();
        let bytes = state_blob.to_vec();
        let timestamp = chrono::Utc::now().timestamp_millis();

        self.write_conn.call(move |conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO execution_checkpoints (session_id, step_index, state_blob, created_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(session_id, step_index) DO UPDATE SET
                    state_blob = excluded.state_blob,
                    created_at = excluded.created_at"
            )?;
            stmt.execute(rusqlite::params![session_str, step_index, bytes, timestamp])?;
            Ok(())
        }).await.map_err(StorageError::from)
    }

    pub async fn load_latest_checkpoint(
        &self,
        session_id: SessionId,
    ) -> Result<Option<(u32, Vec<u8>)>, StorageError> {
        let session_str = session_id.0.to_string();
        self.write_conn.call(move |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT step_index, state_blob FROM execution_checkpoints
                 WHERE session_id = ?1 ORDER BY step_index DESC LIMIT 1"
            )?;
            let mut rows = stmt.query(rusqlite::params![session_str])?;
            if let Some(row) = rows.next()? {
                let step: u32 = row.get(0)?;
                let blob: Vec<u8> = row.get(1)?;
                Ok(Some((step, blob)))
            } else {
                Ok(None)
            }
        }).await.map_err(StorageError::from)
    }
}


4. Etap 3: Demon Sieciowy axum, Transporty Lokalne IPC i Menedżer Sesji (qubit-core / network)

Zastąpienie serwera Caddy wewnętrznym routerem axum eliminuje ryzyko awarii OOM i pozwala zamknąć pełną obsługę protokołów HTTP, WebSocket oraz gniazd uniksowych w pojedynczym pliku wykonywalnym. QubitCore działa w tle 24/7 jako centralny węzeł orkiestracji.

4.1. Architektura Centralnego Hubu Sesji (SessionHub)

SessionHub zarządza globalną mapą aktywnych sesji. Gdy użytkownik otwiera terminal, edytor kodu oraz aplikację desktopową, wszystkie te procesy podłączają się do tej samej instancji SessionInstance na podstawie identyfikatora session_id. Zdarzenia są rozgłaszane w czasie rzeczywistym do wszystkich podłączonych klientów za pomocą kanałów transmisyjnych tokio::sync::broadcast.

// crates/qubit-core/src/session_hub.rs
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use qubit_protocol::session::SessionId;
use qubit_protocol::manifest::{ClientCapabilityManifest, EngineEvent};

pub struct ConnectedClient {
    pub client_id: String,
    pub manifest: ClientCapabilityManifest,
}

pub struct SessionInstance {
    pub session_id: SessionId,
    pub clients: DashMap<String, ConnectedClient>,
    pub event_sender: broadcast::Sender<EngineEvent>,
}

#[derive(Clone)]
pub struct SessionHub {
    sessions: Arc<DashMap<SessionId, Arc<SessionInstance>>>,
}

impl SessionHub {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn get_or_create(&self, session_id: SessionId) -> Arc<SessionInstance> {
        self.sessions.entry(session_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(2048);
            Arc::new(SessionInstance {
                session_id,
                clients: DashMap::new(),
                event_sender: tx,
            })
        }).clone()
    }

    pub fn broadcast(&self, session_id: SessionId, event: EngineEvent) {
        if let Some(session) = self.sessions.get(&session_id) {
            let _ = session.event_sender.send(event);
        }
    }

    pub fn unregister_client(&self, session_id: SessionId, client_id: &str) {
        if let Some(session) = self.sessions.get(&session_id) {
            session.clients.remove(client_id);
            let _ = session.event_sender.send(EngineEvent::ClientDisconnected {
                client_id: client_id.to_string(),
            });
        }
    }
}


4.2. Serwer axum i Obsługa WebSocket

Router w axum udostępnia punkty kontrolne stanu zdrowia systemu, strumienie WebSocket dla synchronizacji stanu w czasie rzeczywistym oraz statyczną dystrybucję interfejsu QubitDash.

// crates/qubit-core/src/server.rs
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::session_hub::SessionHub;
use qubit_protocol::session::SessionId;
use qubit_protocol::manifest::EngineEvent;

pub struct CoreState {
    pub hub: SessionHub,
}

pub fn build_app_router(state: Arc<CoreState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/sessions/:id/ws", get(websocket_session_upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "QUBIT_CORE_RUNNING"
}

async fn websocket_session_upgrade(
    ws: WebSocketUpgrade,
    Path(raw_session_id): Path<uuid::Uuid>,
    State(state): State<Arc<CoreState>>,
) -> impl IntoResponse {
    let session = state.hub.get_or_create(SessionId(raw_session_id));
    ws.on_upgrade(move |socket| handle_session_stream(socket, session))
}

async fn handle_session_stream(mut socket: WebSocket, session: Arc<crate::session_hub::SessionInstance>) {
    let mut rx = session.event_sender.subscribe();

    loop {
        tokio::select! {
            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(event) => {
                        if let Ok(json_text) = serde_json::to_string(&event) {
                            if socket.send(axum::extract::ws::Message::Text(json_text)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("Klient WebSocket opuścił {} zdarzeń", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        // Deserializacja poleceń od klienta
                        tracing::debug!("Odebrano komunikat klienta: {}", text);
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}


4.3. Transport Lokalny IPC dla Narzędzi CLI

Klienci lokalni działający na tym samym węźle (np. proces qubit-cli) komunikują się z silnikiem przez gniazda uniksowe, eliminując narzut parsowania nagłówków HTTP oraz protokołu TCP.

// crates/qubit-core/src/ipc.rs
use std::path::Path;
use tokio::net::UnixListener;
use tracing::{error, info};

pub async fn run_unix_ipc_listener<P: AsRef<Path>>(socket_path: P) -> Result<(), std::io::Error> {
    let path = socket_path.as_ref();
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }

    let listener = UnixListener::bind(path)?;
    info!("Gniazdo IPC aktywne pod adresem: {:?}", path);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    tokio::spawn(async move {
                        // Obsługa asynchronicznego strumienia gniazda przez ramki protokołu
                    });
                }
                Err(err) => {
                    error!("Błąd nasłuchu gniazda IPC: {:?}", err);
                    break;
                }
            }
        }
    });

    Ok(())
}


5. Etap 4: Integracja Protokołu MCP, Dynamiczna Rejestracja i Capability ACL (qubit-mcp)

W architekturze Qubit silnik nie zawiera sztywnych integracji z narzędziami programistycznymi. Zamiast tego implementuje specyfikację Model Context Protocol (MCP) z użyciem oficjalnego SDK rmcp. Kluczowym wymogiem bezpieczeństwa jest ochrona przed atakami Indirect Prompt Injection w sytuacji, gdy jedna sesja łączy narzędzia wykonawcze systemu hosta z analizą niezaufanych danych zewnętrznych.

5.1. Cykl Życia Połączenia MCP i Dynamiczna Rejestracja

Gdy nowy program (np. edytor kodu) łączy się z silnikiem, funkcja serve() przeprowadza procedurę uścisku dłoni i wywołuje handler on_initialized. Po zarejestrowaniu narzędzi klienta, silnik rozsyła notyfikację informującą pozostałe interfejsy o zaktualizowaniu listy dostępnych możliwości.

// crates/qubit-mcp/src/server.rs
use rmcp::{
    model::ServerHandler,
    service::RequestContext,
    McpError,
};
use async_trait::async_trait;

pub struct QubitMcpBridge {
    pub client_identifier: String,
}

#[async_trait]
impl ServerHandler for QubitMcpBridge {
    async fn on_initialized(&self, context: &RequestContext) -> Result<(), McpError> {
        tracing::info!("Pomyślny handshake MCP dla klienta: {}", self.client_identifier);
        
        // Zgłoszenie zmiany dostępnych narzędzi do wszystkich subskrybentów
        context.peer.notify_resource_list_changed().await
            .map_err(|e| McpError::internal_error(format!("Błąd notyfikacji zasobów: {:?}", e)))?;
            
        Ok(())
    }
}


5.2. Wykrywanie Przestrzeni Roboczej przez MCP Roots

Prymityw MCP Roots pozwala klientom przekazać listę identyfikatorów URI wskazujących na katalogi robocze projektów użytkownika. Silnik odpytuje klienta o te ścieżki i automatycznie ogranicza operacje na plikach do wskazanego obszaru.

// crates/qubit-mcp/src/roots.rs
use rmcp::service::RequestContext;
use rmcp::McpError;

pub async fn query_client_roots(context: &RequestContext) -> Result<Vec<String>, McpError> {
    let roots_result = context.peer.list_roots().await
        .map_err(|e| McpError::internal_error(format!("Nie udało się pobrać Roots: {:?}", e)))?;
        
    let root_uris = roots_result.roots.into_iter()
        .map(|entry| entry.uri)
        .collect();
        
    Ok(root_uris)
}


5.3. Capability ACL i Izolacja Niezaufanego Kontekstu (TaintedContext)

Współdzielenie sesji stwarza poważny wektor ataku: analizowany plik binarny lub niezaufane repozytorium może zawierać w ciągach znaków (strings) lub komentarzach ukryte polecenia sterujące modelem LLM. Po dekompilacji model mógłby zlecić wykonanie destrukcyjnego polecenia w powłoce podłączonego edytora IDE (run_terminal).

Architektura Zero-Trust MCP w qubit-mcp eliminuje to zagrożenie poprzez:

1. Oznaczenie kontekstu flagą TaintedContext: Zadania przetwarzające obce, niezaufane dane automatycznie otrzymują ten znacznik.
2. Filtrację narzędzi: W kontekście oznaczonym jako skażony (TaintedContext) narzędzia dotykające systemu operacyjnego hosta (run_terminal, apply_diff, execute_bash) są bezwzględnie blokowane na poziomie dyspozytora.
3. Wymuszoną autoryzację człowieka (Elicitation): Każda próba modyfikacji kodu lub uruchomienia procesu wymaga jawnego zatwierdzenia przez użytkownika w interfejsie QubitDash.

Zasady filtrowania uprawnień dla narzędzi w sesji przedstawia poniższa tabela:

Identyfikator Narzędzia	Dostęp Domyślny	Dostęp w TaintedContext	Wymóg Elicitation (Zgoda Człowieka)
read_file	Dozwolony wewnątrz Roots	Dozwolony (tylko odczyt)	Nie
apply_diff	Dozwolony	Zablokowany	Tak (podgląd diffa w UI)
run_terminal	Dozwolony	Zablokowany	Tak (bezwzględna akceptacja komendy)
decompile_function	Dozwolony w piaskownicy	Dozwolony w piaskownicy	Nie
http_request	Ograniczony domenowo	Zablokowany	Tak (dla nowych domen zewnętrznych)

// crates/qubit-mcp/src/acl.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpSecurityError {
    #[error("Naruszenie zasad bezpieczeństwa: Narzędzie '{0}' jest zablokowane w kontekście TaintedContext")]
    TaintedAccessBlocked(String),
    #[error("Odrzucono autoryzację człowieka (Elicitation denied)")]
    HumanApprovalRejected,
}

pub struct SecurityContext {
    pub is_tainted: bool,
}

impl SecurityContext {
    pub fn enforce_tool_permissions(&self, tool_name: &str) -> Result<(), McpSecurityError> {
        if self.is_tainted {
            match tool_name {
                "run_terminal" | "apply_diff" | "delete_file" | "execute_bash" => {
                    return Err(McpSecurityError::TaintedAccessBlocked(tool_name.to_string()));
                }
                _ => {}
            }
        }
        Ok(())
    }
}


5.4. Bramki Autoryzacji Człowieka przez MCP Elicitation

Mechanizm Elicitation wstrzymuje wykonanie zadania Tokio do momentu nadesłania odpowiedzi z interfejsu użytkownika, wykorzystując ustrukturyzowane formularze JSON Schema wygenerowane przez bibliotekę schemars.

// crates/qubit-mcp/src/elicitation.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommandExecutionApproval {
    pub command: String,
    pub directory: String,
    pub explanation: String,
    pub approve: bool,
}


6. Etap 5: Pętla Wykonawcza, Router Trybów i Kompresja Pamięci (qubit-agents)

Mózg decyzyjny silnika stanowi zaadaptowany i przepisany do języka Rust framework Hermes Agent (Nous Research). Przeniesienie logiki do Rusta eliminuje ograniczenia jednowątkowej pętli Pythona, wprowadzając pełną współbieżność zadań, dynamiczny router trybów oraz zaawansowaną kompresję kontekstu z gwarancją odporności na awarie modeli pomocniczych.

6.1. Router Trybów: Lekki (Reflex) vs. Ciężki (Deep)

Router dynamicznie dobiera profil wykonawczy w zależności od intencji użytkownika:

* Tryb Lekki (Reflex): Minimalizuje narzut CPU i pamięci. Pomija ciężkie zapytania wektorowe i grafy wiedzy, kierując zapytanie bezpośrednio do szybkich modeli językowych. Służy do prostych edycji kodu, odpowiedzi w CLI i automatyzacji w tle.
* Tryb Ciężki (Deep): Aktywuje wieloetapowe planowanie, odpytywanie bazy wektorowej, uruchamianie sub-agentów oraz powoływanie odizolowanych piaskownic kontenerowych (np. pod moduł Reverse Engineeringu).

// crates/qubit-agents/src/router.rs
use qubit_protocol::session::ExecutionMode;

pub struct ModeRouter;

impl ModeRouter {
    pub fn resolve_mode(user_prompt: &str, manual_override: Option<ExecutionMode>) -> ExecutionMode {
        if let Some(mode) = manual_override {
            return mode;
        }

        let normalized = user_prompt.to_lowercase();
        if normalized.contains("dekompiluj")
            || normalized.contains("reverse engineering")
            || normalized.contains("znajdź lukę")
            || normalized.contains("przeanalizuj cały projekt") {
            ExecutionMode::Deep
        } else {
            ExecutionMode::Reflex
        }
    }
}


6.2. Czterofazowy Algorytm Kompresji Kontekstu

Kompresor uruchamia się wewnątrz pętli narzędziowej, gdy wielkość transkryptu przekroczy 50% okna kontekstowego (threshold: 0.50). Algorytm realizuje cztery sekwencyjne fazy:

* Faza 1 (Przycięcie tanich wyjść narzędzi): Usuwanie długich, redundantnych listingów plików lub powtarzalnych komunikatów o statusie.
* Faza 2 (Podział i ochrona ogona): Bezwzględne zabezpieczenie ostatnich 20 wiadomości w historii (protect_last_n: 20). Wyznaczenie środkowego segmentu do streszczenia.
* Faza 3 (Ustrukturyzowane streszczenie): Wygenerowanie podsumowania przez model pomocniczy. Docelowy budżet tokenów wynosi: \text{Budżet Streszczenia} = \text{Tokeny Środka} \times 0.20 z zachowaniem limitu dolnego równego 2 000 tokenów oraz limitu górnego równego \min(\text{Długość Okna} \times 0.05, 12 000) tokenów.
* Faza 4 (Złożenie transkryptu): Ponowne połączenie nagłówka (Head), wygenerowanego streszczenia (Summary z rolą nienaruszającą naprzemienności) oraz nienaruszonego ogona (Tail).

6.3. Ochrona Przed Utratą Kontekstu i Drabina Deterministycznych Fallbacków

Oryginalna implementacja Pythona w Hermes Agent zawierała wadę: jeśli model pomocniczy posiadał mniejsze okno niż model główny, błąd przepełnienia kontekstu powodował ciche porzucenie środkowych wiadomości bez streszczenia.

W qubit-agents moduł metadanych weryfikuje okno modelu pomocniczego przed zleceniem podsumowania. W przypadku awarii sieciowych lub przeciążenia dostawcy silnik stosuje drabinę deterministycznych fallbacków:

// crates/qubit-agents/src/compressor.rs
use tracing::{error, warn};

pub enum SummaryStrategy {
    CallAuxiliaryLlm,
    DeterministicRuleBased,
    AbortDegraded,
}

pub struct FallbackManager {
    consecutive_timeouts: u32,
    consecutive_529_errors: u32,
}

impl FallbackManager {
    pub fn new() -> Self {
        Self {
            consecutive_timeouts: 0,
            consecutive_529_errors: 0,
        }
    }

    pub fn evaluate_api_response(&mut self, http_status: u16) -> SummaryStrategy {
        match http_status {
            408 | 504 => {
                self.consecutive_timeouts += 1;
                if self.consecutive_timeouts >= 2 {
                    warn!("Wykryto powtórny timeout API. Aktywacja DETERMINISTIC_SUMMARY_ROUTE");
                    SummaryStrategy::DeterministicRuleBased
                } else {
                    SummaryStrategy::CallAuxiliaryLlm
                }
            }
            529 => {
                self.consecutive_529_errors += 1;
                if self.consecutive_529_errors >= 3 {
                    error!("Dostawca LLM przeciążony (3x 529). Przejście do deterministycznego streszczenia");
                    SummaryStrategy::DeterministicRuleBased
                } else {
                    SummaryStrategy::CallAuxiliaryLlm
                }
            }
            _ => SummaryStrategy::AbortDegraded,
        }
    }
}


6.4. Proaktywna Pamięć w Tle z Zerowym Opóźnieniem (Zero-Latency Injection)

Pamięć długoterminowa działa asynchronicznie poza ścieżką krytyczną generowania odpowiedzi:

1. Po każdej turze asynchroniczne zadanie Tokio ekstrahuje kluczowe fakty i zapisuje embeddingi w tabeli memory_vectors w sqlite-vec.
2. Zbuforowane wyniki z poprzedniej tury są wstrzykiwane bezpośrednio do promptu systemowego nowej tury, co gwarantuje 0 ms narzutu opóźnienia na czas generowania odpowiedzi.
3. Podsystem chroniony jest bezpiecznikiem Circuit Breaker: po zarejestrowaniu 5 kolejnych błędów z rzędu podsystem pamięci zostaje uśpiony na 2 minuty, zapobiegając destabilizacji głównej pętli dialogowej.

6.5. Innowacja: Air-Gapped Two-Tier Memory & Context Fact Dispatcher (QubitDocs)

Nowa innowacja architektoniczna integrująca otwartą bazę wiedzy (QubitDocs) z wewnętrzną warstwą pamięci operacyjnej QubitCore oraz selekcjonowaniem wyłącznie istotnych faktów kontekstowych przed głównym modelem decyzyjnym:

1. Dwa odizolowane skarbce (Two-Tier Air-Gapped Memory):
   * **System Vault** (`.qubit/system/`): ukryta pamięć operacyjna — logi kompresji, checkpointy oraz destylowane fakty z czatów. Główny model (Claude / Gemini Pro) nie wie o jej istnieniu.
   * **User Vault** (QubitDocs / Obsidian `docs/`): jawna baza wiedzy użytkownika — notatki w czystym Markdown, plany i wiki.

2. Context Fact Filter / Prompt Dispatcher:
   * Mały model pomocniczy (Flint 40B, Groq, Ollama) pełni rolę filtra faktów i dyspozera promptu: po otrzymaniu promptu od użytkownika błyskawicznie analizuje treść i selekcjonuje z bazy QubitDocs tylko te fakty, które są precyzyjnie potrzebne do danego pytania.
   * Do głównego, drogiego modelu trafia zwięzły ekstrakt (np. ~300 tokenów zamiast 80 000 tokenów), co drastycznie ogranicza koszty i chroni limity subskrypcji AI.

3. Promocja ekosystemu: Użytkownik widzi, że Qubit sam buduje mu bazę wiedzy w QubitDocs i Obsidian bez wysiłku.

7. Etap 6: Nadzór Tokio Supervisora, Kontrola Zasobów cgroups v2 i Ochrona OOM (qubit-sandbox / cgroups)

Działanie na maszynach o zasobach 1–2 GB RAM wymaga rygorystycznej ochrony przed jądrowym mechanizmem OOM Killer, który w przypadku wyczerpania pamięci zabija procesy w sposób arbitralny.

7.1. Tokio Supervisor i Izolacja Paniki

Każde zadanie agenta oraz każde wywołanie narzędzia wykonywane jest w odizolowanym zadaniu Tokio z jawnym limitem czasu (timeout) oraz osłoną przed paniką (std::panic::AssertUnwindSafe). Awaria pojedynczego zadania nie destabilizuje demona QubitCore.

// crates/qubit-sandbox/src/supervisor.rs
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::time::Duration;
use futures::FutureExt;
use tokio::time::timeout;
use tracing::error;

pub async fn execute_supervised<F, T>(
    identifier: &str,
    task_future: F,
    time_limit: Duration,
) -> Result<T, String>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = tokio::spawn(async move {
        AssertUnwindSafe(task_future).catch_unwind().await
    });

    match timeout(time_limit, handle).await {
        Ok(join_result) => match join_result {
            Ok(unwind_result) => unwind_result.map_err(|panic_info| {
                error!("Zadanie '{}' rzuciło panikę: {:?}", identifier, panic_info);
                format!("Panic w module: {}", identifier)
            }),
            Err(join_err) => {
                error!("Błąd Tokio JoinHandle w '{}': {:?}", identifier, join_err);
                Err("Tokio Executor Error".into())
            }
        },
        Err(_) => {
            error!("Zadanie '{}' przekroczyło limit czasu", identifier);
            Err("Timeout".into())
        }
    }
}


7.2. Monitorowanie Pamięci Anonimowej w cgroups v2

Całkowite zużycie pamięci raportowane przez jądro Linuksa w pliku memory.current obejmuje bufory pamięci podręcznej plików (file w memory.stat), które są bezpiecznie zwalniane przez jądro w razie potrzeby.

Algorytm Memory Backpressure weryfikuje wyłącznie pamięć anonimową (anon):

\text{Faktyczne Obciążenie Pamięcią} = \frac{\text{memory.stat}[anon]}{\text{Limit Grupy Cgroups}} \times 100\%

// crates/qubit-sandbox/src/memory_guard.rs
use tokio::fs;

pub struct CgroupV2MemorySnapshot {
    pub anon_bytes: u64,
    pub max_limit_bytes: u64,
}

impl CgroupV2MemorySnapshot {
    pub async fn capture() -> Result<Self, std::io::Error> {
        let stat_text = fs::read_to_string("/sys/fs/cgroup/memory.stat").await?;
        let max_text = fs::read_to_string("/sys/fs/cgroup/memory.max").await?;

        let mut anon_bytes = 0;
        for line in stat_text.lines() {
            if line.starts_with("anon ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    anon_bytes = parts[1].parse::<u64>().unwrap_or(0);
                }
                break;
            }
        }

        let max_limit_bytes = max_text.trim().parse::<u64>().unwrap_or(u64::MAX);

        Ok(Self { anon_bytes, max_limit_bytes })
    }

    pub fn calculate_pressure_ratio(&self) -> f64 {
        if self.max_limit_bytes == 0 || self.max_limit_bytes == u64::MAX {
            return 0.0;
        }
        (self.anon_bytes as f64 / self.max_limit_bytes as f64) * 100.0
    }
}


7.3. Działanie Kolejki Memory Backpressure

Monitor pamięci cyklicznie analizuje stan systemu:

* Przekroczenie progu 80%: Wstrzymanie pobierania nowych zadań z kolejki. Nowe polecenia oczekują w stanie zbuforowanym na zwolnienie zasobów.
* Przekroczenie progu 85%: Wywołanie systemowego alertu przeciążenia oraz wymuszenie wyczyszczenia pamięci podręcznej SQLite i zrzucenia zbędnych struktur z pamięci RAM.

Dla procesów potomnych CLI tworzona jest dedykowana podgrupa cgroups v2 (cgroups-rs). W razie ubicia procesu kodem wyjścia 137, silnik odczytuje licznik oom_kill w pliku memory.events, co pozwala jednoznacznie odróżnić przekroczenie limitu pamięci od ręcznego sygnału SIGKILL.

8. Etap 7: Orkiestracja Piaskownic Dockerowych i Moduł Reverse Engineeringu (sandboxes/re-ghidra)

Ciężkie zadania analityczne w Trybie Deep realizowane są w odizolowanych kontenerach zarządzanych za pośrednictwem biblioteki bollard (Docker API). Pozwala to na bezpieczną analizę nieznanych plików binarnych bez ryzyka infekcji systemu hosta.

8.1. Architektura Piaskownicy RE (sandboxes/re-ghidra)

Obraz kontenerowy bazuje na 64-bitowym środowisku Java JDK 21 oraz bezgłowej instalacji środowiska Ghidra (wersja 12.1.4 lub 12.2). Wewnątrz kontenera działają dwa serwery MCP:

1. reversing-mcp: Szybka analiza nagłówków PE/ELF, entropii sekcji, ciągów znaków oraz wywołania narzędzi CLI (binwalk, jadx, tshark).
2. ReVa / GhidraMCP: Głęboka integracja z dekompilatorem Ghidry, udostępniająca przeszukiwanie referencji krzyżowych (xrefs) oraz punktową dekompilację.

Kontener uruchamiany jest z całkowicie odciętą siecią zewnętrzną (--network none) oraz uprawnieniami użytkownika nieuprzywilejowanego.

8.2. Pięcioetapowa Metodyka Progressive Disclosure

Aby nie przepełnić okna kontekstowego modelu językowego dekompilacją całego programu, orkiestrator prowadzi agenta przez pięć ściśle zdefiniowanych faz badawczych:

[Krok 1: Orientacja]
  │  Narzędzia: get_program_info, list_imports, list_strings, entropy
  │  Zużycie: ~500 tokenów. Cel: Identyfikacja architektury, bibliotek i wektorów wejścia.
  ▼
[Krok 2: Punktowe Szukanie & Xrefs]
  │  Narzędzia: get_xrefs_to, autorski skrypt wyszukiwania stałych liczbowych
  │  Zużycie: ~800 tokenów. Cel: Zlokalizowanie adresów procedur kryptograficznych lub deszyfrujących.
  ▼
[Krok 3: Punktowa Dekompilacja]
  │  Narzędzia: decompile_function_by_address (unikanie kolizji nazw)
  │  Zużycie: ~1500 tokenów. Cel: Pobranie kodu C wyłącznie badanej funkcji.
  ▼
[Krok 4: Utrwalenie Wiedzy w Projekcie Ghidry (Write-Back)]
  │  Narzędzia: rename_symbol, add_comment
  │  Cel: Zapis odgadniętych nazw zmiennych i funkcji bezpośrednio do bazy Ghidry.
  ▼
[Krok 5: Analiza Grafu Wywołań & Raportowanie]
  │  Narzędzia: get_callers, get_callees + strumieniowanie zdarzeń do QubitDasha
  │  Cel: Dekompilacja funkcji nadrzędnych, które zawierają już nazwane symbole z Kroku 4.


8.3. Autorski Skrypt Wyszukiwania Stałych Liczbowych

Standardowe implementacje GhidraMCP nie posiadają narzędzia do wyszukiwania stałych skalarnych, co zmusza modele do nieefektywnej dekompilacji setek funkcji. Do piaskownicy dołączany jest autorski skrypt wyszukujący stałe bezpośrednio w instrukcjach maszynowych:

# sandboxes/re-ghidra/scripts/find_scalar_constants.py
from ghidra.program.model.scalar import Scalar

def locate_scalar_in_instructions(target_value):
    matching_addresses = []
    program_listing = currentProgram.getListing()
    instruction_iter = program_listing.getInstructions(True)

    for instr in instruction_iter:
        for idx in range(instr.getNumOperands()):
            scalar = instr.getScalar(idx)
            if scalar and scalar.getValue() == target_value:
                matching_addresses.append(instr.getAddress().toString())
                
    return matching_addresses


9. Etap 8: Przygotowanie Pod SaaS, Bazy Database-per-Tenant i Klaster K8s (qubit-cloud)

Ostatni etap prac przekształca lokalnego demona w skalowalną, wielodostępną usługę subskrypcyjną (Plan Pro) z pełną izolacją kryptograficzną danych klientów.

9.1. Model Database-per-Tenant z SQLCipher

Zamiast łączyć dane wielu użytkowników w jednej centralnej bazie danych, system wdraża model Database-per-Tenant:

* Każdy subskrybent posiada dedykowany plik bazy SQLite w trybie WAL.
* Baza jest szyfrowana w czasie rzeczywistym 256-bitowym kluczem AES za pomocą biblioteki rusqlite z flagą bundled-sqlcipher.
* Klucze szyfrujące zarządzane są przez centralny moduł KMS i wstrzykiwane wyłącznie na czas trwania aktywnej sesji użytkownika.
* Gwarantuje to fizyczną separację tabel wektorowych sqlite-vec oraz uniemożliwia wyciek danych między tenantami na poziomie silnika bazy.

9.2. Orkiestracja Kubernetes przez kube-rs

W środowisku chmurowym sterowanie kontenerami realizowane jest za pomocą oficjalnego crate'u kube-rs:

1. Dla każdego zadania w Trybie Deep silnik tworzy izolowaną przestrzeń nazw (Namespace) z twardymi limitami ResourceQuota.
2. Pody piaskownicy uruchamiane są w restrykcyjnym profilu bezpieczeństwa (Pod Security Standards: Restricted) z zablokowaną eskalacją uprawnień (allowPrivilegeEscalation: false).
3. Po zakończeniu analizy przestrzeń nazw jest usuwana.

9.3. Hybrydowy Compute Offloading i Zewnętrzny Caddy Edge Proxy

Na poziomie produkcyjnym przed klastrem umieszczany jest niezależny serwer Caddy działający jako Edge Proxy:

* Zarządza automatycznymi certyfikatami SSL/TLS (Let's Encrypt / ZeroSSL).
* Komunikuje się z QubitCore przez bezpieczne gniazdo Unix Domain Socket z flagą --resume.
* Posiada wymuszony parametr idle_timeout: 5m, co zapobiega wyciekom pamięci w procesie Go.
* Funkcja Compute Offloading pozwala użytkownikowi podpiąć swój domowy komputer PC (8+ GB RAM) jako zdalny węzeł wykonawczy MCP do chmurowego QubitCore, automatycznie delegując na niego lokalne modele językowe lub ciężkie zadania dekompilacji.

10. Macierz Weryfikacji, Testy Integracyjne i Procedury CI/CD

Wdrożenie każdego etapu podlega ścisłej weryfikacji w zautomatyzowanym potoku Continuous Integration.

10.1. Macierz Zgodności Komponentów i Kryteria Gotowości (DoD)

Poniższa tabela podsumowuje kluczowe technologie, kryteria akceptacji oraz docelowe limity pamięciowe dla poszczególnych etapów wdrożenia silnika:

Etap	Główne Crate'y i Narzędzia	Kryterium Akceptacji (Definition of Done)	Limit Pamięci RAM
Etap 1	qubit-protocol, specta, schemars	Bezbłędna generacja bindings.ts, 100% zgodności typów z TS	< 5 MB (proces CLI)
Etap 2	qubit-storage, rusqlite, sqlite-vec	Zapis wektorów FTS5, czas odzyskania stanu po panic < 10 ms	< 15 MB
Etap 3	qubit-core, axum, tokio::net	Współdzielona sesja na 5 równoległych klientów, obsługa WS i IPC	< 25 MB
Etap 4	qubit-mcp, rmcp	Blokada run_terminal w TaintedContext, Elicitation w UI	< 30 MB
Etap 5	qubit-agents, tokio	Pomyślna kompresja przy 50% progu, testy drabin fallbacku	< 45 MB
Etap 6	qubit-sandbox, cgroups-rs	Wstrzymanie kolejki przy >80% anon RAM, izolacja paniki	< 50 MB
Etap 7	sandboxes/re-ghidra, bollard	5-krokowe Progressive Disclosure, brak wycieku sieciowego	Zależny od kontenera
Etap 8	qubit-cloud, sqlcipher, kube-rs	Szyfrowanie bazy AES-256 per tenant, orkiestracja w K8s	< 60 MB (demon)

10.2. Procedura Wdrożeniowa w Repozytorium GitHub

Wdrożenie kodu do repozytorium https://github.com/Kvbi213/QubitCore.git realizowane jest według następujących kroków:

1. Inicjalizacja Monorepo: Zbudowanie struktury katalogów i dodanie głównego pliku Cargo.toml.
2. Automatyzacja CI (GitHub Actions):
  * Weryfikacja formatowania kodu: cargo fmt --check
  * Statyczna analiza lintera: cargo clippy --all-targets --all-features -- -D warnings
  * Testy jednostkowe i integracyjne: cargo test --workspace
  * Weryfikacja spójności typów TS: uruchomienie generatora cargo run -p qubit-protocol --bin gen_bindings i sprawdzenie czystości drzewa git (git diff --exit-code).
3. Audyt Zależności: Regularne skanowanie bezpieczeństwa za pomocą narzędzi cargo audit oraz cargo deny.
