use futures::{
    future::{self, Ready},
    task::Context,
};
use std::task::Poll;
use tower::Service;

pub struct StatsService(Listener<StatsUpdate>);

impl StatsService {
    /// Create a new (tower) service for broadcasting the running process statistics.
    pub fn new(subscriber: Subscriber) -> Self {
        Self { subscriber }
    }
}

impl Service<ModelRequest> for StatsService {
    type Response = ModelUpdate;
    type Error = ServiceError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ModelRequest) -> Self::Future {
        future::ready(Ok(self.subscriber.rx.recv()))
    }
}
