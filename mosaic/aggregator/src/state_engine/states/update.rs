use async_trait::async_trait;
// use tracing::{info, warn};

use crate::{
    state_engine::{
        channel::{StateEngineRequest, RequestError},
        states::{Collect, State, StateCondition, StateError, StateHandler, StateName},
        StateEngine
    },
};

#[derive(Debug)]
/// [`Update`] state where the aggregation is computed.
pub struct Update {}

#[async_trait]
impl State for StateCondition<Update>
where
    Self: StateHandler,
{
    const NAME: StateName = StateName::Update;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    async fn next(self) -> Option<StateEngine> {
        todo!()
    }
}

impl StateCondition<Update> {
    pub fn new() -> Self {
        todo!()
    }
}

#[async_trait]
impl StateHandler for StateCondition<Update> {
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        Ok(())
    }
}