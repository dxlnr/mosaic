use std::{cmp::Ordering, sync::Arc};

use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
#[cfg(feature = "model-persistence")]
use tracing::warn;
use tracing::{error, info};

use crate::{
    state_engine::{
        events::ModelUpdate,
        states::{Idle, SharedState, State, StateCondition, StateError, StateName},
        StateEngine,
    },
    storage::{Storage, StorageError},
};
use mosaic_core::{
    mask::{Aggregation, MaskObject, UnmaskingError},
    model::Model,
};

/// Errors which can occur during the unmask State.
#[derive(Debug, Display, Error)]
pub enum UnmaskError {
    /// Ambiguous masks were computed by the sum participants.
    AmbiguousMasks,
    /// No mask found.
    NoMask,
    /// Unmasking global model failed: {0}.
    Unmasking(#[from] UnmaskingError),
    /// Fetching best masks failed: {0}.
    FetchBestMasks(#[from] StorageError),
    #[cfg(feature = "model-persistence")]
    /// Saving the global model failed: {0}.
    SaveGlobalModel(crate::storage::StorageError),
    /// Publishing the proof of the global model failed: {0}.
    PublishProof(crate::storage::StorageError),
}

/// The unmask state.
#[derive(Debug)]
pub struct Unmask {
    /// The aggregator for masked models.
    model_agg: Option<Aggregation>,
    /// The global model of the current round.
    global_model: Option<Arc<Model>>,
}

#[async_trait]
impl<T> State<T> for StateCondition<Unmask, T>
where
    T: Storage,
{
    const NAME: StateName = StateName::Unmask;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    fn publish(&mut self) {
        info!("broadcasting the new global model");
        let global_model =
            self.private.global_model.take().expect(
                "unreachable: never fails when `broadcast()` is called after `end_round()`",
            );
        self.shared
            .publisher
            .broadcast_model(ModelUpdate::New(global_model));
    }

    async fn next(self) -> Option<StateEngine<T>> {
        Some(StateCondition::<Idle, _>::new(self.shared).into())
    }
}

impl<T> StateCondition<Unmask, T> {
    /// Creates a new unmask state.
    pub fn new(shared: SharedState<T>, model_agg: Aggregation) -> Self {
        Self {
            private: Unmask {
                model_agg: Some(model_agg),
                global_model: None,
            },
            shared,
        }
    }

    /// Freezes the mask dictionary.
    async fn freeze_mask_dict(
        &mut self,
        mut best_masks: Vec<(MaskObject, u64)>,
    ) -> Result<MaskObject, UnmaskError> {
        let mask = best_masks
            .drain(0..)
            .fold(
                (None, 0),
                |(unique_mask, unique_count), (mask, count)| match unique_count.cmp(&count) {
                    Ordering::Less => (Some(mask), count),
                    Ordering::Greater => (unique_mask, unique_count),
                    Ordering::Equal => (None, unique_count),
                },
            )
            .0
            .ok_or(UnmaskError::AmbiguousMasks)?;

        Ok(mask)
    }

    /// Ends the round by unmasking the global model.
    async fn end_round(&mut self, best_masks: Vec<(MaskObject, u64)>) -> Result<(), UnmaskError> {
        let mask = self.freeze_mask_dict(best_masks).await?;

        // Safe unwrap: State::<Unmask>::new always creates Some(aggregation)
        let model_agg = self.private.model_agg.take().unwrap();

        model_agg
            .validate_unmasking(&mask)
            .map_err(UnmaskError::from)?;
        self.private.global_model = Some(Arc::new(model_agg.unmask(mask)));

        Ok(())
    }
}

impl<T> StateCondition<Unmask, T>
where
    T: Storage,
{
    // /// Broadcasts mask metrics.
    // fn emit_number_of_unique_masks_metrics(&mut self) {
    //     if GlobalRecorder::global().is_none() {
    //         return;
    //     }

    //     let mut store = self.shared.store.clone();
    //     let (round_id, State_name) = (self.shared.state.round_id, Self::NAME);

    //     tokio::spawn(async move {
    //         match store.number_of_unique_masks().await {
    //             Ok(number_of_masks) => metric!(
    //                 Measurement::MasksTotalNumber,
    //                 number_of_masks,
    //                 ("round_id", round_id),
    //                 ("State", State_name as u8),
    //             ),
    //             Err(err) => error!("failed to fetch total number of masks: {}", err),
    //         };
    //     });
    // }

    // /// Gets the two masks with the highest score.
    // async fn best_masks(&mut self) -> Result<Vec<(MaskObject, u64)>, UnmaskError> {
    //     self.shared
    //         .store
    //         .best_masks()
    //         .await
    //         .map_err(UnmaskError::FetchBestMasks)?
    //         .ok_or(UnmaskError::NoMask)
    // }

    /// Persists the global model to the store.
    #[cfg(feature = "model-persistence")]
    async fn save_global_model(&mut self) -> Result<(), UnmaskError> {
        info!("saving global model");
        let global_model = self
            .private
            .global_model
            .as_ref()
            .expect(
                "unreachable: never fails when `save_global_model()` is called after `end_round()`",
            )
            .as_ref();
        let global_model_id = self
            .shared
            .store
            .set_global_model(
                self.shared.state.round_id,
                &self.shared.state.round_params.seed,
                global_model,
            )
            .await
            .map_err(UnmaskError::SaveGlobalModel)?;
        if let Err(err) = self
            .shared
            .store
            .set_latest_global_model_id(&global_model_id)
            .await
        {
            warn!("failed to update latest global model id: {}", err);
        }

        Ok(())
    }

    /// Publishes proof of the global model.
    async fn publish_proof(&mut self) -> Result<(), UnmaskError> {
        info!("publishing proof of the new global model");
        let global_model = self
            .private
            .global_model
            .as_ref()
            .expect(
                "unreachable: never fails when `save_global_model()` is called after `end_round()`",
            )
            .as_ref();
        self.shared
            .store
            .publish_proof(global_model)
            .await
            .map_err(UnmaskError::PublishProof)
    }
}
