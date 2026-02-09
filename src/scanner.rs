//! Main scanner orchestrating all components.

use crate::config::ScanConfig;
use crate::discovery::{BrowserCapture, JsFetcher, SourceMapProber};
use crate::notify::{ConsoleOutput, TelegramNotifier};
use crate::parser::{
    is_likely_false_positive, is_likely_internal, AstParser, BundlerParser, Deobfuscator,
    SourceMapParser, WebpackParser,
};
use crate::registry::NpmChecker;
use crate::types::{
    Confidence, ExtractionMethod, Finding, JsFile, JsSource, NpmCheckResult, Package, Result,
    ScanResult, Severity,
};
use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, trace};

/// Main scanner that orchestrates all scanning components.
pub struct Scanner {
    config: ScanConfig,
    fetcher: Arc<JsFetcher>,
    npm_checker: Arc<NpmChecker>,
    ast_parser: AstParser,
    sourcemap_parser: SourceMapParser,
    webpack_parser: WebpackParser,
    bundler_parser: BundlerParser,
    deobfuscator: Deobfuscator,
    sourcemap_prober: SourceMapProber,
    browser_capture: BrowserCapture,
    console: ConsoleOutput,
    telegram: Option<TelegramNotifier>,
}

impl Scanner {
    /// Create a new scanner with the given configuration.
    pub async fn new(config: ScanConfig) -> Result<Self> {
        let http_config = config.http_config();

        let fetcher = Arc::new(JsFetcher::new(http_config.clone(), config.rate_limit)?);

        let npm_checker = Arc::new(NpmChecker::new(
            config.timeout,
            config.rate_limit,
            3600, // 1 hour cache TTL
        )?);

        let sourcemap_prober =
            SourceMapProber::new(config.timeout, &http_config.user_agent)?;

        // Resolve Chrome executable: explicit flag > managed install > system default
        let chrome_exe = config
            .chrome_path
            .clone()
            .or_else(crate::browser::resolve_chrome_executable);

        let browser_capture = BrowserCapture::new(config.timeout, true)
            .with_fast_mode(config.fast)
            .with_chrome_executable(chrome_exe);

        let console = ConsoleOutput::new(config.verbose, config.json, config.quiet);

        let include_low = config.min_confidence == "low";

        Ok(Self {
            config,
            fetcher,
            npm_checker,
            ast_parser: AstParser::new(include_low),
            sourcemap_parser: SourceMapParser::new(),
            webpack_parser: WebpackParser::new(),
            bundler_parser: BundlerParser::new(),
            deobfuscator: Deobfuscator::new(),
            sourcemap_prober,
            browser_capture,
            console,
            telegram: None,
        })
    }

    /// Configure Telegram notifications.
    pub fn with_telegram(mut self, token: &str, chat_id: &str) -> Result<Self> {
        self.telegram = Some(TelegramNotifier::new(token, chat_id)?);
        Ok(self)
    }

    /// Scan a single target URL.
    pub async fn scan(&self, target: &str) -> Result<ScanResult> {
        let start_time = Instant::now();
        self.console.print_scan_start(target);

        let mut all_js_files: Vec<JsFile> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        // Use headless browser to capture all JS
        self.console.print_progress("Launching browser to capture JS...");
        match self.browser_capture.capture(target).await {
            Ok(js_files) => {
                self.console.print_progress(&format!(
                    "Browser captured {} JS files",
                    js_files.len()
                ));
                all_js_files.extend(js_files);
            }
            Err(e) => {
                errors.push(format!("Browser capture failed: {}", e));
            }
        }

        self.process_captured_js(target, all_js_files, errors, start_time)
            .await
    }

