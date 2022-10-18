use std::sync::Arc;

use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
// use tracing::{debug, info, warn};
use tracing::info;

use crate::{
    aggr::buffer::FedBuffer,
    state_engine::{
        events::ModelUpdate,
        states::{Idle, SharedState, State, StateCondition, StateError, StateName},
        StateEngine,
    },
    storage::Storage,
};

#[cfg(not(feature = "secure"))]
use crate::aggr::{Aggregation, AggregationError};
use mosaic_core::model::Model;

#[cfg(feature = "secure")]
use crate::{
    mask::{Aggregation, MaskObject},
    LocalSeedDict, SeedDict,
};

/// Errors which can occur during the update phase.
#[derive(Debug, Display, Error)]
pub enum UpdateError {
    #[cfg(feature = "secure")]
    /// Seed dictionary does not exists.
    NoSeedDict,
    #[cfg(feature = "secure")]
    /// Fetching seed dictionary failed: {0}.
    FetchSeedDict(crate::storage::StorageError),
    #[cfg(feature = "model-persistence")]
    /// Saving the global model failed: {0}.
    SaveGlobalModel(crate::storage::StorageError),
    /// AggregationError
    AggregationError,
}

#[derive(Debug)]
/// [`Update`] state where the aggregation is computed.
pub struct Update {
    fed_buffer: FedBuffer,
    /// [`Aggregation`]: The aggregator for masked models.
    aggr: Aggregation,
    ///
    global_model: Option<Arc<Model>>,
}

#[async_trait]
impl<T> State<T> for StateCondition<Update, T>
where
    T: Storage,
{
    const NAME: StateName = StateName::Update;

    async fn perform(&mut self) -> Result<(), StateError> {
        #[cfg(not(feature = "secure"))]
        self.aggregate_model()
            .await
            .map_err(|_| StateError::Update(UpdateError::AggregationError))?;

        #[cfg(feature = "model-persistence")]
        self.save_global_model().await?;

        Ok(())
    }

    fn publish(&mut self) {
        info!("publishing the new global model.");
        let global_model = self
            .private
            .global_model
            .take()
            .expect("unreachable: never fails when `publish()` is called after `end_round()`");
        self.shared
            .publisher
            .broadcast_model(ModelUpdate::New(global_model));
    }

    async fn next(self) -> Option<StateEngine<T>> {
        Some(StateCondition::<Idle, _>::new(self.shared).into())
    }
}

impl<T> StateCondition<Update, T> {
    pub fn new(shared: SharedState<T>, fed_buffer: FedBuffer) -> Self {
        #[cfg(not(feature = "secure"))]
        let aggr = Aggregation::default();
        #[cfg(feature = "secure")]
        let aggr = Aggregation::new(shared.aggr.round_params.mask_config, 0);

        Self {
            private: Update {
                fed_buffer,
                aggr,
                global_model: None,
            },
            shared,
        }
    }
}

impl<T> StateCondition<Update, T>
where
    T: Storage,
{
    #[cfg(feature = "secure")]
    /// Updates the local seed dict and aggregates the masked model.
    async fn aggregate_mask(
        &mut self,
        pk: &UpdateParticipantPublicKey,
        local_seed_dict: &LocalSeedDict,
        mask_object: MaskObject,
    ) -> Result<(), RequestError> {
        // Check if aggregation can be performed. It is important to
        // do that _before_ updating the seed dictionary, because we
        // don't want to add the local seed dict if the corresponding
        // masked model is invalid
        debug!("checking whether the masked model can be aggregated");
        self.private
            .aggr
            .validate_aggregation(&mask_object)
            .map_err(|e| {
                warn!("model aggregation error: {}", e);
                RequestError::AggregationFailed
            })?;

        info!("aggregating the masked model and scalar");
        for masked_model in self.private.fed_buffer.local_models.iter() {
            self.private.aggr.aggregate(masked_model.clone());
        }
        // self.private.aggr.aggregate(mask_object);
        Ok(())
    }

    #[cfg(not(feature = "secure"))]
    async fn aggregate_model(&mut self) -> Result<(), AggregationError> {
        self.private.global_model =
            Some(Arc::new(self.private.aggr.aggregate(
                &self.private.fed_buffer.local_models,
            )?));

        Ok(())
    }
}
