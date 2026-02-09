//! depfused - High-performance dependency confusion scanner.
//!
//! This library provides tools for detecting dependency confusion vulnerabilities by:
//! - Collecting JS files via headless browser capture
//! - Parsing them with AST to extract package names
//! - Probing for source maps (even when not explicitly referenced)
//! - Checking if extracted packages exist on npm registry
//!
//! # Example
//!
//! ```no_run
//! use depfused::scanner::Scanner;
//! use depfused::config::ScanConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let scanner = Scanner::new(Default::default()).await.unwrap();
//!     let result = scanner.scan("https://example.com").await.unwrap();
//!     println!("Found {} potential vulnerabilities", result.findings.len());
//! }
//! ```

pub mod config;
pub mod discovery;
pub mod notify;
pub mod parser;
pub mod registry;
pub mod scanner;
pub mod types;

pub mod browser;

pub use config::{Commands, Config, ScanConfig, SetupConfig};
pub use scanner::Scanner;
pub use types::{
    Confidence, DepfusedError, ExtractionMethod, Finding, JsFile, JsSource, NpmCheckResult,
    Package, Result, ScanResult, Severity,
};
