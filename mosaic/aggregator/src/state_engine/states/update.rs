use async_trait::async_trait;
// use tracing::{info, warn};

use crate::state_engine::{
    channel::{RequestError, StateEngineRequest},
    states::{SharedState, Shutdown, State, StateCondition, StateError, StateHandler, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Update`] state where the aggregation is computed.
pub struct Update;

#[async_trait]
impl<T> State<T> for StateCondition<Update, T>
where
    T: Send,
    Self: StateHandler,
{
    const NAME: StateName = StateName::Update;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Shutdown, T>::new(self.shared).into())
    }
}

impl<T> StateCondition<Update, T> {
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Update,
            shared,
        }
    }
}

#[async_trait]
impl<T> StateHandler for StateCondition<Update, T> 
where
    T: Send,
{
    async fn handle_request(&mut self, _req: StateEngineRequest) -> Result<(), RequestError> {
        Ok(())
    }
}
