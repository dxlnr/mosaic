use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    states::{Collect, State, StateCondition, StateName},
    Engine, ServerState,
};

/// The idle state.
#[derive(Debug)]
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), Infallible> {
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Collect>::new(self.shared).into())
    }
}

impl StateCondition<Idle> {
    /// Creates a new idle state.
    pub fn new(mut shared: ServerState) -> Self {
        shared.set_round_id(shared.round_id() + 1);
        Self {
            private: Idle,
            shared,
        }
    }
}
