//! GitHub Release & Update Checker for QubitCore
//! Queries GitHub API for the latest version and compares against current build.

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

pub const DEFAULT_REPO: &str = "Kvbi213/QubitCore";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

/// Query GitHub API for the latest published release
pub async fn fetch_latest_release(repo: &str) -> Result<ReleaseInfo> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::Client::builder()
        .user_agent(format!("qubit-setup/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context(format!("Failed to connect to GitHub API at {url}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API returned HTTP {}: {}", status, body);
    }

    let release = response
        .json::<ReleaseInfo>()
        .await
        .context("Failed to parse GitHub Release JSON payload")?;

    Ok(release)
}

/// Normalize tag string into semver-compliant string (e.g. "v0.1.0" -> "0.1.0")
pub fn parse_clean_version(raw_version: &str) -> Option<Version> {
    let cleaned = raw_version.trim().trim_start_matches('v');
    Version::parse(cleaned).ok()
}

/// Compare current version with latest release tag
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_clean_version(current), parse_clean_version(latest)) {
        (Some(cur), Some(lat)) => lat > cur,
        _ => false,
    }
}

/// Find matching platform archive asset in the release
pub fn find_matching_asset<'a>(release: &'a ReleaseInfo, target_os: &str) -> Option<&'a ReleaseAsset> {
    let search_terms: &[&str] = match target_os {
        "windows" => &["windows", "win64", "x86_64-pc-windows", ".zip", "setup"],
        "linux" => &["linux", "x86_64-unknown-linux", ".tar.gz"],
        "macos" => &["darwin", "macos", "apple", ".tar.gz", ".zip"],
        _ => &["bundle", ".zip", ".tar.gz"],
    };

    // First try: matches OS name and bundle archive
    for asset in &release.assets {
        let lower = asset.name.to_lowercase();
        if target_os == "windows" && (lower.contains("windows") || lower.contains("win64")) && lower.ends_with(".zip") {
            return Some(asset);
        }
        if target_os == "linux" && lower.contains("linux") && (lower.ends_with(".tar.gz") || lower.ends_with(".zip")) {
            return Some(asset);
        }
    }

    // Fallback: look for any zip or tar.gz archive
    for asset in &release.assets {
        let lower = asset.name.to_lowercase();
        for term in search_terms {
            if lower.contains(term) && (lower.ends_with(".zip") || lower.ends_with(".tar.gz")) {
                return Some(asset);
            }
        }
    }

    // Final fallback: first archive asset
    release.assets.iter().find(|a| {
        a.name.ends_with(".zip") || a.name.ends_with(".tar.gz")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_comparison() {
        assert!(is_newer_version("0.1.0", "v0.2.0"));
        assert!(is_newer_version("v0.1.0", "0.1.1"));
        assert!(!is_newer_version("0.2.0", "v0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_asset_matching() {
        let release = ReleaseInfo {
            tag_name: "v0.2.0".into(),
            name: Some("Release 0.2.0".into()),
            body: Some("Release notes".into()),
            published_at: None,
            html_url: "https://github.com/Kvbi213/QubitCore/releases/v0.2.0".into(),
            assets: vec![
                ReleaseAsset {
                    name: "qubit-v0.2.0-x86_64-unknown-linux-gnu.tar.gz".into(),
                    size: 15485000,
                    browser_download_url: "https://example.com/linux.tar.gz".into(),
                    content_type: None,
                },
                ReleaseAsset {
                    name: "qubit-v0.2.0-x86_64-pc-windows-msvc.zip".into(),
                    size: 18230000,
                    browser_download_url: "https://example.com/win.zip".into(),
                    content_type: None,
                },
            ],
        };

        let win_asset = find_matching_asset(&release, "windows");
        assert!(win_asset.is_some());
        assert_eq!(win_asset.unwrap().name, "qubit-v0.2.0-x86_64-pc-windows-msvc.zip");

        let linux_asset = find_matching_asset(&release, "linux");
        assert!(linux_asset.is_some());
        assert_eq!(linux_asset.unwrap().name, "qubit-v0.2.0-x86_64-unknown-linux-gnu.tar.gz");
    }
}
