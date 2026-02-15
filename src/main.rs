//! depfused - High-performance dependency confusion scanner.
//!
//! CLI entry point.

use clap::Parser;
use depfused::{Commands, Config, ScanConfig, Scanner, SetupConfig};
use std::fs;
use std::process::ExitCode;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let config = Config::parse();

    // Set up logging
    let filter = if config.verbose {
        EnvFilter::new("depfused=debug,info")
    } else {
        EnvFilter::new("depfused=info,warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Spawn signal handler to kill Chrome processes on SIGTERM/SIGINT.
    // Without this, Chrome survives after depfused is killed and burns CPU forever.
    tokio::spawn(async {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
            let mut sigint = signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {},
                _ = sigint.recv() => {},
            }
        }

        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }

        eprintln!("\nSignal received, killing Chrome processes...");
        depfused::discovery::kill_all_chrome();
        std::process::exit(130);
    });

    match config.command.clone() {
        Commands::Scan(scan_config) => {
            if let Err(code) = run_scan(scan_config, &config).await {
                return code;
            }
        }
        Commands::Setup(setup_config) => {
            if let Err(code) = run_setup(setup_config).await {
                return code;
            }
        }
    }

    ExitCode::SUCCESS
}

async fn run_scan(scan_config: ScanConfig, global_config: &Config) -> Result<(), ExitCode> {
    // Load targets
    let targets = match scan_config.load_targets() {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to load targets: {}", e);
            return Err(ExitCode::FAILURE);
        }
    };

    if targets.is_empty() {
        error!("No targets specified. Use positional arguments or -f <file>.");
        return Err(ExitCode::FAILURE);
    }

    // Create scanner
    let mut scanner = match Scanner::new(scan_config.clone()).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create scanner: {}", e);
            return Err(ExitCode::FAILURE);
        }
    };

    // Configure Telegram if requested
    if scan_config.telegram {
        if let (Some(ref token), Some(ref chat_id)) =
            (&global_config.telegram_token, &global_config.telegram_chat_id)
        {
            scanner = match scanner.with_telegram(token, chat_id) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to configure Telegram: {}", e);
                    return Err(ExitCode::FAILURE);
                }
            };
        } else {
            error!(
                "Telegram notifications requested but DEPFUSED_TELEGRAM_TOKEN and/or DEPFUSED_TELEGRAM_CHAT_ID not set"
            );
            return Err(ExitCode::FAILURE);
        }
    }

    // Print banner unless JSON mode
    if !scan_config.json {
        print_banner();
    }

    // Run scans
    let results = scanner.scan_multiple(targets).await;

    // Output results
    if scan_config.json {
        let json = serde_json::to_string_pretty(&results).unwrap_or_default();
        if let Some(ref output_path) = scan_config.output {
            if let Err(e) = fs::write(output_path, &json) {
                error!("Failed to write output file: {}", e);
                return Err(ExitCode::FAILURE);
            }
        } else {
            println!("{}", json);
        }
    } else if let Some(ref output_path) = scan_config.output {
        // Write JSON to file even in non-JSON mode
        let json = serde_json::to_string_pretty(&results).unwrap_or_default();
        if let Err(e) = fs::write(output_path, &json) {
            error!("Failed to write output file: {}", e);
            return Err(ExitCode::FAILURE);
        }
        info!("Results written to: {:?}", output_path);
    }

    // Check for vulnerabilities and set exit code
    let total_vulns: usize = results
        .iter()
        .map(|r| {
            r.findings
                .iter()
                .filter(|f| {
                    matches!(
                        f.npm_result,
                        depfused::NpmCheckResult::NotFound { .. }
                            | depfused::NpmCheckResult::ScopeNotClaimed { .. }
                    )
                })
                .count()
        })
        .sum();

    if total_vulns > 0 && !scan_config.json {
        eprintln!(
            "\n{} potential dependency confusion vulnerabilities found!",
            total_vulns
        );
    }

    Ok(())
}

async fn run_setup(setup_config: SetupConfig) -> Result<(), ExitCode> {
    eprintln!("Setting up Chromium browser...");
    match depfused::browser::download_chrome(setup_config.force).await {
        Ok(path) => {
            eprintln!("Chromium ready at: {}", path.display());
            Ok(())
        }
        Err(e) => {
            error!("Setup failed: {}", e);
            Err(ExitCode::FAILURE)
        }
    }
}

fn print_banner() {
    println!();
    println!("\x1b[36m╔══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║                    DEPFUSED v0.1.0                           ║\x1b[0m");
    println!("\x1b[36m║           Dependency Confusion Scanner                       ║\x1b[0m");
    println!("\x1b[36m╚══════════════════════════════════════════════════════════════╝\x1b[0m");
    println!();
}
