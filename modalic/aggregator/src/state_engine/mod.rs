//! The state engine that controls the execution of the aggregation protocol.
//!
//! #
//! The implementation resembles a finite state machine which allows to keep state with in
//! a single `Aggregator` and perform the steps of the protocol in that way.
//!
//! # StateEngine states
//!
pub mod channel;
pub mod events;
pub mod init;
pub mod states;

use derive_more::From;

use crate::{
    state_engine::states::{
        Collect, Failure, Idle, Shutdown, State, StateCondition, Unmask, Update,
    },
    storage::Storage,
};

/// [`StateEngine`] functions as the state machine which handles the progress of the `Aggregator`
/// and keep its state.
///
#[derive(From)]
pub enum StateEngine<T> {
    /// [`Idle`] state.
    Idle(StateCondition<Idle, T>),
    /// [`Collect`] state.
    Collect(StateCondition<Collect, T>),
    /// [`Update`] state.
    Update(StateCondition<Update, T>),
    /// [`Update`] state.
    Unmask(StateCondition<Unmask, T>),
    /// [`Shutdown`] state.
    Shutdown(StateCondition<Shutdown, T>),
    /// [`Failure`] state.
    Failure(StateCondition<Failure, T>),
}

impl<T> StateEngine<T>
where
    T: Storage,
    StateCondition<Idle, T>: State<T>,
    StateCondition<Collect, T>: State<T>,
    StateCondition<Update, T>: State<T>,
    StateCondition<Unmask, T>: State<T>,
    StateCondition<Failure, T>: State<T>,
    StateCondition<Shutdown, T>: State<T>,
{
    pub async fn next(self) -> Option<Self> {
        match self {
            StateEngine::Idle(state) => state.run_state().await,
            StateEngine::Collect(state) => state.run_state().await,
            StateEngine::Update(state) => state.run_state().await,
            StateEngine::Unmask(state) => state.run_state().await,
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
