use async_trait::async_trait;

use crate::{db::{traits::{ModelStorage, StorageResult}}, engine::model::Model};

// pub struct Storage {
//     pub s3: Client,
// }

#[derive(Clone)]
/// A generic store.
pub struct Storage<M>
where
    M: ModelStorage,
{
    /// A model store.
    model: M,
}

// impl<M> Storage<M> {
//     pub async fn init_storage(s3_settings: S3Settings) -> Result<Client, StorageError> {
//         let s3 = Client::new(s3_settings).await?;
//         s3.clone().create_bucket().await?;
//         Ok(s3)
//     }
// }

#[async_trait]
impl<M> ModelStorage for Storage<M>
where
    M: ModelStorage,
{
    async fn get_global_model(&mut self, key: &str) -> StorageResult<Option<Model>> {
        self.model.get_global_model(key).await
    }
}
