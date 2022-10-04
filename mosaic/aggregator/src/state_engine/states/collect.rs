use async_trait::async_trait;

use crate::state_engine::{
    channel::{RequestError, StateEngineRequest},
    states::{SharedState, State, StateCondition, StateError, StateHandler, StateName, Update},
    StateEngine,
};

#[derive(Debug)]
/// [`Collect`] object representing the collect state.
pub struct Collect;

#[async_trait]
impl State for StateCondition<Collect> {
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Update>::new(self.shared).into())
    }
}

impl StateCondition<Collect> {
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
    /// Add message to buffer for current training round described in
    /// [FedBuff](https://arxiv.org/abs/2106.06639).
    ///
    fn add_to_buffer(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        // TODO: use different error
        todo!()
    }
}

#[async_trait]
impl StateHandler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        self.add_to_buffer(req)
    }
}

#[cfg(test)]
mod tests {}
