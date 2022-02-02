//! S3 connection for storing.

use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;

use s3::{bucket::Bucket, BucketConfiguration, creds::Credentials};

use crate::settings::S3Settings;

#[derive(Debug, Display, Error)]
pub enum StorageError {
    /// Failed to create bucket: {0}.
    CreateBucket(String),
}

type ClientResult<T> = Result<T, StorageError>;

#[derive(Clone)]
pub struct Client {
    bucket: Arc<Bucket>,
}

impl Client {
    /// Creates a new S3 client. The client creates and maintains buckets for storing all
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
        .map_err(|_| StorageError::CreateBucket(s3_settings.bucket.to_string()))?;

        let bucket = Bucket::new_with_path_style(
            &s3_settings.bucket.to_string(),
            s3_settings.region,
            credentials,
        )
        .map_err(|_| StorageError::CreateBucket(s3_settings.bucket.to_string()))?;

        Ok(Self {
            bucket: Arc::new(bucket),
        })
    }

    // Downloads the content of a requested object.
    async fn download_object() {
        todo!()
    }

    // Uploads an object with the given key to the given bucket.
    async fn upload_object() {
        todo!()
    }

    // Creates a new bucket with the given bucket name.
    pub async fn create_bucket(self) -> ClientResult<()> {
        let (_, code) = self.bucket.head_object("/").await.map_err(|_| StorageError::CreateBucket(self.bucket.name()))?;

        Bucket::create_with_path_style(&self.bucket.name(), self.bucket.region(), self.bucket.credentials().clone(),BucketConfiguration::default()).await.map_err(|_| StorageError::CreateBucket(self.bucket.name()))?;
        Ok(())
    }
}
