//! Source map parser for extracting package names from sources array.

use crate::parser::{filters, normalize_package_name};
use crate::types::{Confidence, ExtractionMethod, Package, Result};
use std::collections::HashSet;
use tracing::debug;

/// Parser for extracting package information from source maps.
#[derive(Clone)]
pub struct SourceMapParser;

impl SourceMapParser {
    /// Create a new source map parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse source map content and extract packages.
    ///
    /// Returns `(packages, workspace_only_names)` where `workspace_only_names` contains
    /// package names found in `packages/` paths but NOT in `node_modules/` paths.
    /// These are monorepo workspace packages that should be suppressed.
    pub fn parse(&self, content: &str, source_url: &str) -> Result<(Vec<Package>, HashSet<String>)> {
        let map: sourcemap::SourceMap = sourcemap::SourceMap::from_slice(content.as_bytes())
            .map_err(|e| crate::types::DepfusedError::SourceMapError(e.to_string()))?;

        // Classification pass: identify workspace-only packages
        let mut node_modules_names: HashSet<String> = HashSet::new();
        let mut workspace_names: HashSet<String> = HashSet::new();

        for source in map.sources() {
            let path = source
                .strip_prefix("webpack:///")
                .or_else(|| source.strip_prefix("webpack://"))
                .unwrap_or(source);

            if let Some(idx) = path.find("node_modules/") {
                let after = &path[idx + "node_modules/".len()..];
                if let Some(name) = self.extract_package_from_path_segment(after) {
                    node_modules_names.insert(name);
                }
            } else if let Some(idx) = path.find("packages/") {
                let after = &path[idx + "packages/".len()..];
                if let Some(name) = self.extract_package_from_path_segment(after) {
                    workspace_names.insert(name);
                }
            }
        }

        let workspace_only: HashSet<String> = workspace_names
            .difference(&node_modules_names)
            .cloned()
            .collect();

        let mut packages = HashSet::new();

        // Extract from sources array
        for source in map.sources() {
            if let Some(pkgs) = self.extract_packages_from_path(source, source_url) {
                packages.extend(pkgs);
            }
        }

        // Extract from sourcesContent — find require/import of packages in embedded source code
        let source_count = map.get_source_count();
        for i in 0..source_count {
            if let Some(content) = map.get_source_contents(i) {
                self.extract_packages_from_source_content(content, source_url, &mut packages);
            }
        }

        // Filter workspace-only packages
        let result: Vec<Package> = packages
            .into_iter()
            .filter(|p| !workspace_only.contains(&p.name))
            .collect();

        debug!(
            "Extracted {} packages from source map (filtered {} workspace-only): {}",
            result.len(),
            workspace_only.len(),
            source_url
        );

        Ok((result, workspace_only))
    }

    /// Extract package names from a source path.
    fn extract_packages_from_path(&self, path: &str, source_url: &str) -> Option<HashSet<Package>> {
        let mut packages = HashSet::new();

        // Pattern: node_modules/@scope/package/...
        // Pattern: node_modules/package/...
        // Pattern: webpack://node_modules/@scope/package/...
        // Pattern: webpack://@scope/package/...

        let path = path
            .strip_prefix("webpack:///")
            .or_else(|| path.strip_prefix("webpack://"))
            .unwrap_or(path);

        // Look for node_modules pattern
        if let Some(idx) = path.find("node_modules/") {
            let after_nm = &path[idx + "node_modules/".len()..];
            if let Some(pkg_name) = self.extract_package_from_path_segment(after_nm) {
                // Apply filters to reduce false positives
                if !filters::should_filter_package(&pkg_name, Some(path), Some(source_url)) {
                    packages.insert(Package {
                        name: pkg_name,
                        extraction_method: ExtractionMethod::SourceMap,
                        source_url: source_url.to_string(),
                        confidence: Confidence::High,
                    });
                }
            }
        }
        // Check for packages/ directory pattern (monorepo workspaces)
        // e.g., ../packages/private-logger/index.js, packages/@scope/utils/src/index.js
        // NOTE: Low confidence because monorepo workspace names rarely match their
        // published npm name (they almost always have a scope like @company/pkg).
        else if let Some(idx) = path.find("packages/") {
            let after_packages = &path[idx + "packages/".len()..];
            if let Some(pkg_name) = self.extract_package_from_path_segment(after_packages) {
                if !filters::should_filter_package(&pkg_name, Some(path), Some(source_url)) {
                    packages.insert(Package {
                        name: pkg_name,
                        extraction_method: ExtractionMethod::SourceMap,
                        source_url: source_url.to_string(),
                        confidence: Confidence::Low,
                    });
                }
            }
        }
        // Check for direct package reference (webpack://@scope/pkg or webpack:///pkg)
        else if path.starts_with('@') || path.starts_with("~/") {
            let clean_path = path.strip_prefix("~/").unwrap_or(path);
            if let Some(pkg_name) = self.extract_package_from_path_segment(clean_path) {
                // Apply filters to reduce false positives
                if !filters::should_filter_package(&pkg_name, Some(path), Some(source_url)) {
                    packages.insert(Package {
                        name: pkg_name,
                        extraction_method: ExtractionMethod::SourceMap,
                        source_url: source_url.to_string(),
                        confidence: Confidence::Medium,
                    });
                }
            }
        }

        if packages.is_empty() {
            None
        } else {
            Some(packages)
        }
    }

