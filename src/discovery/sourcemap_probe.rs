//! Source map probing - tries to find .map files even when not referenced.

use crate::config::get_sourcemap_variations;
use crate::types::Result;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, trace};

/// Prober for discovering source map files.
#[derive(Clone)]
pub struct SourceMapProber {
    client: Client,
}

impl SourceMapProber {
    /// Create a new source map prober.
    pub fn new(timeout_secs: u64, user_agent: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent)
            .http1_only() // Force HTTP/1.1 to avoid HTTP/2 stream limit issues
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client })
    }

    /// Try to find a source map for a given JS file URL.
    pub async fn probe(&self, js_url: &str) -> Option<(String, String)> {
        let variations = get_sourcemap_variations(js_url);

        for map_url in variations {
            if let Some(content) = self.try_fetch_map(&map_url).await {
                debug!("Found source map at: {}", map_url);
                return Some((map_url, content));
            }
        }

        None
    }

    /// Try to fetch a source map URL.
    async fn try_fetch_map(&self, url: &str) -> Option<String> {
        trace!("Probing source map: {}", url);

        let response = match self.client.get(url).send().await {
            Ok(r) => r,
            Err(_) => return None,
        };

        if !response.status().is_success() {
            return None;
        }

        // Check content type
        if let Some(content_type) = response.headers().get("content-type") {
            if let Ok(ct) = content_type.to_str() {
                // Accept JSON or source map content types
                if !ct.contains("json")
                    && !ct.contains("sourcemap")
                    && !ct.contains("text/plain")
                    && !ct.contains("application/octet-stream")
                {
                    return None;
                }
            }
        }

        let content = match response.text().await {
            Ok(c) => c,
            Err(_) => return None,
        };

        // Verify it looks like a source map
        if self.is_valid_sourcemap(&content) {
            Some(content)
        } else {
            None
        }
    }

    /// Check if content looks like a valid source map.
    fn is_valid_sourcemap(&self, content: &str) -> bool {
        // Quick check for JSON structure with source map fields
        let trimmed = content.trim();

        if !trimmed.starts_with('{') {
            return false;
        }

        // Must have "version" field (required by spec)
        if !content.contains("\"version\"") {
            return false;
        }

        // Should have either "sources" or "mappings"
        content.contains("\"sources\"") || content.contains("\"mappings\"")
    }

    /// Decode an inline base64 source map.
    pub fn decode_inline_sourcemap(data_url: &str) -> Option<String> {
        // Format: data:application/json;base64,<base64-data>
        // or: data:application/json;charset=utf-8;base64,<base64-data>

        if !data_url.starts_with("data:") {
            return None;
        }

        // Find the base64 data
        let base64_marker = ";base64,";
        let base64_start = data_url.find(base64_marker)?;
        let base64_data = &data_url[base64_start + base64_marker.len()..];

        // Decode using base64 0.21 API
        use base64::{engine::general_purpose::STANDARD, Engine};
        let decoded = STANDARD.decode(base64_data).ok()?;

        String::from_utf8(decoded).ok()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_sourcemap() {
        let prober = SourceMapProber::new(30, "test").unwrap();

        let valid = r#"{"version":3,"sources":["src/main.js"],"mappings":"AAAA"}"#;
        assert!(prober.is_valid_sourcemap(valid));

        let invalid = r#"{"name":"not a sourcemap"}"#;
        assert!(!prober.is_valid_sourcemap(invalid));

        let html = "<!DOCTYPE html>";
        assert!(!prober.is_valid_sourcemap(html));
    }

    #[test]
    fn test_decode_inline_sourcemap() {
        // Base64 encoded: {"version":3}
        let data_url = "data:application/json;base64,eyJ2ZXJzaW9uIjozfQ==";
        let decoded = SourceMapProber::decode_inline_sourcemap(data_url);
        assert_eq!(decoded, Some(r#"{"version":3}"#.to_string()));
    }
}
