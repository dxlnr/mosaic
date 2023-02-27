use std::sync::Arc;

use async_trait::async_trait;
use displaydoc::Display;
use http::StatusCode;
use rusoto_core::{credential::StaticProvider, request::TlsError, HttpClient, RusotoError};
use rusoto_s3::{
    CreateBucketError,
    CreateBucketOutput,
    CreateBucketRequest,
    DeleteObjectsError,
    GetObjectError,
    GetObjectOutput,
    GetObjectRequest,
    HeadBucketError,
    HeadBucketRequest,
    ListObjectsV2Error,
    PutObjectError,
    PutObjectOutput,
    PutObjectRequest,
    S3Client,
    StreamingBody,
    S3,
};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tracing::debug;

use crate::{
    settings::{S3BucketsSettings, S3Settings},
    storage::{ModelStorage, StorageResult},
};
use mosaic_core::{common::RoundSeed, mask::Model};

type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Display, Error)]
pub enum ClientError {
    /// Failed to create bucket: {0}.
    CreateBucket(#[from] RusotoError<CreateBucketError>),
    /// Failed to get object: {0}.
    GetObject(#[from] RusotoError<GetObjectError>),
    /// Failed to put object: {0}.
    PutObject(#[from] RusotoError<PutObjectError>),
    /// Failed to list objects: {0}.
    ListObjects(#[from] RusotoError<ListObjectsV2Error>),
    /// Failed to delete objects: {0}.
    DeleteObjects(#[from] RusotoError<DeleteObjectsError>),
    /// Failed to dispatch: {0}.
    Dispatcher(#[from] TlsError),
    /// Failed to serialize: {0}.
    Serialization(bincode::Error),
    /// Failed to deserialize: {0}.
    Deserialization(bincode::Error),
    /// Response contains no body.
    NoBody,
    /// Failed to download body: {0}.
    DownloadBody(std::io::Error),
    /// Object {0} already exists.
    ObjectAlreadyExists(String),
    /// Storage not ready: {0}.
    NotReady(RusotoError<HeadBucketError>),
}

#[derive(Clone)]
pub struct Client {
    buckets: Arc<S3BucketsSettings>,
    client: S3Client,
}

impl Client {
    /// Creates a new S3 client. The client creates and maintains one bucket for storing global models.
    ///
    /// To connect to AWS-compatible services such as Minio, you need to specify a custom region.
    /// ```
    /// use rusoto_core::Region;
    ///
    /// let region = Region::Custom {
    ///     name: String::from("minio"),
    ///     endpoint: String::from("http://127.0.0.1:9000"), // URL of minio
    /// };
    ///
    /// let s3_settings = S3Settings {
    ///     region,
    ///     access_key: String::from("minio"),
    ///     secret_access_key: String::from("minio123"),
    ///     buckets: S3BucketsSettings {
    ///         global_models: String::from("global-models"),
    ///     },
    /// };
    ///
    /// let store = Client::new(s3_settings).unwrap();
    /// ```
    pub fn new(settings: S3Settings) -> ClientResult<Self> {
        let credentials_provider =
            StaticProvider::new_minimal(settings.access_key, settings.secret_access_key);

        let dispatcher = HttpClient::new()?;
        Ok(Self {
            buckets: Arc::new(settings.buckets),
            client: S3Client::new_with(dispatcher, credentials_provider, settings.region),
        })
    }

    /// Creates the `global models` bucket.
    /// This method does not fail if the bucket already exists or is already owned by you.
    pub async fn create_global_models_bucket(&self) -> ClientResult<()> {
        debug!("create {} bucket", &self.buckets.global_models);
        match self.create_bucket(&self.buckets.global_models).await {
            Ok(_)
            | Err(RusotoError::Service(CreateBucketError::BucketAlreadyExists(_)))
            | Err(RusotoError::Service(CreateBucketError::BucketAlreadyOwnedByYou(_))) => Ok(()),
            Err(err) => Err(ClientError::from(err)),
        }
    }

    // Downloads the content of the given object.
    async fn download_object_body(object: GetObjectOutput) -> ClientResult<Vec<u8>> {
        let mut body = Vec::new();
        object
            .body
            .ok_or(ClientError::NoBody)?
            .into_async_read()
            .read_to_end(&mut body)
            .await
            .map_err(ClientError::DownloadBody)?;
        Ok(body)
    }

    // Fetches the metadata of the object with the given key from the given bucket.
    async fn fetch_object_meta(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectOutput, RusotoError<GetObjectError>> {
        // If an object does not exist, S3 / Minio will return an error
        let req = GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        };
        self.client.get_object(req).await
    }

    // Uploads an object with the given key to the given bucket.
    async fn upload_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
    ) -> Result<PutObjectOutput, RusotoError<PutObjectError>> {
        let req = PutObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            body: Some(StreamingBody::from(data)),
            ..Default::default()
        };
        self.client.put_object(req).await
    }

    // Creates a new bucket with the given bucket name.
    async fn create_bucket(
        &self,
        bucket: &str,
    ) -> Result<CreateBucketOutput, RusotoError<CreateBucketError>> {
        let req = CreateBucketRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };
        self.client.create_bucket(req).await
    }
}

#[async_trait]
impl ModelStorage for Client {
    async fn set_global_model(
        &mut self,
        round_id: u64,
        round_seed: &RoundSeed,
        global_model: &Model,
    ) -> StorageResult<String> {
        let id = Self::create_global_model_id(round_id, round_seed);

        debug!("upload global model: {}", id);
        let output = self
            .fetch_object_meta(&self.buckets.global_models, &id)
            .await;
        if output.is_ok() {
            return Err(anyhow::anyhow!(ClientError::ObjectAlreadyExists(
                id.to_string()
            )));
        };

        let data = bincode::serialize(global_model).map_err(ClientError::Serialization)?;
        self.upload_object(&self.buckets.global_models, &id, data)
            .await
            .map(|_| Ok(id))?
    }

    async fn global_model(&mut self, id: &str) -> StorageResult<Option<Model>> {
        debug!("download global model {}", id);
        let output = self
            .fetch_object_meta(&self.buckets.global_models, id)
            .await;
        let object_meta = match output {
            Err(RusotoError::Service(GetObjectError::NoSuchKey(_))) => return Ok(None),
            Err(err) => return Err(anyhow::anyhow!(err)),
            Ok(object) => object,
        };

        let body = Self::download_object_body(object_meta).await?;
        let model = bincode::deserialize(&body).map_err(ClientError::Deserialization)?;
        Ok(Some(model))
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        let req = HeadBucketRequest {
            // we can't use an empty string because S3/Minio would return BAD_REQUEST
            bucket: self.buckets.global_models.clone(),
            ..Default::default()
        };
        let res = self.client.head_bucket(req).await;

        match res {
            // rusoto doesn't return NoSuchBucket if the bucket doesn't exist
            // https://github.com/rusoto/rusoto/issues/1099
            //
            // a workaround is to check if the StatusCode is NOT_FOUND
            Err(RusotoError::Service(HeadBucketError::NoSuchBucket(_))) | Ok(_) => Ok(()),
            Err(RusotoError::Unknown(resp)) => match resp.status {
                // https://github.com/timberio/vector/blob/803c68c031e5872876e1167c428cd41358123d64/src/sinks/aws_s3.rs#L229
                StatusCode::NOT_FOUND => Ok(()),
                _ => Err(anyhow::anyhow!(ClientError::NotReady(
                    RusotoError::Unknown(resp)
                ))),
            },
            Err(e) => Err(anyhow::anyhow!(ClientError::NotReady(e))),
        }
    }
}
