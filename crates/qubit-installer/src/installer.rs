//! Asset Download, Archive Extraction & Service Launcher Installation
//! Downloads release bundles and sets up local directory structure and launcher scripts.

use crate::wizard::InstallConfig;
use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;

/// Embedded icon bytes for Windows launcher
pub const EMBEDDED_ICON: &[u8] = include_bytes!("../resources/icon.ico");

/// Download release asset from URL with visual progress bar and extract into destination
pub async fn download_and_extract_asset(url: &str, dest_dir: &Path) -> Result<()> {
    println!("{} {}", "[*] Pobieranie pakietu z:".bold().cyan(), url);

    let client = reqwest::Client::builder()
        .user_agent(format!("qubit-setup/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let response = client.get(url).send().await.context("Błąd połączenia HTTP")?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut bytes = Vec::new();
    let mut stream = response;

    while let Some(chunk) = stream.chunk().await? {
        bytes.extend_from_slice(&chunk);
        pb.set_position(bytes.len() as u64);
    }
    pb.finish_with_message("Pobieranie ukończone");

    println!("{}", "[+] Rozpakowywanie archiwum...".bold().green());
    extract_archive_bytes(&bytes, url, dest_dir)?;

    Ok(())
}

/// Unpack in-memory archive bytes into target directory (.zip or .tar.gz)
pub fn extract_archive_bytes(bytes: &[u8], filename_hint: &str, dest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dest_dir).context("Nie można utworzyć folderu instalacyjnego")?;

    if filename_hint.ends_with(".zip") {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).context("Błąd odczytu archiwum ZIP")?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue,
            };

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
    } else if filename_hint.ends_with(".tar.gz") || filename_hint.ends_with(".tgz") {
        let gz = flate2::read::GzDecoder::new(bytes);
        let mut archive = tar::Archive::new(gz);
        archive.unpack(dest_dir).context("Błąd rozpakowywania TAR.GZ")?;
    } else {
        // Fallback: save as raw binary file in bin/
        let bin_dir = dest_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        let file_path = bin_dir.join("qubit-bundle.bin");
        std::fs::write(&file_path, bytes)?;
    }

    Ok(())
}

/// Setup full local directory structure, copy resources, and write launcher scripts
pub fn initialize_install_directory(config: &InstallConfig) -> Result<()> {
    let install_dir = &config.install_dir;
    let bin_dir = install_dir.join("bin");
    let data_dir = install_dir.join("data");
    let config_dir = install_dir.join("config");
    let resources_dir = install_dir.join("resources");

    std::fs::create_dir_all(&bin_dir)?;
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(&resources_dir)?;

    // Embed icon into installation resources
    let icon_file = resources_dir.join("icon.ico");
    std::fs::write(&icon_file, EMBEDDED_ICON)?;

    // Generate Windows Batch launcher
    let launcher_bat = install_dir.join("qubit-launcher.bat");
    let bat_content = format!(
        "@echo off\r\n\
         title QubitCore & QubitDash Launcher\r\n\
         cd /d \"%~dp0\"\r\n\
         set QUBIT_DAEMON_PORT={port}\r\n\
         set QUBIT_DB_PATH={db}\r\n\
         set QUBIT_INSTALL_DIR=%~dp0\r\n\
         echo [*] Uruchamianie daemona QubitCore na porcie {port}...\r\n\
         start \"QubitCore Daemon\" /b \"bin\\qubit-core.exe\" --bind 127.0.0.1:{port} --db \"%~dp0\\data\\{db}\"\r\n\
         timeout /t 1 /nobreak >nul\r\n\
         echo [*] Uruchamianie interfejsu QubitDash Plasma Edition...\r\n\
         start \"\" \"bin\\qubit-dash.exe\"\r\n",
        port = config.daemon_port,
        db = config.db_path,
    );
    std::fs::write(&launcher_bat, bat_content)?;

    // Generate Unix Shell launcher
    let launcher_sh = install_dir.join("qubit-launcher.sh");
    let sh_content = format!(
        "#!/usr/bin/env bash\n\
         set -e\n\
         DIR=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")\" && pwd)\"\n\
         export QUBIT_DAEMON_PORT={port}\n\
         export QUBIT_DB_PATH=\"$DIR/data/{db}\"\n\
         export QUBIT_INSTALL_DIR=\"$DIR\"\n\
         echo \"[*] Startowanie daemona QubitCore na porcie {port}...\"\n\
         if [ -f \"$DIR/bin/qubit-core\" ]; then\n\
             \"$DIR/bin/qubit-core\" --bind 127.0.0.1:{port} --db \"$DIR/data/{db}\" &\n\
         fi\n\
         sleep 1\n\
         echo \"[*] Startowanie QubitDash...\"\n\
         if [ -f \"$DIR/bin/qubit-dash\" ]; then\n\
             exec \"$DIR/bin/qubit-dash\"\n\
         fi\n",
        port = config.daemon_port,
        db = config.db_path,
    );
    std::fs::write(&launcher_sh, sh_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&launcher_sh, std::fs::Permissions::from_mode(0o755));
    }

    // Write README.txt
    let readme_path = install_dir.join("README.txt");
    let readme_content = format!(
        "=====================================================\n\
         QubitCore Distributed Agent OS & QubitDash\n\
         Zainstalowano pomyślnie w: {path}\n\
         Wersja: v{ver}\n\
         =====================================================\n\n\
         Uruchomienie:\n\
           Windows: Uruchom qubit-launcher.bat lub kliknij skrót QubitDash na Pulpicie.\n\
           Linux  : ./qubit-launcher.sh\n\n\
         Struktura katalogów:\n\
           bin/       - Pliki wykonywalne (qubit-core, qubit-dash, qubit CLI)\n\
           data/      - Trwała baza danych SQLite WAL (qubit_data.db)\n\
           config/    - Pliki konfiguracji i ACL narzędzi\n\
           resources/ - Ikony i zasoby graficzne\n\
           qubit.env  - Zmienne środowiskowe środowiska wykonawczego\n",
        path = install_dir.display(),
        ver = env!("CARGO_PKG_VERSION"),
    );
    std::fs::write(&readme_path, readme_content)?;

    Ok(())
}

/// Verify that the daemon port is available and directory permissions are correct
pub async fn verify_environment(config: &InstallConfig) -> Result<()> {
    // Check write access
    let test_file = config.install_dir.join(".write_test");
    std::fs::write(&test_file, b"ok")
        .context(format!("Brak uprawnień do zapisu w folderze {}", config.install_dir.display()))?;
    let _ = std::fs::remove_file(test_file);

    // Check port binding
    let bind_addr = format!("127.0.0.1:{}", config.daemon_port);
    match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => {
            drop(listener);
        }
        Err(e) => {
            println!(
                "{}",
                format!("[!] Ostrzeżenie: Port {} może być już zajęty ({})", config.daemon_port, e).yellow()
            );
        }
    }

    Ok(())
}
