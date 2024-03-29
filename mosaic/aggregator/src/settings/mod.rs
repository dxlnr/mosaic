//! Loading and validation of settings.
//!
//! Values defined in the configuration file can be overridden by environment variables. Examples of
//! configuration files can be found in the `configs/` directory located in the repository root.
//!
#[cfg(feature = "tls")]
use std::path::PathBuf;
use std::{fmt, path::Path};

use config::{Config, ConfigError, ValueKind};
use displaydoc::Display;
use redis::{ConnectionInfo, IntoConnectionInfo};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationErrors};

#[cfg(feature = "tls")]
use validator::ValidationError;

use mosaic_core::{
    mask::{BoundType, GroupType, MaskConfig, ModelType},
    model::{DataType, ModelConfig},
};

#[cfg(feature = "model-persistence")]
#[cfg_attr(docsrs, doc(cfg(feature = "model-persistence")))]
pub mod s3;
#[cfg(feature = "model-persistence")]
pub use self::{s3::RestoreSettings, s3::S3BucketsSettings, s3::S3Settings};

#[derive(Debug, Display, Error)]
/// An error related to loading and validation of settings.
pub enum SettingsError {
    /// Configuration loading failed: {0}.
    Loading(#[from] ConfigError),
    /// Validation failed: {0}.
    Validation(#[from] ValidationErrors),
}

#[derive(Debug, Validate, Deserialize)]
/// The combined settings.
///
/// Each section in the configuration file corresponds to the identically named settings field.
pub struct Settings {
    pub api: ApiSettings,
    pub protocol: ProtocolSettings,
    pub mask: MaskSettings,
    pub log: LoggingSettings,
    pub model: ModelSettings,
    #[validate]
    pub metrics: MetricsSettings,
    #[cfg(feature = "redis")]
    #[validate]
    pub redis: RedisSettings,
    #[cfg(feature = "model-persistence")]
    #[validate]
    pub s3: S3Settings,
    #[cfg(feature = "model-persistence")]
    #[validate]
    pub restore: RestoreSettings,
    #[serde(default)]
    pub trust_anchor: TrustAnchorSettings,
}

impl Settings {
    /// Loads and validates the settings via a configuration file.
    ///
    /// # Errors
    /// Fails when the loading of the configuration file or its validation failed.
    pub fn new(path: Option<impl AsRef<Path>>) -> Result<Self, SettingsError> {
        let settings: Settings = Self::load(path)?;
        settings.validate()?;
        Ok(settings)
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
                ValueKind::String("127.0.0.1:8080".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "api.tls_certificate",
                ValueKind::String("/app/ssl/tls.pem".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "api.tls_key",
                ValueKind::String("/app/ssl/tls.key".to_string()),
            )
            .unwrap_or_default()
            .set_default("protocol.training_rounds", ValueKind::I64(1))
            .unwrap_or_default()
            .set_default("protocol.participants", ValueKind::I64(1))
            .unwrap_or_default()
            .set_default("mask.group_type", ValueKind::String("Prime".to_string()))
            .unwrap_or_default()
            .set_default("mask.data_type", ValueKind::String("F32".to_string()))
            .unwrap_or_default()
            .set_default("mask.bound_type", ValueKind::String("B0".to_string()))
            .unwrap_or_default()
            .set_default("mask.model_type", ValueKind::String("M3".to_string()))
            .unwrap_or_default()
            // .set_default("model.length", ValueKind::I64(0))
            // .unwrap_or_default()
            .set_default("model.data_type", ValueKind::String("F32".to_string()))
            .unwrap_or_default()
            .set_default(
                "metrics.influxdb.url",
                ValueKind::String("http://127.0.0.1:8086".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "metrics.influxdb.db",
                ValueKind::String("metrics".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "redis.url",
                ValueKind::String("redis://127.0.0.1/".to_string()),
            )
            .unwrap_or_default()
            .set_default(
                "log.filter",
                ValueKind::String("mosaic=debug,info".to_string()),
            )
            .unwrap_or_default()
            .set_default("s3.access_key", ValueKind::String("".to_string()))
            .unwrap_or_default()
            .set_default("s3.secret_access_key", ValueKind::String("".to_string()))
            .unwrap_or_default()
            .set_default(
                "s3.region",
                ValueKind::Array(vec![
                    config::Value::new(None, ValueKind::String("minio".to_string())),
                    config::Value::new(
                        None,
                        ValueKind::String("http://localhost:9000".to_string()),
                    ),
                ]),
            )
            .unwrap_or_default()
            .set_default("restore.enable", ValueKind::Boolean(true))
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(
    feature = "tls",
    derive(Validate),
    validate(schema(function = "validate_api"))
)]
/// REST API settings.
///
/// Requires at least one of the following arguments if the `tls` feature is enabled:
/// - `tls_certificate` together with `tls_key` for TLS server authentication
// - `tls_client_auth` for TLS client authentication
pub struct ApiSettings {
    /// The address to which the REST API should be bound.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// server_address = "0.0.0.0:8080"
    /// # or
    /// server_address = "127.0.0.1:8080"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__API__SERVER_ADDRESS=127.0.0.1:8080
    /// ```
    pub server_address: std::net::SocketAddr,

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// The path to the server certificate to enable TLS server authentication. Leave this out to
    /// disable server authentication. If this is present, then `tls_key` must also be present.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// tls_certificate = path/to/tls/files/cert.pem
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__API__TLS_CERTIFICATE=path/to/tls/files/certificate.pem
    /// ```
    pub tls_certificate: Option<PathBuf>,

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// The path to the server private key to enable TLS server authentication. Leave this out to
    /// disable server authentication. If this is present, then `tls_certificate` must also be
    /// present.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// tls_key = path/to/tls/files/key.rsa
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__API__TLS_KEY=path/to/tls/files/key.rsa
    /// ```
    pub tls_key: Option<PathBuf>,

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// The path to the trust anchor to enable TLS client authentication. Leave this out to disable
    /// client authentication.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// tls_client_auth = path/to/tls/files/trust_anchor.pem
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__API__TLS_CLIENT_AUTH=path/to/tls/files/trust_anchor.pem
    /// ```
    pub tls_client_auth: Option<PathBuf>,
}

#[cfg(feature = "tls")]
impl ApiSettings {
    /// Checks API settings.
    fn validate_api(&self) -> Result<(), ValidationError> {
        match (&self.tls_certificate, &self.tls_key, &self.tls_client_auth) {
            (Some(_), Some(_), _) | (None, None, Some(_)) => Ok(()),
            _ => Err(ValidationError::new("invalid tls settings")),
        }
    }
}

/// A wrapper for validate derive.
#[cfg(feature = "tls")]
fn validate_api(s: &ApiSettings) -> Result<(), ValidationError> {
    s.validate_api()
}

#[derive(Debug, Deserialize, Clone)]
/// Hyperparameter controlling the Federated Learning training process.
pub struct ProtocolSettings {
    /// Defines the number of training rounds that will be performed.
    ///
    /// # Example
    ///
    /// **TOML**
    /// ```text
    /// [process]
    /// training_rounds = 25
    /// ```
    pub training_rounds: u32,
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
}

impl std::fmt::Display for ProtocolSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[process]\n    training_rounds: {}\n    participants: {}\n",
            self.training_rounds, self.participants
        )
    }
}

#[derive(Debug, Validate, Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
/// Masking settings.
pub struct MaskSettings {
    /// The order of the finite group.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// group_type = "Integer"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__MASK__GROUP_TYPE=Integer
    /// ```
    pub group_type: GroupType,

    /// The data type of the numbers to be masked.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// data_type = "F32"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__MASK__DATA_TYPE=F32
    /// ```
    pub data_type: DataType,

    /// The bounds of the numbers to be masked.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// bound_type = "B0"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__MASK__BOUND_TYPE=B0
    /// ```
    pub bound_type: BoundType,

    /// The maximum number of models to be aggregated.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// model_type = "M3"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__MASK__MODEL_TYPE=M3
    /// ```
    pub model_type: ModelType,
}

impl From<MaskSettings> for MaskConfig {
    fn from(
        MaskSettings {
            group_type,
            data_type,
            bound_type,
            model_type,
        }: MaskSettings,
    ) -> MaskConfig {
        MaskConfig {
            group_type,
            data_type,
            bound_type,
            model_type,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
/// Model settings.
pub struct ModelSettings {
    // /// The expected length of the model. The model length corresponds to the number of elements.
    // /// This value is used to validate the uniform length of the submitted models/masks.
    // ///
    // /// # Examples
    // ///
    // /// **TOML**
    // /// ```text
    // /// [model]
    // /// length = 100
    // /// ```
    // ///
    // /// **Environment variable**
    // /// ```text
    // /// MOSAIC__MODEL__LENGTH=100
    // /// ```
    // pub length: usize,
    /// The data type of the model.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// data_type = "F32"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__MODEL__DATA_TYPE=F32
    /// ```
    pub data_type: DataType,
}

impl From<ModelSettings> for ModelConfig {
    fn from(ModelSettings { data_type }: ModelSettings) -> ModelConfig {
        ModelConfig { data_type }
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Metrics settings.
pub struct MetricsSettings {
    #[validate]
    /// Settings for the InfluxDB backend.
    pub influxdb: InfluxSettings,
}

#[derive(Debug, Deserialize, Validate)]
/// InfluxDB settings.
pub struct InfluxSettings {
    #[validate(url)]
    /// The URL where InfluxDB is running.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [metrics.influxdb]
    /// url = "http://localhost:8086"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__METRICS__INFLUXDB__URL=http://localhost:8086
    /// ```
    pub url: String,

    /// The InfluxDB database name.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [metrics.influxdb]
    /// db = "test"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__METRICS__INFLUXDB__DB=test
    /// ```
    pub db: String,
}

#[derive(Debug, Deserialize)]
/// Redis settings.
pub struct RedisSettings {
    /// The URL where Redis is running.
    ///
    /// The format of the URL is `redis://[<username>][:<passwd>@]<hostname>[:port][/<db>]`.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [redis]
    /// url = "redis://127.0.0.1/"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__REDIS__URL=redis://127.0.0.1/
    /// ```
    #[serde(deserialize_with = "deserialize_redis_url")]
    pub url: ConnectionInfo,
}

fn deserialize_redis_url<'de, D>(deserializer: D) -> Result<ConnectionInfo, D::Error>
where
    D: Deserializer<'de>,
{
    struct ConnectionInfoVisitor;

    impl<'de> Visitor<'de> for ConnectionInfoVisitor {
        type Value = ConnectionInfo;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "redis://[<username>][:<passwd>@]<hostname>[:port][/<db>]"
            )
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .into_connection_info()
                .map_err(|_| de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_str(ConnectionInfoVisitor)
}

#[derive(Debug, Default, Deserialize, Validate)]
/// Trust anchor settings.
pub struct TrustAnchorSettings {}

#[derive(Debug, Deserialize)]
/// Logging settings.
pub struct LoggingSettings {
    /// A comma-separated list of logging directives. More information about logging directives
    /// can be found [here].
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [log]
    /// filter = "info"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// MOSAIC__LOG__FILTER=info
    /// ```
    ///
    /// [here]: https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber/filter/struct.EnvFilter.html#directives
    #[serde(deserialize_with = "deserialize_env_filter")]
    pub filter: EnvFilter,
}

fn deserialize_env_filter<'de, D>(deserializer: D) -> Result<EnvFilter, D::Error>
where
    D: Deserializer<'de>,
{
    struct EnvFilterVisitor;

    impl<'de> Visitor<'de> for EnvFilterVisitor {
        type Value = EnvFilter;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a valid tracing filter directive: https://docs.rs/tracing-subscriber/0.2.6/tracing_subscriber/filter/struct.EnvFilter.html#directives")
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     impl Default for PetSettings {
//         fn default() -> Self {
//             Self {
//                 sum: PetSettingsSum {
//                     prob: 0.01,
//                     count: PetSettingsCount { min: 10, max: 100 },
//                     time: PetSettingsTime {
//                         min: 0,
//                         max: 604800,
//                     },
//                 },
//                 update: PetSettingsUpdate {
//                     prob: 0.1,
//                     count: PetSettingsCount {
//                         min: 100,
//                         max: 10000,
//                     },
//                     time: PetSettingsTime {
//                         min: 0,
//                         max: 604800,
//                     },
//                 },
//                 sum2: PetSettingsSum2 {
//                     count: PetSettingsCount { min: 10, max: 100 },
//                     time: PetSettingsTime {
//                         min: 0,
//                         max: 604800,
//                     },
//                 },
//             }
//         }
//     }

//     impl Default for MaskSettings {
//         fn default() -> Self {
//             Self {
//                 group_type: GroupType::Prime,
//                 data_type: DataType::F32,
//                 bound_type: BoundType::B0,
//                 model_type: ModelType::M3,
//             }
//         }
//     }

//     #[test]
//     fn test_settings_new() {
//         assert!(Settings::new(Some("../../configs/config.toml")).is_ok());
//         assert!(Settings::new(Some("")).is_err());
//     }

//     #[test]
//     fn test_validate_pet() {
//         assert!(PetSettings::default().validate_pet().is_ok());
//     }

//     #[test]
//     fn test_validate_pet_counts() {
//         assert_eq!(SUM_COUNT_MIN, 1);
//         assert_eq!(UPDATE_COUNT_MIN, 3);

//         let mut pet = PetSettings::default();
//         pet.sum.count.min = 0;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum.count.min = 11;
//         pet.sum.count.max = 10;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.update.count.min = 2;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.update.count.min = 11;
//         pet.update.count.max = 10;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum2.count.min = 0;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum2.count.min = 11;
//         pet.sum2.count.max = 10;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum2.count.min = 11;
//         pet.sum.count.max = 10;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum2.count.max = 11;
//         pet.sum.count.max = 10;
//         assert!(pet.validate().is_err());
//     }

//     #[test]
//     fn test_validate_pet_times() {
//         let mut pet = PetSettings::default();
//         pet.sum.time.min = 2;
//         pet.sum.time.max = 1;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.update.time.min = 2;
//         pet.update.time.max = 1;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum2.time.min = 2;
//         pet.sum2.time.max = 1;
//         assert!(pet.validate().is_err());
//     }

//     #[test]
//     fn test_validate_pet_probabilities() {
//         let mut pet = PetSettings::default();
//         pet.sum.prob = 0.;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.sum.prob = 1.;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.update.prob = 0.;
//         assert!(pet.validate().is_err());

//         let mut pet = PetSettings::default();
//         pet.update.prob = 1. + f64::EPSILON;
//         assert!(pet.validate().is_err());
//     }

//     #[cfg(feature = "tls")]
//     #[test]
//     fn test_validate_api() {
//         let server_address = ([0, 0, 0, 0], 0).into();
//         let some_path = Some(std::path::PathBuf::new());

//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: some_path.clone(),
//             tls_key: some_path.clone(),
//             tls_client_auth: some_path.clone(),
//         }
//         .validate()
//         .is_ok());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: some_path.clone(),
//             tls_key: some_path.clone(),
//             tls_client_auth: None,
//         }
//         .validate()
//         .is_ok());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: None,
//             tls_key: None,
//             tls_client_auth: some_path.clone(),
//         }
//         .validate()
//         .is_ok());

//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: some_path.clone(),
//             tls_key: None,
//             tls_client_auth: some_path.clone(),
//         }
//         .validate()
//         .is_err());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: None,
//             tls_key: some_path.clone(),
//             tls_client_auth: some_path.clone(),
//         }
//         .validate()
//         .is_err());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: some_path.clone(),
//             tls_key: None,
//             tls_client_auth: None,
//         }
//         .validate()
//         .is_err());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: None,
//             tls_key: some_path,
//             tls_client_auth: None,
//         }
//         .validate()
//         .is_err());
//         assert!(ApiSettings {
//             server_address,
//             tls_certificate: None,
//             tls_key: None,
//             tls_client_auth: None,
//         }
//         .validate()
//         .is_err());
//     }
// }
