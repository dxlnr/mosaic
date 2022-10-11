use std::sync::Arc;

use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::{
    aggr::buffer::FedBuffer,
    state_engine::{
        channel::RequestError,
        states::{SharedState, Shutdown, State, StateCondition, StateError, StateName},
        StateEngine,
    },
    storage::{Storage, StorageError},
};

use modalic_core::{
    mask::{Aggregation, MaskObject, UnmaskingError},
    model::Model,
    LocalSeedDict, SeedDict, UpdateParticipantPublicKey,
};

/// Errors which can occur during the update phase.
#[derive(Debug, Display, Error)]
pub enum UpdateError {
    /// Seed dictionary does not exists.
    NoSeedDict,
    /// Fetching seed dictionary failed: {0}.
    FetchSeedDict(StorageError),
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
        Ok(())
    }

    async fn next(self) -> Option<StateEngine<T>> {
        Some(StateCondition::<Shutdown, _>::new(self.shared).into())
    }
}

// #[async_trait]
// impl<T> StateHandler for StateCondition<Update, T>
// where
//     T: Storage,
// {
//     async fn handle_request(&mut self, _req: StateEngineRequest) -> Result<(), RequestError> {
//         Ok(())
//     }
// }

impl<T> StateCondition<Update, T> {
    pub fn new(shared: SharedState<T>, fed_buffer: FedBuffer) -> Self {
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
}
