//! Desktop and Start Menu Shortcut Creator for Qubit
//! Supports Windows (.lnk via PowerShell WScript.Shell and .bat) and Linux (.desktop).

use crate::wizard::InstallConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Creates desktop and start menu shortcuts according to user configuration
pub fn create_shortcuts(config: &InstallConfig) -> Result<()> {
    if !config.create_desktop_shortcut && !config.create_start_menu_shortcut {
        return Ok(());
    }

    if cfg!(windows) {
        create_windows_shortcuts(config)?;
    } else {
        create_linux_shortcuts(config)?;
    }

    Ok(())
}

/// Windows shortcut creation logic
fn create_windows_shortcuts(config: &InstallConfig) -> Result<()> {
    let launcher_bat = config.install_dir.join("qubit-launcher.bat");
    let icon_path = config.install_dir.join("resources").join("icon.ico");

    let desktop_dir = std::env::var("USERPROFILE")
        .map(|p| PathBuf::from(p).join("Desktop"))
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Public\\Desktop"));

    let start_menu_dir = std::env::var("APPDATA")
        .map(|p| PathBuf::from(p).join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("Qubit"))
        .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Qubit"));

    // 1. Desktop shortcut
    if config.create_desktop_shortcut && desktop_dir.exists() {
        let lnk_target = desktop_dir.join("QubitDash.lnk");
        create_windows_lnk(
            &lnk_target,
            &launcher_bat,
            &config.install_dir,
            &icon_path,
            "QubitDash Plasma Edition",
        )?;
    }

    // 2. Start Menu shortcut
    if config.create_start_menu_shortcut {
        let _ = std::fs::create_dir_all(&start_menu_dir);
        let lnk_target = start_menu_dir.join("QubitDash.lnk");
        create_windows_lnk(
            &lnk_target,
            &launcher_bat,
            &config.install_dir,
            &icon_path,
            "QubitDash Plasma Edition",
        )?;
    }

    Ok(())
}

/// Creates a Windows .lnk shortcut via PowerShell COM WScript.Shell
fn create_windows_lnk(
    lnk_path: &Path,
    target_path: &Path,
    working_dir: &Path,
    icon_path: &Path,
    description: &str,
) -> Result<()> {
    let ps_command = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{lnk}'); \
         $s.TargetPath = '{target}'; \
         $s.WorkingDirectory = '{workdir}'; \
         $s.Description = '{desc}'; \
         if (Test-Path '{icon}') {{ $s.IconLocation = '{icon}' }}; \
         $s.Save()",
        lnk = lnk_path.display(),
        target = target_path.display(),
        workdir = working_dir.display(),
        icon = icon_path.display(),
        desc = description,
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_command])
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // Fallback: create a .bat direct launcher on desktop if PowerShell fails
            let bat_fallback = lnk_path.with_extension("bat");
            let bat_content = format!(
                "@echo off\r\ncd /d \"{}\"\r\ncall \"{}\"\r\n",
                working_dir.display(),
                target_path.display()
            );
            std::fs::write(&bat_fallback, bat_content)
                .context("Failed to write fallback shortcut .bat")?;
            Ok(())
        }
    }
}

/// Linux desktop entries creation logic
fn create_linux_shortcuts(config: &InstallConfig) -> Result<()> {
    let launcher_sh = config.install_dir.join("qubit-launcher.sh");
    let icon_path = config.install_dir.join("resources").join("icon.ico");

    let desktop_entry_content = format!(
        "[Desktop Entry]\n\
         Name=QubitDash\n\
         Comment=QubitCore & QubitDash Autonomous Agent OS\n\
         Exec=\"{exec}\"\n\
         Path={workdir}\n\
         Icon={icon}\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Development;System;\n",
        exec = launcher_sh.display(),
        workdir = config.install_dir.display(),
        icon = icon_path.display(),
    );

    // Desktop
    if config.create_desktop_shortcut {
        if let Ok(home) = std::env::var("HOME") {
            let desktop_dir = PathBuf::from(home).join("Desktop");
            if desktop_dir.exists() {
                let shortcut_file = desktop_dir.join("qubit.desktop");
                let _ = std::fs::write(&shortcut_file, &desktop_entry_content);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&shortcut_file, std::fs::Permissions::from_mode(0o755));
                }
            }
        }
    }

    // Applications Menu (~/.local/share/applications)
    if config.create_start_menu_shortcut {
        if let Ok(home) = std::env::var("HOME") {
            let app_dir = PathBuf::from(home).join(".local").join("share").join("applications");
            let _ = std::fs::create_dir_all(&app_dir);
            let shortcut_file = app_dir.join("qubit.desktop");
            let _ = std::fs::write(&shortcut_file, &desktop_entry_content);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&shortcut_file, std::fs::Permissions::from_mode(0o755));
            }
        }
    }

    Ok(())
}
