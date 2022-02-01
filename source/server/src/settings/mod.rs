use std::{fmt, path::Path};

use config::{Config, ConfigError};
use displaydoc::Display;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use s3::region::Region;
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationErrors};

use crate::engine::model::DataType;

#[derive(Debug, Display, Error)]
/// An error related to loading and validation of settings.
pub enum SettingsError {
    /// Loading configuration file failed: {0}.
    Loading(#[from] ConfigError),
    /// Validation failed: {0}.
    Validation(#[from] ValidationErrors),
}

#[derive(Debug, Validate, Deserialize)]
pub struct Settings {
    pub api: APISettings,
    pub model: ModelSettings,
    pub process: ProcessSettings,
    pub log: LogSettings,
    pub s3: S3Settings,
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

#[derive(Debug, Deserialize, Clone)]
pub struct S3Settings {
    /// Defines the user (access) key.
    pub access_key: String,
    /// Defines the user secret key (password)
    pub secret_access_key: String,
    /// The Regional AWS endpoint.
    /// The region is specified using the [Region code](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints)
    /// 
    /// For MinIO this has to be specified in a custom manner.
    /// 
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// region = ["minio", "http://localhost:9001"]
    /// ```
    #[serde(deserialize_with = "deserialize_s3_region")]
    pub region: Region,
    /// Bucket name that should be targeted.
    pub bucket: String,
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
            write!(formatter, "check for valid tracing filter: https://docs.rs/tracing-subscriber/0.2.6/tracing_subscriber/filter/struct.EnvFilter.html#directives")
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

fn deserialize_s3_region<'de, D>(deserializer: D) -> Result<Region, D::Error>
where
    D: Deserializer<'de>,
{
    struct S3Visitor;
    impl <'de> Visitor<'de> for S3Visitor {
        type Value = Region;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "[<region>, <endpoint>] -> [\"minio\", \"http://localhost:9000\"]")
        }
        
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut seq = value.split_whitespace();

            let region: &str = seq
                .next()
                .ok_or_else(|| de::Error::custom("No region specified."))?;
            let endpoint: Option<&str> = seq.next();

            match (region, endpoint) {
                (region, Some(endpoint)) => Ok(Region::Custom {
                    region: region.to_string(),
                    endpoint: endpoint.to_string(),
                }),
                (region, None) => region.parse().map_err(de::Error::custom),
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {   
            let region: String = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("No region in [s3].region specified. region = [<region>, <endpoint>]"))?;

            let endpoint: String = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("No endpoint in [s3].region specified. region = [<region>, <endpoint>]"))?;

            Ok(Region::Custom { region, endpoint })
        }
    }
    deserializer.deserialize_any(S3Visitor)
}