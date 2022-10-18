//! A NoOp [`AggregatorStorage`] backend.

use crate::{
    aggr::Aggregator,
    storage::{
        AggregatorStorage,
        StorageResult,
    },
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct AggrNoOp;

#[async_trait]
impl AggregatorStorage for AggrNoOp {
    async fn set_coordinator_state(&mut self, _state: &Aggregator) -> StorageResult<()> {
        Ok(())
    }

    async fn coordinator_state(&mut self) -> StorageResult<Option<Aggregator>> {
        Ok(None)
    }

    async fn delete_coordinator_data(&mut self) -> StorageResult<()> {
        Ok(())
    }

    async fn delete_dicts(&mut self) -> StorageResult<()> {
        Ok(())
    }

    async fn set_latest_global_model_id(&mut self, _id: &str) -> StorageResult<()> {
        Ok(())
    }

    async fn latest_global_model_id(&mut self) -> StorageResult<Option<String>> {
        Ok(None)
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        Ok(())
    }
}
