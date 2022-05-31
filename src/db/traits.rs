//! Functional storage API traits.
//!
use async_trait::async_trait;

use crate::{core::model::Model, db::s3::StorageError};

/// The result of the storage operation.
pub type StorageResult<T> = Result<T, StorageError>;

#[async_trait]
/// An abstract model storage.
pub trait ModelStorage
where
    Self: Clone + Send + Sync + 'static,
{
    async fn get_global_model(&mut self) -> StorageResult<Option<Model>>;
    async fn set_global_model(&mut self, data: &[u8]) -> StorageResult<()>;
}
