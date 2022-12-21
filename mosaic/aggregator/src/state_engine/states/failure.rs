use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::{
    state_engine::{
        states::{SharedState, State, StateCondition, StateError, StateName},
        StateEngine,
    },
    storage::Storage,
};

#[derive(Debug)]
/// [`Failure`] state of the [`StateEngine`]
///
pub struct Failure {
    pub(in crate::state_engine) _error: StateError,
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
    pub fn new(shared: SharedState<T>, error: StateError) -> Self {
        Self {
            private: Failure { _error: error },
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
    async fn _wait_for_store_readiness(&mut self) {
        while let Err(err) = <T as Storage>::is_ready(&mut self.shared.store).await {
            error!("store not ready: {}", err);
            debug!("try again in 5 sec");
            sleep(Duration::from_secs(5)).await;
        }
    }
}
