//! Mosaic Server.
//!
//! This binary serves as entry point for the server implementation and executes it.
//! Every single instance is designed for performing an individual training process
//! and terminates when finished.

use std::{path::PathBuf, process};

use server::settings::{LogSettings, Settings};

use structopt::StructOpt;
use tokio::signal;
use tracing::warn;
use tracing_subscriber::*;

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short, parse(from_os_str))]
    config_path: PathBuf,
}

fn init_logging(settings: LogSettings) {
    let _fmt_subscriber = FmtSubscriber::builder()
        .with_env_filter(settings.filter)
        .with_ansi(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_buf = match Config::from_args_safe() {
        Ok(path_buf) => Some(path_buf.config_path),
        Err(_) => {
            println!("\n\tWARN: Aggregation Server runs without external configuration, default values are used.\n");
            None
        }
    };

    let settings = Settings::new(path_buf).unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1);
    });

    let Settings {
        api: api_settings,
        log: logging,
    } = settings;

    init_logging(logging);

    tokio::select! {
        biased;

        _ =  signal::ctrl_c() => {}
    }

    Ok(())
}
