//! Package extraction parsers.
//!
//! This module handles extracting package names from:
//! - JavaScript AST (require/import statements)
//! - Source maps (sources array paths)
//! - Webpack chunk manifests
//! - Vite, Parcel, Turbopack, esbuild bundler patterns
//! - Obfuscated/encoded strings (base64, hex, unicode, fromCharCode)

pub mod ast_parser;
pub mod bundlers;
pub mod deobfuscate;
pub mod filters;
pub mod sourcemap;
pub mod webpack;

pub use ast_parser::AstParser;
pub use bundlers::BundlerParser;
pub use deobfuscate::Deobfuscator;
pub use filters::should_filter_package;
pub use sourcemap::SourceMapParser;
pub use webpack::WebpackParser;

/// Normalize a package name (handle scoped packages, strip versions, etc.).
pub fn normalize_package_name(name: &str) -> Option<String> {
    let trimmed = name.trim();

    // Skip empty names
    if trimmed.is_empty() {
        return None;
    }

    // Skip relative imports
    if trimmed.starts_with('.') || trimmed.starts_with('/') {
        return None;
    }

    // Skip node built-ins
    if is_node_builtin(trimmed) {
        return None;
    }

    // Handle scoped packages (@scope/package)
    if trimmed.starts_with('@') {
        // Must have a scope and package name
        let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
        if parts.len() >= 2 {
            // @scope/package or @scope/package/subpath
            let scope = parts[0];
            let package = parts[1];

            // Validate scope name
            if !is_valid_scope(scope) {
                return None;
            }

            // Validate package name
            if !is_valid_package_name(package) {
                return None;
            }

            return Some(format!("{}/{}", scope, package));
        }
        return None;
    }

    // Regular package - extract just the package name (first segment)
    let package_name = trimmed.split('/').next()?;

    if !is_valid_package_name(package_name) {
        return None;
    }

    Some(package_name.to_string())
}

/// Check if a name is a Node.js built-in module.
fn is_node_builtin(name: &str) -> bool {
    const BUILTINS: &[&str] = &[
        "assert",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "http2",
        "https",
        "inspector",
        "module",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "sys",
        "timers",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "wasi",
        "worker_threads",
        "zlib",
    ];

    let base = name.strip_prefix("node:").unwrap_or(name);
    BUILTINS.contains(&base)
}

/// Validate a package name according to npm rules.
fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 214 {
        return false;
    }

    // Must not start with . or _
    if name.starts_with('.') || name.starts_with('_') {
        return false;
    }

    // Must be lowercase and URL-safe
    let valid_chars = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.');

    valid_chars
}

