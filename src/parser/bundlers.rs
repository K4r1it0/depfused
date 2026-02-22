//! Bundler-specific parsers for Vite, Parcel, Turbopack, esbuild, and others.

use crate::parser::{filters, normalize_package_name};
use crate::types::{Confidence, ExtractionMethod, Package};
use regex::Regex;
use std::collections::HashSet;
use tracing::{debug, trace};

/// Parser for various JavaScript bundler output formats.
#[derive(Clone)]
pub struct BundlerParser {
    /// Vite/Rollup patterns
    vite_patterns: Vec<Regex>,
    /// Parcel patterns
    parcel_patterns: Vec<Regex>,
    /// Turbopack patterns (Next.js 13+)
    turbopack_patterns: Vec<Regex>,
    /// esbuild patterns
    esbuild_patterns: Vec<Regex>,
    /// SWC patterns
    swc_patterns: Vec<Regex>,
    /// Generic minified patterns
    minified_patterns: Vec<Regex>,
}

impl BundlerParser {
    /// Create a new bundler parser with all patterns initialized.
    pub fn new() -> Self {
        Self {
            vite_patterns: Self::build_vite_patterns(),
            parcel_patterns: Self::build_parcel_patterns(),
            turbopack_patterns: Self::build_turbopack_patterns(),
            esbuild_patterns: Self::build_esbuild_patterns(),
            swc_patterns: Self::build_swc_patterns(),
            minified_patterns: Self::build_minified_patterns(),
        }
    }

