use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
use tracing::warn;

use crate::{
    state_engine::{StateEngine, mpc::Smpc},
};

/// Handling state errors when running ['StateEngine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Request channel error: {0}.
    RequestChannel(&'static str),
}

// #[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
// /// The name of the current state.
// pub enum StateName {
//     #[display(fmt = "Idle")]
//     Idle,
//     #[display(fmt = "Train")]
//     Train,
//     #[display(fmt = "Stop")]
//     Stop,
// }

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait StateCondition<S> {
    /// Performs the attached tasks of current state.
    async fn perform(&mut self) -> Result<(), StateError>;

    /// Moves from the current state to the next state.
    async fn next(self) -> Option<StateEngine>;
}

#[allow(dead_code)]
pub struct State<S> {
    /// Private Identifier of the state.
    /// 
    pub(in crate::state_engine) private: S,
    /// [`SharedState`] that the client holds.
    ///
    pub(in crate::state_engine) shared: SharedState,
    /// .
    pub (in crate::state_engine) smpc: Smpc,
}

impl<S> State<S> {
    /// Create a new [`State`].
    pub fn new(shared: SharedState, smpc: Smpc, private: S) -> Self {
        Self {
            private,
            shared,
            smpc,
        }
    }
}

impl<S> State<S>
where
    Self: StateCondition<S>
{
    /// Runs the current [`State`] to completion.
    pub async fn run_state(mut self) -> Option<StateEngine> {
        async move {
            if let Err(err) = self.perform().await {
                warn!("client error : {:?}", err);
            }
            self.next().await
        }
        .await
    }
}

/// [`SharedState`]
pub struct SharedState {}

impl SharedState {
    /// Init new [`SharedState`] for the client.
    pub fn new() -> Self {
        Self {}
    }
}