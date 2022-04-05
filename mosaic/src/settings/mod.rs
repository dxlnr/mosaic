//! Settings module which allows to manipulate the server from the outside.
//! 
//! Important settings regarding the training process can be configured using **.toml**.
//! Therefore this module serves as an entry point to define specialised Federated Learning training processes without
//! touching the code.

use std::{fmt, path::Path};

use config::{Config, ConfigError};
use displaydoc::Display;
use s3::region::Region;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationErrors};

use crate::core::{aggregator::traits::Scheme, model::DataType};

#[derive(Debug, Display, Error)]
/// An error related to loading and validation of settings.
pub enum SettingsError {
    /// Loading configuration file failed: {0}.
    Loading(#[from] ConfigError),
    /// Validation failed: {0}.
    Validation(#[from] ValidationErrors),
    /// Parsing error
    ParsingError,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Settings {
    /// Defines all the relevant API information and how to interact with the server.
    pub api: APISettings,
    pub job: JobSettings,
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
pub struct JobSettings {
    pub job_id: u32,
    pub job_token: String,
    pub route: String,
}

#[derive(Debug, Deserialize, Clone)]
/// Defines all the relevant API information and how to interact with the server.
pub struct APISettings {
    /// Defines the static IP address for the communication server.
    /// The communication server enables clients to interact via the grpc protocol.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// address = "127.0.0.1:8081"
    /// ```
    pub address: std::net::SocketAddr,
    /// Defines the Rest API where the server exposes data from the running process.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// rest_api = "127.0.0.1:8000"
    /// ```
    pub rest_api: std::net::SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelSettings {
    pub length: usize,
    /// The DataType the model is encoded with.
    ///
    /// Options that are available: F64 & F32
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [model]
    /// data_type = "F32"
    /// ```
    pub data_type: DataType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessSettings {
    /// Defines the number of training rounds that will be performed.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [process]
    /// rounds = 25
    /// ```
    pub rounds: u32,
    /// Sets the number of participants one global epoch should at least contain.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [process]
    /// participants = 10
    /// ```
    pub participants: u32,
    /// Sets the aggregation strategy as key ingredient for Federated Learning.
    ///
    /// Options that are available:
    /// * **FedAvg**: Most basic algorithm performing iterative model averaging.
    ///
    ///     This method is based on local models using stochastic gradient descent (SGD) for optimization.
    ///     SGD can be applied naively to the federated optimization problem,
    ///     where a single batch gradient calculation is done per round of communication.
    ///     This approach is computationally efficient, but requires very large numbers of rounds of training to produce good models.
    ///
    ///     For more information check out [McMahan et al. Communication-Efficient Learning of Deep Networks from Decentralized Data](https://arxiv.org/abs/1602.05629)
    ///
    /// * **FedAdaGrad**: Based on FedOpt and one of the federated versions of adaptive optimizers.
    ///
    ///     For more information check out [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/abs/2003.00295)
    ///
    /// * **FedAdam**: Based on FedOpt and one of the federated versions of adaptive optimizers.
    ///
    ///     For more information check out [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/abs/2003.00295)
    ///
    /// * **FedYogi**: Based on FedOpt and one of the federated versions of adaptive optimizers.
    ///
    ///     For more information check out [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/abs/2003.00295)
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [process]
    /// strategy = "FedAvg"
    /// ```
    pub strategy: Scheme,
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
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// region = ["minio", "http://localhost:9000"]
    /// ```
    #[serde(deserialize_with = "deserialize_s3_region")]
    pub region: Region,
    /// Bucket name that should be targeted.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// bucket = "mnist-cnn-testing"
    /// ```
    pub bucket: String,
    /// Name of the overall global model.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// global_model = "cnn_global"
    /// ```
    pub global_model: String,
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
    impl<'de> Visitor<'de> for S3Visitor {
        type Value = Region;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "[<region>, <endpoint>] -> [\"minio\", \"http://localhost:9000\"]"
            )
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
            let region: String = seq.next_element()?.ok_or_else(|| {
                de::Error::custom(
                    "No region in [s3].region specified. region = [<region>, <endpoint>]",
                )
            })?;

            let endpoint: String = seq.next_element()?.ok_or_else(|| {
                de::Error::custom(
                    "No endpoint in [s3].region specified. region = [<region>, <endpoint>]",
                )
            })?;

            Ok(Region::Custom { region, endpoint })
        }
    }
    deserializer.deserialize_any(S3Visitor)
}
