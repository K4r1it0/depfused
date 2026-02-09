//! Configuration handling for the scanner.

use crate::types::HttpConfig;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// High-performance dependency confusion scanner.
#[derive(Parser, Debug, Clone)]
#[command(name = "depfused")]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Telegram bot token for notifications
    #[arg(long, env = "DEPFUSED_TELEGRAM_TOKEN", global = true)]
    pub telegram_token: Option<String>,

    /// Telegram chat ID for notifications
    #[arg(long, env = "DEPFUSED_TELEGRAM_CHAT_ID", global = true)]
    pub telegram_chat_id: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Scan targets for dependency confusion vulnerabilities
    Scan(ScanConfig),
    /// Download and set up a managed Chromium browser
    Setup(SetupConfig),
}

/// Configuration for the setup command.
#[derive(Parser, Debug, Clone)]
pub struct SetupConfig {
    /// Force re-download even if Chromium is already installed
    #[arg(long)]
    pub force: bool,
}

/// Configuration for the scan command.
#[derive(Parser, Debug, Clone)]
pub struct ScanConfig {
    /// Target URL(s) to scan
    #[arg(required_unless_present = "file")]
    pub targets: Vec<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// File containing URLs to scan (one per line)
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Enable Telegram notifications for findings
    #[arg(long)]
    pub telegram: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Maximum retries for failed requests
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Rate limit (requests per second)
    #[arg(long, default_value = "10")]
    pub rate_limit: u32,

    /// Skip npm registry checks (only extract packages)
    #[arg(long)]
    pub skip_npm_check: bool,

    /// Only check scoped packages (@scope/pkg)
    #[arg(long)]
    pub scoped_only: bool,

    /// Custom User-Agent string
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Minimum confidence level to report (low, medium, high)
    #[arg(long, default_value = "low")]
    pub min_confidence: String,

    /// Number of sites to scan in parallel (default: 1)
    #[arg(long, short = 'p', default_value = "1")]
    pub parallel: usize,

    /// Fast mode: reduce wait times for quicker scans (may miss some lazy-loaded JS)
    #[arg(long)]
    pub fast: bool,

    /// Quiet mode: only show output for targets with vulnerabilities
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Path to Chrome/Chromium executable (overrides auto-detection)
    #[arg(long)]
    pub chrome_path: Option<PathBuf>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            verbose: false,
            file: None,
            telegram: false,
            json: false,
            output: None,
            timeout: 30,
            max_retries: 3,
            rate_limit: 10,
            skip_npm_check: false,
            scoped_only: false,
            user_agent: None,
            min_confidence: "low".to_string(),
            parallel: 1,
            fast: false,
            quiet: false,
            chrome_path: None,
        }
    }
}

impl ScanConfig {
    /// Get HTTP configuration from scan config.
    pub fn http_config(&self) -> HttpConfig {
        HttpConfig {
            timeout_secs: self.timeout,
            max_retries: self.max_retries,
            user_agent: self.user_agent.clone().unwrap_or_else(|| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
            }),
        }
    }

    /// Load targets from file if specified.
    pub fn load_targets(&self) -> crate::types::Result<Vec<String>> {
        let mut targets = self.targets.clone();

        if let Some(ref file_path) = self.file {
            let content = std::fs::read_to_string(file_path)?;
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    targets.push(trimmed.to_string());
                }
            }
        }

        // Normalize URLs
        let normalized: Vec<String> = targets
            .into_iter()
            .map(|t| {
                if !t.starts_with("http://") && !t.starts_with("https://") {
                    format!("https://{}", t)
                } else {
                    t
                }
            })
            .collect();

        Ok(normalized)
    }
}

/// Source map URL variations to try.
pub fn get_sourcemap_variations(js_url: &str) -> Vec<String> {
    let mut variations = Vec::new();

    // Direct .map suffix
    variations.push(format!("{}.map", js_url));

    // Try without .min if present
    if js_url.contains(".min.js") {
        let without_min = js_url.replace(".min.js", ".js");
        variations.push(format!("{}.map", without_min));
    }

    // Try adding .min if not present
    if js_url.ends_with(".js") && !js_url.contains(".min.") {
        let with_min = js_url.replace(".js", ".min.js");
        variations.push(format!("{}.map", with_min));
    }

    // Try common sourcemap directories
    if let Some(filename) = js_url.rsplit('/').next() {
        if let Some(base_url) = js_url.strip_suffix(filename) {
            variations.push(format!("{}sourcemaps/{}.map", base_url, filename));
            variations.push(format!("{}_sourcemaps/{}.map", base_url, filename));
            variations.push(format!("{}maps/{}.map", base_url, filename));
        }
    }

    variations
}
