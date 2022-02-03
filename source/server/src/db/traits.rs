//! Functional storage API traits.
//! 
use async_trait::async_trait;

use crate::{db::s3::StorageError, engine::model::Model};

/// The result of the storage operation.
pub type StorageResult<T> = Result<T, StorageError>;
 
#[async_trait]
/// An abstract model storage.
pub trait ModelStorage
where
    Self: Clone + Send + Sync + 'static,
{
    async fn get_global_model(&mut self, key: &str) -> StorageResult<Option<Model>>;
}