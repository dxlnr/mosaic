use std::path::Path;

use config::{Config, ConfigError};
use displaydoc::Display;
use serde::Deserialize;

use thiserror::Error;
//use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Display, Error)]
/// An error related to loading and validation of settings.
pub enum SettingsError {
    /// Configuration loading failed: {0}.
    Loading(#[from] ConfigError),
    /// Validation failed: {0}.
    Validation(#[from] ValidationErrors),
}

#[derive(Debug, Validate, Deserialize)]
pub struct Settings {
    //pub log: LoggingSettings,
    pub api: APISettings,
    pub model: ModelSettings,
}

impl Settings {
    /// Loads and validates the settings via a configuration file.
    ///
    /// # Errors
    /// Fails when the loading of the configuration file or its validation failed.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SettingsError> {
        let settings: Settings = Self::load(path)?;
        settings.validate()?;
        Ok(settings)
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        config.merge(config::File::from(path.as_ref()))?;
        config.try_into()
    }
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct LoggingSettings {
//     pub filter: EnvFilter,
// }

#[derive(Debug, Deserialize, Clone)]
pub struct APISettings {
    pub address: std::net::SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelSettings {
    pub length: usize,
}
