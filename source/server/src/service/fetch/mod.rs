mod model;
use futures::future::poll_fn;
use std::io::Error;
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
    pub async fn forward(&mut self, model: Model) -> Result<(), Error> {
        poll_fn(|cx| self.model_service.poll_ready(cx)).await?;
        self.model_service.call(model).await
    }
}
