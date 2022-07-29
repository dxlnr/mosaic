//! S3 (Simple Storage Service) connection for storing objects regarding the process through a web service interface.
//!
//! Storage Service is handled by [MinIO](https://github.com/minio/minio) which is also compatible with AWS.

use async_trait::async_trait;
use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

use s3::{bucket::Bucket, creds::Credentials, BucketConfiguration};

use crate::{
    core::model::{DataType, Model},
    db::traits::{ModelStorage, StorageResult},
    settings::S3Settings,
};

#[derive(Debug, Display, Error)]
pub enum StorageError {
    /// Failed to create bucket: {0}.
    CreateBucket(String),
    /// Failed to download data: {0}.
    DownloadData(anyhow::Error),
    /// Connection to MinIO failed: {0}.
    ConnectionError(&'static str),
    /// Initialization of Client failed: {0}.
    InitClient(String),
    /// Failed to upload data: {0}.
    UploadError(anyhow::Error),
}

#[derive(Clone)]
pub struct SettingsParams {
    pub global_model_name: String,
}

impl SettingsParams {
    pub fn new(global_model_name: &str) -> Self {
        Self {
            global_model_name: global_model_name.to_string(),
        }
    }
}

type ClientResult<T> = Result<T, StorageError>;

#[derive(Clone)]
pub struct S3Client {
    bucket: Arc<Bucket>,
    params: SettingsParams,
}

impl S3Client {
    /// Creates a new S3 client. The client instantiates, creates and maintains buckets for storing all
    /// the data created during the process.
    ///
    pub async fn new(s3_settings: S3Settings) -> ClientResult<Self> {
        let credentials = Credentials::new(
            Some(&s3_settings.access_key),
            Some(&s3_settings.secret_access_key),
            None,
            None,
            None,
        )
        .map_err(|_| StorageError::InitClient("Setting credentials failed".to_string()))?;

        let bucket = Bucket::new_with_path_style(
            &s3_settings.bucket.to_string(),
            s3_settings.region,
            credentials,
        )
        .map_err(|_| {
            StorageError::InitClient(format!(
                "Unable to instantiate bucket {}.",
                s3_settings.bucket
            ))
        })?;

        Ok(Self {
            bucket: Arc::new(bucket),
            params: SettingsParams::new(&s3_settings.global_model),
        })
    }

    /// Downloads the content of a requested object.
    async fn download_object(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        let (data, code) = self
            .bucket
            .get_object(key)
            .await
            .map_err(StorageError::DownloadData)?;

        match code {
            200 => Ok(Some(data)),
            _ => Ok(None),
        }
    }

    /// Uploads an object with the given key to the given bucket.
    async fn upload_object(&self, key: &str, data: &[u8]) -> ClientResult<()> {
        let (_, _code) = self
            .bucket
            .put_object(key, data)
            .await
            .map_err(StorageError::UploadError)?;
        Ok(())
    }

    /// Checks if a connection to the storage bucket can be established.
    pub async fn check_conn(&self) -> ClientResult<()> {
        self.bucket.head_object("/").await.map_err(|_| {
            StorageError::ConnectionError(
                "Unable to establish connection to MinIO. Learning proceeds without external storage",
            )
        })?;

        Ok(())
    }

    /// Returns bucket and or creates a new bucket with the given bucket name.
    pub async fn create_bucket(self) -> ClientResult<()> {
        info!(
            "Instantiating S3 Bucket ['{}'] on {}",
            &self.bucket.name(),
            &self.bucket.region()
        );

        Bucket::create_with_path_style(
            &self.bucket.name(),
            self.bucket.region(),
            self.bucket.credentials().clone(),
            BucketConfiguration::default(),
        )
        .await
        .map_err(|_| StorageError::CreateBucket(self.bucket.name()))?;

        Ok(())
    }
}

#[async_trait]
impl ModelStorage for S3Client {
    /// Downloads the global model from bucket.
    async fn get_global_model(&mut self) -> StorageResult<Option<Model>> {
        let data = self.download_object(&self.params.global_model_name).await?;

        let mut model: Model = Default::default();
        if let Some(b) = data {
            model.deserialize(b, &DataType::F32)
        };

        if model.is_empty() {
            warn!(
                "No pretrained global model found in S3 Bucket ['{}'].",
                &self.bucket.name()
            );
        }
        Ok(Some(model))
    }
    /// Uploads the global model to bucket.
    async fn set_global_model(&mut self, data: &[u8]) -> StorageResult<()> {
        self.upload_object(&self.params.global_model_name, data)
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Noop;
