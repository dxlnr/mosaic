use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
use tracing::{info, warn};

use crate::{
    state_engine::{channel::{ResponseSender, StateEngineRequest}, StateEngine},
};

/// Handling state errors when running ['StateEngine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Request channel error: {0}.
    RequestChannel(&'static str),
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
/// The name of the current state.
pub enum StateName {
    #[display(fmt = "Idle")]
    Idle,
    #[display(fmt = "Collect")]
    Collect,
    #[display(fmt = "Update")]
    Update,
    #[display(fmt = "Failure")]
    Failure,
    #[display(fmt = "Shutdown")]
    Shutdown,
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait State {
    /// The name of the current state.
    const NAME: StateName;

    /// Performs the attached tasks of current state.
    async fn perform(&mut self) -> Result<(), StateError>;

    /// Publishes data of current state (Default: None).
    fn publish(&mut self) {}

    /// Moves from the current state to the next state.
    async fn next(self) -> Option<StateEngine>;
}

#[allow(dead_code)]
pub struct StateCondition<S> {
    pub(in crate::state_engine) private: S,
    // /// Shared state that the Aggregator holds.
    // pub shared: ServerState,
}

impl<S> StateCondition<S>
where
    Self: State,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<StateEngine> {
        info!("Server runs in state: {:?}", &Self::NAME);
        async move {
            if let Err(err) = self.perform().await {
                warn!("{:?}", err);
            }
            self.next().await
        }
        .await
    }
    /// Receives the next ['Request'].
    pub async fn next_request(&mut self) -> Result<(StateEngineRequest, ResponseSender), StateError> {
        todo!()
    }
}
