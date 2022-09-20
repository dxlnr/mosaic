use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
use tracing::{info, warn};

use crate::{
    state_engine::StateEngine, client::EventSender
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

// /// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
// #[async_trait]
// pub trait State {
//     /// The name of the current state.
//     const NAME: StateName;

//     /// Performs the attached tasks of current state.
//     async fn perform(&mut self) -> Result<(), StateError>;

//     /// Publishes data of current state (Default: None).
//     fn publish(&mut self) {}

//     /// Moves from the current state to the next state.
//     async fn next(self) -> Option<StateEngine>;
// }

#[allow(dead_code)]
pub struct State<S> {
    /// Private Identifier of the state.
    /// 
    pub(in crate::state_engine) private: S,
    /// [`SharedState`] that the client holds.
    ///
    pub(in crate::state_engine) shared: SharedState,
}

// impl<S> State<S>

// {
//     /// Runs the current [`State`] to completion.
//     pub async fn run_state(mut self) -> Option<StateEngine> {
//         info!("Client runs in state: {:?}", &Self::NAME);
//         async move {
//             if let Err(err) = self.perform().await {
//                 warn!("{:?}", err);
//             }
//             self.next().await
//         }
//         .await
//     }
// }

/// [`SharedState`]
pub struct SharedState {
    /// [`EventSender`]
    event_sender: EventSender,
}

impl SharedState {
    /// Init new [`SharedState`] for the client.
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender
        }
    }
}