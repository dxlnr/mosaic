use std::{fmt, path::Path};

use config::{Config, ConfigError};
use displaydoc::Display;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};

use crate::engine::model::DataType;
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
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
    pub process: ProcessSettings,
    pub log: LogSettings,
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

#[derive(Debug, Deserialize)]
pub struct LogSettings {
    /// Tokio tracing filter which filters spans and events based on a set of filter directives.
    #[serde(deserialize_with = "deserialize_env_filter")]
    pub filter: EnvFilter,
}

#[derive(Debug, Deserialize, Clone)]
pub struct APISettings {
    pub address: std::net::SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelSettings {
    pub length: usize,
    pub data_type: DataType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessSettings {
    /// Defines the number of training rounds.
    pub rounds: u32,
    /// Sets the number of participants.
    pub participants: u32,
}

fn deserialize_env_filter<'de, D>(deserializer: D) -> Result<EnvFilter, D::Error>
where
    D: Deserializer<'de>,
{
    struct EnvFilterVisitor;
    impl<'de> Visitor<'de> for EnvFilterVisitor {
        type Value = EnvFilter;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "tokio tracing")
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            EnvFilter::try_new(value)
                .map_err(|_| de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
        }
    }
    deserializer.deserialize_str(EnvFilterVisitor)
}
