//! JavaScript file fetcher with retry support.

use crate::types::{ContentHashSet, HttpConfig, JsFile, JsSource, Result};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

/// Fetcher for JavaScript files with rate limiting and deduplication.
pub struct JsFetcher {
    client: Client,
    config: HttpConfig,
    rate_limiter: Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
    seen_hashes: Arc<RwLock<ContentHashSet>>,
}

impl JsFetcher {
    /// Create a new JS fetcher.
    pub fn new(config: HttpConfig, rate_limit: u32) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(5))
            .http1_only()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;

        let quota = Quota::per_second(NonZeroU32::new(rate_limit).unwrap_or(NonZeroU32::new(10).unwrap()));
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            client,
            config,
            rate_limiter,
            seen_hashes: Arc::new(RwLock::new(ContentHashSet::new())),
        })
    }

    /// Fetch a single JS file with retries.
    pub async fn fetch_one(&self, url: &str, source: JsSource) -> Option<JsFile> {
        // Rate limit
        self.rate_limiter.until_ready().await;

        let mut retries = 0;
        let mut last_error = None;

        while retries <= self.config.max_retries {
            match self.do_fetch(url).await {
                Ok(content) => {
                    // Calculate content hash
                    let hash = Self::hash_content(&content);

                    // Check for duplicate content
                    {
                        let seen = self.seen_hashes.read().await;
                        if seen.contains(&hash) {
                            trace!("Skipping duplicate content: {}", url);
                            return None;
                        }
                    }

                    // Add to seen hashes
                    {
                        let mut seen = self.seen_hashes.write().await;
                        seen.insert(hash.clone());
                    }

                    // Check for source map reference
                    let source_map_url = self.extract_sourcemap_url(&content, url);

                    debug!("Fetched JS file: {} ({} bytes)", url, content.len());

                    return Some(JsFile {
                        url: url.to_string(),
                        content,
                        content_hash: hash,
                        source,
                        source_map_url,
                    });
                }
                Err(e) => {
                    // Check if this is a client error (4xx) that shouldn't be retried
                    let should_retry = if let crate::types::DepfusedError::HttpError(ref http_err) = e {
                        if let Some(status) = http_err.status() {
                            // Don't retry on 4xx client errors (404, 403, 401, etc.)
                            // These won't succeed on retry
                            !status.is_client_error()
                        } else {
                            true // Retry on network errors without status
                        }
                    } else {
                        true // Retry on other error types
                    };

                    last_error = Some(e);

                    if !should_retry {
                        // Fail fast on client errors (404, 403, etc.)
                        debug!("Not retrying {} - client error", url);
                        break;
                    }

                    retries += 1;
                    if retries <= self.config.max_retries {
                        trace!("Retry {} for {}", retries, url);
                        tokio::time::sleep(Duration::from_millis(500 * retries as u64)).await;
                    }
                }
            }
        }

        if let Some(e) = last_error {
            if retries > 1 {
                warn!("Failed to fetch {} after {} retries: {}", url, retries - 1, e);
            } else {
                // Don't mention retries if we didn't actually retry
                debug!("Failed to fetch {}: {}", url, e);
            }
        }

        None
    }

    /// Perform the actual HTTP fetch.
    async fn do_fetch(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(crate::types::DepfusedError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let content = response.text().await?;
        Ok(content)
    }

    /// Extract source map URL from JS content (delegates to standalone function).
    fn extract_sourcemap_url(&self, content: &str, base_url: &str) -> Option<String> {
        extract_sourcemap_url(content, base_url)
    }

    /// Calculate SHA256 hash of content.
    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

}

/// Extract source map URL from JS content (standalone, reusable).
pub fn extract_sourcemap_url(content: &str, base_url: &str) -> Option<String> {
    // Check for sourceMappingURL comment
    // //# sourceMappingURL=...
    // or //@ sourceMappingURL=... (deprecated)
    let patterns = [
        r"//[#@]\s*sourceMappingURL\s*=\s*(\S+)",
        r"/\*[#@]\s*sourceMappingURL\s*=\s*(\S+?)\s*\*/",
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(url_match) = caps.get(1) {
                    let map_url = url_match.as_str().trim();

                    // Handle inline base64 source maps
                    if map_url.starts_with("data:") {
                        return Some(map_url.to_string());
                    }

                    // Resolve relative URL
                    if map_url.starts_with("http://") || map_url.starts_with("https://") {
                        return Some(map_url.to_string());
                    }

                    // Resolve relative to base URL
                    if let Ok(base) = url::Url::parse(base_url) {
                        if let Ok(resolved) = base.join(map_url) {
                            return Some(resolved.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = "console.log('hello');";
        let hash = JsFetcher::hash_content(content);
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_extract_sourcemap_url() {
        let config = HttpConfig::default();
        let fetcher = JsFetcher::new(config, 10).unwrap();

        let content = r#"
            console.log('test');
            //# sourceMappingURL=main.js.map
        "#;

        let result = fetcher.extract_sourcemap_url(content, "https://example.com/js/main.js");
        assert_eq!(
            result,
            Some("https://example.com/js/main.js.map".to_string())
        );
    }
}
