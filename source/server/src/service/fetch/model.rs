use futures::task::Context;
use std::io::Error;
use std::task::Poll;
use tower::Service;

use crate::engine::{channel::RequestReceiver, model::Model};

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

/// [`ModelService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct ModelRequest;

pub struct ModelService {
    pub handle: RequestReceiver,
}

impl ModelService {
    /// Create a new (tower) service with the a handler for returning
    /// a global model to the client.
    pub fn new(handle: RequestReceiver) -> Self {
        Self { handle }
    }
}

impl Service<ModelRequest> for ModelService {
    type Response = Model;
    type Error = Error;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ModelRequest) -> Self::Future {
        todo!()
    }
}
