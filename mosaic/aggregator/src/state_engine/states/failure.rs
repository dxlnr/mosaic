use async_trait::async_trait;

use crate::state_engine::{
    states::{SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Failure`] state of the [`StateEngine`]
///
pub struct Failure {
    pub(in crate::state_engine) error: StateError,
}

#[async_trait]
impl<T> State<T> for StateCondition<Failure, T> 
where
    T: Send,
{
    const NAME: StateName = StateName::Failure;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl<T> StateCondition<Failure, T> {
    pub fn new(error: StateError, shared: SharedState<T>) -> Self {
        Self {
            private: Failure { error },
            shared,
        }
    }
}
