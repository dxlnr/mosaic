use async_trait::async_trait;
use std::io::Error;

use crate::engine::{
    states::{Idle, Shutdown, State, StateCondition, StateName},
    Engine, ServerState,
};

/// The failure state.
#[derive(Debug)]
pub struct Failure;

#[async_trait]
impl State for StateCondition<Failure> {
    const NAME: StateName = StateName::Failure;

    async fn perform(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        // if ERROR {
        //     Some(StateCondition::<Shutdown>::new(self.shared).into())
        // } else {
        //     Some(StateCondition::<Idle>::new(self.shared).into())
        // }
        Some(StateCondition::<Shutdown>::new(self.shared).into())
    }
}

impl StateCondition<Failure> {
    /// Creates a new failure state.
    pub fn new(mut shared: ServerState) -> Self {
        shared.set_round_id(shared.round_id() + 1);
        Self {
            private: Failure,
            shared,
        }
    }
}