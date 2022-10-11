use std::collections::HashMap;

use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::{
    aggr::buffer::FedBuffer,
    state_engine::{
        channel::{RequestError, StateEngineRequest, UpdateRequest},
        states::{
            SharedState, State, StateCondition, StateError, StateHandler, StateName, Update,
            UpdateError,
        },
        StateEngine,
    },
    storage::Storage,
};

use modalic_core::{
    mask::{Aggregation, MaskObject},
    model::ModelObject,
    LocalSeedDict, SeedDict, UpdateParticipantPublicKey,
};

#[derive(Debug)]
/// [`Collect`] object representing the collect state.
pub struct Collect {
    /// [`FedBuffer`]
    fed_buffer: FedBuffer,
}

#[async_trait]
impl<T> State<T> for StateCondition<Collect, T>
where
    T: Storage,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        self.process().await?;
        // self.get_fedbuff().await?;
        Ok(())
    }

    async fn next(self) -> Option<StateEngine<T>> {
        Some(StateCondition::<Update, _>::new(self.shared, self.private.fed_buffer).into())
    }
}

impl<T> StateCondition<Collect, T> {
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Collect {
                fed_buffer: FedBuffer::default(),
            },
            shared,
        }
    }
}

#[async_trait]
impl<T> StateHandler for StateCondition<Collect, T>
where
    T: Storage,
{
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        #[cfg(feature = "secure")]
        {
            if let StateEngineRequest::Update(UpdateRequest {
                participant_pk,
                local_seed_dict,
                masked_model,
            }) = req
            {
                // self.update_seed_dict_and_aggregate_mask(
                //     &participant_pk,
                //     &local_seed_dict,
                //     masked_model,
                // )
                // .await
                self.update_fedbuffer(&participant_pk, &local_seed_dict, masked_model)
                    .await
            } else {
                Err(RequestError::MessageRejected)
            }
        }
        #[cfg(not(feature = "secure"))]
        {
            if let StateEngineRequest::Update(UpdateRequest {
                participant_pk,
                model_object,
            }) = req
            {
                self.update_fedbuffer(&participant_pk, model_object).await
            } else {
                Err(RequestError::MessageRejected)
            }
        }
    }
}

impl<T> StateCondition<Collect, T>
where
    T: Storage,
{
    /// Add message to buffer for current training round described in
    /// [FedBuff](https://arxiv.org/abs/2106.06639).
    ///
    async fn update_fedbuffer(
        &mut self,
        pk: &UpdateParticipantPublicKey,
        // local_seed_dict: &LocalSeedDict,
        mask_object: ModelObject,
    ) -> Result<(), RequestError> {
        #[cfg(not(feature = "redis"))]
        {
            let _ = self
                .private
                .fed_buffer
                .seed_dict
                .insert(HashMap::from([(*pk, local_seed_dict.clone())]));
            self.private.fed_buffer.local_models.push(mask_object);
        }
        #[cfg(feature = "redis")]
        {
            debug!("updating the global seed dictionary");
            self.add_local_seed_dict(pk, local_seed_dict)
                .await
                .map_err(|err| {
                    warn!("invalid local seed dictionary, ignoring update message");
                    err
                })?;
        }
        Ok(())
    }
    // /// Updates the local seed dict and aggregates the masked model.
    // async fn update_seed_dict_and_aggregate_mask(
    //     &mut self,
    //     pk: &UpdateParticipantPublicKey,
    //     local_seed_dict: &LocalSeedDict,
    //     mask_object: MaskObject,
    // ) -> Result<(), RequestError> {
    //     // // Check if aggregation can be performed. It is important to
    //     // // do that _before_ updating the seed dictionary, because we
    //     // // don't want to add the local seed dict if the corresponding
    //     // // masked model is invalid
    //     // debug!("checking whether the masked model can be aggregated");
    //     // self.private
    //     //     .model_agg
    //     //     .validate_aggregation(&mask_object)
    //     //     .map_err(|e| {
    //     //         warn!("model aggregation error: {}", e);
    //     //         RequestError::AggregationFailed
    //     //     })?;

    //     // Try to update local seed dict first. If this fail, we do
    //     // not want to aggregate the model.

    //     info!("updating the global seed dictionary");
    //     self.add_local_seed_dict(pk, local_seed_dict)
    //         .await
    //         .map_err(|err| {
    //             warn!("invalid local seed dictionary, ignoring update message");
    //             err
    //         })?;

    //     info!("aggregating the masked model and scalar");
    //     self.private.model_agg.aggregate(mask_object);
    //     Ok(())
    // }

    #[cfg(feature = "redis")]
    /// Adds a local seed dictionary to the global seed dictionary.
    ///
    /// # Error
    ///
    /// Fails if the local seed dict cannot be added due to a PET or [`StorageError`].
    async fn add_local_seed_dict(
        &mut self,
        pk: &UpdateParticipantPublicKey,
        local_seed_dict: &LocalSeedDict,
    ) -> Result<(), RequestError> {
        self.shared
            .store
            .add_local_seed_dict(pk, local_seed_dict)
            .await?
            .into_inner()
            .map_err(RequestError::from)
    }

    #[cfg(feature = "redis")]
    /// Gets the global seed dict from the store.
    async fn seed_dict(&mut self) -> Result<(), UpdateError> {
        self.private.seed_dict = self
            .shared
            .store
            .seed_dict()
            .await
            .map_err(UpdateError::FetchSeedDict)?
            .ok_or(UpdateError::NoSeedDict)?
            .into();

        Ok(())
    }
}
