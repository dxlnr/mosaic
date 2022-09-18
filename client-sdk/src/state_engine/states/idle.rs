use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName, Train},
    StateEngine,
};

#[derive(Debug)]
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Train>::new(self.shared).into())
    }
}

impl StateCondition<Idle> {
    /// Init a new [`Idle`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }
}