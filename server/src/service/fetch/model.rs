use futures::{
    future::{self, Ready},
    task::Context,
};
use std::io::Error;
use std::task::Poll;
use tower::Service;

use crate::{core::model::ModelUpdate, engine::watch::Subscriber, service::fetch::ModelRequest};

#[derive(Debug, Clone)]
pub struct ModelService {
    pub subscriber: Subscriber,
}

impl ModelService {
    /// Create a new (tower) service with the a handler for returning
    /// a global model to the client.
    pub fn new(subscriber: Subscriber) -> Self {
        Self { subscriber }
    }
}

impl Service<ModelRequest> for ModelService {
    type Response = ModelUpdate;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ModelRequest) -> Self::Future {
        future::ready(Ok(self.subscriber.rx.recv()))
    }
}
