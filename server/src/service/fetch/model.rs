use futures::{
    future::{self, Ready},
    task::Context,
};
use std::task::Poll;
use tower::Service;

use crate::{
    core::model::ModelUpdate,
    engine::watch::{Listener, Subscriber},
    service::error::ServiceError,
};

/// [`ModelService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct ModelRequest;

#[derive(Debug, Clone)]
pub struct ModelService(Listener<ModelUpdate>);

impl ModelService {
    /// Create a new (tower) service with the a handler for returning a global model to the client.
    pub fn new(subs: &Subscriber) -> Self {
        Self(subs.get_listener_model())
    }
}

impl Service<ModelRequest> for ModelService {
    type Response = ModelUpdate;
    type Error = ServiceError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ModelRequest) -> Self::Future {
        future::ready(Ok(self.0.recv().event))
    }
}
