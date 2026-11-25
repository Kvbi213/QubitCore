use qubit_protocol::{ChatMessage, ExecuteTaskRequest, ExecutionMode, MessageRole};
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