/// Validate a scope name (including the @).
fn is_valid_scope(scope: &str) -> bool {
    if !scope.starts_with('@') {
        return false;
    }

    let name = &scope[1..];
    if name.is_empty() || name.len() > 214 {
        return false;
    }

    // Must be lowercase and URL-safe
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Check if a package name looks like an internal/private package.
/// Check if a package name is likely a false positive (not a real package).
pub fn is_likely_false_positive(name: &str) -> bool {
    // Design system component patterns (responsive breakpoint sizes)
    // Pattern: @scope/component-{xs|sm|md|lg|xl}
    // Example: @allocation-list/asset-list-xs, @asset-list/holding-list-md
    // These are NOT npm packages - they're internal component identifiers
    // CHECK THIS BEFORE the general scoped package early return!
    if name.starts_with('@') && (name.ends_with("-xs") || name.ends_with("-sm")
        || name.ends_with("-md") || name.ends_with("-lg") || name.ends_with("-xl")) {
        // Strong indicator: scope name ends with -list and package name also contains -list
        if name.contains("-list/") && name.contains("-list-") {
            return true;
        }
    }

    // Skip scoped packages (they're more reliable)
    if name.starts_with('@') {
        return false;
    }

    // Very short names (1-2 chars) are usually variables, not packages
    if name.len() <= 2 {
        return true;
    }

    // Names ending with _id or _ID (webpack/bundler identifiers)
    if name.ends_with("_id") || name.ends_with("_ID") || name.ends_with("Id") {
        return true;
    }

    // Common webpack/bundler artifacts
    let bundler_artifacts = [
        "template_id",
        "chunk_id",
        "module_id",
        "webpack_require",
        "webpackChunk",
        "installedModules",
        "installedChunks",
        "__webpack",
        "list-v",
        "list-a",
        "list-b",
        "list-c",
        "list-d",
        "list-e",
    ];

    for artifact in bundler_artifacts {
        if name == artifact || name.starts_with(artifact) {
            return true;
        }
    }

    // Very short hyphenated names (likely webpack list identifiers: list-v, item-x, etc.)
    // Pattern: word-letter where total length <= 8 and ends with single letter
    if name.len() <= 8 && name.contains('-') {
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() == 2 {
            let last_part = parts[1];
            // If last part is a single letter or very short (1-2 chars), likely an artifact
            if last_part.len() <= 2 && last_part.chars().all(|c| c.is_ascii_alphabetic()) {
                return true;
            }
        }
    }

    // Very generic single-word names that are likely variables
    let generic_names = [
        "id", "key", "value", "data", "config", "options", "params",
        "result", "response", "request", "error", "callback",
    ];

    if generic_names.contains(&name) {
        return true;
    }

    // Hex hashes from obfuscated code (e.g., cce448c, 806d289, 02cd8bbf69bb5ae8)
    // Any string 6+ chars that is only hex digits with mixed letters and numbers
    if name.len() >= 6 && name.chars().all(|c| c.is_ascii_hexdigit()) {
        let has_letter = name.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = name.chars().any(|c| c.is_ascii_digit());

        // If mixed hex (has both letters and numbers), likely a hash
        if has_letter && has_digit {
            return true;
        }
    }

    // JavaScript built-in objects, properties, and methods
    // These exist as npm packages but are almost always false positives
    // when extracted from obfuscated code
    let js_builtins = [
        "constructor", "prototype", "object", "function", "array", "string",
        "number", "boolean", "symbol", "undefined", "null",
        "keys", "values", "entries", "length", "name", "apply", "call",
        "bind", "create", "define", "freeze", "seal", "assign",
        "hasownproperty", "tostring", "valueof", "getprototypeof",
        "isprototypeof", "propertyisenumerable",
    ];

    // Check lowercase version for case-insensitive matching
    let name_lower = name.to_lowercase();
    if js_builtins.contains(&name_lower.as_str()) {
        return true;
    }

    // Very short names from deobfuscation (3-4 chars)
    // These are usually variable names or obfuscation artifacts, not real packages
    if name.len() >= 3 && name.len() <= 4 {
        // All lowercase, or mixed alphanumeric (e.g., xt1, b558, g3ec)
        let all_lowercase = name.chars().all(|c| c.is_ascii_lowercase());
        let has_digit = name.chars().any(|c| c.is_ascii_digit());
        let all_alphanumeric = name.chars().all(|c| c.is_ascii_alphanumeric());

        if all_lowercase || (has_digit && all_alphanumeric) {
            return true;
        }
    }

    // WebpackChunk-specific patterns (internal webpack module names)
    let webpack_suffixes = ["-handler", "-tgl", "-btn", "-grp", "-chkbox"];
    for suffix in webpack_suffixes {
        if name.ends_with(suffix) {
            return true;
        }
    }

    // WebpackChunk patterns (internal names)
    let webpack_patterns = [
        "consent-", "opt-out-", "privacy-", "purpose-", "feature-",
        "checkbox-", "legclaim-", "spl-", "header-id", "leg-",
        "close-pc-", "list-save-", "search-", "groups-", "option-",
        "cookie-", "label-", "purposes-", "header-container",
        "portal-", "uw-", // DigitalOcean/UserWay internal modules
    ];
    for pattern in webpack_patterns {
        if name.starts_with(pattern) || name.contains(pattern) {
            return true;
        }
    }

    // Brand names and specific false positives
    let brand_names = ["rakbank"]; // Banking/company brands unlikely to be npm packages
    if brand_names.contains(&name_lower.as_str()) {
        return true;
    }

    // Webpack hashed module names (e.g., react-d494828cb1d95eaa, design-system-f4677b5ea6850f41)
    // Pattern: name-<long hex hash> or name-name-<long hex hash>
    if name.contains('-') {
        let parts: Vec<&str> = name.split('-').collect();
        if let Some(last_part) = parts.last() {
            // If last part is a long hex string (12+ chars), likely a webpack hash
            if last_part.len() >= 12 && last_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return true;
            }
        }
    }

    // Domain names (contain common TLDs)
    // These are often extracted from obfuscated code but are NOT npm packages
    let tlds = [
        ".com", ".org", ".net", ".io", ".co", ".me", ".ai", ".dev", ".app",
        ".edu", ".gov", ".mil", ".int", ".biz", ".info", ".name", ".pro",
        ".ae", ".uk", ".ca", ".au", ".de", ".fr", ".jp", ".cn", ".in",
        ".br", ".ru", ".it", ".es", ".nl", ".se", ".no", ".dk", ".fi",
        ".gr", ".si", ".la", ".be", ".ch", ".at",
    ];
    for tld in tlds {
        if name.contains(tld) {
            return true;
        }
    }

    // More common variable/property names
    let more_generic_names = [
        "initialized", "loaded", "ready", "active", "enabled", "disabled",
        "visible", "hidden", "selected", "focused", "checked", "valid",
    ];
    if more_generic_names.contains(&name_lower.as_str()) {
        return true;
    }

    // DOM event names — extracted by deobfuscator from event handlers, never real packages
    let dom_events = [
        "mousedown", "mouseup", "mousemove", "mouseover", "mouseout", "mouseenter",
        "mouseleave", "touchstart", "touchend", "touchmove", "touchcancel",
        "keydown", "keyup", "keypress", "beforeunload", "visibilitychange",
        "readystatechange", "onmessage", "ontouchend", "ontouchstart",
        "pointerdown", "pointerup", "pointermove", "contextmenu",
        "focusin", "focusout", "compositionstart", "compositionend",
    ];
    if dom_events.contains(&name_lower.as_str()) {
        return true;
    }

    // Referrer policy values and other web API constants
    let web_constants = [
        "unsafe-url", "no-referrer", "same-origin", "strict-origin",
        "evenodd", "alphabetic", "experimental-webgl",
    ];
    if web_constants.contains(&name) {
        return true;
    }

    false
}

