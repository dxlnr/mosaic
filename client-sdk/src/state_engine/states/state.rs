use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
// use tracing::warn;

use crate::state_engine::{smpc::Smpc, TransitionState};

/// Handling state errors when running ['StateEngine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Request channel error: {0}.
    RequestChannel(&'static str),
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait StateCondition<S> {
    /// Performs the attached tasks of current state.
    async fn proceed(mut self) -> TransitionState;
}

pub trait IntoNextState<S> {
    /// Moves from the current state to the next state.
    fn into_next_state(self) -> State<S>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct State<S> {
    /// Private Identifier of the state.
    ///
    pub(super) private: S,
    /// [`SharedState`] that the client holds.
    ///
    pub(super) shared: SharedState,
    /// StateEngine Message Passing Communication object.
    /// 
    pub(super) smpc: Smpc,
}

impl<S> State<S>
// where 
//     State<S>: Into<StateEngine>,
{
    /// Create a new [`State`].
    pub fn new(shared: SharedState, smpc: Smpc, private: S) -> Self {
        Self {
            private,
            shared,
            smpc,
        }
    }
}

#[derive(Debug)]
/// [`SharedState`]
pub struct SharedState {}

impl SharedState {
    /// Init new [`SharedState`] for the client.
    pub fn new() -> Self {
        Self {}
    }
}
