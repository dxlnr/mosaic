//! StateEngine implements the clients protocol logic.
//!  
pub mod states;

use derive_more::From;

use crate::state_engine::states::{Idle, StateCondition};

/// [`StateEngine`]
#[derive(From)]
pub enum StateEngine {
    Idle(StateCondition<Idle>),
}

impl StateEngine {
    pub async fn next(self) -> Option<Self> {
        match self {
            StateEngine::Idle(state) => state.run_state().await,
        }
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}