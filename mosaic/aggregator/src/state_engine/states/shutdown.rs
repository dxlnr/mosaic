use async_trait::async_trait;

use crate::{
    state_engine::{
        states::{Collect, StateError, State, StateCondition, StateName},
        StateEngine,
    },
};

#[derive(Debug)]
/// [`Shutdown`] state of the [`StateEngine`]
pub struct Shutdown;


#[async_trait]
impl State for StateCondition<Shutdown> {
    const NAME: StateName = StateName::Shutdown;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    async fn next(self) -> Option<StateEngine> {
        None
    }
}

impl StateCondition<Shutdown> {
    pub fn new() -> Self {
        todo!()
    }
}