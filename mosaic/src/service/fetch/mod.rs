mod model;
mod stats;

use async_trait::async_trait;
use futures::future::poll_fn;
use std::task::{Context, Poll};
use tower::{layer::Layer, Service, ServiceBuilder};

use self::{
    model::{ModelRequest, ModelService},
    stats::{StatsRequest, StatsService},
};
use crate::{
    core::model::ModelUpdate,
    engine::watch::Subscriber,
    rest::stats::StatsUpdate,
    service::error::{into_service_error, ServiceError},
};

/// An interface for retrieving data from running engine process.
#[async_trait]
pub trait Fetch {
    /// Fetch the statistics from the running process / update round.
    async fn fetch_stats(&mut self) -> Result<StatsUpdate, ServiceError>;

    /// Fetch the latest global model.
    async fn fetch_model(&mut self) -> Result<ModelUpdate, ServiceError>;
}

#[derive(Debug, Clone)]
pub struct Fetcher<M, S> {
    pub model_service: M,
    pub stats_service: S,
}

#[async_trait]
impl<M, S> Fetch for Fetcher<M, S>
where
    Self: Send + Sync + 'static,

    M: Service<ModelRequest, Response = ModelUpdate> + Send + 'static,
    <M as Service<ModelRequest>>::Future: Send + Sync + 'static,
    <M as Service<ModelRequest>>::Error: Into<Box<dyn std::error::Error + 'static + Sync + Send>>,

    S: Service<StatsRequest, Response = StatsUpdate> + Send + 'static,
    <S as Service<StatsRequest>>::Future: Send + Sync + 'static,
    <S as Service<StatsRequest>>::Error: Into<Box<dyn std::error::Error + 'static + Sync + Send>>,
{
    async fn fetch_stats(&mut self) -> Result<StatsUpdate, ServiceError> {
        poll_fn(|cx| <S as Service<StatsRequest>>::poll_ready(&mut self.stats_service, cx))
            .await
            .map_err(into_service_error)?;
        Ok(self
            .stats_service
            .call(StatsRequest)
            .await
            .map_err(into_service_error)?)
    }
    async fn fetch_model(&mut self) -> Result<ModelUpdate, ServiceError> {
        poll_fn(|cx| <M as Service<ModelRequest>>::poll_ready(&mut self.model_service, cx))
            .await
            .map_err(into_service_error)?;
        Ok(self
            .model_service
            .call(ModelRequest)
            .await
            .map_err(into_service_error)?)
    }
}

impl<M, S> Fetcher<M, S> {
    pub fn new(model_service: M, stats_service: S) -> Self {
        Self {
            model_service,
            stats_service,
        }
    }
}

pub(in crate::service) struct FetcherService<S>(S);

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

pub(in crate::service) struct FetcherLayer;

impl<S> Layer<S> for FetcherLayer {
    type Service = FetcherService<S>;

    fn layer(&self, service: S) -> Self::Service {
        FetcherService(service)
    }
}

/// Construct a [`Fetcher`] service
pub fn init_fetcher(subs: &Subscriber) -> impl Fetch + Sync + Send + Clone + 'static {
    let model = ServiceBuilder::new()
        .buffer(100)
        .concurrency_limit(100)
        .layer(FetcherLayer)
        .service(ModelService::new(subs));

    let stats = ServiceBuilder::new()
        .buffer(100)
        .concurrency_limit(100)
        .layer(FetcherLayer)
        .service(StatsService::new(subs));

    Fetcher::new(model, stats)
}
