//! In-memory caching layer for registry checks.

use crate::types::NpmCheckResult;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache entry with TTL.
#[derive(Debug, Clone)]
struct CacheEntry {
    result: NpmCheckResult,
    expires_at: Instant,
}

/// Thread-safe cache for npm registry check results.
#[derive(Debug, Clone)]
pub struct RegistryCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl RegistryCache {
    /// Create a new cache with the given TTL in seconds.
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get a cached result if it exists and hasn't expired.
    pub fn get(&self, package_name: &str) -> Option<NpmCheckResult> {
        let entry = self.cache.get(package_name)?;
        if Instant::now() < entry.expires_at {
            return Some(entry.result.clone());
        }
        // Entry expired, remove it
        drop(entry);
        self.cache.remove(package_name);
        None
    }

    /// Store a result in the cache.
    pub fn set(&self, package_name: &str, result: NpmCheckResult) {
        let entry = CacheEntry {
            result,
            expires_at: Instant::now() + self.ttl,
        };
        self.cache.insert(package_name.to_string(), entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_get() {
        let cache = RegistryCache::new(60);

        let result = NpmCheckResult::Exists {
            name: "lodash".to_string(),
            latest_version: Some("4.17.21".to_string()),
        };

        cache.set("lodash", result);

        let cached = cache.get("lodash");
        assert!(cached.is_some());

        if let Some(NpmCheckResult::Exists { name, .. }) = cached {
            assert_eq!(name, "lodash");
        } else {
            panic!("Expected Exists result");
        }
    }

    #[test]
    fn test_cache_miss() {
        let cache = RegistryCache::new(60);
        assert!(cache.get("nonexistent").is_none());
    }
}
