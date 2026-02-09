//! npm registry checker for verifying package existence.

use crate::registry::cache::RegistryCache;
use crate::types::{NpmCheckResult, Package, Result};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::Deserialize;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, trace, warn};

/// npm registry API response for package info.
#[derive(Debug, Deserialize)]
struct NpmPackageInfo {
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "dist-tags")]
    dist_tags: Option<DistTags>,
}

#[derive(Debug, Deserialize)]
struct DistTags {
    latest: Option<String>,
}

/// npm registry API response for scoped search.
#[derive(Debug, Deserialize)]
struct NpmSearchResponse {
    objects: Vec<NpmSearchObject>,
}

#[derive(Debug, Deserialize)]
struct NpmSearchObject {
    #[allow(dead_code)]
    package: NpmSearchPackage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NpmSearchPackage {
    name: String,
    scope: Option<String>,
}

/// Checker for verifying packages against npm registry.
pub struct NpmChecker {
    client: Client,
    cache: RegistryCache,
    rate_limiter: Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
    registry_url: String,
}

impl NpmChecker {
    /// Create a new npm checker.
    pub fn new(timeout_secs: u64, rate_limit: u32, cache_ttl_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("depfused/0.1")
            .http1_only() // Force HTTP/1.1 to avoid HTTP/2 stream limit issues
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;

        let quota = Quota::per_second(NonZeroU32::new(rate_limit).unwrap_or(NonZeroU32::new(5).unwrap()));
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            client,
            cache: RegistryCache::new(cache_ttl_secs),
            rate_limiter,
            registry_url: "https://registry.npmjs.org".to_string(),
        })
    }

    /// Check if a package exists on npm.
    pub async fn check_package(&self, package: &Package) -> NpmCheckResult {
        // Check cache first
        if let Some(cached) = self.cache.get(&package.name) {
            trace!("Cache hit for {}", package.name);
            return cached;
        }

        // Rate limit
        self.rate_limiter.until_ready().await;

        let result = self.do_check(&package.name).await;

        // Cache the result
        self.cache.set(&package.name, result.clone());

        result
    }

    /// Perform the actual npm registry check.
    async fn do_check(&self, package_name: &str) -> NpmCheckResult {
        // Handle scoped packages differently
        if package_name.starts_with('@') {
            return self.check_scoped_package(package_name).await;
        }

        self.check_regular_package(package_name).await
    }

    /// Check a regular (non-scoped) package.
    async fn check_regular_package(&self, package_name: &str) -> NpmCheckResult {
        let url = format!("{}/{}", self.registry_url, urlencoding::encode(package_name));
        trace!("Checking npm: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Package exists
                    match response.json::<NpmPackageInfo>().await {
                        Ok(info) => {
                            debug!("Package exists: {}", package_name);
                            NpmCheckResult::Exists {
                                name: package_name.to_string(),
                                latest_version: info.dist_tags.and_then(|dt| dt.latest),
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse npm response for {}: {}", package_name, e);
                            NpmCheckResult::Exists {
                                name: package_name.to_string(),
                                latest_version: None,
                            }
                        }
                    }
                } else if response.status().as_u16() == 404 {
                    // Package doesn't exist - potential vulnerability
                    debug!("Package NOT FOUND: {}", package_name);
                    NpmCheckResult::NotFound {
                        name: package_name.to_string(),
                    }
                } else {
                    NpmCheckResult::Error {
                        name: package_name.to_string(),
                        error: format!("HTTP {}", response.status()),
                    }
                }
            }
            Err(e) => NpmCheckResult::Error {
                name: package_name.to_string(),
                error: e.to_string(),
            },
        }
    }

    /// Check a scoped package (@scope/name).
    async fn check_scoped_package(&self, package_name: &str) -> NpmCheckResult {
        // First check if the package itself exists
        let url = format!(
            "{}/{}",
            self.registry_url,
            urlencoding::encode(package_name)
        );
        trace!("Checking scoped npm package: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Package exists
                    match response.json::<NpmPackageInfo>().await {
                        Ok(info) => {
                            debug!("Scoped package exists: {}", package_name);
                            NpmCheckResult::Exists {
                                name: package_name.to_string(),
                                latest_version: info.dist_tags.and_then(|dt| dt.latest),
                            }
                        }
                        Err(_) => NpmCheckResult::Exists {
                            name: package_name.to_string(),
                            latest_version: None,
                        },
                    }
                } else if response.status().as_u16() == 404 {
                    // Package doesn't exist - check if scope is claimed
                    self.check_scope_ownership(package_name).await
                } else {
                    NpmCheckResult::Error {
                        name: package_name.to_string(),
                        error: format!("HTTP {}", response.status()),
                    }
                }
            }
            Err(e) => NpmCheckResult::Error {
                name: package_name.to_string(),
                error: e.to_string(),
            },
        }
    }

    /// Check if a scope is claimed on npm.
    ///
    /// A scope is claimed if ANY of the following are true:
    /// 1. A user with that name exists - checks /-/user/org.couchdb.user:{scope}
    ///    Returns {"ok": true} if user exists
    /// 2. An organization with that name exists - checks /-/org/{scope}/package
    ///    Returns {} (empty object) if org exists with 0 packages
    ///    Returns {"error": "Scope not found"} if org doesn't exist
    /// 3. Packages with that scope exist - searches /-/v1/search
    ///
    /// Only if ALL checks fail is the scope unclaimed and exploitable.
    async fn check_scope_ownership(&self, package_name: &str) -> NpmCheckResult {
        // Extract scope from package name
        let scope = package_name.split('/').next().unwrap_or("");

        if scope.is_empty() || !scope.starts_with('@') {
            return NpmCheckResult::NotFound {
                name: package_name.to_string(),
            };
        }

        let scope_name = &scope[1..]; // Remove @ prefix for user/org checks

        // Check 1: Does a user with this name exist?
        let user_url = format!(
            "https://registry.npmjs.org/-/user/org.couchdb.user:{}",
            urlencoding::encode(scope_name)
        );
        trace!("Checking if user exists: {}", user_url);

        match self.client.get(&user_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Try to parse the response
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Check if user exists: ok=true, name field, or _id field
                            let has_ok = json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                            let has_name = json.get("name").is_some();
                            let has_id = json.get("_id").is_some();

                            if has_ok || has_name || has_id {
                                debug!("Scope claimed by USER '{}': {}", scope_name, package_name);
                                return NpmCheckResult::NotFound {
                                    name: package_name.to_string(),
                                };
                            }
                        }
                    }
                }
            }
            Err(e) => trace!("User check error: {}", e),
        }

        // Check 2: Does an organization with this name exist?
        let org_url = format!(
            "https://registry.npmjs.org/-/org/{}/package",
            urlencoding::encode(scope_name)
        );
        trace!("Checking if org exists: {}", org_url);

        match self.client.get(&org_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Check if response contains an error field
                            if let Some(error) = json.get("error") {
                                // Response like {"error": "Scope not found"} means unclaimed
                                trace!("Org API returned error: {}", error);
                            } else {
                                // No error field means org exists (even if empty object {})
                                // Orgs with 0 packages return {} (empty object)
                                debug!("Scope claimed by ORG '{}': {}", scope_name, package_name);
                                return NpmCheckResult::NotFound {
                                    name: package_name.to_string(),
                                };
                            }
                        }
                    }
                }
            }
            Err(e) => trace!("Org check error: {}", e),
        }

        // Check 3: Are there any packages with this scope?
        // Use size=5 to get enough results to verify scope match
        let search_url = format!(
            "https://registry.npmjs.org/-/v1/search?text={}&size=5",
            urlencoding::encode(scope) // Include the @ prefix
        );
        trace!("Checking for packages in scope: {}", search_url);

        // Build the scope prefix to verify results actually belong to this scope
        let scope_prefix = format!("{}/", scope); // e.g. "@myscope/"

        match self.client.get(&search_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<NpmSearchResponse>().await {
                        Ok(search_result) => {
                            // Verify at least one result actually belongs to this scope
                            // npm search is text-based, so "@internal" can match unrelated packages
                            let has_true_scope_match = search_result.objects.iter().any(|obj| {
                                obj.package.name.starts_with(&scope_prefix)
                            });
                            if has_true_scope_match {
                                debug!("Scope claimed by PACKAGES: {}", package_name);
                                return NpmCheckResult::NotFound {
                                    name: package_name.to_string(),
                                };
                            }
                        }
                        Err(e) => trace!("Search parse error: {}", e),
                    }
                }
            }
            Err(e) => trace!("Search error: {}", e),
        }

        // If we reach here, no user, no org, and no packages found
        // The scope is UNCLAIMED and exploitable!
        debug!("Scope UNCLAIMED (no user, no org, no packages): {}", scope);
        NpmCheckResult::ScopeNotClaimed {
            scope: scope.to_string(),
            name: package_name.to_string(),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, ExtractionMethod};

    fn make_test_package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            extraction_method: ExtractionMethod::Import,
            source_url: "test.js".to_string(),
            confidence: Confidence::High,
        }
    }

    #[tokio::test]
    async fn test_check_existing_package() {
        let checker = NpmChecker::new(10, 5, 60).unwrap();
        let package = make_test_package("lodash");

        let result = checker.check_package(&package).await;

        match result {
            NpmCheckResult::Exists { name, .. } => {
                assert_eq!(name, "lodash");
            }
            _ => panic!("Expected lodash to exist"),
        }
    }

    #[tokio::test]
    async fn test_check_nonexistent_package() {
        let checker = NpmChecker::new(10, 5, 60).unwrap();
        // Use a very unlikely package name
        let package = make_test_package("this-package-definitely-does-not-exist-12345xyz");

        let result = checker.check_package(&package).await;

        match result {
            NpmCheckResult::NotFound { .. } | NpmCheckResult::Error { .. } => {
                // Expected - either not found or error is acceptable
            }
            NpmCheckResult::Exists { .. } => panic!("Package should not exist"),
            _ => {}
        }
    }
}