    /// Process captured JS files: extract packages, check npm, produce findings.
    /// Shared pipeline used by both `scan()` and `scan_host_group()`.
    async fn process_captured_js(
        &self,
        target: &str,
        mut all_js_files: Vec<JsFile>,
        errors: Vec<String>,
        start_time: Instant,
    ) -> Result<ScanResult> {
        let mut all_packages: HashSet<Package> = HashSet::new();

        // Discover lazy-loaded chunks referenced in captured JS (iterate to find nested chunks)
        let mut seen_urls: HashSet<String> = all_js_files.iter().map(|f| f.url.clone()).collect();
        let mut scan_from = 0; // index to start scanning from
        for _depth in 0..3 {
            let chunk_urls = discover_lazy_chunks(&all_js_files[scan_from..], target);
            let new_urls: Vec<String> = chunk_urls.into_iter().filter(|u| !seen_urls.contains(u)).collect();
            if new_urls.is_empty() {
                break;
            }
            self.console.print_progress(&format!(
                "Discovered {} lazy-loaded chunk URLs, fetching...",
                new_urls.len()
            ));
            scan_from = all_js_files.len();
            for chunk_url in new_urls {
                seen_urls.insert(chunk_url.clone());
                if let Some(js_file) = self.fetcher.fetch_one(&chunk_url, JsSource::Probe).await {
                    all_js_files.push(js_file);
                }
            }
        }

        // Process all JS files in parallel
        let js_files_count = all_js_files.len();
        self.console.print_progress(&format!(
            "Processing {} JS files in parallel...",
            js_files_count
        ));

        // Spawn parallel tasks for each JS file
        let tasks: Vec<_> = all_js_files
            .into_iter()
            .map(|js_file| {
                let fetcher = self.fetcher.clone();
                let sourcemap_prober = self.sourcemap_prober.clone();
                let sourcemap_parser = self.sourcemap_parser.clone();
                let ast_parser = self.ast_parser.clone();
                let webpack_parser = self.webpack_parser.clone();
                let bundler_parser = self.bundler_parser.clone();
                let deobfuscator = self.deobfuscator.clone();
                let target = target.to_string();

                tokio::spawn(async move {
                    let mut packages = HashSet::new();

                    // Skip very large files (>5MB) - they're rarely useful and slow to parse
                    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
                    if js_file.content.len() > MAX_FILE_SIZE {
                        debug!("Skipping large file ({} bytes): {}", js_file.content.len(), js_file.url);
                        return packages;
                    }

                    // 1. Fetch and parse source map (if exists)
                    if let Some(ref map_url) = js_file.source_map_url {
                        if map_url.starts_with("data:") {
                            // Inline source map
                            if let Some(content) = SourceMapProber::decode_inline_sourcemap(map_url) {
                                if let Ok(pkgs) = sourcemap_parser.parse(&content, map_url) {
                                    packages.extend(pkgs);
                                }
                            }
                        } else {
                            // Fetch external source map
                            if let Some(map_js) = fetcher.fetch_one(map_url, JsSource::Probe).await {
                                if let Ok(pkgs) = sourcemap_parser.parse(&map_js.content, map_url) {
                                    packages.extend(pkgs);
                                }
                            }
                        }
                    }

                    // Also probe for .map even if not referenced (but only for bundled files)
                    // Check if file looks like it's bundled (has webpack/vite/bundler patterns)
                    let is_likely_bundled = js_file.content.contains("webpackChunk")
                        || js_file.content.contains("__vite__")
                        || js_file.content.contains("parcelRequire")
                        || js_file.content.contains("__commonJS")
                        || js_file.content.contains("__toESM")
                        || js_file.content.contains("__require")
                        || js_file.content.contains("/*#__PURE__*/")
                        || js_file.url.contains(".bundle.js")
                        || js_file.url.ends_with("/bundle.js")
                        || js_file.url.contains("chunk")
                        || js_file.url.contains("vendor")
                        || js_file.url.contains("/main-")
                        || js_file.url.contains("/main.")
                        || js_file.content.len() > 50_000;

                    if is_likely_bundled {
                        if let Some((map_url, content)) = sourcemap_prober.probe(&js_file.url).await {
                            if let Ok(pkgs) = sourcemap_parser.parse(&content, &map_url) {
                                packages.extend(pkgs);
                            }
                        }
                    }

                    // 2. Parse JS with AST
                    if let Ok(pkgs) = ast_parser.parse(&js_file.content, &js_file.url) {
                        packages.extend(pkgs);
                    }

                    // 3. Webpack-specific extraction
                    if webpack_parser.is_webpack_bundle(&js_file.content) {
                        let webpack_packages = webpack_parser
                            .extract_packages(&js_file.content, &js_file.url);
                        packages.extend(webpack_packages);

                        // Check for Next.js and extract build manifests
                        if let Some(build_id) = webpack_parser.extract_nextjs_build_id(&js_file.content)
                        {
                            let manifest_urls = webpack_parser
                                .get_nextjs_manifest_urls(&target, &build_id);
                            for url in manifest_urls {
                                if let Some(manifest) = fetcher.fetch_one(&url, JsSource::Probe).await {
                                    if let Ok(pkgs) = ast_parser.parse(&manifest.content, &manifest.url) {
                                        packages.extend(pkgs);
                                    }
                                }
                            }
                        }
                    }

                    // 4. Bundler-specific extraction
                    let bundler_packages = bundler_parser
                        .extract_packages(&js_file.content, &js_file.url);
                    packages.extend(bundler_packages);

                    // 5. Deobfuscation extraction
                    if deobfuscator.is_likely_obfuscated(&js_file.content) {
                        debug!("Detected obfuscated content in {}, running deobfuscation", js_file.url);
                        let deob_packages = deobfuscator
                            .extract_packages(&js_file.content, &js_file.url);
                        packages.extend(deob_packages);
                    }

                    packages
                })
            })
            .collect();

        // Wait for all tasks to complete and collect results
        for task in tasks {
            match task.await {
                Ok(packages) => all_packages.extend(packages),
                Err(e) => debug!("Task join error: {}", e),
            }
        }

        // Deduplicate packages by name, keeping the highest confidence version
        let all_packages = deduplicate_packages(all_packages);

        self.console.print_info(&format!(
            "Extracted {} unique packages",
            all_packages.len()
        ));

        // Filter packages based on configuration
        let mut packages_to_check: Vec<Package> = all_packages
            .into_iter()
            .filter(|p| {
                if self.config.scoped_only && !p.name.starts_with('@') {
                    return false;
                }
                // Filter by confidence
                match self.config.min_confidence.as_str() {
                    "high" => p.confidence == Confidence::High,
                    "medium" => p.confidence >= Confidence::Medium,
                    _ => true,
                }
            })
            .collect();

        // Sort packages by name for consistent/reproducible results
        packages_to_check.sort_by(|a, b| a.name.cmp(&b.name));

        // Check packages against npm registry (in parallel)
        let mut findings: Vec<Finding> = Vec::new();

        if !self.config.skip_npm_check {
            self.console
                .print_progress("Checking packages against npm registry...");

            let pb = self
                .console
                .create_progress_bar(packages_to_check.len() as u64, "Checking npm");

            // Check all packages in parallel using buffer_unordered
            let npm_checker = &self.npm_checker;
            let concurrency = 50; // Check up to 50 packages concurrently

            let results: Vec<(Package, NpmCheckResult)> = stream::iter(packages_to_check.iter())
                .map(|package| async move {
                    let result = npm_checker.check_package(package).await;
                    (package.clone(), result)
                })
                .buffer_unordered(concurrency)
                .collect()
                .await;

            for (package, result) in results {
                if let Some(ref pb) = pb {
                    pb.inc(1);
                }

                let finding = self.create_finding(package, result);

                // Filter out false positives: NotFound for scoped packages means scope is claimed
                // and we can't exploit it (not a dependency confusion vulnerability)
                let should_report = match &finding.npm_result {
                    // CRITICAL: Scope unclaimed - attacker can register and publish
                    NpmCheckResult::ScopeNotClaimed { .. } => true,
                    // Check if NotFound is exploitable (only for unscoped packages)
                    NpmCheckResult::NotFound { name } => {
                        // If it's a scoped package and NotFound, scope IS claimed
                        // We can't publish to claimed scopes = NOT a dependency confusion vuln
                        !name.starts_with('@')
                    }
                    // Include safe packages (Info) and errors (Low) in results
                    NpmCheckResult::Exists { .. } => true,
                    NpmCheckResult::Error { .. } => true,
                };

                // Only report valid findings (exploitable + info)
                if should_report {
                    // Print to console if it's a potential vulnerability
                    if matches!(
                        finding.npm_result,
                        NpmCheckResult::NotFound { .. } | NpmCheckResult::ScopeNotClaimed { .. }
                    ) {
                        self.console.print_finding(&finding);

                        // Send Telegram notification for high/critical findings
                        if let Some(ref telegram) = self.telegram {
                            if finding.severity >= Severity::High {
                                if let Err(e) = telegram.send_finding(&finding, target).await {
                                    debug!("Failed to send Telegram notification: {}", e);
                                }
                            }
                        }
                    }

                    findings.push(finding);
                }
            }

            if let Some(pb) = pb {
                pb.finish_and_clear();
            }

            // Sort findings by package name for consistent output
            findings.sort_by(|a, b| {
                a.package.name.cmp(&b.package.name)
            });
        }

        let duration = start_time.elapsed().as_secs_f64();

        let result = ScanResult {
            target: target.to_string(),
            js_files_count,
            packages_found: packages_to_check.len(),
            findings,
            duration_secs: duration,
            errors,
        };

        self.console.print_summary(&result);

        // Send summary to Telegram
        if let Some(ref telegram) = self.telegram {
            let vuln_count = result
                .findings
                .iter()
                .filter(|f| {
                    matches!(
                        f.npm_result,
                        NpmCheckResult::NotFound { .. } | NpmCheckResult::ScopeNotClaimed { .. }
                    )
                })
                .count();

            if let Err(e) = telegram
                .send_summary(target, result.findings.len(), vuln_count)
                .await
            {
                debug!("Failed to send Telegram summary: {}", e);
            }
        }

        Ok(result)
    }

