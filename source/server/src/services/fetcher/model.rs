use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures::future::{self, Ready};
use tower::Service;

/// [`ModelService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct ModelRequest;

/// [`ModelService`]'s response type.
///
/// The response is `None` when no model is currently available.
pub type ModelResponse = Option<Arc<Vec<f64>>>;

/// A service that serves the latest available global model
pub struct ModelService();

impl ModelService {
    pub fn new() -> Self {
        Self()
    }
}

impl Service<ModelRequest> for ModelService {
    type Response = ModelResponse;
    type Error = std::convert::Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ModelRequest) -> Self::Future {
        todo!();
    }
}
