use futures::task::Context;
use std::io::Error;
use std::task::Poll;
use tower::Service;

use crate::{engine::channel::RequestSender, message::Message};

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

#[derive(Debug, Clone)]
pub struct ClientService {
    pub response: RequestSender,
}

impl ClientService {
    /// Create a new (tower) service with a handler for responding
    /// to the client that sent a request to the ['Engine'].
    pub fn new(response: RequestSender) -> Self {
        Self { response }
    }
}

impl Service<Message> for ClientService {
    type Response = ();
    type Error = Error;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Message) -> Self::Future {
        todo!()
    }
}
