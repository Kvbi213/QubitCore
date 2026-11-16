//! QubitCore & QubitDash Autonomous Setup & Bootstrapper (qubit-setup)
//! Lightweight Rust installer with GitHub update verification, initial setup wizard,
//! configuration generation, and desktop shortcut provisioning.

mod github;
mod installer;
mod shortcut;
mod wizard;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use wizard::InstallConfig;

#[derive(Parser, Debug)]
#[command(
    name = "qubit-setup",
    author = "Kvbi213 <lisjakub2106@gmail.com>",
    version,
    about = "QubitCore & QubitDash Autonomous Installer & Bootstrapper"
)]
struct Args {
    /// GitHub repository to query for releases
    #[arg(short, long, default_value = github::DEFAULT_REPO)]
    repo: String,

    /// Unattended mode (uses defaults without prompting)
    #[arg(short, long, default_value_t = false)]
    unattended: bool,

    /// Check for updates on GitHub and exit
    #[arg(long, default_value_t = false)]
    check_update_only: bool,

    /// Installation target directory
    #[arg(short = 'd', long)]
    install_dir: Option<PathBuf>,

    /// QubitCore background daemon port
    #[arg(short, long)]
    port: Option<u16>,

    /// SQLite database filename or relative path
    #[arg(long)]
    db_path: Option<String>,

    /// Disable creation of Desktop and Start Menu shortcuts
    #[arg(long, default_value_t = false)]
    no_shortcuts: bool,
}

fn print_banner() {
    let current_ver = env!("CARGO_PKG_VERSION");
    println!("{}", "======================================================================".cyan());
    println!("{}", "      ⚡ QUBIT CORE & QUBIT DASH — AUTONOMOUS SETUP ENGINE ⚡         ".bold().cyan());
    println!("{}", format!("                  Wersja instalatora: v{current_ver}").dimmed());
    println!("{}", "======================================================================".cyan());
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    print_banner();
    let args = Args::parse();
    let current_version = env!("CARGO_PKG_VERSION");

    // 1. Sprawdzanie aktualizacji na GitHubie
    println!("{} Sprawdzanie wydań w repozytorium GitHub: {}...", "[1/5]".bold().blue(), args.repo.cyan());
    let release_info = match github::fetch_latest_release(&args.repo).await {
        Ok(rel) => {
            let is_newer = github::is_newer_version(current_version, &rel.tag_name);
            if is_newer {
                println!(
                    "{} Wykryto nowszą wersję QubitCore: {} (obecna: v{})",
                    "[!]".bold().yellow(),
                    rel.tag_name.bold().green(),
                    current_version
                );
                if let Some(notes) = &rel.body {
                    println!("\n{}:\n{}\n", "Notatki do wydania".underline(), notes.trim());
                }
            } else {
                println!(
                    "{} Posiadasz najnowszą wersję: v{} (GitHub: {})",
                    "[✓]".bold().green(),
                    current_version,
                    rel.tag_name.dimmed()
                );
            }
            Some(rel)
        }
        Err(err) => {
            println!(
                "{} Nie udało się pobrać informacji z GitHuba: {}. Kontynuowanie w trybie lokalnym.",
                "[!]".bold().yellow(),
                err
            );
            None
        }
    };

    if args.check_update_only {
        println!("{}", "[+] Sprawdzanie aktualizacji zakończone.".green());
        return Ok(());
    }

    // 2. Kreator wstępnej konfiguracji
    println!("\n{} Konfiguracja parametrów instalacji...", "[2/5]".bold().blue());
    let mut config = InstallConfig::default();

    if let Some(dir) = args.install_dir {
        config.install_dir = dir;
    }
    if let Some(port) = args.port {
        config.daemon_port = port;
    }
    if let Some(db) = args.db_path {
        config.db_path = db;
    }
    if args.no_shortcuts {
        config.create_desktop_shortcut = false;
        config.create_start_menu_shortcut = false;
    }

    if !args.unattended {
        config = wizard::run_interactive_wizard(config)?;
    } else {
        println!("{}", "[*] Uruchomiono w trybie nienadzorowanym (unattended).".dimmed());
        println!("    Katalog instalacji: {}", config.install_dir.display());
        println!("    Port daemona      : {}", config.daemon_port);
    }

    // 3. Przygotowanie katalogu i pobieranie pakietu
    println!("\n{} Przygotowywanie struktury katalogów i pobieranie plików...", "[3/5]".bold().blue());
    installer::initialize_install_directory(&config)?;

    // Download matching binary bundle if release asset is available
    if let Some(rel) = &release_info {
        let target_os = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        if let Some(asset) = github::find_matching_asset(rel, target_os) {
            let _ = installer::download_and_extract_asset(&asset.browser_download_url, &config.install_dir).await;
        } else {
            println!(
                "{}",
                format!("[*] Brak gotowego archiwum dla platformy '{}' w wydaniu {}. Zainicjalizowano strukturę bazową.", target_os, rel.tag_name).dimmed()
            );
        }
    }

    // 4. Generowanie konfiguracji środowiskowej i skrótów
    println!("\n{} Zapisywanie konfiguracji qubit.env i tworzenie skrótów...", "[4/5]".bold().blue());
    let env_path = wizard::create_env_file(&config, &config.install_dir)?;
    println!("  {} Zapisano {}", "[✓]".green(), env_path.display());

    shortcut::create_shortcuts(&config)?;
    if config.create_desktop_shortcut || config.create_start_menu_shortcut {
        println!("  {} Utworzono skróty systemowe", "[✓]".green());
    }

    // 5. Weryfikacja środowiska
    println!("\n{} Weryfikacja środowiska wykonawczego...", "[5/5]".bold().blue());
    installer::verify_environment(&config).await?;

    // Final summary
    println!("\n{}", "======================================================================".green());
    println!("{}", "       🎉 INSTALACJA QUBITCORE & QUBITDASH ZAKOŃCZONA SUKCESEM!        ".bold().green());
    println!("{}", "======================================================================".green());
    println!("  Lokalizacja : {}", config.install_dir.display().to_string().cyan());
    println!("  Port daemona: {}", config.daemon_port.to_string().cyan());
    println!("  Baza SQLite : {}/data/{}", config.install_dir.display(), config.db_path.cyan());
    println!();
    println!("Aby uruchomić system:");
    if cfg!(windows) {
        println!("  Uruchom: {}", format!("{}\\qubit-launcher.bat", config.install_dir.display()).yellow());
        println!("  Lub kliknij dwukrotnie skrót {} na Pulpicie.", "QubitDash".bold());
    } else {
        println!("  Uruchom: {}", format!("{}/qubit-launcher.sh", config.install_dir.display()).yellow());
    }
    println!();

    Ok(())
}
