use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{Collect, SharedState, State, StateCondition, StateError, StateName},
    StateEngine,
};

#[derive(Debug)]
/// [`Idle`] state of the [`StateEngine`]
/// 
/// The initialziation of supporting processes happens in the idle state.
///
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(StateCondition::<Collect>::new(self.shared).into())
    }
}

impl StateCondition<Idle> {
    /// Init a new [`Idle`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }
}
