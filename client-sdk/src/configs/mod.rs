use std::{fmt, path::Path};

use config::{Config, ConfigError, ValueKind};
use derive_more::Display;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Display, Error)]
/// An error related to loading and validation of [`Conf`].
pub enum ConfError {
    /// Loading configuration file failed: {0}.
    Loading(#[from] ConfigError),
    /// Validation failed: {0}.
    Validation(#[from] ValidationErrors),
    /// Parsing error
    ParsingError,
    /// BuildError: Cannot construct a Conf.
    BuildError,
}

#[derive(Debug, Default, Deserialize)]
/// Defines the way the logging of the server is done via filter.
pub struct LogConf {
    /// Tokio tracing filter which filters spans and events based on a set of filter directives.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [log]
    /// filter = "mosaic=debug,info"
    /// ```
    #[serde(deserialize_with = "deserialize_env_filter")]
    pub filter: EnvFilter,
}

#[derive(Debug, Deserialize, Clone)]
/// Defines all the relevant API information and how to interact with the server.
pub struct APIConf {
    /// Defines the static IP address for the communication server.
    /// The communication server enables clients to interact via the grpc protocol.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// server_address = "[::]:8080"
    /// ```
    pub server_address: std::net::SocketAddr,
}

impl APIConf {
    pub fn addr_as_str(self) -> std::borrow::Cow<'static, str> {
        todo!()
    }
}

// https://serde.rs/impl-deserialize.html
fn deserialize_env_filter<'de, D>(deserializer: D) -> Result<EnvFilter, D::Error>
where
    D: Deserializer<'de>,
{
    struct EnvFilterVisitor;
    impl<'de> Visitor<'de> for EnvFilterVisitor {
        type Value = EnvFilter;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "check for valid tracing filter: \
            https://docs.rs/tracing-subscriber/0.2.6/tracing_subscriber/filter/struct.EnvFilter.html#directives")
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

#[derive(Debug, Validate, Deserialize)]
pub struct Conf {
    /// Defines all the relevant API information and how to interact with the server.
    pub api: APIConf,
    /// Defines the way the logging of the server is done via filter.
    pub log: LogConf,
}

impl Conf {
    /// Loads and validates the [`Conf`] via a configuration file.
    ///
    /// # Errors
    /// Fails when the loading of the configuration file or its validation failed.
    pub fn init_from_path(path: Option<impl AsRef<Path>>) -> Result<Self, ConfError> {
        let conf = Self::load(path)?;
        conf.validate()?;
        Ok(conf)
    }
    /// Constructs the [`Conf`] from external params
    ///
    /// # Errors
    pub fn init_from_params(api_server_address: Option<String>) -> Result<Self, ConfError> {
        let builder = if api_server_address.is_some() {
            Self::set_default().set_override(
                "api.server_address",
                ValueKind::String(api_server_address.unwrap()),
            )?
        } else {
            Self::set_default()
        };

        let conf = builder.build()?.try_deserialize()?;

        Ok(conf)
    }

    fn load(path: Option<impl AsRef<Path>>) -> Result<Self, ConfigError> {
        match path {
            None => Self::set_default().build()?.try_deserialize(),
            Some(path) => Self::set_default()
                .add_source(config::File::from(path.as_ref()))
                .build()?
                .try_deserialize(),
        }
    }

    fn set_default() -> config::ConfigBuilder<config::builder::DefaultState> {
        Config::builder()
            .set_default(
                "api.server_address",
                ValueKind::String("[::]:8080".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "log.filter",
                ValueKind::String("mosaic=debug,info".to_string()),
            )
            .unwrap_or_default()
    }
}
