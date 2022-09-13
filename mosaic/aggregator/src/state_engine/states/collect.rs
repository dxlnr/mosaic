use async_trait::async_trait;

use crate::state_engine::{
    channel::{RequestError, StateEngineRequest},
    states::{State, StateCondition, StateError, StateHandler, StateName, Update},
    StateEngine,
};

#[derive(Debug)]
/// [`Collect`] object representing the collect state.
pub struct Collect {}

#[async_trait]
impl State for StateCondition<Collect>
where
    Self: StateHandler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    async fn next(self) -> Option<StateEngine> {
        todo!()
    }
}

impl StateCondition<Collect> {
    pub fn new() -> Self {
        todo!()
    }
    /// Add message to buffer for current training round described in
    /// [FedBuff](https://arxiv.org/abs/2106.06639).
    ///
    fn add_to_buffer(&mut self) -> Result<(), StateError> {
        // TODO: use different error
        todo!()
    }
}

#[async_trait]
impl StateHandler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        // TODO: use different error
        todo!()
    }
}

#[cfg(test)]
mod tests {}
