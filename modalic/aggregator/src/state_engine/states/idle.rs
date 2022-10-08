use async_trait::async_trait;
use displaydoc::Display;
// use sodiumoxide::crypto::hash::sha256;
use thiserror::Error;
use tracing::{debug, info};

use crate::{
    storage::{Storage, StorageError},
    state_engine::{
    states::{Collect, SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
    }
};
use modalic_core::crypto::EncryptKeyPair;

/// Errors which can occur during the idle phase.
#[derive(Debug, Display, Error)]
pub enum IdleError {
    /// Setting the coordinator state failed: {0}.
    SetCoordinatorState(StorageError),
    /// Deleting the dictionaries failed: {0}.
    DeleteDictionaries(StorageError),
}

#[derive(Debug)]
/// [`Idle`] state of the [`StateEngine`]
///
/// The initialziation of supporting processes happens in the idle state.
///
pub struct Idle;

#[async_trait]
impl<T> State<T> for StateCondition<Idle, T> 
where
    T: Storage,
{
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        // self.delete_dicts().await?;

        self.gen_round_keypair();
        // self.update_round_probabilities();
        // self.update_round_seed();

        self.set_aggr_state_to_store().await?;
        
        Ok(())
    }

    fn publish(&mut self) {
        self.publish_keys();
    }

    async fn next(self) -> Option<StateEngine<T>> {
        Some(StateCondition::<Collect, _>::new(self.shared).into())
    }
}

impl<T> StateCondition<Idle, T> {
    /// Init a new [`Idle`] state.
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }

    /// Generates fresh round credentials.
    fn gen_round_keypair(&mut self) {
        debug!("updating the keys");
        self.shared.aggr.keys = EncryptKeyPair::generate();
        self.shared.aggr.round_params.pk = self.shared.aggr.keys.public;
    }

    /// Broadcasts the keys.
    fn publish_keys(&mut self) {
        debug!("broadcasting new keys");
        self.shared
            .publisher
            .broadcast_keys(self.shared.aggr.keys.clone());
    }
}

impl<T> StateCondition<Idle, T>
where
    T: Storage,
{
    /// Persists the coordinator state to the store.
    async fn set_aggr_state_to_store(&mut self) -> Result<(), IdleError> {
        info!("storing new coordinator state");
        self.shared
            .store
            .set_coordinator_state(&self.shared.aggr)
            .await
            .map_err(IdleError::SetCoordinatorState)
    }
}
