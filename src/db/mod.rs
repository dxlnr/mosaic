//! Handling the connection to external storage capabilities, like [MinIO](https://github.com/minio/minio).
//! 
//! The storage layer is used for pushing logs, metadata and the global model to an external object storage.
//! The storage is a third party service and independent of the server application. 
//! 
//! This module serves as an api layer to these third party services.
pub mod s3;
pub mod storage;
pub mod traits;
