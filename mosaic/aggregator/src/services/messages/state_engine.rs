use futures::task::Context;
use std::task::Poll;
use tower::Service;

// use mosaic_core::message::Message;
use crate::{
    state_engine::channel::{StateEngineRequest, RequestSender},
    services::{error::ServiceError, messages::BoxedServiceFuture},
};

/// [`StateEngineService`]
#[derive(Debug, Clone)]
pub struct StateEngineService {
    pub handle: RequestSender,
}

impl StateEngineService {
    /// Create a new (tower) service with a handler for forwarding
    /// requests from gRPC [`AggrServer`] to the [`StateEngine`].
    /// 
    pub fn new(handle: RequestSender) -> Self {
        Self { handle }
    }
}

impl Service<StateEngineRequest> for StateEngineService {
    type Response = ();
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: StateEngineRequest) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move {
            handle
                .send(req)
                .await
                .map_err(|_| ServiceError::RequestError)
        })
    }
}
