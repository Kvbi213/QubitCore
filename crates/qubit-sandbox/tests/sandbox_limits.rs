use qubit_sandbox::{classify_memory_pressure, BackpressureLevel, SandboxConfig, SandboxRunner};
use tempfile::tempdir;

#[tokio::test]
async fn kills_timed_out_process_and_truncates_large_output() {
    let dir = tempdir().unwrap();
    let cfg = SandboxConfig {
        timeout_ms: 150,
        max_output_bytes: 256,
    };
    let out = SandboxRunner::run("sh", &["-c", "sleep 2"], dir.path(), &cfg)
        .await
        .unwrap();
    assert!(out.timed_out);

    let flood_cfg = SandboxConfig {
        timeout_ms: 2000,
        max_output_bytes: 128,
    };
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
