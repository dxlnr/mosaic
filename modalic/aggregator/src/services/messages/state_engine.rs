use std::task::Poll;

use futures::task::Context;
use tower::Service;
use modalic_core::message::Message;

use crate::{
    services::messages::{BoxedServiceFuture, ServiceError},
    state_engine::channel::RequestSender,
};

/// A service that hands the requests to the [`StateEngine`] that runs in the background.
///
/// [`StateEngine`]: crate::state_machine::StateEngine
#[derive(Debug, Clone)]
pub struct StateEngine {
    handle: RequestSender,
}

impl StateEngine {
    /// Create a new service with the given handle for forwarding
    /// requests to the state machine. The handle should be obtained
    /// via [`init()`].
    ///
    /// [`init()`]: crate::state_machine::initializer::StateEngineInitializer::init
    pub fn new(handle: RequestSender) -> Self {
        Self { handle }
    }
}

impl Service<Message> for StateEngine {
    type Response = ();
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Message) -> Self::Future {
        let handle = self.handle.clone();
        Box::pin(async move {
            handle
                .request(req.into(), tracing::Span::none())
                .await
                .map_err(ServiceError::StateEngine)
        })
    }
}
