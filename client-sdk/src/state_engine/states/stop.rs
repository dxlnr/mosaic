use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
pub struct Stop;

#[async_trait]
impl State for StateCondition<Stop> {
    const NAME: StateName = StateName::Stop;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl StateCondition<Stop> {
    /// Init a new [`Stop`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Stop,
            shared,
        }
    }
}