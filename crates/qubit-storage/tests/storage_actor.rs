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
