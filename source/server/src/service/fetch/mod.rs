mod model;
use futures::future::poll_fn;
use std::io::Error;
use std::sync::Arc;
use tower::Service;

use self::model::ModelService;
use crate::engine::{model::Model, watch::Subscriber};

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
    pub async fn forward(&mut self, model: Model) -> Result<Arc<Model>, Error> {
        poll_fn(|cx| self.model_service.poll_ready(cx)).await?;
        self.model_service.call(model).await
    }
}
