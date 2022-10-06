use async_trait::async_trait;
use tracing::debug;

use crate::{
    state_engine::{
        states::{State, StateError, StateName, StateCondition, SharedState},
        StateEngine,
    },
    storage::Storage,
};

/// The shutdown state.
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl<T> State<T> for StateCondition<Shutdown, T>
where
    T: Storage,
{
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), StateError> {
        debug!("clearing the request channel");
        self.shared.request_rx.close();
        while self.shared.request_rx.recv().await.is_some() {}

        Ok(())
    }

    async fn next(self) -> Option<StateEngine<T>> {
        None
    }
}

impl<T> StateCondition<Shutdown, T> {
    /// Creates a new shutdown state.
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Shutdown,
            shared,
        }
    }
}

