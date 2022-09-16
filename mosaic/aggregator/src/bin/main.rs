//! Mosaic Aggregator Executable.
//!
use std::process;

use aggregator::configs::{CliConfig, LogSettings, AggrSettings};

use structopt::StructOpt;
use tokio::signal;
// use tracing::warn;
use tracing_subscriber::*;

fn init_logging(settings: LogSettings) {
    let _fmt_subscriber = FmtSubscriber::builder()
        .with_env_filter(settings.filter)
        .with_ansi(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_buf = match CliConfig::from_args_safe() {
        Ok(path_buf) => Some(path_buf.config_path),
        Err(_) => {
            println!("\n\tWARN: Aggregator runs without external configuration, default values are used.\n");
            None
        }
    };

    let settings = AggrSettings::new(path_buf).unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1);
    });

    let AggrSettings {
        api: _api_settings,
        log: logging,
    } = settings;

    init_logging(logging);

    tokio::select! {
        biased;

        _ =  signal::ctrl_c() => {}
    }

    Ok(())
}