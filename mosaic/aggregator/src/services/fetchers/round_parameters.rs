use std::task::{Context, Poll};

use futures::future::{self, Ready};
use tower::Service;
use tracing::error_span;
use tracing_futures::{Instrument, Instrumented};

use crate::state_engine::events::{EventListener, EventSubscriber};
use mosaic_core::common::RoundParameters;

/// [`RoundParamsService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct RoundParamsRequest;

/// [`RoundParamsService`]'s response type
pub type RoundParamsResponse = RoundParameters;

/// A service that serves the round parameters for the current round.
pub struct RoundParamsService(EventListener<RoundParameters>);

impl RoundParamsService {
    pub fn new(events: &EventSubscriber) -> Self {
        Self(events.params_listener())
    }
}

impl Service<RoundParamsRequest> for RoundParamsService {
    type Response = RoundParameters;
    type Error = std::convert::Infallible;
    type Future = Instrumented<Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: RoundParamsRequest) -> Self::Future {
        future::ready(Ok(self.0.get_latest().event))
            .instrument(error_span!("round_params_fetch_request"))
    }
}
