use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName, Stop},
    StateEngine,
};

#[derive(Debug)]
pub struct Train;

#[async_trait]
impl State for StateCondition<Train> {
    const NAME: StateName = StateName::Train;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Stop>::new(self.shared).into())
    }
}

impl StateCondition<Train> {
    /// Init a new [`Train`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Train,
            shared,
        }
    }
}