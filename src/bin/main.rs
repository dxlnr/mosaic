//! Mosaic Server.
//!
//! This binary serves as entry point for the server implementation and executes it.
//! Every single instance is designed for performing an individual training process and terminates when finished.

use std::{path::PathBuf, process};

use mosaic::{
    engine::EngineInitializer,
    proxy::server::start,
    rest::serve,
    service::{fetch::init_fetcher, messages::MessageHandler},
    settings::{LogSettings, Settings},
};
use structopt::StructOpt;
use tokio::signal;
use tracing::warn;
use tracing_subscriber::*;

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short, parse(from_os_str))]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_buf = match Config::from_args_safe() {
        Ok(path_buf) => Some(path_buf.config_path),
        Err(_) => {
            println!("\nAggregation Server runs without external configuration, default values are used.\n");
            None
        }
    };

    let settings = Settings::new(path_buf).unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1);
    });

    let Settings {
        api: api_settings,
        job: job_settings,
        model: model_settings,
        process: process_settings,
        log: logging,
        s3: s3_settings,
    } = settings;

    init_logging(logging);

    let (engine, tx, subscriber) =
        EngineInitializer::new(job_settings, model_settings, process_settings, s3_settings)
            .init()
            .await?;
    let message_handler = MessageHandler::new(tx);
    let fetcher = init_fetcher(&subscriber);

    tokio::select! {
        biased;

        _ =  signal::ctrl_c() => {}
        _ = engine.run() => {
            warn!("Training finished: Terminating the engine.")
        }
        rest = serve(&api_settings, fetcher.clone()) => {
            match rest {
                Ok(()) => warn!("Shutting down: rest http server terminated."),
                Err(_) => {
                    warn!("Shutting down the rest http server as an error occured.");
                },
            }
        }
        result = start(&api_settings, message_handler, fetcher) => {
            match result {
                Ok(()) => warn!("Shutting down: gRPC server terminated."),
                Err(_error) => {
                    warn!("Shutting down the gRPC server as an error occured.");
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
