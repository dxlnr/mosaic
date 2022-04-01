use futures::{
    future::{self, Ready},
    task::Context,
};
use std::task::Poll;
use tower::Service;

use crate::{
    engine::watch::{Listener, Subscriber},
    rest::stats::StatsUpdate,
    service::error::ServiceError,
};

/// [`ModelService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct StatsRequest;

#[derive(Debug, Clone)]
pub struct StatsService(Listener<StatsUpdate>);

impl StatsService {
    /// Create a new (tower) service for broadcasting the running process statistics.
    pub fn new(subs: &Subscriber) -> Self {
        Self(subs.get_listener_stats())
    }
}

impl Service<StatsRequest> for StatsService {
    type Response = StatsUpdate;
    type Error = ServiceError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: StatsRequest) -> Self::Future {
        future::ready(Ok(self.0.recv().event))
    }
}