    /// Scan a group of URLs that share the same host using a single browser instance.
    async fn scan_host_group(&self, urls: Vec<String>) -> Vec<ScanResult> {
        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        let capture_results = self.browser_capture.capture_multiple(&url_refs).await;

        let mut results = Vec::with_capacity(urls.len());
        for (target, capture_result) in capture_results {
            let start_time = Instant::now();
            self.console.print_scan_start(&target);

            let (js_files, errors) = match capture_result {
                Ok(files) => {
                    self.console.print_progress(&format!(
                        "Browser captured {} JS files",
                        files.len()
                    ));
                    (files, Vec::new())
                }
                Err(e) => (Vec::new(), vec![format!("Browser capture failed: {}", e)]),
            };

            match self
                .process_captured_js(&target, js_files, errors, start_time)
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => results.push(ScanResult {
                    target,
                    js_files_count: 0,
                    packages_found: 0,
                    findings: vec![],
                    duration_secs: 0.0,
                    errors: vec![e.to_string()],
                }),
            }
        }

        results
    }

    /// Scan multiple targets, grouping by host to reuse browser instances.
    pub async fn scan_multiple(&self, targets: Vec<String>) -> Vec<ScanResult> {
        let parallel_count = self.config.parallel.max(1);

        // For a single target, skip grouping overhead
        if targets.len() == 1 {
            let result = match self.scan(&targets[0]).await {
                Ok(r) => r,
                Err(e) => ScanResult {
                    target: targets[0].clone(),
                    js_files_count: 0,
                    packages_found: 0,
                    findings: vec![],
                    duration_secs: 0.0,
                    errors: vec![e.to_string()],
                },
            };
            return vec![result];
        }

        // Group targets by host for browser reuse
        let host_groups = crate::browser::group_by_host(&targets);

        // Build an index to restore original ordering
        let mut url_to_index: HashMap<String, usize> = HashMap::new();
        for (i, t) in targets.iter().enumerate() {
            url_to_index.insert(t.clone(), i);
        }

        // Process host groups in parallel
        let all_group_results: Vec<Vec<ScanResult>> = stream::iter(host_groups)
            .map(|(_host, group_urls)| async move {
                self.scan_host_group(group_urls).await
            })
            .buffer_unordered(parallel_count)
            .collect()
            .await;

        // Flatten and re-order to match original input order
        let mut indexed: Vec<(usize, ScanResult)> = all_group_results
            .into_iter()
            .flatten()
            .map(|r| {
                let idx = url_to_index.get(&r.target).copied().unwrap_or(usize::MAX);
                (idx, r)
            })
            .collect();

        indexed.sort_by_key(|(idx, _)| *idx);
        indexed.into_iter().map(|(_, r)| r).collect()
    }

    /// Create a finding from a package and npm check result.
    fn create_finding(&self, package: Package, npm_result: NpmCheckResult) -> Finding {
        let severity = match &npm_result {
            NpmCheckResult::ScopeNotClaimed { .. } => Severity::Critical,
            NpmCheckResult::NotFound { .. } => {
                if is_likely_internal(&package.name) {
                    Severity::High
                } else {
                    Severity::Medium
                }
            }
            NpmCheckResult::Exists { .. } => Severity::Info,
            NpmCheckResult::Error { .. } => Severity::Low,
        };

        let mut notes = Vec::new();

        if is_likely_internal(&package.name) {
            notes.push("Package name suggests internal/private usage".to_string());
        }

        if package.confidence == Confidence::Low {
            notes.push("Low confidence extraction - verify manually".to_string());
        }

        Finding {
            package,
            npm_result,
            severity,
            notes,
        }
    }
}

