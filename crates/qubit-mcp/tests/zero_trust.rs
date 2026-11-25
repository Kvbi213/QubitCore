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
    std::os::unix::fs::symlink(
        outside.path().join("secret.txt"),
        root.path().join("link.txt"),
    )
    .unwrap();

    let guard = RootGuard::new(vec![root.path().to_path_buf()]).unwrap();
    assert!(guard.resolve_safe(&root.path().join("ok.txt")).is_ok());
    assert!(matches!(
        guard.resolve_safe(Path::new("../../etc/passwd")),
        Err(McpError::OutsideRoot)
    ));

    #[cfg(unix)]
    assert!(matches!(
        guard.resolve_safe(&root.path().join("link.txt")),
        Err(McpError::OutsideRoot)
    ));
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
    let id = gate
        .submit(Uuid::new_v4(), "run_terminal", "rm -rf /tmp/x")
        .await;
    assert_eq!(gate.status(id).await, Some(ElicitationStatus::Pending));
    gate.resolve(id, ElicitationStatus::Approved).await.unwrap();
    assert_eq!(gate.status(id).await, Some(ElicitationStatus::Approved));
}
