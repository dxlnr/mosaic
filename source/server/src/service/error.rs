use displaydoc::Display;
use thiserror::Error;

/// Errors for the message parsing service.
#[derive(Debug, Display, Error)]
pub enum ServiceError {
    /// Error while operating the message parsing service.
    ParsingError,
    /// Error while sending a request to the engine.
    RequestError,
}