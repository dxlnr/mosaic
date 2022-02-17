mod model;

use async_trait::async_trait;
use futures::future::poll_fn;
use std::task::{Context, Poll};
use tower::{layer::Layer, Service, ServiceBuilder};

use self::model::{ModelRequest, ModelService};
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
pub struct Fetcher<M> {
    pub model_service: M,
}

#[async_trait]
impl<M> Fetch for Fetcher<M>
where
    Self: Send + Sync + 'static,

    M: Service<ModelRequest, Response = ModelUpdate> + Send + 'static,
    <M as Service<ModelRequest>>::Future: Send + Sync + 'static,
    <M as Service<ModelRequest>>::Error: Into<Box<dyn std::error::Error + 'static + Sync + Send>>,
{
    async fn fetch_stats(&mut self) -> Result<StatsUpdate, ServiceError> {
        todo!()
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

// impl Fetcher {
//     pub fn new(rx: Subscriber) -> Self {
//         Fetcher {
//             model_service: ModelService::new(&rx),
//         }
//     }
//     pub async fn fetch(&mut self) -> Result<ModelUpdate, ServiceError> {
//         poll_fn(|cx| self.model_service.poll_ready(cx)).await?;
//         self.model_service.call(ModelRequest).await
//     }
// }

impl<M> Fetcher<M> {
    pub fn new(model_service: M) -> Self {
        Self { model_service }
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

    Fetcher::new(model)
}
