//! The state engine that controls the execution of the aggregation protocol.
//!
//! #
//! The implementation resembles a finite state machine which allows to keep state with in
//! a single `Aggregator` and perform the steps of the protocol in that way.
//!
//! # StateEngine states
//!
pub mod channel;
pub mod event;
pub mod init;
pub mod states;

use derive_more::From;

use crate::state_engine::states::{Collect, Failure, Idle, Shutdown, StateCondition, Update};

/// [`StateEngine`] functions as the state machine which handles the progress of the `Aggregator`
/// and keep its state.
///
#[derive(From)]
pub enum StateEngine {
    /// [`Idle`] state.
    Idle(StateCondition<Idle>),
    /// [`Collect`] state.
    Collect(StateCondition<Collect>),
    /// [`Update`] state.
    Update(StateCondition<Update>),
    /// [`Shutdown`] state.
    Shutdown(StateCondition<Shutdown>),
    /// [`Failure`] state.
    Failure(StateCondition<Failure>),
}

impl StateEngine {
    pub async fn next(self) -> Option<Self> {
        match self {
            StateEngine::Idle(state) => state.run_state().await,
            StateEngine::Collect(state) => state.run_state().await,
            StateEngine::Update(state) => state.run_state().await,
            StateEngine::Shutdown(state) => state.run_state().await,
            StateEngine::Failure(state) => state.run_state().await,
        }
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}
