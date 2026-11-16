//! Qubit Storage Engine
//! Embedded SQLite storage with WAL mode, optimized memory pragmas (<60 MB RAM budget),
//! and a dedicated single-threaded asynchronous writer actor (Actor Pattern via Tokio mpsc + oneshot)
//! to completely eliminate SQLITE_BUSY and database locking hazards under concurrent agent swarms.

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Lock error: {0}")]
    Lock(String),
    #[error("Channel communication error: {0}")]
    Channel(String),
    #[error("Writer actor terminated unexpectedly")]
    WriterTerminated,
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRecord {
    pub id: String,
    pub agent_id: Option<String>,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub id: i64,
    pub agent_id: String,
    pub step_index: i64,
    pub state_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunkRecord {
    pub id: String,
    pub agent_id: Option<String>,
    pub content: String,
    pub embedding_json: Option<String>,
    pub created_at: String,
}

/// Commands processed exclusively by the dedicated writer actor
enum WriteCommand {
    InsertMessage {
        id: String,
        agent_id: Option<String>,
        role: String,
        content: String,
        metadata: Option<String>,
        responder: oneshot::Sender<Result<()>>,
    },
    UpsertAgent {
        id: String,
        name: String,
        role: String,
        status: String,
        responder: oneshot::Sender<Result<()>>,
    },
    UpdateAgentStatus {
        id: String,
        status: String,
        responder: oneshot::Sender<Result<()>>,
    },
    SaveCheckpoint {
        agent_id: String,
        step_index: i64,
        state_json: String,
        responder: oneshot::Sender<Result<()>>,
    },
    InsertMemoryChunk {
        id: String,
        agent_id: Option<String>,
        content: String,
        embedding_json: Option<String>,
        responder: oneshot::Sender<Result<()>>,
    },
    SetConfig {
        key: String,
        value: String,
        responder: oneshot::Sender<Result<()>>,
    },
    WalCheckpoint {
        mode: String,
        responder: oneshot::Sender<Result<qubit_protocol::WalCheckpointResultDto>>,
    },
    ExecuteRaw {
        sql: String,
        responder: oneshot::Sender<Result<usize>>,
    },
}

#[derive(Clone)]
pub struct Storage {
    writer_tx: mpsc::Sender<WriteCommand>,
    reader_conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Storage {
    /// Initialize storage with dedicated background writer actor and concurrent reader connection
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let is_memory = path_ref.to_string_lossy() == ":memory:";

        let (writer_conn, reader_conn) = if is_memory {
            // Shared in-memory database using URI
            let uri = "file:qubit_mem?mode=memory&cache=shared";
            let flags = OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE;
            let w = Connection::open_with_flags(uri, flags)?;
            let r = Connection::open_with_flags(uri, flags)?;
            (w, r)
        } else {
            if let Some(parent) = path_ref.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let w = Connection::open(path_ref)?;
            let r = Connection::open_with_flags(
                path_ref,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
            )?;
            (w, r)
        };

        // Apply strict low-footprint pragmas on writer
        writer_conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA mmap_size = 268435456;
             PRAGMA cache_size = -2000;
             PRAGMA busy_timeout = 5000;",
        )?;

        // Apply read pragmas on reader
        let _ = reader_conn.execute_batch(
            "PRAGMA query_only = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -2000;
             PRAGMA mmap_size = 268435456;",
        );

        // Run baseline migrations synchronously on write connection before starting actor
        Self::run_migrations(&writer_conn)?;

        // Spawn dedicated background writer actor
        let (tx, rx) = mpsc::channel(2048);
        std::thread::Builder::new()
            .name("qubit-storage-writer".into())
            .spawn(move || {
                Self::writer_actor_loop(writer_conn, rx);
            })
            .map_err(|e| StorageError::Io(e))?;

        Ok(Self {
            writer_tx: tx,
            reader_conn: Arc::new(Mutex::new(reader_conn)),
            db_path: path_ref.to_path_buf(),
        })
    }

    /// Open in-memory storage for testing
    pub fn open_in_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    /// Path to current database
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Baseline schema migrations
    fn run_migrations(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                agent_id TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS memory_chunks (
                id TEXT PRIMARY KEY,
                agent_id TEXT,
                content TEXT NOT NULL,
                embedding_json TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS system_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_messages_agent ON chat_messages(agent_id);
            CREATE INDEX IF NOT EXISTS idx_checkpoints_agent ON agent_checkpoints(agent_id);
            CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory_chunks(agent_id);",
        )?;

        Ok(())
    }

    /// Main loop executed inside the dedicated writer thread
    fn writer_actor_loop(mut conn: Connection, mut rx: mpsc::Receiver<WriteCommand>) {
        debug!("Qubit storage writer actor started.");
        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                WriteCommand::InsertMessage {
                    id,
                    agent_id,
                    role,
                    content,
                    metadata,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "INSERT INTO chat_messages (id, agent_id, role, content, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![id, agent_id, role, content, metadata],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::UpsertAgent {
                    id,
                    name,
                    role,
                    status,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "INSERT INTO agents (id, name, role, status, updated_at) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
                             ON CONFLICT(id) DO UPDATE SET
                                name = excluded.name,
                                role = excluded.role,
                                status = excluded.status,
                                updated_at = CURRENT_TIMESTAMP",
                            params![id, name, role, status],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::UpdateAgentStatus {
                    id,
                    status,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "UPDATE agents SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                            params![status, id],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::SaveCheckpoint {
                    agent_id,
                    step_index,
                    state_json,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "INSERT INTO agent_checkpoints (agent_id, step_index, state_json) VALUES (?1, ?2, ?3)",
                            params![agent_id, step_index, state_json],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::InsertMemoryChunk {
                    id,
                    agent_id,
                    content,
                    embedding_json,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "INSERT INTO memory_chunks (id, agent_id, content, embedding_json) VALUES (?1, ?2, ?3, ?4)",
                            params![id, agent_id, content, embedding_json],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::SetConfig {
                    key,
                    value,
                    responder,
                } => {
                    let res = conn
                        .execute(
                            "INSERT INTO system_config (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
                             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
                            params![key, value],
                        )
                        .map(|_| ())
                        .map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
                WriteCommand::WalCheckpoint { mode, responder } => {
                    let start = std::time::Instant::now();
                    let pragma_sql = match mode.to_uppercase().as_str() {
                        "RESTART" => "PRAGMA wal_checkpoint(RESTART)",
                        "TRUNCATE" => "PRAGMA wal_checkpoint(TRUNCATE)",
                        "FULL" => "PRAGMA wal_checkpoint(FULL)",
                        _ => "PRAGMA wal_checkpoint(PASSIVE)",
                    };
                    let res = (|| -> Result<qubit_protocol::WalCheckpointResultDto> {
                        let mut stmt = conn.prepare(pragma_sql)?;
                        let mut rows = stmt.query([])?;
                        if let Some(row) = rows.next()? {
                            let busy: i64 = row.get(0)?;
                            let log: i64 = row.get(1)?;
                            let checkpointed: i64 = row.get(2)?;
                            Ok(qubit_protocol::WalCheckpointResultDto {
                                success: busy == 0,
                                checkpoint_mode: mode.clone(),
                                wal_pages_checkpointed: checkpointed,
                                wal_pages_remaining: log - checkpointed,
                                duration_ms: start.elapsed().as_millis() as u64,
                            })
                        } else {
                            Ok(qubit_protocol::WalCheckpointResultDto {
                                success: true,
                                checkpoint_mode: mode.clone(),
                                wal_pages_checkpointed: 0,
                                wal_pages_remaining: 0,
                                duration_ms: start.elapsed().as_millis() as u64,
                            })
                        }
                    })();
                    let _ = responder.send(res);
                }
                WriteCommand::ExecuteRaw { sql, responder } => {
                    let res = conn.execute(&sql, []).map_err(StorageError::Sqlite);
                    let _ = responder.send(res);
                }
            }
        }
        info!("Qubit storage writer actor gracefully terminated.");
    }

    // =========================================================================
    // ASYNCHRONOUS WRITE OPERATIONS (Dispatched via Writer Actor)
    // =========================================================================

    /// Insert a chat message into persistent storage asynchronously
    pub async fn insert_message(&self, id: &str, role: &str, content: &str) -> Result<()> {
        self.insert_message_full(id, None, role, content, None)
            .await
    }

    /// Insert a message associated with a specific agent and optional metadata
    pub async fn insert_message_full(
        &self,
        id: &str,
        agent_id: Option<&str>,
        role: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::InsertMessage {
                id: id.to_string(),
                agent_id: agent_id.map(String::from),
                role: role.to_string(),
                content: content.to_string(),
                metadata: metadata.map(String::from),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Insert or update agent descriptor
    pub async fn upsert_agent(
        &self,
        id: &str,
        name: &str,
        role: &str,
        status: &str,
    ) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::UpsertAgent {
                id: id.to_string(),
                name: name.to_string(),
                role: role.to_string(),
                status: status.to_string(),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Update agent lifecycle status
    pub async fn update_agent_status(&self, id: &str, status: &str) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::UpdateAgentStatus {
                id: id.to_string(),
                status: status.to_string(),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Record agent execution checkpoint
    pub async fn save_checkpoint(
        &self,
        agent_id: &str,
        step_index: i64,
        state_json: &str,
    ) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::SaveCheckpoint {
                agent_id: agent_id.to_string(),
                step_index,
                state_json: state_json.to_string(),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Record a memory chunk with optional embedding vector
    pub async fn insert_memory_chunk(
        &self,
        id: &str,
        agent_id: Option<&str>,
        content: &str,
        embedding_json: Option<&str>,
    ) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::InsertMemoryChunk {
                id: id.to_string(),
                agent_id: agent_id.map(String::from),
                content: content.to_string(),
                embedding_json: embedding_json.map(String::from),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    // =========================================================================
    // CONCURRENT READ OPERATIONS (Non-blocking via Reader Connection)
    // =========================================================================

    /// Get single agent by ID
    pub fn get_agent(&self, id: &str) -> Result<Option<AgentRecord>> {
        let conn = self
            .reader_conn
            .lock()
            .map_err(|e| StorageError::Lock(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, role, status, created_at, updated_at FROM agents WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(AgentRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all registered agents
    pub fn list_agents(&self) -> Result<Vec<AgentRecord>> {
        let conn = self
            .reader_conn
            .lock()
            .map_err(|e| StorageError::Lock(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, role, status, created_at, updated_at FROM agents ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Retrieve conversation messages with optional agent filtering
    pub fn get_messages(
        &self,
        agent_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ChatMessageRecord>> {
        let conn = self
            .reader_conn
            .lock()
            .map_err(|e| StorageError::Lock(e.to_string()))?;

        let mut list = Vec::new();
        if let Some(aid) = agent_id {
            let mut stmt = conn.prepare(
                "SELECT id, agent_id, role, content, metadata, created_at FROM chat_messages
                 WHERE agent_id = ?1 ORDER BY created_at ASC LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![aid, limit as i64], |row| {
                Ok(ChatMessageRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    metadata: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?;
            for r in rows {
                list.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, agent_id, role, content, metadata, created_at FROM chat_messages
                 ORDER BY created_at ASC LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit as i64], |row| {
                Ok(ChatMessageRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    metadata: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?;
            for r in rows {
                list.push(r?);
            }
        }

        Ok(list)
    }

    /// Retrieve checkpoints recorded for an agent
    pub fn get_checkpoints(&self, agent_id: &str) -> Result<Vec<CheckpointRecord>> {
        let conn = self
            .reader_conn
            .lock()
            .map_err(|e| StorageError::Lock(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, agent_id, step_index, state_json, created_at FROM agent_checkpoints
             WHERE agent_id = ?1 ORDER BY step_index ASC",
        )?;

        let rows = stmt.query_map(params![agent_id], |row| {
            Ok(CheckpointRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                step_index: row.get(2)?,
                state_json: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Trigger WAL checkpoint through the dedicated background writer actor
    pub async fn trigger_wal_checkpoint(
        &self,
        mode: &str,
    ) -> Result<qubit_protocol::WalCheckpointResultDto> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::WalCheckpoint {
                mode: mode.to_string(),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Set system configuration key-value pair through writer actor
    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let (responder, rx) = oneshot::channel();
        self.writer_tx
            .send(WriteCommand::SetConfig {
                key: key.to_string(),
                value: value.to_string(),
                responder,
            })
            .await
            .map_err(|e| StorageError::Channel(e.to_string()))?;

        rx.await.map_err(|_| StorageError::WriterTerminated)?
    }

    /// Retrieve configuration value by key
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self
            .reader_conn
            .lock()
            .map_err(|e| StorageError::Lock(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT value FROM system_config WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_writer_actor_and_concurrent_reads() {
        let storage = Storage::open_in_memory().expect("open in-memory storage");

        // Concurrent writes through the writer actor
        let mut handles = Vec::new();
        for i in 0..10 {
            let s = storage.clone();
            handles.push(tokio::spawn(async move {
                s.insert_message(&format!("msg-{}", i), "user", &format!("Content {}", i))
                    .await
                    .expect("insert message");
            }));
        }

        for h in handles {
            h.await.expect("join handle");
        }

        // Verify messages in reader connection
        let messages = storage.get_messages(None, 100).expect("get messages");
        assert_eq!(messages.len(), 10);
    }

    #[tokio::test]
    async fn test_agent_upsert_and_checkpoint() {
        let storage = Storage::open_in_memory().expect("open in-memory storage");
        let agent_id = uuid::Uuid::new_v4().to_string();

        storage
            .upsert_agent(&agent_id, "TestAgent", "Auditor", "idle")
            .await
            .expect("upsert agent");

        let agent = storage
            .get_agent(&agent_id)
            .expect("get agent")
            .expect("found");
        assert_eq!(agent.name, "TestAgent");
        assert_eq!(agent.status, "idle");

        storage
            .save_checkpoint(&agent_id, 1, r#"{"step": 1, "state": "ok"}"#)
            .await
            .expect("save checkpoint");

        let checkpoints = storage.get_checkpoints(&agent_id).expect("get checkpoints");
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].step_index, 1);
    }

    #[tokio::test]
    async fn test_wal_checkpoint_and_config() {
        let storage = Storage::open_in_memory().expect("open in-memory storage");

        // Test configuration persistence
        storage
            .set_config("routing.reflex_threshold_ms", "250")
            .await
            .expect("set config");

        let val = storage
            .get_config("routing.reflex_threshold_ms")
            .expect("get config");
        assert_eq!(val, Some("250".to_string()));

        // Test WAL checkpoint trigger via writer actor
        let cp_res = storage
            .trigger_wal_checkpoint("PASSIVE")
            .await
            .expect("checkpoint trigger");
        assert!(cp_res.success);
        assert_eq!(cp_res.checkpoint_mode, "PASSIVE");
    }
}

