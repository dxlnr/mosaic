use futures::task::Context;
use std::task::Poll;
use tower::Service;

use crate::{engine::channel::RequestSender, message::Message, service::{error::ServiceError, messages::BoxedServiceFuture}};

#[derive(Debug, Clone)]
pub struct EngineService {
    pub handle: RequestSender,
}

impl EngineService {
    /// Create a new (tower) service with a handler for forwarding
    /// requests from gRPC setup to the engine.
    pub fn new(handle: RequestSender) -> Self {
        Self { handle }
    }
}

impl Service<Message> for EngineService {
    type Response = ();
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Message) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move { handle.send(req).await })
    }
}
