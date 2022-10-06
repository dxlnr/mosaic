//! Initializes the [`StateEngine`].
//!

use derive_more::Display;
use thiserror::Error;

use crate::{
    aggr::Aggregator,
    state_engine::{
        channel::{RequestReceiver, RequestSender},
        event::{EventPublisher, EventSubscriber},
        states::{Idle, SharedState, StateCondition, StateName},
        StateEngine,
    },
};

/// Errors occuring during the initialization process and the [`StateEngine`].
#[derive(Debug, Display, Error)]
pub enum StateEngineInitError {
    /// Initialization of storage connection failed: {0}
    // StorageInit(StorageError),
    StorageInit(Box<dyn std::error::Error>),
}

/// Responsible for the initialization of the [`StateEngine`].
///
/// Takes various settings and links them to the process.
pub struct StateEngineInitializer {}

impl StateEngineInitializer {
    /// Creates a new [`EngineInitializer`] which sets up the engine running the aggregation algorithm.
    pub fn new() -> Self {
        Self {}
    }
    /// Initializes the [`StateEngine`] and the communication handlers.
    pub async fn init(
        self,
        ) -> Result<(StateEngine, RequestSender, EventSubscriber), StateEngineInitError> {
        let (publisher, subscriber) = EventPublisher::new(0, StateName::Idle);
        let (rx, tx) = RequestSender::new();

        let shared = SharedState::new(Aggregator::new(), rx, publisher);

        Ok((
            StateEngine::Idle(StateCondition::<Idle>::new(shared)),
            tx,
            subscriber,
        ))
    }

    // async fn from_previous_state() {}
    // async fn try_restore_state() {}

    // /// Establishes the storage connection via instantiation of [`S3Client`].
    // pub async fn init_storage(&self) -> Result<(), StorageError> {
    //     todo!()
    // }
}
