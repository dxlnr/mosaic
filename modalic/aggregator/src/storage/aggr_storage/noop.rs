//! A NoOp [`AggregatorStorage`] backend.

use crate::{
    aggr::Aggregator,
    // state_engine::coordinator::Aggregator,
    storage::{
        AggregatorStorage,
        trust_anchor::noop::NoOp,
        // AggregatorStorage,
        LocalSeedDictAdd,
        MaskScoreIncr,
        ModelStorage,
        Storage,
        StorageResult,
        SumPartAdd,
        TrustAnchor,
    },
};
use async_trait::async_trait;
use modalic_core::{
    common::RoundSeed,
    mask::{MaskObject, Model},
    LocalSeedDict,
    SeedDict,
    SumDict,
    SumParticipantEphemeralPublicKey,
    SumParticipantPublicKey,
    UpdateParticipantPublicKey,
};

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

    // async fn add_sum_participant(
    //     &mut self,
    //     pk: &SumParticipantPublicKey,
    //     ephm_pk: &SumParticipantEphemeralPublicKey,
    // ) -> StorageResult<SumPartAdd> {
    //     todo!()
    // }

    // async fn sum_dict(&mut self) -> StorageResult<Option<SumDict>> {
    //     todo!()
    // }

    // async fn add_local_seed_dict(
    //     &mut self,
    //     update_pk: &UpdateParticipantPublicKey,
    //     local_seed_dict: &LocalSeedDict,
    // ) -> StorageResult<LocalSeedDictAdd> {
    //     todo!()
    // }

    // async fn seed_dict(&mut self) -> StorageResult<Option<SeedDict>> {
    //     todo!()
    // }

    // async fn incr_mask_score(
    //     &mut self,
    //     pk: &SumParticipantPublicKey,
    //     mask: &MaskObject,
    // ) -> StorageResult<MaskScoreIncr> {
    //     todo!()
    // }

    // async fn best_masks(&mut self) -> StorageResult<Option<Vec<(MaskObject, u64)>>> {
    //     todo!()
    // }

    // async fn number_of_unique_masks(&mut self) -> StorageResult<u64> {
    //     todo!()
    // }
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
