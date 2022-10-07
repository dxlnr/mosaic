use async_trait::async_trait;
use displaydoc::Display;
// use sodiumoxide::crypto::hash::sha256;
use thiserror::Error;
// use tracing::{debug, info, warn};

use crate::{
    storage::{Storage, StorageError},
    state_engine::{
    states::{Collect, SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
    }
};

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

        // self.gen_round_keypair();
        // self.update_round_probabilities();
        // self.update_round_seed();

        // self.set_coordinator_state().await?;
        
        Ok(())
    }

    fn publish(&mut self) {
        // self.shared.publisher.publish_state(Self::NAME);
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
}
