use async_trait::async_trait;
use tracing::warn;

use crate::state_engine::{
    states::{Collect, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Idle`] state of the [`StateEngine`]
/// The initialziation of supporting processes is happens in the idle state.
///
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        todo!()
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Collect>::new().into())
    }
}

impl StateCondition<Idle> {
    /// Creates a new idle state.
    pub fn new() -> Self {
        todo!()
    }
}
