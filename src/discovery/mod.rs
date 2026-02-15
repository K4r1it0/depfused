//! JS file discovery module.
//!
//! This module handles discovering JavaScript files from:
//! - Browser capture (headless Chrome - primary method)
//! - Source map references

pub mod browser_capture;
pub mod js_fetcher;
pub mod sourcemap_probe;

pub use browser_capture::BrowserCapture;
pub use browser_capture::kill_all_chrome;
pub use js_fetcher::{JsFetcher, extract_sourcemap_url};
pub use sourcemap_probe::SourceMapProber;
