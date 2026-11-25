use qubit_mcp::{McpError, RootGuard, SecurityContext, StandardTools};
use qubit_sandbox::SandboxConfig;
use tempfile::tempdir;

#[tokio::test]
async fn test_read_write_edit_and_list_tools() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let guard = RootGuard::new(vec![root.clone()]).unwrap();
    let clean_sec = SecurityContext { tainted: false };
    let tainted_sec = SecurityContext { tainted: true };

    // 1. write_file with clean context
    let file_path = root.join("test_note.txt");
    let written = StandardTools::write_file(&guard, &clean_sec, &file_path, "Hello Qubit MCP", false)
        .expect("write failed");
    assert!(written.exists());

    // 2. write_file blocked in tainted context
    let err = StandardTools::write_file(&guard, &tainted_sec, &file_path, "Blocked", true)
        .expect_err("should block mutating tool");
    assert!(matches!(err, McpError::TaintedBlocked));

    // 3. read_file
    let content = StandardTools::read_file(&guard, &file_path, None, None).expect("read failed");
    assert_eq!(content, "Hello Qubit MCP");

    // 4. edit_file
    StandardTools::edit_file(&guard, &clean_sec, &file_path, "MCP", "Ecosystem")
        .expect("edit failed");
    let updated = StandardTools::read_file(&guard, &file_path, None, None).expect("read failed");
    assert_eq!(updated, "Hello Qubit Ecosystem");

    // 5. list_dir
    let entries = StandardTools::list_dir(&guard, &root, None).expect("list failed");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "test_note.txt");
    assert!(!entries[0].is_dir);

    // 6. run_command through sandbox
    let cfg = SandboxConfig::default();
    let out = StandardTools::run_command(
        &guard,
        &clean_sec,
        "echo",
        &["mcp-cmd-ok"],
        &root,
        None,
        &cfg,
    )
    .await
    .expect("command execution failed");
    assert_eq!(out.exit_code, 0);
    assert!(out.stdout.contains("mcp-cmd-ok"));
}
