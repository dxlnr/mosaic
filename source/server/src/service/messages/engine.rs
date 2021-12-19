use futures::task::Context;
use std::task::Poll;
use tower::Service;

// use std::convert::Infallible;
use std::io::Error;

use crate::{engine::channel::RequestSender, message::Message};

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

#[derive(Debug, Clone)]
pub struct EngineService {
    pub handle: RequestSender,
}

impl EngineService {
    /// Create a new (tower) service with the a handler for forwarding
    /// requests from gRPC setup to the engine.
    pub fn new(handle: RequestSender) -> Self {
        Self { handle }
    }
}

impl Service<Message> for EngineService {
    type Response = ();
    type Error = Error;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Message) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move { handle.sending(req).await })
    }
}
