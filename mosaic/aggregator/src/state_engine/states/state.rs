use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;
use tracing::{info, warn};

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
pub trait State<T> {
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
pub struct StateCondition<S, T> {
    /// Private Identifier of the state.
    /// 
    pub(in crate::state_engine) private: S,
    /// [`SharedState`] that the Aggregator holds.
    ///
    pub(in crate::state_engine) shared: SharedState<T>,
}

impl<S, T> StateCondition<S, T>
where
    Self: State<T>,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<StateEngine<T>> {
        info!("Server runs in state: {:?}", &Self::NAME);
        async move {
            if let Err(err) = self.perform().await {
                warn!("server failed to perform task of state {:?}", &Self::NAME);
                return Some(self.into_failure_state(err));
            }
            self.next().await
        }
        .await
    }
    /// Receives the next ['Request'].
    pub async fn next_request(
        &mut self,
    ) -> Result<(StateEngineRequest, ResponseSender), StateError> {
        todo!()
    }

    fn into_failure_state(self, err: StateError) -> StateEngine {
        StateCondition::<Failure, T>::new(err, self.shared).into()
    }
}

/// [`SharedState`]
pub struct SharedState<T> {
    /// [`Aggregator`]
    pub aggr: Aggregator<T>,
    /// [`RequestReceiver`] for enabling receiving requests from the client.
    /// 
    pub rx: RequestReceiver,
    /// [`EventPublisher`] responsible for publishing the latest updates.
    ///
    pub publisher: EventPublisher,
}

impl<T> SharedState<T> {
    /// Init new [`SharedState`] for the aggregation server.
    pub fn new(aggr: Aggregator<T>, rx: RequestReceiver, publisher: EventPublisher) -> Self {
        SharedState {
            aggr,
            rx,
            publisher,
        }
    }
}
