use qubit_cli::{run_cli, CliCommand, CliError};

#[tokio::test]
async fn returns_clear_error_when_daemon_unreachable() {
    let err = run_cli(CliCommand::Status, "http://127.0.0.1:49999")
        .await
        .unwrap_err();
    assert!(matches!(err, CliError::DaemonUnavailable(_)));
}
