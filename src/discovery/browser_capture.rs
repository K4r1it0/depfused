//! Browser-based JavaScript capture using Chrome DevTools Protocol.
//!
//! This module captures JavaScript files from real browser sessions using chromiumoxide,
//! ensuring we get all JavaScript files including dynamically loaded ones.
//!
//! Requires: Chrome or Chromium browser installed

use crate::discovery::js_fetcher::{JsFetcher, extract_sourcemap_url};
use crate::types::{DepfusedError, JsFile, JsSource, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::{
    EventResponseReceived, GetResponseBodyParams, ResourceType,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Counter for generating unique browser profile directories
static BROWSER_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Browser-based network capture using Chrome DevTools Protocol.
pub struct BrowserCapture {
    /// Timeout for page load in seconds
    timeout_secs: u64,
    /// Whether to run headless
    headless: bool,
    /// Fast mode: reduce wait times (may miss some lazy-loaded JS)
    fast_mode: bool,
    /// Explicit path to Chrome/Chromium executable
    chrome_executable: Option<std::path::PathBuf>,
}

impl BrowserCapture {
    /// Create a new browser capture instance.
    pub fn new(timeout_secs: u64, headless: bool) -> Self {
        Self {
            timeout_secs,
            headless,
            fast_mode: false,
            chrome_executable: None,
        }
    }

    /// Enable fast mode for quicker scans.
    pub fn with_fast_mode(mut self, fast: bool) -> Self {
        self.fast_mode = fast;
        self
    }

    /// Set an explicit Chrome/Chromium executable path.
    pub fn with_chrome_executable(mut self, path: Option<std::path::PathBuf>) -> Self {
        self.chrome_executable = path;
        self
    }

    /// Build a BrowserConfig with the given temp directory.
    fn build_browser_config(
        &self,
        temp_dir: &std::path::Path,
        chrome_exe: Option<&std::path::Path>,
    ) -> Result<BrowserConfig> {
        let mut config_builder = BrowserConfig::builder()
            .user_data_dir(temp_dir);

        if let Some(exe) = chrome_exe {
            config_builder = config_builder.chrome_executable(exe);
        }

        if !self.headless {
            config_builder = config_builder.with_head();
        }

        config_builder = config_builder
            .no_sandbox()
            .viewport(None);

        config_builder.build().map_err(|e| {
            DepfusedError::ConfigError(format!("Failed to build browser config: {}", e))
        })
    }

    /// Launch a browser, with auto-download fallback if no Chrome is found.
    async fn launch_browser(
        &self,
        temp_dir: &std::path::Path,
    ) -> Result<(Browser, impl futures::Stream<Item = std::result::Result<(), chromiumoxide::error::CdpError>>)> {
        // Resolve Chrome executable: explicit path > previously downloaded > system Chrome
        let chrome_exe = self.chrome_executable.clone()
            .or_else(crate::browser::resolve_chrome_executable);

        // Try building config and launching — config build can fail if no Chrome is found
        let launch_result = match self.build_browser_config(temp_dir, chrome_exe.as_deref()) {
            Ok(config) => Browser::launch(config).await,
            Err(e) => Err(chromiumoxide::error::CdpError::msg(e.to_string())),
        };

        match launch_result {
            Ok(pair) => Ok(pair),
            Err(e) => {
                // If we had an explicit or resolved chrome path, don't try auto-download
                if chrome_exe.is_some() {
                    return Err(DepfusedError::ConfigError(format!(
                        "Failed to launch browser with Chrome at {:?}: {}",
                        chrome_exe.unwrap(),
                        e
                    )));
                }

                warn!(
                    "Chrome not found, downloading Chromium automatically... (run `depfused setup` to pre-install)"
                );
                let exe = crate::browser::download_chrome(false).await?;

                let config = self.build_browser_config(temp_dir, Some(&exe))?;
                Browser::launch(config).await.map_err(|e| {
                    DepfusedError::ConfigError(format!(
                        "Failed to launch browser even after downloading Chromium: {}",
                        e
                    ))
                })
            }
        }
    }

    /// Capture JavaScript files from a URL using headless browser.
    pub async fn capture(&self, url: &str) -> Result<Vec<JsFile>> {
        info!("Capturing with browser (native Rust): {}", url);

        // Create unique temporary directory for this browser instance
        // This allows multiple browser instances to run in parallel without conflicts
        let instance_id = BROWSER_INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("depfused-browser-{}-{}", std::process::id(), instance_id));

        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            debug!("Failed to create temp dir {:?}: {}", temp_dir, e);
        }

        // Store temp_dir for cleanup later
        let temp_dir_for_cleanup = temp_dir.clone();

        // Launch browser (with auto-download fallback)
        let (browser, mut handler) = self.launch_browser(&temp_dir).await?;

        // Spawn handler task
        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        // Capture JS files (with hard timeout to prevent hangs)
        let page_timeout = Duration::from_secs(self.timeout_secs + 15);
        let result = match tokio::time::timeout(page_timeout, self.capture_js_files(&browser, url)).await {
            Ok(r) => r,
            Err(_) => {
                warn!("Hard timeout after {}s for {}, skipping", page_timeout.as_secs(), url);
                Ok(Vec::new())
            }
        };

        // Clean up browser
        drop(browser);
        handler_task.abort();

        // Clean up temporary directory in background to not block
        tokio::spawn(async move {
            // Small delay to ensure browser has fully exited
            tokio::time::sleep(Duration::from_millis(100)).await;
            if let Err(e) = std::fs::remove_dir_all(&temp_dir_for_cleanup) {
                debug!("Failed to cleanup temp dir {:?}: {}", temp_dir_for_cleanup, e);
            }
        });

        result
    }

    /// Capture JavaScript files from multiple URLs using a single browser instance.
    /// Returns a vec of (url, result) pairs in the same order as input.
    pub async fn capture_multiple(&self, urls: &[&str]) -> Vec<(String, Result<Vec<JsFile>>)> {
        if urls.is_empty() {
            return Vec::new();
        }

        info!("Capturing {} URLs with shared browser instance", urls.len());

        let instance_id = BROWSER_INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir()
            .join(format!("depfused-browser-{}-{}", std::process::id(), instance_id));

        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            debug!("Failed to create temp dir {:?}: {}", temp_dir, e);
        }

        let temp_dir_for_cleanup = temp_dir.clone();
        let mut results = Vec::with_capacity(urls.len());

        // Launch browser
        let browser_result = self.launch_browser(&temp_dir).await;
        let (browser, mut handler) = match browser_result {
            Ok(pair) => pair,
            Err(e) => {
                // Return error for all URLs
                for url in urls {
                    results.push((
                        url.to_string(),
                        Err(DepfusedError::ConfigError(format!(
                            "Browser launch failed: {}",
                            e
                        ))),
                    ));
                }
                return results;
            }
        };

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        // Process each URL with the shared browser, restarting every 50 pages
        const RESTART_EVERY: usize = 50;
        let mut pages_used = 0;
        let mut current_browser = Some(browser);
        let mut current_handler_task = handler_task;

        for url in urls {
            // Restart browser if we've processed too many pages (prevent memory buildup)
            if pages_used > 0 && pages_used % RESTART_EVERY == 0 {
                info!("Restarting browser after {} pages to free memory", pages_used);
                current_browser.take(); // drops the old browser
                current_handler_task.abort();

                match self.launch_browser(&temp_dir).await {
                    Ok((new_browser, mut new_handler)) => {
                        current_handler_task = tokio::spawn(async move {
                            while let Some(event) = new_handler.next().await {
                                if event.is_err() {
                                    break;
                                }
                            }
                        });
                        current_browser = Some(new_browser);
                    }
                    Err(e) => {
                        results.push((
                            url.to_string(),
                            Err(DepfusedError::ConfigError(format!(
                                "Browser restart failed: {}",
                                e
                            ))),
                        ));
                        continue;
                    }
                }
            }

            if let Some(ref browser) = current_browser {
                let page_timeout = Duration::from_secs(self.timeout_secs + 15);
                let result = match tokio::time::timeout(page_timeout, self.capture_js_files(browser, url)).await {
                    Ok(r) => r,
                    Err(_) => {
                        warn!("Hard timeout after {}s for {}, killing browser and restarting", page_timeout.as_secs(), url);
                        // Kill the hung browser — Chrome may be spinning CPU
                        current_browser.take();
                        current_handler_task.abort();
                        match self.launch_browser(&temp_dir).await {
                            Ok((new_browser, mut new_handler)) => {
                                current_handler_task = tokio::spawn(async move {
                                    while let Some(event) = new_handler.next().await {
                                        if event.is_err() {
                                            break;
                                        }
                                    }
                                });
                                current_browser = Some(new_browser);
                                pages_used = 0;
                            }
                            Err(e) => {
                                warn!("Browser restart after timeout failed: {}", e);
                            }
                        }
                        Ok(Vec::new())
                    }
                };
                results.push((url.to_string(), result));
                pages_used += 1;
            } else {
                results.push((
                    url.to_string(),
                    Err(DepfusedError::ConfigError(
                        "Browser not available after restart failure".to_string(),
                    )),
                ));
            }
        }

        // Cleanup
        drop(current_browser);
        current_handler_task.abort();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if let Err(e) = std::fs::remove_dir_all(&temp_dir_for_cleanup) {
                debug!("Failed to cleanup temp dir {:?}: {}", temp_dir_for_cleanup, e);
            }
        });

        results
    }

    /// Capture JavaScript files from the page.
    async fn capture_js_files(&self, browser: &Browser, url: &str) -> Result<Vec<JsFile>> {
        // Create new page
        let page = browser.new_page("about:blank").await.map_err(|e| {
            DepfusedError::ConfigError(format!("Failed to create page: {}", e))
        })?;

        // Storage for captured JS
        let js_files: Arc<Mutex<HashMap<String, JsFile>>> = Arc::new(Mutex::new(HashMap::new()));
        let js_files_clone = js_files.clone();

        // Set up network event listener
        let mut response_events = page.event_listener::<EventResponseReceived>().await.map_err(|e| {
            DepfusedError::ConfigError(format!("Failed to set up event listener: {}", e))
        })?;

        // Clone page for the event handler
        let page_clone = page.clone();

        // Spawn task to handle response events
        let capture_task = tokio::spawn(async move {
            while let Some(event) = response_events.next().await {
                let response = &event.response;
                let url = response.url.clone();
                let mime_type = response.mime_type.clone();

                // Check if this is JavaScript (exclude JSON API responses)
                let is_script = matches!(event.r#type, ResourceType::Script);
                let is_js = is_script
                    || mime_type.contains("javascript")
                    || url.ends_with(".js")
                    || url.contains(".js?");

                if is_js {
                    debug!("Captured JS response: {} ({})", url, mime_type);

                    // Try to get response body
                    if let Ok(body) = page_clone.execute(
                        GetResponseBodyParams::new(event.request_id.clone())
                    ).await {
                        let content = if body.base64_encoded {
                            // Decode base64
                            use base64::Engine;
                            match base64::engine::general_purpose::STANDARD.decode(&body.body) {
                                Ok(decoded) => String::from_utf8_lossy(&decoded).to_string(),
                                Err(_) => body.body.clone(),
                            }
                        } else {
                            body.body.clone()
                        };

                        if !content.is_empty() {
                            let source_map_url = extract_sourcemap_url(&content, &url);
                            let content_hash = JsFetcher::hash_content(&content);
                            let js_file = JsFile {
                                url: url.clone(),
                                content,
                                content_hash,
                                source: JsSource::Browser,
                                source_map_url,
                            };

                            let mut files = js_files_clone.lock().await;
                            files.insert(url, js_file);
                        }
                    }
                }
            }
        });

        // Navigate to URL with timeout
        debug!("Navigating to: {}", url);
        let navigate_result = tokio::time::timeout(
            Duration::from_secs(self.timeout_secs),
            page.goto(url)
        ).await;

        match navigate_result {
            Ok(Ok(_)) => debug!("Navigation completed"),
            Ok(Err(e)) => warn!("Navigation error (continuing): {}", e),
            Err(_) => warn!("Navigation timeout (continuing with captured content)"),
        }

        // Adaptive wait: Stop early if no new JS files loaded in last 500ms
        let max_wait_secs = if self.fast_mode { 1 } else { 3 };
        let check_interval_ms = 500;
        let max_checks = (max_wait_secs * 1000) / check_interval_ms;

        let mut last_count = js_files.lock().await.len();
        let mut no_change_count = 0;

        debug!("Waiting up to {}s for lazy-loaded content (adaptive)...", max_wait_secs);
        for _ in 0..max_checks {
            tokio::time::sleep(Duration::from_millis(check_interval_ms)).await;

            let current_count = js_files.lock().await.len();
            if current_count == last_count {
                no_change_count += 1;
                // Wait for 1.5 seconds of no new files before stopping (3 checks * 500ms)
                // This provides more consistent results across runs
                if no_change_count >= 3 {
                    debug!("No new JS files loaded for 1.5s, stopping early");
                    break;
                }
            } else {
                no_change_count = 0;
                last_count = current_count;
            }
        }

        // Stop capture task
        capture_task.abort();

        // Collect results
        let files = js_files.lock().await;
        let result: Vec<JsFile> = files.values().cloned().collect();

        info!("Captured {} JavaScript files from {}", result.len(), url);
        Ok(result)
    }
}

impl Default for BrowserCapture {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            headless: true,
            fast_mode: false,
            chrome_executable: None,
        }
    }
}
