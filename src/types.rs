//! Core types and errors for the dependency confusion scanner.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Errors that can occur during scanning.
#[derive(Error, Debug)]
pub enum DepfusedError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Source map parse error: {0}")]
    SourceMapError(String),

    #[error("AST parse error: {0}")]
    AstParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rate limited by {0}")]
    RateLimited(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Telegram error: {0}")]
    TelegramError(String),
}

pub type Result<T> = std::result::Result<T, DepfusedError>;

/// Represents a discovered JavaScript file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsFile {
    /// The URL where the JS file was found.
    pub url: String,
    /// The content of the JS file.
    pub content: String,
    /// SHA256 hash of the content for deduplication.
    pub content_hash: String,
    /// Source of discovery (browser capture or probe).
    pub source: JsSource,
    /// Associated source map URL if found.
    pub source_map_url: Option<String>,
}

/// Source of JS file discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JsSource {
    /// Captured via headless browser.
    Browser,
    /// Discovered through probing source maps.
    Probe,
}

/// Represents an extracted package reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Package {
    /// Package name (e.g., "@company/pkg" or "lodash").
    pub name: String,
    /// How the package was discovered.
    pub extraction_method: ExtractionMethod,
    /// The source JS file URL where it was found.
    pub source_url: String,
    /// Confidence level of the extraction.
    pub confidence: Confidence,
}

/// Method used to extract the package name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExtractionMethod {
    /// From import statement (import x from 'pkg').
    Import,
    /// From require call (require('pkg')).
    Require,
    /// From dynamic import (import('pkg')).
    DynamicImport,
    /// From source map sources array.
    SourceMap,
    /// From webpack chunk manifest.
    WebpackChunk,
    /// From comment in JS file.
    Comment,
    /// From error message string.
    ErrorMessage,
    /// From deobfuscated/encoded strings.
    Deobfuscate,
}

/// Confidence level of package extraction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Confidence {
    /// Low confidence (e.g., from error messages).
    Low,
    /// Medium confidence (e.g., from comments).
    Medium,
    /// High confidence (e.g., from AST parsing).
    High,
}

/// Result of checking a package against npm registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NpmCheckResult {
    /// Package exists on npm.
    Exists {
        name: String,
        latest_version: Option<String>,
    },
    /// Package does not exist (potential vulnerability).
    NotFound { name: String },
    /// Scoped package where scope is not claimed.
    ScopeNotClaimed { scope: String, name: String },
    /// Error checking the package.
    Error { name: String, error: String },
}

/// A confirmed or potential dependency confusion finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// The package that may be vulnerable.
    pub package: Package,
    /// Result of npm registry check.
    pub npm_result: NpmCheckResult,
    /// Severity assessment.
    pub severity: Severity,
    /// Additional context/notes.
    pub notes: Vec<String>,
}

/// Severity level of a finding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational only.
    Info,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity (confirmed missing package).
    High,
    /// Critical (scoped package with unclaimed scope).
    Critical,
}

/// Complete scan result for a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Target URL that was scanned.
    pub target: String,
    /// All JS files discovered.
    pub js_files_count: usize,
    /// All packages extracted.
    pub packages_found: usize,
    /// Confirmed findings.
    pub findings: Vec<Finding>,
    /// Scan duration in seconds.
    pub duration_secs: f64,
    /// Any errors encountered during scan.
    pub errors: Vec<String>,
}

/// Deduplication set for content hashes.
pub type ContentHashSet = HashSet<String>;

/// Configuration for HTTP requests.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 3,
            user_agent: "Mozilla/5.0 (compatible; depfused/0.1)".to_string(),
        }
    }
}
