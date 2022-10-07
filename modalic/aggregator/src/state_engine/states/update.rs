use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
// use tracing::{debug, info, warn};

use crate::{
    state_engine::{
    channel::{RequestError, StateEngineRequest},
    states::{SharedState, Shutdown, State, StateCondition, StateError, StateHandler, StateName},
    StateEngine},
    storage::{Storage, StorageError},
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
pub struct Update;

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

impl<T> StateCondition<Update, T> {
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Update,
            shared,
        }
    }
}

#[async_trait]
impl<T> StateHandler for StateCondition<Update, T> 
where
    T: Storage,
{
    async fn handle_request(&mut self, _req: StateEngineRequest) -> Result<(), RequestError> {
        Ok(())
    }
}
