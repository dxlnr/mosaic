use crate::settings::S3Settings;

use crate::db::s3::{Client, StorageError};

pub struct Storage {
    pub s3: Client,
}

impl Storage {
    pub async fn init_storage(s3_settings: S3Settings) -> Result<Client, StorageError> {
        // let s3 = Client::new(s3_settings).await?;
        // &s3.create_bucket().await?;
        // Ok(())
        Ok(Client::new(s3_settings).await?)
    }
}
