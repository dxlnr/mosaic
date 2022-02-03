use async_trait::async_trait;

use crate::engine::{
    states::{error::StateError, Shutdown, State, StateCondition, StateName},
    Engine, ServerState,
};

/// The failure state.
#[derive(Debug)]
pub struct Failure {
    pub(in crate::engine) error: StateError,
}

#[async_trait]
impl State for StateCondition<Failure> {
    const NAME: StateName = StateName::Failure;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Shutdown>::new(self.shared).into())
    }
}

impl StateCondition<Failure> {
    /// Creates a new failure state.
    pub fn new(shared: ServerState, error: StateError) -> Self {
        Self {
            private: Failure { error },
            shared,
        }
    }
}