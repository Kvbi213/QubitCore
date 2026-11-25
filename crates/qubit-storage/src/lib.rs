use chrono::{DateTime, Utc};
use qubit_protocol::{ChatMessage, MessageRole};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("storage actor channel closed")]
    ActorClosed,
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocRecord {
    pub id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

enum StorageCmd {
    CreateSession {
        title: String,
        reply: oneshot::Sender<Result<Uuid, StorageError>>,
    },
    AppendMessage {
        msg: ChatMessage,
        reply: oneshot::Sender<Result<(), StorageError>>,
    },
    ListMessages {
        session_id: Uuid,
        reply: oneshot::Sender<Result<Vec<ChatMessage>, StorageError>>,
    },
    UpsertDoc {
        doc: DocRecord,
        reply: oneshot::Sender<Result<(), StorageError>>,
    },
    SearchDocs {
        query: String,
        limit: usize,
        reply: oneshot::Sender<Result<Vec<DocRecord>, StorageError>>,
    },
    WalCheckpoint {
        reply: oneshot::Sender<Result<(), StorageError>>,
    },
}

#[derive(Clone)]
pub struct StorageHandle {
    tx: mpsc::Sender<StorageCmd>,
}

impl StorageHandle {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let conn = Connection::open(&path_buf)?;
        conn.execute_batch(include_str!("schema.sql"))?;

        let (tx, mut rx) = mpsc::channel::<StorageCmd>(256);
        std::thread::Builder::new()
            .name("qubit-sqlite-writer".to_string())
            .spawn(move || {
                while let Some(cmd) = rx.blocking_recv() {
                    match cmd {
                        StorageCmd::CreateSession { title, reply } => {
                            let _ = reply.send(handle_create_session(&conn, &title));
                        }
                        StorageCmd::AppendMessage { msg, reply } => {
                            let _ = reply.send(handle_append_message(&conn, &msg));
                        }
                        StorageCmd::ListMessages { session_id, reply } => {
                            let _ = reply.send(handle_list_messages(&conn, session_id));
                        }
                        StorageCmd::UpsertDoc { doc, reply } => {
                            let _ = reply.send(handle_upsert_doc(&conn, &doc));
                        }
                        StorageCmd::SearchDocs { query, limit, reply } => {
                            let _ = reply.send(handle_search_docs(&conn, &query, limit));
                        }
                        StorageCmd::WalCheckpoint { reply } => {
                            let res = conn
                                .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                                .map_err(StorageError::from);
                            let _ = reply.send(res);
                        }
                    }
                }
            })
            .expect("spawn sqlite writer thread");

        Ok(Self { tx })
    }

    pub async fn create_session(&self, title: &str) -> Result<Uuid, StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::CreateSession {
                title: title.to_string(),
                reply,
            })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }

    pub async fn append_message(&self, msg: ChatMessage) -> Result<(), StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::AppendMessage { msg, reply })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }

    pub async fn list_messages(&self, session_id: Uuid) -> Result<Vec<ChatMessage>, StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::ListMessages { session_id, reply })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }

    pub async fn upsert_doc(&self, doc: DocRecord) -> Result<(), StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::UpsertDoc { doc, reply })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }

    pub async fn search_docs(&self, query: &str, limit: usize) -> Result<Vec<DocRecord>, StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::SearchDocs {
                query: query.to_string(),
                limit,
                reply,
            })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }

    pub async fn wal_checkpoint(&self) -> Result<(), StorageError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(StorageCmd::WalCheckpoint { reply })
            .await
            .map_err(|_| StorageError::ActorClosed)?;
        rx.await.map_err(|_| StorageError::ActorClosed)?
    }
}

fn handle_create_session(conn: &Connection, title: &str) -> Result<Uuid, StorageError> {
    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sessions (id, title, created_at) VALUES (?1, ?2, ?3)",
        params![id.to_string(), title, now],
    )?;
    Ok(id)
}

fn role_to_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}

fn str_to_role(raw: &str) -> MessageRole {
    match raw {
        "system" => MessageRole::System,
        "assistant" => MessageRole::Assistant,
        "tool" => MessageRole::Tool,
        _ => MessageRole::User,
    }
}

fn handle_append_message(conn: &Connection, msg: &ChatMessage) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, tainted, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            msg.id.to_string(),
            msg.session_id.to_string(),
            role_to_str(msg.role),
            msg.content,
            if msg.tainted { 1i64 } else { 0i64 },
            msg.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn handle_list_messages(conn: &Connection, session_id: Uuid) -> Result<Vec<ChatMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, tainted, created_at
         FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![session_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let sid_str: String = row.get(1)?;
        let role_str: String = row.get(2)?;
        let content: String = row.get(3)?;
        let tainted_int: i64 = row.get(4)?;
        let ts_str: String = row.get(5)?;
        Ok((id_str, sid_str, role_str, content, tainted_int, ts_str))
    })?;

    let mut out = Vec::new();
    for r in rows {
        let (id_str, sid_str, role_str, content, tainted_int, ts_str) = r?;
        let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
        let sid = Uuid::parse_str(&sid_str).unwrap_or(session_id);
        let created_at = DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        out.push(ChatMessage {
            id,
            session_id: sid,
            role: str_to_role(&role_str),
            content,
            tainted: tainted_int != 0,
            created_at,
        });
    }
    Ok(out)
}

fn handle_upsert_doc(conn: &Connection, doc: &DocRecord) -> Result<(), StorageError> {
    let tags_json = serde_json::to_string(&doc.tags)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO docs (id, title, body, tags_json, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            body = excluded.body,
            tags_json = excluded.tags_json,
            updated_at = excluded.updated_at",
        params![doc.id, doc.title, doc.body, tags_json, now],
    )?;
    conn.execute("DELETE FROM docs_fts WHERE id = ?1", params![doc.id])?;
    conn.execute(
        "INSERT INTO docs_fts (id, title, body) VALUES (?1, ?2, ?3)",
        params![doc.id, doc.title, doc.body],
    )?;
    Ok(())
}

fn sanitize_fts5_query(raw: &str) -> String {
    let tokens: Vec<String> = raw
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect();
    tokens.join(" ")
}

fn handle_search_docs(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<DocRecord>, StorageError> {
    let safe_query = sanitize_fts5_query(query);
    if safe_query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT d.id, d.title, d.body, d.tags_json
         FROM docs_fts f
         JOIN docs d ON d.id = f.id
         WHERE docs_fts MATCH ?1
         ORDER BY bm25(docs_fts)
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![safe_query, limit as i64], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let body: String = row.get(2)?;
        let tags_json: String = row.get(3)?;
        Ok((id, title, body, tags_json))
    })?;

    let mut out = Vec::new();
    for r in rows {
        let (id, title, body, tags_json) = r?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        out.push(DocRecord {
            id,
            title,
            body,
            tags,
        });
    }
    Ok(out)
}
