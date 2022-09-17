use derive_more::Display;
use thiserror::Error;

/// Errors for the message parsing service.
#[derive(Debug, Display, Error)]
pub enum ServiceError {
    /// Error while operating the message parsing service: {0}
    ParsingError(String),
    /// Error while trying to read parameters.
    ParamsError,
    /// Error while trying to read the process meta data.
    MetaDataError,
    /// Error while sending a request to the engine.
    RequestError,
    /// Internal error: {0}
    InternalError(String),
    /// Fetching error: {0}
    FetchError(anyhow::Error),
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

pub fn into_service_error<E: Into<Box<dyn std::error::Error + 'static + Sync + Send>>>(
    e: E,
) -> ServiceError {
    ServiceError::FetchError(anyhow::anyhow!("Fetcher failed: {:?}", e.into()))
}
