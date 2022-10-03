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
impl<T> State<T> for StateCondition<Collect, T>
where
    T: Send,
    Self: StateHandler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Update, T>::new(self.shared).into())
    }
}

impl<T> StateCondition<Collect, T> {
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
    // /// Add message to buffer for current training round described in
    // /// [FedBuff](https://arxiv.org/abs/2106.06639).
    // ///
    // fn add_to_buffer(&mut self) -> Result<(), StateError> {
    //     // TODO: use different error
    //     todo!()
    // }
}

#[async_trait]
impl<T> StateHandler for StateCondition<Collect, T>
where
    T: Send
{
    async fn handle_request(&mut self, _req: StateEngineRequest) -> Result<(), RequestError> {
        // TODO: use different error
        todo!()
    }
}

#[cfg(test)]
mod tests {}
