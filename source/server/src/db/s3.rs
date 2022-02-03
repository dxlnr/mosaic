//! S3 connection for storing.

use async_trait::async_trait;
use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use s3::{bucket::Bucket, creds::Credentials, BucketConfiguration};

use crate::{
    db::traits::{ModelStorage, StorageResult},
    engine::model::Model,
    settings::S3Settings,
};

use crate::engine::model::DataType;

#[derive(Debug, Display, Error)]
pub enum StorageError {
    /// Failed to create bucket: {0}.
    CreateBucket(String),
    /// Failed to download some data: {0}.
    DownloadData(anyhow::Error),
}

type ClientResult<T> = Result<T, StorageError>;

#[derive(Clone)]
pub struct Client {
    bucket: Arc<Bucket>,
}

impl Client {
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
    async fn download_object(&self, key: &str) -> ClientResult<Vec<u8>> {
        let (data, _) = self
            .bucket
            .get_object(key)
            .await
            .map_err(StorageError::DownloadData)?;
        Ok(data)
    }

    // Uploads an object with the given key to the given bucket.
    // async fn upload_object() {
    //     todo!()
    // }

    // Creates a new bucket with the given bucket name.
    pub async fn create_bucket(self) -> ClientResult<()> {
        info!(
            "Instantiating S3 Bucket ['{}'] on {}",
            &self.bucket.name(),
            &self.bucket.region()
        );
        let (_, _code) = self
            .bucket
            .head_object("/")
            .await
            .map_err(|_| StorageError::CreateBucket(self.bucket.name()))?;

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
impl ModelStorage for Client {
    async fn get_global_model(&mut self, key: &str) -> StorageResult<Option<Model>> {
        let data = self.download_object(key).await?;
        println!("{:?}", &data.len());
        let mut model: Model = Default::default();
        model.deserialize(data, &DataType::F32);
        Ok(Some(model))
    }
}

#[derive(Clone)]
pub struct Noop;
