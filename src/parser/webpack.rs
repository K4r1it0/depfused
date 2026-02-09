//! Webpack-specific parsing for chunk manifests and runtime patterns.

use crate::parser::{filters, normalize_package_name};
use crate::types::{Confidence, ExtractionMethod, Package};
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;

/// Parser for webpack-specific patterns and chunk manifests.
#[derive(Clone)]
pub struct WebpackParser {
    /// Patterns for detecting webpack runtime code.
    chunk_patterns: Vec<Regex>,
}

impl WebpackParser {
    /// Create a new webpack parser.
    pub fn new() -> Self {
        let chunk_patterns = vec![
            // webpackJsonp push pattern
            Regex::new(r#"window\["webpackJsonp"\]|webpackJsonp"#).unwrap(),
            // __webpack_require__ pattern
            Regex::new(r"__webpack_require__").unwrap(),
            // webpack chunk loading
            Regex::new(r"__webpack_chunk_load__").unwrap(),
            // Modern webpack runtime
            Regex::new(r#"self\["webpackChunk"#).unwrap(),
        ];

        Self { chunk_patterns }
    }

    /// Check if JS content is a webpack bundle.
    pub fn is_webpack_bundle(&self, content: &str) -> bool {
        self.chunk_patterns.iter().any(|p| p.is_match(content))
    }

    /// Extract package names from webpack module comments/ids.
    pub fn extract_packages(&self, content: &str, source_url: &str) -> Vec<Package> {
        let mut packages = HashSet::new();

        // Pattern 1: Module ID comments
        if let Ok(module_id_pattern) = Regex::new(r#"/\*\s*\d+\s*\*/\s*["']([^"']+)["']"#) {
            for cap in module_id_pattern.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                    if let Some(pkg) = self.extract_package_from_webpack_path(path.as_str()) {
                        // Apply filters to reduce false positives
                        if filters::should_filter_package(&pkg, Some(content), Some(source_url)) {
                            debug!("Filtered package (Pattern 1): {}", pkg);
                            continue;
                        }

                        packages.insert(Package {
                            name: pkg,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }

        // Pattern 2: Module exports pattern with package names in comments
        if let Ok(exports_pattern) = Regex::new(r#"/\*\*\*/\s*["'](@?[\w-]+(?:/[\w.-]+)*)["']\s*:"#) {
            for cap in exports_pattern.captures_iter(content) {
                if let Some(pkg_path) = cap.get(1) {
                    if let Some(pkg) = normalize_package_name(pkg_path.as_str()) {
                        // Apply filters to reduce false positives
                        if filters::should_filter_package(&pkg, Some(content), Some(source_url)) {
                            debug!("Filtered package (Pattern 2): {}", pkg);
                            continue;
                        }

                        packages.insert(Package {
                            name: pkg,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }

        // Pattern 3: __webpack_require__.m map keys
        // Note: Webpack outputs `: (function` not just `: function`, so we make the paren optional
        if let Ok(require_map_pattern) = Regex::new(r#"["']((?:\./)?node_modules/[^"']+)["']\s*:\s*\(?function"#) {
            for cap in require_map_pattern.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                    if let Some(pkg) = self.extract_package_from_webpack_path(path.as_str()) {
                        // Apply filters to reduce false positives
                        if filters::should_filter_package(&pkg, Some(content), Some(source_url)) {
                            debug!("Filtered package (Pattern 3): {}", pkg);
                            continue;
                        }

                        packages.insert(Package {
                            name: pkg,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }

        // Pattern 4: Vendor chunk splitting comments
        if let Ok(vendor_pattern) = Regex::new(r#"vendors?[~-](@?[\w-]+(?:/[\w.-]+)*)"#) {
            for cap in vendor_pattern.captures_iter(content) {
                if let Some(pkg) = cap.get(1) {
                    if let Some(normalized) = normalize_package_name(pkg.as_str()) {
                        // Apply filters to reduce false positives
                        // This pattern has highest FP rate (CSS classes, etc.)
                        if filters::should_filter_package(&normalized, Some(content), Some(source_url)) {
                            debug!("Filtered package (Pattern 4): {}", normalized);
                            continue;
                        }

                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::Medium,
                        });
                    }
                }
            }
        }

        let result: Vec<Package> = packages.into_iter().collect();
        debug!(
            "Extracted {} packages from webpack patterns: {}",
            result.len(),
            source_url
        );

        result
    }

    /// Extract package name from webpack path.
    fn extract_package_from_webpack_path(&self, path: &str) -> Option<String> {
        // Remove leading ./ if present
        let path = path.strip_prefix("./").unwrap_or(path);

        // Find node_modules
        if let Some(idx) = path.find("node_modules/") {
            let after_nm = &path[idx + "node_modules/".len()..];

            // Handle scoped packages
            if after_nm.starts_with('@') {
                let parts: Vec<&str> = after_nm.split('/').collect();
                if parts.len() >= 2 {
                    let full_name = format!("{}/{}", parts[0], parts[1]);
                    return normalize_package_name(&full_name);
                }
            } else {
                let package = after_nm.split('/').next()?;
                return normalize_package_name(package);
            }
        }

        None
    }

    /// Detect Next.js build patterns and extract build ID.
    pub fn extract_nextjs_build_id(&self, content: &str) -> Option<String> {
        // Pattern: "_next/static/[buildId]/..."
        let pattern = Regex::new(r"_next/static/([a-zA-Z0-9_-]+)/").ok()?;

        pattern
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Generate Next.js manifest URLs from build ID.
    pub fn get_nextjs_manifest_urls(&self, base_url: &str, build_id: &str) -> Vec<String> {
        let origin = if let Ok(url) = url::Url::parse(base_url) {
            format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""))
        } else {
            return vec![];
        };

        vec![
            format!("{}/_next/static/{}//_buildManifest.js", origin, build_id),
            format!("{}/_next/static/{}//_ssgManifest.js", origin, build_id),
            format!("{}/_next/static/chunks/webpack.js", origin),
            format!("{}/_next/static/chunks/main.js", origin),
            format!("{}/_next/static/chunks/framework.js", origin),
            format!("{}/_next/static/chunks/pages/_app.js", origin),
        ]
    }
}

impl Default for WebpackParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_webpack_bundle() {
        let parser = WebpackParser::new();

        assert!(parser.is_webpack_bundle("(window.webpackJsonp=window.webpackJsonp||[]).push"));
        assert!(parser.is_webpack_bundle("__webpack_require__(123)"));
        assert!(!parser.is_webpack_bundle("console.log('hello')"));
    }

    #[test]
    fn test_extract_package_from_webpack_path() {
        let parser = WebpackParser::new();

        assert_eq!(
            parser.extract_package_from_webpack_path("./node_modules/lodash/index.js"),
            Some("lodash".to_string())
        );
        assert_eq!(
            parser.extract_package_from_webpack_path("./node_modules/@company/utils/src/index.js"),
            Some("@company/utils".to_string())
        );
        assert_eq!(
            parser.extract_package_from_webpack_path("./src/app.js"),
            None
        );
    }

    #[test]
    fn test_extract_nextjs_build_id() {
        let parser = WebpackParser::new();

        let content = r#"
            "/_next/static/abc123def/_buildManifest.js"
        "#;

        let build_id = parser.extract_nextjs_build_id(content);
        assert_eq!(build_id, Some("abc123def".to_string()));
    }
}