    /// Extract package names from sourcesContent text (require/import statements).
    fn extract_packages_from_source_content(
        &self,
        content: &str,
        source_url: &str,
        packages: &mut HashSet<Package>,
    ) {
        // Only process content that likely has require/import of packages
        // Match: require("pkg"), require('pkg'), import "pkg", import 'pkg', from "pkg", from 'pkg'
        let patterns = [
            r#"require\s*\(\s*["']([^"'./][^"']*)["']\s*\)"#,
            r#"from\s+["']([^"'./][^"']*)["']"#,
            r#"import\s+["']([^"'./][^"']*)["']"#,
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for caps in re.captures_iter(content) {
                    if let Some(name_match) = caps.get(1) {
                        // FP: Skip commented-out require/import in source content
                        // e.g. "//const x = require('@scope/pkg')" in sourcesContent
                        let match_start = caps.get(0).unwrap().start();
                        let before = &content[..match_start];
                        // Find the start of the current line
                        let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
                        let line_prefix = content[line_start..match_start].trim();
                        if line_prefix.starts_with("//") || line_prefix.starts_with('*') {
                            continue;
                        }

                        let raw_name = name_match.as_str();
                        if let Some(pkg_name) = normalize_package_name(raw_name) {
                            if !filters::should_filter_package(&pkg_name, None, Some(source_url)) {
                                packages.insert(Package {
                                    name: pkg_name,
                                    extraction_method: ExtractionMethod::SourceMap,
                                    source_url: source_url.to_string(),
                                    confidence: Confidence::Low,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Extract package name from path segment after node_modules.
    fn extract_package_from_path_segment(&self, segment: &str) -> Option<String> {
        let segment = segment.trim_start_matches('/');

        // Scoped package: @scope/package/...
        if segment.starts_with('@') {
            let parts: Vec<&str> = segment.split('/').collect();
            if parts.len() >= 2 {
                let scope = parts[0];
                let package = parts[1];
                let full_name = format!("{}/{}", scope, package);
                return normalize_package_name(&full_name);
            }
        }
        // Regular package: package/...
        else {
            let package = segment.split('/').next()?;
            return normalize_package_name(package);
        }

        None
    }
}

impl Default for SourceMapParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_node_modules_path() {
        let parser = SourceMapParser::new();

        // Regular package
        let result = parser.extract_packages_from_path(
            "webpack:///node_modules/lodash/lodash.js",
            "test.js.map",
        );
        assert!(result.is_some());
        let packages: Vec<_> = result.unwrap().into_iter().collect();
        assert!(packages.iter().any(|p| p.name == "lodash"));

        // Scoped package
        let result = parser.extract_packages_from_path(
            "webpack:///node_modules/@company/utils/index.js",
            "test.js.map",
        );
        assert!(result.is_some());
        let packages: Vec<_> = result.unwrap().into_iter().collect();
        assert!(packages.iter().any(|p| p.name == "@company/utils"));
    }

    #[test]
    fn test_extract_from_webpack_direct() {
        let parser = SourceMapParser::new();

        let result =
            parser.extract_packages_from_path("webpack:///@internal/auth/src/index.js", "test.js.map");
        assert!(result.is_some());
        let packages: Vec<_> = result.unwrap().into_iter().collect();
        assert!(packages.iter().any(|p| p.name == "@internal/auth"));
    }

    #[test]
    fn test_parse_full_sourcemap() {
        let parser = SourceMapParser::new();
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///node_modules/lodash/index.js",
                "webpack:///node_modules/@company/utils/src/index.js",
                "webpack:///src/app.js"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"@company/utils"));
        assert_eq!(names.len(), 2);
        assert!(workspace.is_empty());
    }

    #[test]
    fn test_workspace_only_suppressed() {
        let parser = SourceMapParser::new();
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///packages/my-private-lib/src/index.js",
                "webpack:///node_modules/lodash/index.js"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(!names.contains(&"my-private-lib"));
        assert!(workspace.contains("my-private-lib"));
    }

    #[test]
    fn test_package_in_both_kept() {
        let parser = SourceMapParser::new();
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///packages/shared-utils/src/index.js",
                "webpack:///node_modules/shared-utils/dist/index.js"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"shared-utils"));
        assert!(!workspace.contains("shared-utils"));
    }

    #[test]
    fn test_node_modules_only_kept() {
        let parser = SourceMapParser::new();
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///node_modules/lodash/index.js",
                "webpack:///node_modules/react/index.js"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"react"));
        assert!(workspace.is_empty());
    }

    #[test]
    fn test_sources_content_workspace_filtered() {
        let parser = SourceMapParser::new();
        // Source map where my-private-lib appears in packages/ path AND is require'd in sourcesContent
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///packages/my-private-lib/src/index.js",
                "webpack:///node_modules/lodash/index.js"
            ],
            "sourcesContent": [
                "const utils = require('my-private-lib');\nmodule.exports = utils;",
                "module.exports = {};"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        // my-private-lib should be filtered from both sources path AND sourcesContent extraction
        assert!(!names.contains(&"my-private-lib"));
        assert!(workspace.contains("my-private-lib"));
    }

    #[test]
    fn test_empty_workspace_no_change() {
        let parser = SourceMapParser::new();
        let sourcemap_json = r#"{
            "version": 3,
            "sources": [
                "webpack:///node_modules/lodash/index.js",
                "webpack:///src/app.js",
                "webpack:///src/utils/helpers.js"
            ],
            "mappings": "AAAA",
            "names": []
        }"#;

        let (packages, workspace) = parser.parse(sourcemap_json, "bundle.js.map").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(workspace.is_empty());
    }
}
