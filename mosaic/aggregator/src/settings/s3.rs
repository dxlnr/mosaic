//! S3 settings.
use std::fmt;

use fancy_regex::Regex;
use rusoto_core::Region;
use serde::{
    de::{self, value, Deserializer, Visitor},
    Deserialize,
};
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct S3Settings {
    /// The [access key ID](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html).
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// access_key = "AKIAIOSFODNN7EXAMPLE"
    /// ```
    pub access_key: String,

    /// The [secret access key](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html).
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
    /// ```
    pub secret_access_key: String,

    /// The Regional AWS endpoint.
    ///
    /// The region is specified using the [Region code](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints)
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// region = ["eu-west-1"]
    /// ```
    ///
    /// To connect to AWS-compatible services such as Minio, you need to specify a custom region.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3]
    /// region = ["minio", "http://localhost:8000"]
    /// ```
    #[serde(deserialize_with = "deserialize_s3_region")]
    pub region: Region,
    #[validate]
    #[serde(default)]
    pub buckets: S3BucketsSettings,
}

#[derive(Debug, Validate, Deserialize)]
/// S3 buckets settings.
pub struct S3BucketsSettings {
    /// The bucket name in which the global models are stored.
    /// Defaults to `global-models`.
    ///
    /// Please follow the [rules for bucket naming](https://docs.aws.amazon.com/AmazonS3/latest/dev/BucketRestrictions.html)
    /// when creating the name.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [s3.buckets]
    /// global_models = "global-models"
    /// ```
    #[validate(custom = "validate_s3_bucket_name")]
    pub global_models: String,
}

// Default value for the global models bucket
impl Default for S3BucketsSettings {
    fn default() -> Self {
        Self {
            global_models: String::from("global-models"),
        }
    }
}

// Validates the bucket name
// [Rules for AWS bucket naming](https://docs.aws.amazon.com/AmazonS3/latest/dev/BucketRestrictions.html)
fn validate_s3_bucket_name(bucket_name: &str) -> Result<(), ValidationError> {
    // https://stackoverflow.com/questions/50480924/regex-for-s3-bucket-name#comment104807676_58248645
    // I had to use fancy_regex here because the std regex does not support `look-around`
    let re =
        Regex::new(r"(?!^(\d{1,3}\.){3}\d{1,3}$)(^[a-z0-9]([a-z0-9-]*(\.[a-z0-9])?)*$(?<!\-))")
            .unwrap();
    match re.is_match(bucket_name) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ValidationError::new("invalid bucket name\n See here: https://docs.aws.amazon.com/AmazonS3/latest/dev/BucketRestrictions.html")),
        // something went wrong with the regex engine
        Err(_) => Err(ValidationError::new("can not validate bucket name")),
    }
}

// A small wrapper to support the list type for environment variable values.
// config-rs always converts a environment variable value to a string
// https://github.com/mehcode/config-rs/blob/master/src/env.rs#L114 .
// Strings however, are not supported by the deserializer of rusoto_core::Region (only sequences).
// Therefore we use S3RegionVisitor to implement `visit_str` and thus support
// the deserialization of rusoto_core::Region from strings.
fn deserialize_s3_region<'de, D>(deserializer: D) -> Result<Region, D::Error>
where
    D: Deserializer<'de>,
{
    struct S3RegionVisitor;

    impl<'de> Visitor<'de> for S3RegionVisitor {
        type Value = Region;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sequence of \"name Optional<endpoint>\"")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut seq = value.split_whitespace();

            let name: &str = seq
                .next()
                .ok_or_else(|| de::Error::custom("region is missing name"))?;
            let endpoint: Option<&str> = seq.next();

            match (name, endpoint) {
                (name, Some(endpoint)) => Ok(Region::Custom {
                    name: name.to_string(),
                    endpoint: endpoint.to_string(),
                }),
                (name, None) => name.parse().map_err(de::Error::custom),
            }
        }

        // delegate the call for sequences to the deserializer of rusoto_core::Region
        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(S3RegionVisitor)
}

#[derive(Debug, Deserialize, Validate)]
/// Restore settings.
pub struct RestoreSettings {
    /// If set to `false`, the restoring of coordinator state is prevented.
    /// Instead, the state is reset and the coordinator is started with the
    /// settings of the configuration file.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [restore]
    /// enable = true
    /// ```
    pub enable: bool,
}

