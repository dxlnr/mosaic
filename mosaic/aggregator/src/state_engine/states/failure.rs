use async_trait::async_trait;

use crate::state_engine::{
    states::{Collect, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Failure`] state of the [`StateEngine`]
pub struct Failure {
    pub(in crate::state_engine) error: StateError,
}

#[async_trait]
impl State for StateCondition<Failure> {
    const NAME: StateName = StateName::Failure;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl StateCondition<Failure> {
    pub fn new() -> Self {
        todo!()
    }
}