/// Get extraction method priority for deduplication.
fn extraction_priority(method: &ExtractionMethod) -> u8 {
    match method {
        ExtractionMethod::Import | ExtractionMethod::Require | ExtractionMethod::DynamicImport => 3,
        ExtractionMethod::SourceMap | ExtractionMethod::WebpackChunk => 2,
        ExtractionMethod::Comment | ExtractionMethod::ErrorMessage | ExtractionMethod::Deobfuscate => 1,
    }
}

/// Check if a package should be filtered out as likely false positive.
fn should_skip_package(pkg: &Package) -> bool {
    if is_likely_false_positive(&pkg.name) {
        return true;
    }

    // Filter WebpackChunk extractions without hyphens or scopes
    if pkg.extraction_method == ExtractionMethod::WebpackChunk
        && !pkg.name.contains('-')
        && !pkg.name.contains('/')
        && pkg.name.len() < 20
    {
        return true;
    }

    // Filter Comment extractions without hyphens or scopes
    if pkg.extraction_method == ExtractionMethod::Comment
        && !pkg.name.contains('-')
        && !pkg.name.starts_with('@')
    {
        return true;
    }

    false
}

/// Deduplicate packages by name, keeping the one with highest confidence.
fn deduplicate_packages(packages: HashSet<Package>) -> HashSet<Package> {
    let mut by_name: HashMap<String, Package> = HashMap::new();

    for pkg in packages {
        if should_skip_package(&pkg) {
            trace!("Skipping artifact: {}", pkg.name);
            continue;
        }

        let should_insert = match by_name.get(&pkg.name) {
            Some(existing) => {
                pkg.confidence > existing.confidence
                    || (pkg.confidence == existing.confidence
                        && extraction_priority(&pkg.extraction_method)
                            > extraction_priority(&existing.extraction_method))
            }
            None => true,
        };

        if should_insert {
            by_name.insert(pkg.name.clone(), pkg);
        }
    }

    by_name.into_values().collect()
}

