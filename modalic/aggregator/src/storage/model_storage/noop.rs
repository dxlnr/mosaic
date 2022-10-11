//! A NoOp [`ModelStorage`] backend.

use crate::storage::{ModelStorage, StorageResult};
use async_trait::async_trait;
use modalic_core::{common::RoundSeed, model::Model};

#[derive(Clone)]
pub struct ModelNoOp;

#[async_trait]
impl ModelStorage for ModelNoOp {
    async fn set_global_model(
        &mut self,
        round_id: u64,
        round_seed: &RoundSeed,
        _global_model: &Model,
    ) -> StorageResult<String> {
        Ok(Self::create_global_model_id(round_id, round_seed))
    }

    async fn global_model(&mut self, _id: &str) -> StorageResult<Option<Model>> {
        Err(anyhow::anyhow!("No-op model store"))
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        Ok(())
    }
}
