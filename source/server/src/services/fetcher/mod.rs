mod model;
pub use self::model::{ModelResponse, ModelService};
use std::{
    sync::Arc,
    task::{Context, Poll},
};

// use futures::future::{self, Ready};
use crate::engine::model::Model;
use tower::{layer::Layer, Service, ServiceBuilder};

#[derive(Default, Debug, Clone)]
pub struct Fetchers {
    pub model: Model,
}

impl Fetchers {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
}

pub(in crate::services) struct FetcherService<S>(S);

impl<S, R> Service<R> for FetcherService<S>
where
    S: Service<R>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        self.0.call(req)
    }
}

pub(in crate::services) struct FetcherLayer;

impl<S> Layer<S> for FetcherLayer {
    type Service = FetcherService<S>;

    fn layer(&self, service: S) -> Self::Service {
        FetcherService(service)
    }
}

pub fn fetcher() -> Fetchers {
    // let model = ServiceBuilder::new()
    //     .buffer(100)
    //     .concurrency_limit(100)
    //     .layer(FetcherLayer)
    //     .service(ModelService::new());

    Fetchers::new(Model::new())
}