pub fn is_likely_internal(name: &str) -> bool {
    // Scoped packages with company-like scopes
    if name.starts_with('@') {
        let scope = name.split('/').next().unwrap_or("");
        // Common indicators of internal packages
        let internal_indicators = [
            "internal",
            "private",
            "corp",
            "company",
            "team",
            "org",
            "enterprise",
        ];

        for indicator in internal_indicators {
            if scope.contains(indicator) {
                return true;
            }
        }
    }

    // Package names with internal indicators
    let internal_indicators = [
        "internal",
        "private",
        "-internal",
        "-private",
        "_internal",
        "_private",
    ];

    internal_indicators.iter().any(|ind| name.contains(ind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_package_name() {
        assert_eq!(normalize_package_name("lodash"), Some("lodash".to_string()));
        assert_eq!(
            normalize_package_name("@company/utils"),
            Some("@company/utils".to_string())
        );
        assert_eq!(
            normalize_package_name("@scope/pkg/subpath"),
            Some("@scope/pkg".to_string())
        );
        assert_eq!(
            normalize_package_name("lodash/fp"),
            Some("lodash".to_string())
        );
        assert_eq!(normalize_package_name("./local"), None);
        assert_eq!(normalize_package_name("fs"), None); // Node built-in
        assert_eq!(normalize_package_name("node:fs"), None);
    }

    #[test]
    fn test_is_valid_package_name() {
        assert!(is_valid_package_name("lodash"));
        assert!(is_valid_package_name("my-package"));
        assert!(is_valid_package_name("my_package"));
        assert!(is_valid_package_name("package123"));
        assert!(!is_valid_package_name(".hidden"));
        assert!(!is_valid_package_name("_private"));
        assert!(!is_valid_package_name("UPPERCASE")); // Must be lowercase
    }

    #[test]
    fn test_is_likely_internal() {
        assert!(is_likely_internal("@company-internal/utils"));
        assert!(is_likely_internal("@private/auth"));
        assert!(is_likely_internal("my-internal-lib"));
        assert!(!is_likely_internal("lodash"));
        assert!(!is_likely_internal("@angular/core"));
    }
}
