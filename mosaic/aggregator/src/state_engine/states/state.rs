use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use thiserror::Error;
use tracing::{debug, debug_span, info, warn, Span};
use tracing_futures::Instrument;

use crate::{
    aggr::Aggregator,
    state_engine::{
        channel::{RequestReceiver, ResponseSender, StateEngineRequest},
        events::EventPublisher,
        states::{IdleError, UpdateError},
        Failure, StateEngine,
    },
    storage::Storage,
};

/// Handling state errors when running ['StateEngine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Request channel error: {0}.
    RequestChannel(&'static str),
    /// Idle phase failed: {0}.
    Idle(#[from] IdleError),
    /// Update phase failed: {0}.
    Update(#[from] UpdateError),
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
    #[display(fmt = "Unmask")]
    Unmask,
    #[display(fmt = "Failure")]
    Failure,
    #[display(fmt = "Shutdown")]
    Shutdown,
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait State<T>
where
    T: Storage,
{
    /// The name of the current state.
    const NAME: StateName;

    /// Performs the attached tasks of current state.
    async fn perform(&mut self) -> Result<(), StateError>;

    /// Publishes data of current state (Default: None).
    fn publish(&mut self) {}

    /// Moves from the current state to the next state.
    async fn next(self) -> Option<StateEngine<T>>;
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
    S: Send,
    T: Storage,
    Self: State<T>,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<StateEngine<T>> {
        info!("Aggregator runs in state: {:?}", &Self::NAME);
        let span = debug_span!("run_state", state = %&Self::NAME);

        async move {
            self.shared.publisher.broadcast_state(Self::NAME);

            if let Err(err) = self.perform().await {
                warn!(
                    "Aggregator failed to perform task of state {:?}",
                    &Self::NAME
                );
                return Some(self.into_failure_state(err));
            }

            self.publish();

            debug!("Transitioning to the next state.");
            self.next().await
        }
        .instrument(span)
        .await
    }
    /// Receives the next ['StateEngineRequest'].
    pub async fn next_request(
        &mut self,
    ) -> Result<(StateEngineRequest, Span, ResponseSender), StateError> {
        info!("Aggregator waiting for the next incoming request.");
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
    ) -> Result<Option<(StateEngineRequest, Span, ResponseSender)>, StateError> {
        match self.shared.rx.try_recv() {
            Some(Some(item)) => Ok(Some(item)),
            None => {
                debug!("No pending request.");
                Ok(None)
            }
            Some(None) => {
                warn!("Failed to get next pending request: Channel will be shut down.");
                Err(StateError::RequestChannel(
                    "All message senders have been dropped!",
                ))
            }
        }
    }

    fn into_failure_state(self, err: StateError) -> StateEngine<T> {
        StateCondition::<Failure, _>::new(self.shared, err).into()
    }
}

/// [`SharedState`]
pub struct SharedState<T> {
    /// [`Aggregator`]
    pub(in crate::state_engine) aggr: Aggregator,
    /// [`RequestReceiver`] for enabling receiving requests from the client.
    ///
    pub(in crate::state_engine) rx: RequestReceiver,
    /// [`EventPublisher`] responsible for publishing the latest updates.
    ///
    pub(in crate::state_engine) publisher: EventPublisher,
    /// The store for storing coordinator and model data.
    pub(in crate::state_engine) store: T,
}

impl<T> SharedState<T> {
    /// Init new [`SharedState`] for the aggregation server.
    pub fn new(aggr: Aggregator, publisher: EventPublisher, rx: RequestReceiver, store: T) -> Self {
        SharedState {
            aggr,
            rx,
            publisher,
            store,
        }
    }
}
