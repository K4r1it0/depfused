//! npm registry checking module.
//!
//! Verifies if packages exist on npm, checks scope ownership,
//! and caches results to avoid duplicate API calls.

mod cache;
pub mod npm;

pub use npm::NpmChecker;
