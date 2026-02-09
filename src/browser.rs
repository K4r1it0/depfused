//! Browser management: auto-download Chromium, resolve executable paths, host grouping.

use crate::types::{DepfusedError, Result};
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use url::Url;

/// Returns the managed Chrome installation directory: `~/.depfused/chrome/`
pub fn managed_chrome_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        DepfusedError::ConfigError("Could not determine home directory".to_string())
    })?;
    Ok(home.join(".depfused").join("chrome"))
}

/// Checks the managed directory for a previously-downloaded Chrome executable.
/// Returns `Some(path)` if found, `None` otherwise.
pub fn resolve_chrome_executable() -> Option<PathBuf> {
    let chrome_dir = managed_chrome_dir().ok()?;
    if !chrome_dir.exists() {
        return None;
    }

    // The fetcher places the executable inside a platform-specific subdirectory.
    // Walk the directory to find the first chrome/chromium executable.
    find_chrome_in_dir(&chrome_dir)
}

/// Download Chromium to the managed directory using `BrowserFetcher`.
/// Returns the path to the downloaded executable.
pub async fn download_chrome(force: bool) -> Result<PathBuf> {
    let chrome_dir = managed_chrome_dir()?;

    // If already downloaded and not forcing, return existing path
    if !force {
        if let Some(exe) = find_chrome_in_dir(&chrome_dir) {
            info!("Chrome already installed at {:?}", exe);
            return Ok(exe);
        }
    }

    // Clean directory if forcing re-download
    if force && chrome_dir.exists() {
        info!("Removing existing Chrome installation for re-download...");
        std::fs::remove_dir_all(&chrome_dir).map_err(|e| {
            DepfusedError::IoError(e)
        })?;
    }

    // Create directory
    tokio::fs::create_dir_all(&chrome_dir).await.map_err(|e| {
        DepfusedError::IoError(e)
    })?;

    info!("Downloading Chromium to {:?}...", chrome_dir);

    let fetcher = BrowserFetcher::new(
        BrowserFetcherOptions::builder()
            .with_path(&chrome_dir)
            .build()
            .map_err(|e| {
                DepfusedError::ConfigError(format!("Failed to configure browser fetcher: {}", e))
            })?,
    );

    let info = fetcher.fetch().await.map_err(|e| {
        DepfusedError::ConfigError(format!("Failed to download Chromium: {}", e))
    })?;

    info!("Chromium downloaded to {:?}", info.executable_path);
    Ok(info.executable_path)
}

/// Group a list of URLs by their host (scheme + host + port).
/// URLs that fail to parse go into a special "" key.
pub fn group_by_host(urls: &[String]) -> Vec<(String, Vec<String>)> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    // Track insertion order
    let mut order: Vec<String> = Vec::new();

    for url_str in urls {
        let key = Url::parse(url_str)
            .ok()
            .and_then(|u| {
                let host = u.host_str()?.to_string();
                let port = u.port().map(|p| format!(":{}", p)).unwrap_or_default();
                Some(format!("{}://{}{}", u.scheme(), host, port))
            })
            .unwrap_or_default();

        let entry = groups.entry(key.clone()).or_default();
        if entry.is_empty() {
            order.push(key);
        }
        entry.push(url_str.clone());
    }

    order
        .into_iter()
        .filter_map(|key| {
            groups.remove(&key).map(|urls| (key, urls))
        })
        .collect()
}

/// Search a directory recursively for a Chrome/Chromium executable.
fn find_chrome_in_dir(dir: &std::path::Path) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }

    let entries = walk_dir_recursive(dir);
    for entry in entries {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let is_exe = name == "chrome"
            || name == "chromium"
            || name == "Chromium"
            || name == "Google Chrome"
            || name == "chrome.exe"
            || name == "chromium.exe"
            // macOS app bundle
            || name == "Chromium.app";

        if is_exe {
            // For macOS .app bundles, find the actual binary inside
            if name.ends_with(".app") {
                let inner = entry
                    .path()
                    .join("Contents/MacOS/Chromium");
                if inner.exists() {
                    return Some(inner);
                }
                continue;
            }
            if entry.path().is_file() {
                return Some(entry.path().to_path_buf());
            }
        }
    }
    None
}

/// Simple recursive directory walk (avoids adding walkdir dependency).
fn walk_dir_recursive(dir: &std::path::Path) -> Vec<std::fs::DirEntry> {
    let mut results = Vec::new();
    walk_recursive(dir, &mut results);
    results
}

fn walk_recursive(dir: &std::path::Path, results: &mut Vec<std::fs::DirEntry>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            results.push(entry);
            if path.is_dir() {
                walk_recursive(&path, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_host() {
        let urls = vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
            "https://other.com/page1".to_string(),
            "http://localhost:9001".to_string(),
            "http://localhost:9002".to_string(),
        ];

        let groups = group_by_host(&urls);
        assert_eq!(groups.len(), 4);

        // Check first group is example.com with 2 URLs
        assert_eq!(groups[0].0, "https://example.com");
        assert_eq!(groups[0].1.len(), 2);

        // Check other.com has 1 URL
        assert_eq!(groups[1].0, "https://other.com");
        assert_eq!(groups[1].1.len(), 1);

        // localhost entries are split by port
        assert_eq!(groups[2].0, "http://localhost:9001");
        assert_eq!(groups[2].1.len(), 1);

        assert_eq!(groups[3].0, "http://localhost:9002");
        assert_eq!(groups[3].1.len(), 1);
    }

    #[test]
    fn test_managed_chrome_dir() {
        let dir = managed_chrome_dir().unwrap();
        assert!(dir.ends_with("chrome"));
        assert!(dir.to_string_lossy().contains(".depfused"));
    }
}
