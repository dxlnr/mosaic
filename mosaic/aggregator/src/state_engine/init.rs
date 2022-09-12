//! Initializes the [`StateEngine`].
//! 

use derive_more::{Display, From};
use thiserror::Error;

use crate::{
    state_engine::{
        channel::{RequestReceiver, RequestSender},
        event::EventSubscriber,
        StateEngine,
    },
};

/// Errors occuring during the initialization process and the [`StateEngine`].
#[derive(Debug, Display, Error)]
pub enum EngineInitializationError {
    /// Initialization of storage connection failed: {0}
    // StorageInit(StorageError),
    StorageInit(Box<dyn std::error::Error>),
}

/// Responsible for the initialization of the [`StateEngine`].
///
/// Takes various settings and links them to the process.
pub struct EngineInitializer {}

impl EngineInitializer {
    /// Creates a new [`EngineInitializer`] which sets up the engine running the aggregation algorithm.
    pub fn new() -> Self {
        todo!()
    }
    /// Initializes the [`StateEngine`] and the communication handlers.
    pub async fn init(self) -> Result<(StateEngine, RequestSender, EventSubscriber), EngineInitializationError> {
        todo!()
    }
    // /// Establishes the storage connection via instantiation of [`S3Client`].
    // pub async fn init_storage(&self) -> Result<(), StorageError> {
    //     todo!()
    // }
}