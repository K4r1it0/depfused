//! Notification module for alerts and output.
//!
//! This module handles:
//! - Colored console output
//! - Telegram bot notifications
//! - JSON output formatting

pub mod console;
pub mod telegram;

pub use console::ConsoleOutput;
pub use telegram::TelegramNotifier;
