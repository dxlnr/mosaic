use displaydoc::Display;
use thiserror::Error;

/// Errors for the message parsing service.
#[derive(Debug, Display, Error)]
pub enum ServiceError {
    /// Error while operating the message parsing service: {0}
    ParsingError(String),
    /// Error while trying to read parameters.
    ParamsError,
    /// Error while sending a request to the engine.
    RequestError,
    /// Internal error: {0}
    InternalError(String),

}

impl From<Box<dyn std::error::Error>> for ServiceError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        match e.downcast::<ServiceError>() {
            Ok(e) => *e,
            Err(e) => ServiceError::InternalError(format!("{}", e)),
        }
    }
}

impl From<Box<dyn std::error::Error + Sync + Send>> for ServiceError {
    fn from(e: Box<dyn std::error::Error + Sync + Send>) -> Self {
        ServiceError::from(e as Box<dyn std::error::Error>)
    }
}