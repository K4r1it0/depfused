//! String deobfuscation for detecting encoded package names.
//!
//! Handles common obfuscation patterns:
//! - Base64 encoding (atob, Buffer.from)
//! - Hex encoding (\x6c\x6f\x64\x61\x73\x68)
//! - Unicode escapes (\u006c\u006f\u0064\u0061\u0073\u0068)
//! - String.fromCharCode(108,111,100,97,115,104)
//! - Array.join patterns: ["l","o","d","a","s","h"].join("")

use crate::parser::{filters, normalize_package_name};
use crate::types::{Confidence, ExtractionMethod, Package};
use base64::{engine::general_purpose::STANDARD, Engine};
use regex::Regex;
use std::collections::HashSet;
use tracing::{debug, trace};

/// Deobfuscator for extracting package names from obfuscated strings.
#[derive(Clone)]
pub struct Deobfuscator {
    base64_patterns: Vec<Regex>,
    hex_patterns: Vec<Regex>,
    unicode_patterns: Vec<Regex>,
    from_char_code_patterns: Vec<Regex>,
    array_join_patterns: Vec<Regex>,
}

impl Deobfuscator {
    /// Create a new deobfuscator with pre-compiled patterns.
    pub fn new() -> Self {
        Self {
            base64_patterns: vec![
                Regex::new(r#"require\s*\(\s*atob\s*\(\s*["']([A-Za-z0-9+/=]+)["']\s*\)"#).unwrap(),
                Regex::new(r#"import\s*\(\s*atob\s*\(\s*["']([A-Za-z0-9+/=]+)["']\s*\)"#).unwrap(),
                Regex::new(r#"(?:window\.)?atob\s*\(\s*["']([A-Za-z0-9+/=]+)["']\s*\)"#).unwrap(),
                Regex::new(r#"Buffer\.from\s*\(\s*["']([A-Za-z0-9+/=]+)["']\s*,\s*["']base64["']\s*\)"#).unwrap(),
            ],
            hex_patterns: vec![
                Regex::new(r#"["']((?:\\x[0-9a-fA-F]{2})+)["']"#).unwrap(),
                Regex::new(r#"require\s*\(\s*["']((?:\\x[0-9a-fA-F]{2})+)["']\s*\)"#).unwrap(),
            ],
            unicode_patterns: vec![
                Regex::new(r#"["']((?:\\u[0-9a-fA-F]{4})+)["']"#).unwrap(),
                Regex::new(r#"require\s*\(\s*["']((?:\\u[0-9a-fA-F]{4})+)["']\s*\)"#).unwrap(),
            ],
            from_char_code_patterns: vec![
                Regex::new(r#"String\.fromCharCode\s*\(\s*([\d,\s]+)\s*\)"#).unwrap(),
                Regex::new(r#"require\s*\(\s*String\.fromCharCode\s*\(\s*([\d,\s]+)\s*\)\s*\)"#).unwrap(),
                Regex::new(r#"String\s*\[\s*["']fromCharCode["']\s*\]\s*\(\s*([\d,\s]+)\s*\)"#).unwrap(),
            ],
            array_join_patterns: vec![
                Regex::new(r#"\[\s*((?:["'][^"']*["']\s*,?\s*)+)\]\s*\.join\s*\(\s*["']['"]?\s*\)"#).unwrap(),
                Regex::new(r#"require\s*\(\s*\[\s*((?:["'][^"']*["']\s*,?\s*)+)\]\s*\.join"#).unwrap(),
            ],
        }
    }

    /// Extract packages from potentially obfuscated content.
    pub fn extract_packages(&self, content: &str, source_url: &str) -> Vec<Package> {
        let mut packages = HashSet::new();

        // Try each decoding method
        self.extract_with_decoder(content, source_url, &self.base64_patterns, Self::decode_base64, &mut packages);
        self.extract_with_decoder(content, source_url, &self.hex_patterns, Self::decode_hex, &mut packages);
        self.extract_with_decoder(content, source_url, &self.unicode_patterns, Self::decode_unicode, &mut packages);
        self.extract_with_decoder(content, source_url, &self.from_char_code_patterns, Self::decode_char_codes, &mut packages);
        self.extract_with_decoder(content, source_url, &self.array_join_patterns, Self::decode_array_join, &mut packages);

        // String concatenation patterns
        self.extract_concat_packages(content, source_url, &mut packages);

        let result: Vec<Package> = packages.into_iter().collect();
        if !result.is_empty() {
            debug!("Extracted {} packages from deobfuscation: {}", result.len(), source_url);
        }
        result
    }

    /// Generic extraction using patterns and a decoder function.
    fn extract_with_decoder(
        &self,
        content: &str,
        source_url: &str,
        patterns: &[Regex],
        decoder: fn(&str) -> Option<String>,
        packages: &mut HashSet<Package>,
    ) {
        for pattern in patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(encoded) = cap.get(1) {
                    if let Some(decoded) = decoder(encoded.as_str()) {
                        if let Some(normalized) = normalize_package_name(&decoded) {
                            // Apply filters - deobfuscate has very high FP rate
                            if filters::should_filter_package(&normalized, Some(content), Some(source_url)) {
                                trace!("Filtered deobfuscated package: {}", normalized);
                                continue;
                            }

                            trace!("Deobfuscated: {} -> {}", encoded.as_str(), normalized);
                            packages.insert(Package {
                                name: normalized,
                                extraction_method: ExtractionMethod::Deobfuscate,
                                source_url: source_url.to_string(),
                                confidence: Confidence::Low,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Decode base64 string.
    fn decode_base64(encoded: &str) -> Option<String> {
        STANDARD.decode(encoded).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// Decode hex escaped string (\x6c\x6f -> "lo").
    fn decode_hex(encoded: &str) -> Option<String> {
        let hex_re = Regex::new(r"\\x([0-9a-fA-F]{2})").ok()?;
        let result: String = hex_re.captures_iter(encoded)
            .filter_map(|cap| cap.get(1))
            .filter_map(|m| u8::from_str_radix(m.as_str(), 16).ok())
            .map(|b| b as char)
            .collect();

        if result.is_empty() { None } else { Some(result) }
    }

    /// Decode unicode escape sequences (\u006c -> "l").
    fn decode_unicode(encoded: &str) -> Option<String> {
        let unicode_re = Regex::new(r"\\u([0-9a-fA-F]{4})").ok()?;
        let result: String = unicode_re.captures_iter(encoded)
            .filter_map(|cap| cap.get(1))
            .filter_map(|m| u32::from_str_radix(m.as_str(), 16).ok())
            .filter_map(char::from_u32)
            .collect();

        if result.is_empty() { None } else { Some(result) }
    }

    /// Decode String.fromCharCode arguments.
    fn decode_char_codes(codes_str: &str) -> Option<String> {
        let result: String = codes_str.split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .filter_map(char::from_u32)
            .collect();

        if result.is_empty() { None } else { Some(result) }
    }

    /// Decode array join pattern: ["l","o","d"] -> "lod".
    fn decode_array_join(array_str: &str) -> Option<String> {
        let element_re = Regex::new(r#"["']([^"']*)["']"#).ok()?;
        let result: String = element_re.captures_iter(array_str)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect();

        if result.is_empty() { None } else { Some(result) }
    }

    /// Extract packages from string concatenation patterns.
    fn extract_concat_packages(&self, content: &str, source_url: &str, packages: &mut HashSet<Package>) {
        // Scoped package concatenation: "@" + "company" + "/" + "utils"
        let scoped_re = Regex::new(
            r#"["']@["']\s*\+\s*["']([\w-]+)["']\s*\+\s*["']/["']\s*\+\s*["']([\w.-]+)["']"#
        ).unwrap();

        for cap in scoped_re.captures_iter(content) {
            if let (Some(scope), Some(name)) = (cap.get(1), cap.get(2)) {
                let full_name = format!("@{}/{}", scope.as_str(), name.as_str());
                if let Some(normalized) = normalize_package_name(&full_name) {
                    trace!("Concat decoded: {}", normalized);
                    packages.insert(Package {
                        name: normalized,
                        extraction_method: ExtractionMethod::Deobfuscate,
                        source_url: source_url.to_string(),
                        confidence: Confidence::Low,
                    });
                }
            }
        }
    }

    /// Check if content appears to be obfuscated.
    pub fn is_likely_obfuscated(&self, content: &str) -> bool {
        let indicators = [
            r#"\\x[0-9a-fA-F]{2}"#,
            r#"\\u[0-9a-fA-F]{4}"#,
            r#"String\.fromCharCode"#,
            r#"\["fromCharCode"\]"#,
            r#"atob\s*\("#,
            r#"\.split\s*\(\s*["']["']\s*\)\.reverse"#,
            r#"eval\s*\("#,
            r#"Function\s*\("#,
        ];

        let score: usize = indicators.iter()
            .filter(|&pattern| Regex::new(pattern).map(|re| re.is_match(content)).unwrap_or(false))
            .count();

        // Also check for excessive single-letter variables (minified code)
        let short_var_count = Regex::new(r"\b[a-z]\s*=")
            .map(|re| re.find_iter(content).count())
            .unwrap_or(0);

        score >= 2 || (score >= 1 && short_var_count > 50)
    }
}

impl Default for Deobfuscator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64() {
        assert_eq!(Deobfuscator::decode_base64("bG9kYXNo"), Some("lodash".to_string()));
        assert_eq!(Deobfuscator::decode_base64("QGNvbXBhbnkvdXRpbHM="), Some("@company/utils".to_string()));
    }

    #[test]
    fn test_decode_hex() {
        let hex = r"\x6c\x6f\x64\x61\x73\x68";
        assert_eq!(Deobfuscator::decode_hex(hex), Some("lodash".to_string()));
    }

    #[test]
    fn test_decode_unicode() {
        let unicode = r"\u006c\u006f\u0064\u0061\u0073\u0068";
        assert_eq!(Deobfuscator::decode_unicode(unicode), Some("lodash".to_string()));
    }

    #[test]
    fn test_decode_from_char_code() {
        assert_eq!(
            Deobfuscator::decode_char_codes("108,111,100,97,115,104"),
            Some("lodash".to_string())
        );
    }

    #[test]
    fn test_decode_array_join() {
        let array = r#""l","o","d","a","s","h""#;
        assert_eq!(Deobfuscator::decode_array_join(array), Some("lodash".to_string()));
    }

    #[test]
    fn test_extract_atob_packages() {
        let deob = Deobfuscator::new();
        let content = r#"var pkg = atob("bG9kYXNo"); require(pkg);"#;
        let packages = deob.extract_packages(content, "test.js");
        assert!(packages.iter().any(|p| p.name == "lodash"));
    }

    #[test]
    fn test_extract_from_char_code_packages() {
        let deob = Deobfuscator::new();
        let content = r#"require(String.fromCharCode(108,111,100,97,115,104));"#;
        let packages = deob.extract_packages(content, "test.js");
        assert!(packages.iter().any(|p| p.name == "lodash"));
    }

    #[test]
    fn test_is_likely_obfuscated() {
        let deob = Deobfuscator::new();

        let obfuscated = r#"
            var a = String.fromCharCode(108,111,100);
            var b = atob("YXNo");
            eval(a + b);
        "#;
        assert!(deob.is_likely_obfuscated(obfuscated));

        let normal = r#"
            import lodash from 'lodash';
            const result = lodash.map([1,2,3], x => x * 2);
        "#;
        assert!(!deob.is_likely_obfuscated(normal));
    }
}
