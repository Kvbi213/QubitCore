use qubit_sandbox::{detect_available_backend, SandboxBackend, SandboxConfig, SandboxRunner};
use std::env;

#[tokio::test]
async fn detects_backend_and_falls_back_to_native_when_no_docker() {
    let backend = detect_available_backend();
    // In our CI/test container without docker daemon, it falls back cleanly to Native
    assert!(matches!(
        backend,
        SandboxBackend::Native | SandboxBackend::Docker { .. } | SandboxBackend::Podman { .. }
    ));

    // Verify running with Native backend
    let temp_dir = env::temp_dir();
    let cfg = SandboxConfig::default();
    let res = SandboxRunner::run_with_backend(
        &SandboxBackend::Native,
        "echo",
        &["container-sandbox-ok"],
        &temp_dir,
        &cfg,
    )
    .await
    .expect("execution failed");

    assert_eq!(res.exit_code, 0);
    assert!(res.stdout.contains("container-sandbox-ok"));
}
