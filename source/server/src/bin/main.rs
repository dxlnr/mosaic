//! mosaic server.
//!
//! This binary serves as entry point for the server implementation.
use std::{path::PathBuf, process};

use server::{
    engine::EngineInitializer,
    server::start,
    service::messages::MessageHandler,
    settings::{LogSettings, Settings},
};
use structopt::StructOpt;
use tokio::signal;
use tracing::warn;
use tracing_subscriber::*;

#[derive(Debug, StructOpt)]
struct Config {
    config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::from_args();

    let settings = Settings::new(cfg.config_path).unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1);
    });

    let Settings {
        api: api_settings,
        model: model_settings,
        process: process_settings,
        log: logging,
    } = settings;
    init_logging(logging);

    let (engine, tx, subscriber) = EngineInitializer::new(model_settings, process_settings)
        .init()
        .await;
    let message_handler = MessageHandler::new(tx);

    tokio::select! {
        biased;

        _ =  signal::ctrl_c() => {}
        _ = engine.run() => {
            warn!("Training finished: Terminating the engine.")
        }
        result = start(api_settings, message_handler, subscriber) => {
            match result {
                Ok(()) => warn!("Shutting down: gRPC server terminated."),
                Err(_error) => {
                    warn!("Shutting down the server as an error occured.");
                },
            }
        }
    }
    Ok(())
}

fn init_logging(settings: LogSettings) {
    let _fmt_subscriber = FmtSubscriber::builder()
        .with_env_filter(settings.filter)
        .with_ansi(true)
        .init();
}
