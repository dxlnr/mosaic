//! A generic store.

use async_trait::async_trait;

use crate::{
    aggr::Aggregator,
    storage::{
        trust_anchor::noop::NoOp,
        AggregatorStorage,
        ModelStorage,
        Storage,
        StorageResult,
        TrustAnchor,
    },
};
#[cfg(feature = "secure")]
use mosaic_core::{mask::MaskObject, LocalSeedDict, SeedDict};

use mosaic_core::{
    common::RoundSeed, model::Model, UpdateParticipantPublicKey,
};

#[derive(Clone)]
/// A generic store.
pub struct Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    /// An aggregator store.
    aggregator: A,
    /// A model store.
    model: M,
    /// A trust anchor.
    trust_anchor: T,
}

impl<A, M, T> Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    pub fn new_with_trust_anchor(aggregator: A, model: M, trust_anchor: T) -> Self {
        Self {
            aggregator,
            model,
            trust_anchor,
        }
    }
}

impl<A, M> Store<A, M, NoOp>
where
    A: AggregatorStorage,
    M: ModelStorage,
{
    /// Areates a new [`Store`].
    pub fn new(aggregator: A, model: M) -> Self {
        Self {
            aggregator,
            model,
            trust_anchor: NoOp,
        }
    }
}

#[async_trait]
impl<A, M, T> AggregatorStorage for Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    async fn set_aggregator_state(&mut self, state: &Aggregator) -> StorageResult<()> {
        self.aggregator.set_aggregator_state(state).await
    }

    async fn aggregator_state(&mut self) -> StorageResult<Option<Aggregator>> {
        self.aggregator.aggregator_state().await
    }

    // async fn add_sum_participant(
    //     &mut self,
    //     pk: &SumParticipantPublicKey,
    //     ephm_pk: &SumParticipantEphemeralPublicKey,
    // ) -> StorageResult<SumPartAdd> {
    //     self.aggregator.add_sum_participant(pk, ephm_pk).await
    // }

    // async fn sum_dict(&mut self) -> StorageResult<Option<SumDict>> {
    //     self.aggregator.sum_dict().await
    // }

    // async fn add_local_seed_dict(
    //     &mut self,
    //     update_pk: &UpdateParticipantPublicKey,
    //     local_seed_dict: &LocalSeedDict,
    // ) -> StorageResult<LocalSeedDictAdd> {
    //     self.aggregator
    //         .add_local_seed_dict(update_pk, local_seed_dict)
    //         .await
    // }

    // async fn seed_dict(&mut self) -> StorageResult<Option<SeedDict>> {
    //     self.aggregator.seed_dict().await
    // }

    // async fn incr_mask_score(
    //     &mut self,
    //     pk: &SumParticipantPublicKey,
    //     mask: &MaskObject,
    // ) -> StorageResult<MaskScoreIncr> {
    //     self.aggregator.incr_mask_score(pk, mask).await
    // }

    // async fn best_masks(&mut self) -> StorageResult<Option<Vec<(MaskObject, u64)>>> {
    //     self.aggregator.best_masks().await
    // }

    // async fn number_of_unique_masks(&mut self) -> StorageResult<u64> {
    //     self.aggregator.number_of_unique_masks().await
    // }

    async fn delete_aggregator_data(&mut self) -> StorageResult<()> {
        self.aggregator.delete_aggregator_data().await
    }

    async fn delete_dicts(&mut self) -> StorageResult<()> {
        self.aggregator.delete_dicts().await
    }

    async fn set_latest_global_model_id(&mut self, id: &str) -> StorageResult<()> {
        self.aggregator.set_latest_global_model_id(id).await
    }

    async fn latest_global_model_id(&mut self) -> StorageResult<Option<String>> {
        self.aggregator.latest_global_model_id().await
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        self.aggregator.is_ready().await
    }
}

#[async_trait]
impl<A, M, T> ModelStorage for Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    async fn set_global_model(
        &mut self,
        round_id: u64,
        round_seed: &RoundSeed,
        global_model: &Model,
    ) -> StorageResult<String> {
        self.model
            .set_global_model(round_id, round_seed, global_model)
            .await
    }

    async fn global_model(&mut self, id: &str) -> StorageResult<Option<Model>> {
        self.model.global_model(id).await
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        self.model.is_ready().await
    }
}

#[async_trait]
impl<A, M, T> TrustAnchor for Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    async fn publish_proof(&mut self, global_model: &Model) -> StorageResult<()> {
        self.trust_anchor.publish_proof(global_model).await
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        self.trust_anchor.is_ready().await
    }
}

#[async_trait]
impl<A, M, T> Storage for Store<A, M, T>
where
    A: AggregatorStorage,
    M: ModelStorage,
    T: TrustAnchor,
{
    async fn is_ready(&mut self) -> StorageResult<()> {
        tokio::try_join!(
            self.aggregator.is_ready(),
            self.model.is_ready(),
            self.trust_anchor.is_ready()
        )
        .map(|_| ())
    }
}
