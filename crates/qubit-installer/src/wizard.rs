//! Initial Setup Wizard & Configuration Generator
//! Prompts for target directories, ports, and creates environment configuration.

use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub install_dir: PathBuf,
    pub daemon_port: u16,
    pub db_path: String,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            daemon_port: 4000,
            db_path: "qubit.db".to_string(),
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
        }
    }
}

/// Resolves standard platform install directory:
/// Windows: %LOCALAPPDATA%\Qubit
/// Linux / macOS: ~/.qubit
pub fn default_install_dir() -> PathBuf {
    if cfg!(windows) {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_appdata).join("Qubit");
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            return PathBuf::from(userprofile).join("AppData").join("Local").join("Qubit");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".qubit");
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("qubit-dist")
}

/// Run terminal-based interactive setup wizard
pub fn run_interactive_wizard(mut config: InstallConfig) -> Result<InstallConfig> {
    println!("\n{}", "=== KREATOR KONFIGURACJI WSTĘPNEJ QUBIT ===".bold().cyan());
    println!("{}", "Naciśnij [Enter], aby zaakceptować domyślną wartość w nawiasie [ ].\n".dimmed());

    // 1. Installation path
    let default_dir_str = config.install_dir.to_string_lossy().to_string();
    print!("{} [{}]: ", "1. Ścieżka instalacji".bold(), default_dir_str.yellow());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if !trimmed.is_empty() {
        config.install_dir = PathBuf::from(trimmed);
    }

    // 2. Daemon port
    print!("{} [{}]: ", "2. Port daemona QubitCore".bold(), config.daemon_port.to_string().yellow());
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if !trimmed.is_empty() {
        if let Ok(port) = trimmed.parse::<u16>() {
            config.daemon_port = port;
        } else {
            println!("{}", "Niepoprawny port, zastosowano domyślny 4000.".red());
        }
    }

    // 3. SQLite Database path
    print!("{} [{}]: ", "3. Ścieżka bazy SQLite WAL".bold(), config.db_path.yellow());
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if !trimmed.is_empty() {
        config.db_path = trimmed.to_string();
    }

    // 4. Desktop shortcut
    print!("{} [T/n]: ", "4. Utworzyć skrót na Pulpicie?".bold());
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    config.create_desktop_shortcut = !trimmed.starts_with('n');

    // 5. Start Menu shortcut
    print!("{} [T/n]: ", "5. Dodać skrót do Menu Start?".bold());
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    config.create_start_menu_shortcut = !trimmed.starts_with('n');

    println!("\n{}", "Podsumowanie konfiguracji:".bold().green());
    println!("  Ścieżka instalacji : {}", config.install_dir.display().to_string().cyan());
    println!("  Port daemona       : {}", config.daemon_port.to_string().cyan());
    println!("  Baza SQLite        : {}", config.db_path.cyan());
    println!("  Skrót na Pulpicie  : {}", if config.create_desktop_shortcut { "Tak".green() } else { "Nie".dimmed() });
    println!("  Skrót w Menu Start : {}", if config.create_start_menu_shortcut { "Tak".green() } else { "Nie".dimmed() });
    println!();

    Ok(config)
}

/// Generates `qubit.env` environment file in target directory
pub fn create_env_file(config: &InstallConfig, target_dir: &Path) -> Result<PathBuf> {
    let env_file_path = target_dir.join("qubit.env");
    let content = format!(
        "# QubitCore & QubitDash Runtime Environment\n\
         # Wygenerowano przez qubit-setup v{version}\n\n\
         QUBIT_DAEMON_PORT={port}\n\
         QUBIT_DB_PATH={db}\n\
         QUBIT_INSTALL_DIR={install_dir}\n\
         QUBIT_EPHEMERAL_MODE=true\n\
         RUST_LOG=info,qubit_core=info,qubit_dash=info\n",
        version = env!("CARGO_PKG_VERSION"),
        port = config.daemon_port,
        db = config.db_path,
        install_dir = config.install_dir.display(),
    );

    std::fs::write(&env_file_path, content)
        .context(format!("Nie udało się zapisać pliku {}", env_file_path.display()))?;

    Ok(env_file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = InstallConfig::default();
        assert_eq!(cfg.daemon_port, 4000);
        assert_eq!(cfg.db_path, "qubit.db");
        assert!(cfg.create_desktop_shortcut);
        assert!(cfg.create_start_menu_shortcut);
    }

    #[test]
    fn test_create_env_file() {
        let temp_dir = std::env::temp_dir().join(format!("qubit_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let cfg = InstallConfig {
            install_dir: temp_dir.clone(),
            daemon_port: 4500,
            db_path: "test.db".into(),
            create_desktop_shortcut: false,
            create_start_menu_shortcut: false,
        };

        let env_path = create_env_file(&cfg, &temp_dir).expect("write env file");
        assert!(env_path.exists());

        let contents = std::fs::read_to_string(&env_path).expect("read env file");
        assert!(contents.contains("QUBIT_DAEMON_PORT=4500"));
        assert!(contents.contains("QUBIT_DB_PATH=test.db"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
