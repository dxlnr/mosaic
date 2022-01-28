use async_trait::async_trait;
use std::io::Error;

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

    async fn perform(&mut self) -> Result<(), Error> {
        // let global = self.shared.global_model.clone();
        // let _ = self.shared.publisher.broadcast_model(global);
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        // if self.shared.round_id() > self.shared.round_params.training_rounds {
        //     Some(StateCondition::<Shutdown>::new(self.shared).into())
        // } else {
        //     Some(StateCondition::<Collect>::new(self.shared).into())
        // }
        Some(StateCondition::<Collect>::new(self.shared).into())
    }
}

impl StateCondition<Idle> {
    /// Creates a new idle state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }
}
