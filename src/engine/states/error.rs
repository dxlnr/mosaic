/// Error handling while running the process via ['Engine'].
use displaydoc::Display;
use thiserror::Error;

use crate::engine::StorageError;

/// Handling state errors when iterating ['Engine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Some error in the Idle state: {0}.
    IdleError(StorageError),
    /// Request channel error: {0}.
    RequestChannel(&'static str),
    /// Some error in the Aggregation state: {0}.
    AggregationError(StorageError),
    /// No local features available for aggregation.
    FeatureError(&'static str),
    /// Error while trying to send a post request.
    PostRequest,
}