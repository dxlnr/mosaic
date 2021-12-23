use futures::{
    future::{self, Ready},
    task::Context,
};
use std::io::Error;
use std::task::Poll;
use tower::Service;

use crate::engine::{model::Model, watch::Subscriber};

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

// /// [`ModelService`]'s request type
// #[derive(Default, Clone, Eq, PartialEq, Debug)]
// pub struct ModelRequest;

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

impl Service<Model> for ModelService {
    type Response = Model;
    type Error = Error;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Model) -> Self::Future {
        // let mut subs = self.subscriber.clone();
        // self.0.call(req)
        future::ready(self.recv())
    }
}
