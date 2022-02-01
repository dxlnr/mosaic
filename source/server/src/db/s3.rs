//! S3 connection for storing.

use std::sync::Arc;

use s3::bucket::Bucket;

use crate::settings::S3Settings;

#[derive(Clone)]
pub struct Client {
    bucket: Arc<Bucket>,
}

impl Client {
    /// Creates a new S3 client. The client creates and maintains buckets for storing all 
    /// the data created during the process.
    ///
    pub fn new(setting: S3Settings) -> Self{
        todo!()

    }

    // Downloads the content of a requested object.
    async fn download_object_body() {
        todo!()
    }

    // Uploads an object with the given key to the given bucket.
    async fn upload_object() {
        todo!()
    }

    // Creates a new bucket with the given bucket name.
    async fn create_bucket() {
        todo!()
    }
}