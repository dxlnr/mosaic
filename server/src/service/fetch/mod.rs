mod model;
use futures::future::poll_fn;
use tower::Service;

use self::model::ModelService;
use crate::{core::model::ModelUpdate, engine::watch::Subscriber, service::error::ServiceError};

/// [`ModelService`]'s request type
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct ModelRequest;

#[derive(Debug, Clone)]
pub struct Fetcher {
    pub model_service: ModelService,
}

impl Fetcher {
    pub fn new(rx: Subscriber) -> Self {
        Fetcher {
            model_service: ModelService::new(rx),
        }
    }
    pub async fn fetch(&mut self) -> Result<ModelUpdate, ServiceError> {
        poll_fn(|cx| self.model_service.poll_ready(cx)).await?;
        self.model_service.call(ModelRequest).await
    }
}
