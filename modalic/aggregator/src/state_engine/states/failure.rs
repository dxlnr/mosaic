use std::time::Duration;

use async_trait::async_trait;
use displaydoc::Display;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{error, debug};

use crate::{
    state_engine::{
    states::{IdleError, SharedState, State, StateCondition, UpdateError, StateName},
    StateEngine,},
    storage::Storage,
};

/// Errors which can occur during the execution of the [`StateMachine`].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Request channel error: {0}.
    RequestChannel(&'static str),
    /// Idle phase failed: {0}.
    Idle(#[from] IdleError),
    /// Update phase failed: {0}.
    Update(#[from] UpdateError),
}

#[derive(Debug)]
/// [`Failure`] state of the [`StateEngine`]
///
pub struct Failure {
    pub(in crate::state_engine) error: StateError,
}

#[async_trait]
impl<T> State<T> for StateCondition<Failure, T> 
where
    T: Storage,
{
    const NAME: StateName = StateName::Failure;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine<T>> {
        None
    }
}

impl<T> StateCondition<Failure, T> {
    pub fn new(error: StateError, shared: SharedState<T>) -> Self {
        Self {
            private: Failure { error },
            shared,
        }
    }
}

impl<T> StateCondition<Failure, T>
where
    T: Storage,
{
    /// Waits until the [`Store`] is ready.
    ///
    /// [`Store`]: crate::storage::Store
    async fn wait_for_store_readiness(&mut self) {
        while let Err(err) = <T as Storage>::is_ready(&mut self.shared.store).await {
            error!("store not ready: {}", err);
            debug!("try again in 5 sec");
            sleep(Duration::from_secs(5)).await;
        }
    }
}