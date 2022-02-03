use async_trait::async_trait;
use tracing::debug;

use crate::engine::{
    states::{error::StateError, State, StateCondition, StateName},
    Engine, ServerState,
};

/// The shutdown state.
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl State for StateCondition<Shutdown> {
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), StateError> {
        debug!("Closing request channel.");
        self.shared.rx.close();
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        None
    }
}

impl StateCondition<Shutdown> {
    /// Creates a new idle state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Shutdown,
            shared,
        }
    }
}
