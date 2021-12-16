use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    channel::EngineRequest,
    states::{Handler, Shutdown, State, StateCondition, StateName},
    Engine, ServerState,
};

/// The collect state.
#[derive(Debug)]
pub struct Collect;

#[async_trait]
impl State for StateCondition<Collect>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), Infallible> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Shutdown>::new(self.shared).into())
    }
}

impl StateCondition<Collect> {
    /// Creates a new collect state.
    pub fn new(mut shared: ServerState) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
}

#[async_trait]
impl Handler for StateCondition<Collect> {
    async fn handle_request(&mut self, _req: EngineRequest) -> Result<(), Infallible> {
        Ok(())
    }
}
