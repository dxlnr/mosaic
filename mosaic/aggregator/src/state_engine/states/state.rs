use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
use futures::StreamExt;
use tracing::{debug, info, warn};

use crate::{
    aggr::Aggregator,
    state_engine::{
        channel::{RequestReceiver, ResponseSender, StateEngineRequest},
        event::EventPublisher,
        Failure,
        StateEngine,
    },
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
    /// Private Identifier of the state.
    /// 
    pub(in crate::state_engine) private: S,
    /// [`SharedState`] that the Aggregator holds.
    ///
    pub(in crate::state_engine) shared: SharedState,
}

impl<S> StateCondition<S>
where
    Self: State,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<StateEngine> {
        info!("Aggregator runs in state: {:?}", &Self::NAME);

        async move {
            if let Err(err) = self.perform().await {
                warn!("Aggregator failed to perform task of state {:?}", &Self::NAME);
                return Some(self.into_failure_state(err));
            }

            self.publish();

            debug!("transitioning to the next state.");
            self.next().await
        }
        .await
    }
    /// Receives the next ['StateEngineRequest'].
    pub async fn next_request(
        &mut self,
    ) -> Result<(StateEngineRequest, ResponseSender), StateError> {
        debug!("Aggregator waiting for the next incoming request");
        self.shared
            .rx
            .next()
            .await
            .ok_or(StateError::RequestChannel(
                "error when receiving next request.",
            ))
    }

    pub fn try_next_request(
        &mut self,
    ) -> Result<Option<(StateEngineRequest, ResponseSender)>, StateError> {
        match self.shared.rx.try_recv() {
            Some(Some(item)) => Ok(Some(item)),
            None => {
                debug!("no pending request");
                Ok(None)
            }
            Some(None) => {
                warn!("failed to get next pending request: Channel will be shut down.");
                Err(StateError::RequestChannel(
                    "all message senders have been dropped!",
                ))
            }
        }
    }

    fn into_failure_state(self, err: StateError) -> StateEngine {
        StateCondition::<Failure>::new(err, self.shared).into()
    }
}

/// [`SharedState`]
pub struct SharedState {
    /// [`Aggregator`]
    pub aggr: Aggregator,
    /// [`RequestReceiver`] for enabling receiving requests from the client.
    /// 
    pub rx: RequestReceiver,
    /// [`EventPublisher`] responsible for publishing the latest updates.
    ///
    pub publisher: EventPublisher,
}

impl SharedState {
    /// Init new [`SharedState`] for the aggregation server.
    pub fn new(aggr: Aggregator, rx: RequestReceiver, publisher: EventPublisher) -> Self {
        SharedState {
            aggr,
            rx,
            publisher,
        }
    }
}
