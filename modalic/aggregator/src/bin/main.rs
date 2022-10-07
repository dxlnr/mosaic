use std::{path::PathBuf, process};

use structopt::StructOpt;
use tokio::signal;
use tracing::warn;
use tracing_subscriber::*;

#[cfg(feature = "metrics")]
use xaynet_server::{metrics, settings::InfluxSettings};

use aggregator::{
    rest::{serve, RestError},
    services,
    settings::{LoggingSettings, RedisSettings, Settings},
    state_engine::init::StateEngineInitializer,
    storage::{coordinator_storage::redis, Storage, Store},
};
#[cfg(feature = "model-persistence")]
use xaynet_server::{settings::S3Settings, storage::model_storage::s3};

#[derive(Debug, StructOpt)]
#[structopt(name = "Aggregator")]
struct CliConf {
    /// Path of the configuration file
    #[structopt(short, parse(from_os_str))]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let path_buf = match CliConf::from_args_safe() {
        Ok(path_buf) => Some(path_buf.config_path),
        Err(_) => {
            println!("\n\tWARN: Aggregator runs without external configuration, default values are used.\n");
            None
        }
    };

    let settings = Settings::new(path_buf).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let Settings {
        pet: pet_settings,
        mask: mask_settings,
        api: api_settings,
        log: log_settings,
        model: model_settings,
        redis: redis_settings,
        ..
    } = settings;

    init_tracing(log_settings);

    // This should already called internally when instantiating the
    // state machine but it doesn't hurt making sure the crypto layer
    // is correctly initialized
    sodiumoxide::init().unwrap();

    #[cfg(feature = "metrics")]
    init_metrics(settings.metrics.influxdb);

    let store = init_store(
        redis_settings,
        #[cfg(feature = "model-persistence")]
        settings.s3,
    )
    .await;

    let (state_machine, requests_tx, event_subscriber) = StateEngineInitializer::new(
        pet_settings,
        mask_settings,
        model_settings,
        #[cfg(feature = "model-persistence")]
        settings.restore,
        store,
    )
    .init()
    .await
    .expect("failed to initialize state machine");

    let fetcher = services::fetchers::fetcher(&event_subscriber);
    let message_handler =
        services::messages::PetMessageHandler::new(&event_subscriber, requests_tx);

    tokio::select! {
        biased;

        _ =  signal::ctrl_c() => {}
        _ = state_machine.run() => {
            warn!("shutting down: Service terminated");
        }
        result = serve(api_settings, fetcher, message_handler) => {
            match result {
                Ok(()) => warn!("shutting down: REST server terminated"),
                Err(RestError::InvalidTlsConfig) => {
                    warn!("shutting down: invalid TLS settings for REST server");
                },
            }
        }
    }
}

fn init_tracing(settings: LoggingSettings) {
    let _fmt_subscriber = FmtSubscriber::builder()
        .with_env_filter(settings.filter)
        .with_ansi(true)
        .init();
}

#[cfg(feature = "metrics")]
fn init_metrics(settings: InfluxSettings) {
    let recorder = metrics::Recorder::new(settings);
    if metrics::GlobalRecorder::install(recorder).is_err() {
        warn!("failed to install metrics recorder");
    };
}

async fn init_store(
    redis_settings: RedisSettings,
    #[cfg(feature = "model-persistence")] s3_settings: S3Settings,
) -> impl Storage {
    let coordinator_store = redis::Client::new(redis_settings.url)
        .await
        .expect("failed to establish a connection to Redis");

    let model_store = {
        #[cfg(not(feature = "model-persistence"))]
        {
            aggregator::storage::model_storage::noop::NoOp
        }

        #[cfg(feature = "model-persistence")]
        {
            let s3 = s3::Client::new(s3_settings).expect("failed to create S3 client");
            s3.create_global_models_bucket()
                .await
                .expect("failed to create bucket for global models");
            s3
        }
    };

    Store::new(coordinator_store, model_store)
}
