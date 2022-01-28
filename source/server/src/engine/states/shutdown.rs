use async_trait::async_trait;
use std::io::Error;
use tracing::debug;
use std::{thread, time::Duration};

use crate::engine::{
    states::{State, StateCondition, StateName},
    Engine, ServerState,
};

/// The shutdown state.
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl State for StateCondition<Shutdown> {
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), Error> {
        debug!("Closing request channel.");
        self.shared.rx.close();
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        thread::sleep(Duration::from_secs(10));
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