    /// Build Vite/Rollup specific patterns.
    fn build_vite_patterns() -> Vec<Regex> {
        vec![
            // Vite chunk imports: import { x } from "/node_modules/.vite/deps/lodash.js"
            Regex::new(r#"from\s*["']/node_modules/\.vite/deps/([^"'?]+)"#).unwrap(),
            // Vite optimized deps: /@id/__x00__@company/utils
            Regex::new(r#"/@id/__x00__(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
            // Rollup chunk naming: chunk-lodash-abc123.js
            Regex::new(r#"chunk[_-](@?[\w-]+(?:/[\w.-]+)?)[_-][a-f0-9]+"#).unwrap(),
            // Rollup vendor chunk: vendor.lodash.js or vendor-@company-utils.js
            Regex::new(r#"vendor[._-](@?[\w-]+(?:/[\w.-]+)?)"#).unwrap(),
            // Vite import analysis: import_lodash, import_@company_utils
            Regex::new(r#"__vite__import(?:Analysis)?[_-](\d+)[_-](@?[\w-]+)"#).unwrap(),
            // Rollup external: /*#__PURE__*/require('lodash')
            Regex::new(r#"/\*#__PURE__\*/\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Vite pre-bundled: /node_modules/.vite/deps/@company_utils.js
            Regex::new(r#"/node_modules/\.vite/deps/(@[\w-]+[_-][\w.-]+|[\w.-]+)\.js"#).unwrap(),
        ]
    }

    /// Build Parcel specific patterns.
    fn build_parcel_patterns() -> Vec<Regex> {
        vec![
            // Parcel 2 module map: "node_modules/lodash/index.js": [function(...)
            Regex::new(r#"["']node_modules/([^"']+)["']\s*:\s*\[?\s*function"#).unwrap(),
            // Parcel require runtime: parcelRequire("lodash")
            Regex::new(r#"parcelRequire\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Parcel 2 dependency map: $parcel$require("@company/utils")
            Regex::new(r#"\$parcel\$require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Parcel module id comment: /* @company/utils */
            Regex::new(r#"/\*\s*(@[\w-]+/[\w.-]+)\s*\*/"#).unwrap(),
            // Parcel asset id: $abc123$exports from node_modules/@company/utils
            Regex::new(r#"\$[a-f0-9]+\$exports.*node_modules/(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
            // Parcel 2 bundle: Object(parcel_require("pkg"))
            Regex::new(r#"parcel[_-]?require\s*\(\s*["'](@?[\w-]+(?:/[\w.-]+)?)["']\s*\)"#).unwrap(),
        ]
    }

    /// Build Turbopack specific patterns (Next.js 13+).
    fn build_turbopack_patterns() -> Vec<Regex> {
        vec![
            // Turbopack module: [project]/node_modules/@company/utils/index.js
            Regex::new(r#"\[project\]/node_modules/(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
            // Turbopack chunk: __turbopack_require__("[project]/node_modules/lodash")
            Regex::new(r#"__turbopack_require__\s*\(\s*["']\[project\]/node_modules/([^"'\]]+)"#).unwrap(),
            // Turbopack import: __turbopack_import__("@company/utils")
            Regex::new(r#"__turbopack_import__\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Turbopack external: __turbopack_external_require__("lodash")
            Regex::new(r#"__turbopack_external_require__\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Turbopack binding: turbopack_binding["@company/utils"]
            Regex::new(r#"turbopack[_-]?binding\s*\[\s*["']([^"']+)["']\s*\]"#).unwrap(),
            // Next.js 13+ app router chunks: (self.__next_f=self.__next_f||[]).push
            Regex::new(r#"__next_[a-z]+.*["']node_modules/(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
            // Turbopack module id: "turbopack://[project]/node_modules/@scope/pkg"
            Regex::new(r#"turbopack://\[project\]/node_modules/(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
        ]
    }

    /// Build esbuild specific patterns.
    fn build_esbuild_patterns() -> Vec<Regex> {
        vec![
            // esbuild require: __require("lodash")
            Regex::new(r#"__require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // esbuild commonjs: __commonJS({ "node_modules/lodash/index.js"(exports, module)
            Regex::new(r#"__commonJS\s*\(\s*\{\s*["']node_modules/([^"']+)["']"#).unwrap(),
            // esbuild esm: __esm({ "node_modules/@company/utils/index.js"()
            Regex::new(r#"__esm\s*\(\s*\{\s*["']node_modules/([^"']+)["']"#).unwrap(),
            // esbuild export: __export(lodash_exports, { ... })
            Regex::new(r#"__export\s*\(\s*(\w+)_exports"#).unwrap(),
            // esbuild toESM: __toESM(require_lodash())
            Regex::new(r#"__toESM\s*\(\s*require_([a-zA-Z0-9_]+)\s*\(\s*\)\s*\)"#).unwrap(),
            // esbuild init: var init_lodash = __esm({...})
            Regex::new(r#"var\s+init_([a-zA-Z0-9_]+)\s*=\s*__esm"#).unwrap(),
            // esbuild banner comment: // node_modules/@company/utils/index.js
            Regex::new(r#"//\s*node_modules/(@[\w-]+/[\w.-]+|[\w.-]+)"#).unwrap(),
            // esbuild chunk: chunk-ABCD1234.js containing @company/utils
            Regex::new(r#"chunk-[A-Z0-9]+\.js.*["'](@[\w-]+/[\w.-]+|[\w.-]+)["']"#).unwrap(),
        ]
    }

    /// Build SWC specific patterns.
    fn build_swc_patterns() -> Vec<Regex> {
        vec![
            // SWC helpers: _interop_require_default(require("lodash"))
            Regex::new(r#"_interop_require_\w+\s*\(\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // SWC export star: _export_star(require("@company/utils"), exports)
            Regex::new(r#"_export_star\s*\(\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // SWC class private: _class_private_field_get from @swc/helpers
            Regex::new(r#"from\s*["'](@swc/[\w.-]+)["']"#).unwrap(),
        ]
    }

    /// Build patterns for minified/obfuscated code.
    fn build_minified_patterns() -> Vec<Regex> {
        vec![
            // Minified require with short var: var a=require("lodash")
            Regex::new(r#"(?:var|let|const)\s+[a-z]\s*=\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Minified import: import a from"lodash" (no space)
            Regex::new(r#"import\s+[a-z]\s+from\s*["']([^"']+)["']"#).unwrap(),
            // Bracket notation require: e["require"]("lodash")
            Regex::new(r#"\w\s*\[\s*["']require["']\s*\]\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Object.assign pattern with package: Object.assign(e,require("lodash"))
            Regex::new(r#"Object\.assign\s*\([^,]+,\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Spread require: {...require("lodash")}
            Regex::new(r#"\{\s*\.\.\.require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Module.exports = require: module.exports=require("lodash")
            Regex::new(r#"module\.exports\s*=\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Ternary require: a?require("lodash"):null
            Regex::new(r#"\?\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Logical AND require: a&&require("lodash")
            Regex::new(r#"&&\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Array element require: [require("lodash")]
            Regex::new(r#"\[\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
            // Comma expression require: (0,require("lodash"))
            Regex::new(r#"\(\s*\d+\s*,\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap(),
        ]
    }

    /// Detect which bundler was used to create the content.
    pub fn detect_bundler(&self, content: &str) -> Option<BundlerType> {
        // Check for Turbopack (Next.js 13+)
        if content.contains("__turbopack_") || content.contains("[project]/node_modules") {
            return Some(BundlerType::Turbopack);
        }

        // Check for Vite
        if content.contains(".vite/deps") || content.contains("/@id/__x00__") {
            return Some(BundlerType::Vite);
        }

        // Check for Parcel
        if content.contains("parcelRequire") || content.contains("$parcel$") {
            return Some(BundlerType::Parcel);
        }

        // Check for esbuild
        if content.contains("__commonJS") || content.contains("__toESM") || content.contains("__require") {
            return Some(BundlerType::Esbuild);
        }

        // Check for SWC
        if content.contains("_interop_require_") || content.contains("@swc/helpers") {
            return Some(BundlerType::Swc);
        }

        // Check for Rollup
        if content.contains("/*#__PURE__*/") && content.contains("require") {
            return Some(BundlerType::Rollup);
        }

        None
    }

    /// Extract packages from content using all bundler patterns.
    pub fn extract_packages(&self, content: &str, source_url: &str) -> Vec<Package> {
        let mut packages = HashSet::new();

        // Detect bundler type for logging
        if let Some(bundler) = self.detect_bundler(content) {
            debug!("Detected bundler: {:?} for {}", bundler, source_url);
        }

        // Run all pattern matchers
        self.extract_vite_packages(content, source_url, &mut packages);
        self.extract_parcel_packages(content, source_url, &mut packages);
        self.extract_turbopack_packages(content, source_url, &mut packages);
        self.extract_esbuild_packages(content, source_url, &mut packages);
        self.extract_swc_packages(content, source_url, &mut packages);
        self.extract_minified_packages(content, source_url, &mut packages);

        let result: Vec<Package> = packages
            .into_iter()
            .filter(|p| !filters::should_filter_package(&p.name, Some(content), Some(source_url)))
            .collect();
        if !result.is_empty() {
            debug!(
                "Extracted {} packages from bundler patterns: {}",
                result.len(),
                source_url
            );
        }

        result
    }

    /// Extract packages using Vite/Rollup patterns.
    fn extract_vite_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.vite_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    let raw_name = pkg_match.as_str();
                    // Handle Vite's underscore conversion: @company_utils -> @company/utils
                    let name = self.convert_vite_name(raw_name);
                    if let Some(normalized) = normalize_package_name(&name) {
                        trace!("Vite pattern matched: {} -> {}", raw_name, normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk, // Reuse for bundler
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }
    }

    /// Convert Vite naming conventions back to package names.
    fn convert_vite_name(&self, name: &str) -> String {
        // Vite converts @ to empty and / to _ in some cases
        // e.g., @company/utils -> company_utils or @company_utils
        let mut result = name.to_string();

        // Handle .js extension
        if result.ends_with(".js") {
            result = result[..result.len() - 3].to_string();
        }

        // Handle @scope_package -> @scope/package
        if result.starts_with('@') && !result.contains('/') {
            if let Some(idx) = result[1..].find('_') {
                let scope = &result[0..=idx];
                let pkg = &result[idx + 2..];
                result = format!("{}/{}", scope, pkg);
            }
        }

        // Handle scope_package -> @scope/package (without @)
        if !result.starts_with('@') && result.contains('_') {
            // Check if first part looks like a scope
            if let Some(idx) = result.find('_') {
                let potential_scope = &result[..idx];
                // Common scope-like names
                if ["company", "internal", "private", "org", "team"].iter()
                    .any(|s| potential_scope.contains(s)) {
                    let pkg = &result[idx + 1..];
                    result = format!("@{}/{}", potential_scope, pkg);
                }
            }
        }

        result
    }

    /// Extract packages using Parcel patterns.
    fn extract_parcel_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.parcel_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    if let Some(normalized) = self.extract_package_from_path(pkg_match.as_str()) {
                        trace!("Parcel pattern matched: {}", normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }
    }

    /// Extract packages using Turbopack patterns.
    fn extract_turbopack_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.turbopack_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    if let Some(normalized) = self.extract_package_from_path(pkg_match.as_str()) {
                        trace!("Turbopack pattern matched: {}", normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }
    }

    /// Extract packages using esbuild patterns.
    fn extract_esbuild_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.esbuild_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    let raw = pkg_match.as_str();

                    // Handle esbuild's naming: require_lodash -> lodash
                    let name = if raw.starts_with("require_") || raw.starts_with("init_") {
                        self.convert_esbuild_name(&raw[raw.find('_').unwrap() + 1..])
                    } else {
                        raw.to_string()
                    };

                    if let Some(normalized) = self.extract_package_from_path(&name) {
                        trace!("esbuild pattern matched: {} -> {}", raw, normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }
    }

    /// Convert esbuild naming conventions back to package names.
    fn convert_esbuild_name(&self, name: &str) -> String {
        // esbuild converts special chars: @company/utils -> _company_utils
        let mut result = name.to_string();

        // Convert leading underscore to @
        if result.starts_with('_') {
            result = format!("@{}", &result[1..]);
        }

        // Convert remaining underscores to / (only first one for scoped)
        if result.starts_with('@') {
            if let Some(idx) = result[1..].find('_') {
                let before = &result[..idx + 1];
                let after = &result[idx + 2..];
                result = format!("{}/{}", before, after.replace('_', "-"));
            }
        }

        result
    }

    /// Extract packages using SWC patterns.
    fn extract_swc_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.swc_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    if let Some(normalized) = normalize_package_name(pkg_match.as_str()) {
                        trace!("SWC pattern matched: {}", normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::WebpackChunk,
                            source_url: source_url.to_string(),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }
    }

    /// Extract packages using minified code patterns.
    fn extract_minified_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        for pattern in &self.minified_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(pkg_match) = cap.get(1) {
                    if let Some(normalized) = normalize_package_name(pkg_match.as_str()) {
                        trace!("Minified pattern matched: {}", normalized);
                        packages.insert(Package {
                            name: normalized,
                            extraction_method: ExtractionMethod::Require,
                            source_url: source_url.to_string(),
                            confidence: Confidence::Medium, // Lower confidence for minified
                        });
                    }
                }
            }
        }
    }

    /// Extract package name from a file path.
    fn extract_package_from_path(&self, path: &str) -> Option<String> {
        let path = path.strip_prefix("./").unwrap_or(path);

        // Handle scoped packages
        if path.starts_with('@') {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                let name = format!("{}/{}", parts[0], parts[1]);
                return normalize_package_name(&name);
            }
        } else {
            // Regular package
            let pkg = path.split('/').next()?;
            return normalize_package_name(pkg);
        }

        None
    }
}

impl Default for BundlerParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of bundlers that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundlerType {
    Vite,
    Rollup,
    Parcel,
    Turbopack,
    Esbuild,
    Swc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_turbopack() {
        let parser = BundlerParser::new();
        let content = r#"__turbopack_require__("[project]/node_modules/lodash/index.js")"#;
        assert_eq!(parser.detect_bundler(content), Some(BundlerType::Turbopack));
    }

    #[test]
    fn test_detect_vite() {
        let parser = BundlerParser::new();
        let content = r#"import { x } from "/node_modules/.vite/deps/lodash.js""#;
        assert_eq!(parser.detect_bundler(content), Some(BundlerType::Vite));
    }

    #[test]
    fn test_detect_parcel() {
        let parser = BundlerParser::new();
        let content = r#"parcelRequire("lodash")"#;
        assert_eq!(parser.detect_bundler(content), Some(BundlerType::Parcel));
    }

    #[test]
    fn test_detect_esbuild() {
        let parser = BundlerParser::new();
        let content = r#"var lodash = __require("lodash")"#;
        assert_eq!(parser.detect_bundler(content), Some(BundlerType::Esbuild));
    }

    #[test]
    fn test_extract_turbopack_packages() {
        let parser = BundlerParser::new();
        let content = r#"
            __turbopack_require__("[project]/node_modules/@company/utils/index.js");
            __turbopack_import__("lodash");
        "#;
        let packages = parser.extract_packages(content, "test.js");
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"@company/utils"));
        assert!(names.contains(&"lodash"));
    }

    #[test]
    fn test_extract_esbuild_packages() {
        let parser = BundlerParser::new();
        let content = r#"
            __commonJS({ "node_modules/lodash/index.js"(exports) {} });
            __esm({ "node_modules/@company/utils/src/index.js"() {} });
        "#;
        let packages = parser.extract_packages(content, "test.js");
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"@company/utils"));
    }

    #[test]
    fn test_extract_parcel_packages() {
        let parser = BundlerParser::new();
        let content = r#"
            parcelRequire("@company/sdk");
            $parcel$require("lodash");
        "#;
        let packages = parser.extract_packages(content, "test.js");
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"@company/sdk"));
        assert!(names.contains(&"lodash"));
    }

    #[test]
    fn test_extract_minified_packages() {
        let parser = BundlerParser::new();
        let content = r#"
            var a=require("lodash");
            let b=require("@company/utils");
            module.exports=require("express");
        "#;
        let packages = parser.extract_packages(content, "test.js");
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"@company/utils"));
        assert!(names.contains(&"express"));
    }

    #[test]
    fn test_convert_vite_name() {
        let parser = BundlerParser::new();

        assert_eq!(parser.convert_vite_name("@company_utils.js"), "@company/utils");
        assert_eq!(parser.convert_vite_name("lodash.js"), "lodash");
    }
}
