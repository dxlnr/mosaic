use async_trait::async_trait;
// use tracing::{info, warn};

use crate::state_engine::{
    channel::{RequestError, StateEngineRequest},
    states::{SharedState, Shutdown, State, StateCondition, StateError, StateHandler, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Update`] state where the aggregation is computed.
pub struct Update;

#[async_trait]
impl State for StateCondition<Update> {
    const NAME: StateName = StateName::Update;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Shutdown>::new(self.shared).into())
    }
}

impl StateCondition<Update> {
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Update,
            shared,
        }
    }
}

#[async_trait]
impl StateHandler for StateCondition<Update> {
    async fn handle_request(&mut self, _req: StateEngineRequest) -> Result<(), RequestError> {
        Ok(())
    }
}
