//! StateEngine implements the clients protocol logic.
//!  
pub mod states;

use derive_more::From;

use crate::state_engine::states::{Idle, SharedState, StateCondition, Stop, Train};

/// [`StateEngine`]
#[derive(From)]
pub enum StateEngine {
    /// [`Idle`] state of client.
    Idle(StateCondition<Idle>),
    /// [`Idle`] state of client.
    Train(StateCondition<Train>),
    /// [`Idle`] state of client.
    Stop(StateCondition<Stop>),
}

impl StateEngine {
    pub fn new() -> Self {
        let shared = SharedState::new();

        StateEngine::Idle(StateCondition::<Idle>::new(shared))
    }

    pub async fn next(self) -> Option<Self> {
        match self {
            StateEngine::Idle(state) => state.run_state().await,
            StateEngine::Train(state) => state.run_state().await,
            StateEngine::Stop(state) => state.run_state().await,
        }
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}