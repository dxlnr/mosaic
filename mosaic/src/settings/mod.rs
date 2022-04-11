//! Settings module which allows to manipulate the server from the outside.
//!
//! Important settings regarding the training process can be configured using **.toml**.
//! Therefore this module serves as an entry point to define specialised Federated Learning training processes without
//! touching the code.

use std::{
    fmt,
    path::{Path, PathBuf},
};

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
    /// Defines information regarding the specific job that is performed.
    pub job: JobSettings,
    /// Settings regarding the model that is trained.
    pub model: ModelSettings,
    /// Hyperparameter regarding the Federated Learning training process.
    pub process: ProcessSettings,
    /// Defines the way the logging of the server is done via filter.
    pub log: LogSettings,
    /// Handling storage regarding the training process.
    pub s3: S3Settings,
}

impl Settings {
    /// Loads and validates the settings via a configuration file.
    ///
    /// # Errors
    /// Fails when the loading of the configuration file or its validation failed.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SettingsError> {
        // let mut settings = Default::default();
        let settings = Self::load(path)?;
        settings.validate()?;
        Ok(settings)
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let _default_settings: Settings = Default::default();

        Config::builder()
            .add_source(config::File::from(
                PathBuf::from("mosaic/src/settings/default.toml").as_ref(),
            ))
            .add_source(config::File::from(path.as_ref()))
            .build()?
            .try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        let api = APISettings {
            address: std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                8080,
            ),
            rest_api: std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                8000,
            ),
        };
        let job = JobSettings {
            job_id: 0,
            job_token: "modalic".to_string(),
            route: "http://localhost:5000/api/communication_rounds/create_communication_round"
                .to_string(),
        };
        let model = ModelSettings {
            data_type: DataType::F32,
            precision: 53,
        };
        let process = ProcessSettings {
            rounds: 0,
            participants: 0,
            strategy: Scheme::FedAvg,
        };
        let log = Default::default();
        let s3 = S3Settings {
            access_key: "modalic".to_string(),
            secret_access_key: "modalic".to_string(),
            region: Region::Custom {
                region: "minio".to_string(),
                endpoint: "http://localhost:9000".to_string(),
            },
            bucket: "modalic-testing".to_string(),
            global_model: "global_model".to_string(),
        };
        Self {
            api,
            job,
            model,
            process,
            log,
            s3,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
/// Defines the way the logging of the server is done via filter.
pub struct LogSettings {
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
/// General Settings regarding the Job. Only important if used in conjunction with the Modalic backend.
pub struct JobSettings {
    /// Defines the Job ID which is unique.
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
    /// address = "127.0.0.1:8080"
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
/// Settings regarding the model that is trained during runtime.
pub struct ModelSettings {
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
    /// Sets the precision the Float values of the model are encoded with.
    /// The precision has to be set during construction of the Float variable.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [model]
    /// precision = 53
    /// ```
    pub precision: u32,
}

#[derive(Debug, Deserialize, Clone)]
/// Hyperparameter regarding the Federated Learning training process.
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
/// Defines the connection to external storage drive by handling the access points to MinIO and
/// sets the information regarding the stored elements.
///
///
/// Server will work also without setting up the connection to MinIO or AWS services.
pub struct S3Settings {
    /// Defines the user (access) key. Essentially the user name.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// access_key = "modalic"
    /// ```
    pub access_key: String,
    /// Defines the user secret key (password). This is sensitive information and you might not want to state that directly
    /// within the config but use a *.env* for that.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// secret_access_key = "12345678"
    /// ```
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
