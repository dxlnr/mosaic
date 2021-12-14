use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    phases::{Phase, PhaseName, PhaseState},
    Engine, ServerState,
};

/// The shutdown state.
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl Phase for PhaseState<Shutdown> {
    const NAME: PhaseName = PhaseName::Shutdown;

    async fn perform(&mut self) -> Result<(), Infallible> {
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        None
    }
}

impl PhaseState<Shutdown> {
    /// Creates a new idle state.
    pub fn new(mut shared: ServerState) -> Self {
        Self {
            private: Shutdown,
            shared,
        }
    }
}
