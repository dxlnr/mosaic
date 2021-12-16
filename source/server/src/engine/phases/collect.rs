use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    channel::EngineRequest,
    phases::{Handler, Phase, PhaseName, PhaseState, Shutdown},
    Engine, ServerState,
};

/// The collect state.
#[derive(Debug)]
pub struct Collect;

#[async_trait]
impl Phase for PhaseState<Collect>
where
    Self: Handler,
{
    const NAME: PhaseName = PhaseName::Collect;

    async fn perform(&mut self) -> Result<(), Infallible> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(PhaseState::<Shutdown>::new(self.shared).into())
    }
}

impl PhaseState<Collect> {
    /// Creates a new collect state.
    pub fn new(mut shared: ServerState) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
}

#[async_trait]
impl Handler for PhaseState<Collect> {
    async fn handle_request(&mut self, req: EngineRequest) -> Result<(), Infallible> {
        Ok(())
    }
}
