use std::{path::PathBuf, process};

use server::{engine, server::start, services, settings::Settings};
use structopt::StructOpt;
use tracing::warn;
//use tracing_subscriber::*;

#[derive(Debug, StructOpt)]
struct Config {
    config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::from_args();

    let settings = Settings::new(cfg.config_path).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    let Settings { api: api_settings } = settings;

    // let message_handler = engine::message::MessageHandler::new();
    // let fetcher = services::fetcher::fetcher();

    tokio::select! {
        result = start(api_settings) => {
            match result {
                Ok(()) => warn!("shutting down: gRPC server terminated."),
                Err(_error) => {
                    warn!("shutting down as an error occured.");
                },
            }
        }
    }
    Ok(())
}
