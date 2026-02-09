//! Colored console output for scan results.

use crate::types::{Confidence, Finding, NpmCheckResult, ScanResult, Severity};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// Console output handler with colors and formatting.
pub struct ConsoleOutput {
    verbose: bool,
    json_mode: bool,
    quiet: bool,
}

impl ConsoleOutput {
    /// Create a new console output handler.
    pub fn new(verbose: bool, json_mode: bool, quiet: bool) -> Self {
        Self { verbose, json_mode, quiet }
    }

    /// Print scan start message.
    pub fn print_scan_start(&self, target: &str) {
        if self.json_mode || self.quiet {
            return;
        }

        println!(
            "{} Scanning: {}",
            "[*]".bright_blue(),
            target.bright_white()
        );
    }

    /// Print scan progress (only in verbose mode).
    pub fn print_progress(&self, message: &str) {
        if self.json_mode || !self.verbose {
            return;
        }

        println!("{} {}", "[.]".dimmed(), message.dimmed());
    }

    /// Print info message.
    pub fn print_info(&self, message: &str) {
        if self.json_mode || self.quiet {
            return;
        }

        println!("{} {}", "[*]".bright_blue(), message);
    }

    /// Print a finding.
    pub fn print_finding(&self, finding: &Finding) {
        if self.json_mode {
            return;
        }

        let severity_color = match finding.severity {
            Severity::Critical => "CRITICAL".on_red().white().bold(),
            Severity::High => "HIGH".red().bold(),
            Severity::Medium => "MEDIUM".yellow().bold(),
            Severity::Low => "LOW".blue(),
            Severity::Info => "INFO".dimmed(),
        };

        let status = match &finding.npm_result {
            NpmCheckResult::NotFound { .. } => "NOT FOUND ON NPM".red().bold(),
            NpmCheckResult::ScopeNotClaimed { scope, .. } => {
                format!("SCOPE {} UNCLAIMED", scope).on_red().white().bold()
            }
            NpmCheckResult::Exists { .. } => "exists".green(),
            NpmCheckResult::Error { error, .. } => format!("error: {}", error).yellow(),
        };

        println!();
        println!(
            "{} {} [{}]",
            "===".bright_cyan(),
            finding.package.name.bright_white().bold(),
            severity_color
        );
        println!("    |-- Status: {}", status);
        println!("    |-- Source: {}", finding.package.source_url.dimmed());
        println!("    |-- Method: {:?}", finding.package.extraction_method);
        println!("    +-- Confidence: {}", format_confidence(finding.package.confidence));

        for note in &finding.notes {
            println!("        {}", note.dimmed());
        }
    }

    /// Print scan summary.
    pub fn print_summary(&self, result: &ScanResult) {
        if self.json_mode {
            if let Ok(json) = serde_json::to_string_pretty(result) {
                println!("{}", json);
            }
            return;
        }

        let vulnerability_count = result
            .findings
            .iter()
            .filter(|f| {
                matches!(
                    f.npm_result,
                    NpmCheckResult::NotFound { .. } | NpmCheckResult::ScopeNotClaimed { .. }
                )
            })
            .count();

        // In quiet mode, only print if there are vulnerabilities
        if self.quiet && vulnerability_count == 0 {
            return;
        }

        // Print scan start if we're showing summary in quiet mode
        if self.quiet && vulnerability_count > 0 {
            println!();
            println!(
                "{} Scanning: {}",
                "[*]".bright_blue(),
                result.target.bright_white()
            );
        }

        println!();
        println!("{}", "=== Scan Summary ===".bright_cyan());
        println!("  Target:    {}", result.target);
        println!("  Duration:  {:.2}s", result.duration_secs);
        println!("  JS files:  {}", result.js_files_count);
        println!("  Packages:  {}", result.packages_found);

        if vulnerability_count > 0 {
            println!(
                "  {}",
                format!("POTENTIAL VULNERABILITIES FOUND: {}", vulnerability_count)
                    .red()
                    .bold()
            );
        } else {
            println!(
                "  {}",
                "No dependency confusion vulnerabilities found.".green()
            );
        }

        if !result.errors.is_empty() {
            println!();
            println!("{}", "Errors encountered:".yellow());
            for error in &result.errors {
                println!("  - {}", error.dimmed());
            }
        }

        println!();
    }

    /// Create a progress bar.
    pub fn create_progress_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if self.json_mode {
            return None;
        }

        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        Some(pb)
    }
}

/// Format confidence level with color.
fn format_confidence(confidence: Confidence) -> colored::ColoredString {
    match confidence {
        Confidence::High => "High".green(),
        Confidence::Medium => "Medium".yellow(),
        Confidence::Low => "Low".dimmed(),
    }
}

impl Default for ConsoleOutput {
    fn default() -> Self {
        Self::new(false, false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_output_creation() {
        let output = ConsoleOutput::new(true, false, false);
        assert!(output.verbose);
        assert!(!output.json_mode);
    }

    #[test]
    fn test_format_confidence() {
        // Just test that it doesn't panic
        format_confidence(Confidence::High);
        format_confidence(Confidence::Medium);
        format_confidence(Confidence::Low);
    }
}
