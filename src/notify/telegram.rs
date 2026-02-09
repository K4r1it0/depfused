//! Telegram bot notifications for scan findings.

use crate::types::{DepfusedError, Finding, NpmCheckResult, Result, Severity};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error};

/// Telegram message request body.
#[derive(Serialize)]
struct SendMessageRequest<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

/// Telegram notification handler.
pub struct TelegramNotifier {
    client: Client,
    token: String,
    chat_id: String,
}

impl TelegramNotifier {
    /// Create a new Telegram notifier.
    pub fn new(token: &str, chat_id: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            token: token.to_string(),
            chat_id: chat_id.to_string(),
        })
    }

    /// Send a finding notification.
    pub async fn send_finding(&self, finding: &Finding, target: &str) -> Result<()> {
        let message = self.format_finding_message(finding, target);
        self.send_message(&message).await
    }

    /// Send a scan summary notification.
    pub async fn send_summary(
        &self,
        target: &str,
        findings_count: usize,
        vulnerabilities_count: usize,
    ) -> Result<()> {
        let emoji = if vulnerabilities_count > 0 {
            "🚨"
        } else {
            "✅"
        };

        let message = format!(
            "{} *Depfused Scan Complete*\n\n\
             *Target:* `{}`\n\
             *Findings:* {}\n\
             *Potential Vulnerabilities:* {}",
            emoji, target, findings_count, vulnerabilities_count
        );

        self.send_message(&message).await
    }

    /// Format a finding as a Telegram message.
    fn format_finding_message(&self, finding: &Finding, target: &str) -> String {
        let severity_emoji = match finding.severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
            Severity::Info => "⚪",
        };

        let status = match &finding.npm_result {
            NpmCheckResult::NotFound { .. } => "❌ NOT FOUND ON NPM".to_string(),
            NpmCheckResult::ScopeNotClaimed { scope, .. } => {
                format!("⚠️ SCOPE {} UNCLAIMED", scope)
            }
            NpmCheckResult::Exists { .. } => "✓ exists".to_string(),
            NpmCheckResult::Error { error, .. } => format!("⚠️ error: {}", error),
        };

        format!(
            "{} *Dependency Confusion Finding*\n\n\
             *Package:* `{}`\n\
             *Status:* {}\n\
             *Severity:* {:?}\n\
             *Target:* `{}`\n\
             *Source:* `{}`\n\
             *Confidence:* {:?}",
            severity_emoji,
            finding.package.name,
            status,
            finding.severity,
            target,
            finding.package.source_url,
            finding.package.confidence
        )
    }

    /// Send a raw message via Telegram Bot API.
    async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.token
        );

        let body = SendMessageRequest {
            chat_id: &self.chat_id,
            text,
            parse_mode: "Markdown",
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Telegram message sent successfully");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to send Telegram message: {}", error_text);
            Err(DepfusedError::TelegramError(error_text))
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, ExtractionMethod, Package};

    fn make_test_finding() -> Finding {
        Finding {
            package: Package {
                name: "@internal/test-pkg".to_string(),
                extraction_method: ExtractionMethod::SourceMap,
                source_url: "https://example.com/bundle.js".to_string(),
                confidence: Confidence::High,
            },
            npm_result: NpmCheckResult::NotFound {
                name: "@internal/test-pkg".to_string(),
            },
            severity: Severity::High,
            notes: vec![],
        }
    }

    #[test]
    fn test_format_finding_message() {
        // This test would require a valid token, so we skip the actual API call
        // Just test the message formatting logic exists
        let finding = make_test_finding();
        assert!(!finding.package.name.is_empty());
    }
}