/// Discover lazy-loaded chunk URLs referenced in captured JS files.
///
/// Scans JS content for patterns like:
/// - `import("./chunk-ABC123.js")`
/// - `e.p+"chunk-ABC123."+e.h()+".js"` (webpack)
/// - `"./chunk-DIHBRSVG.js"` (literal chunk references)
fn discover_lazy_chunks(js_files: &[JsFile], _target: &str) -> Vec<String> {
    let mut chunk_urls = HashSet::new();

    // Regex for chunk filename patterns commonly found in bundles
    let chunk_patterns = [
        // Literal chunk filenames: "chunk-XXXX.js", "./chunk-XXXX.js"
        r#"["']\./?(chunk-[a-zA-Z0-9_-]+\.js)["']"#,
        // Lazy import patterns: import("./chunk-XXXX.js")
        r#"import\s*\(\s*["']\./?(chunk-[a-zA-Z0-9_-]+\.js)["']\s*\)"#,
        // Angular-style lazy chunks
        r#"["']\./?(chunk-[a-zA-Z0-9_-]+\.mjs)["']"#,
    ];

    for js_file in js_files {
        // Determine base URL for resolving relative chunk paths
        let base_url = if let Some(pos) = js_file.url.rfind('/') {
            &js_file.url[..=pos]
        } else {
            continue;
        };

        for pattern in &chunk_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for caps in re.captures_iter(&js_file.content) {
                    if let Some(chunk_name) = caps.get(1) {
                        let chunk_url = format!("{}{}", base_url, chunk_name.as_str());
                        chunk_urls.insert(chunk_url);
                    }
                }
            }
        }
    }

    chunk_urls.into_iter().collect()
}
