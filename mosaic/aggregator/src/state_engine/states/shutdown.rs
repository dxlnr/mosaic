use async_trait::async_trait;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Shutdown`] state of the [`StateEngine`]
pub struct Shutdown;

#[async_trait]
impl State for StateCondition<Shutdown> {
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl StateCondition<Shutdown> {
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Shutdown,
            shared,
        }
    }
}
