use async_trait::async_trait;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Shutdown`] state of the [`StateEngine`]
pub struct Shutdown;

#[async_trait]
impl<T> State<T> for StateCondition<Shutdown, T> 
where
    T: Send,
{
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl<T> StateCondition<Shutdown, T> {
    pub fn new(shared: SharedState<T>) -> Self {
        Self {
            private: Shutdown,
            shared,
        }
    }
}
