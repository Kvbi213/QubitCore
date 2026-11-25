use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QubitConfig {
    pub port: u16,
    pub host: String,
    pub ram_limit_mb: usize,
    pub storage_path: Option<String>,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub zero_trust_mcp: bool,
}

impl Default for QubitConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            host: "127.0.0.1".to_string(),
            ram_limit_mb: 60,
            storage_path: None,
            api_url: None,
            api_key: None,
            model: None,
            zero_trust_mcp: true,
        }
    }
}

impl QubitConfig {
    pub fn load() -> Self {
        let mut cfg = Self::default();

        // 1. Try reading config file from common paths
        let candidate_paths = vec![
            PathBuf::from("qubit_config.json"),
            PathBuf::from("config/qubit_config.json"),
            dirs_fallback().join("qubit_config.json"),
        ];

        for p in candidate_paths {
            if p.is_file() {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(parsed) = serde_json::from_str::<QubitConfig>(&content) {
                        cfg = parsed;
                        break;
                    }
                }
            }
        }

        // 2. Override with explicit environment variables
        if let Ok(p_str) = env::var("QUBIT_PORT") {
            if let Ok(p) = p_str.parse::<u16>() {
                cfg.port = p;
            }
        }
        if let Ok(h) = env::var("QUBIT_HOST") {
            cfg.host = h;
        }
        if let Ok(s) = env::var("QUBIT_STORAGE_PATH") {
            cfg.storage_path = Some(s);
        }
        if let Ok(k) = env::var("QUBIT_API_KEY") {
            cfg.api_key = Some(k);
        }
        if let Ok(u) = env::var("QUBIT_API_URL") {
            cfg.api_url = Some(u);
        }
        if let Ok(m) = env::var("QUBIT_MODEL") {
            cfg.model = Some(m);
        }

        cfg
    }
}

fn dirs_fallback() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local) = env::var("LOCALAPPDATA") {
            return Path::new(&local).join("Qubit");
        }
    }
    if let Ok(home) = env::var("HOME") {
        return Path::new(&home).join(".qubit");
    }
    PathBuf::from(".qubit")
}
