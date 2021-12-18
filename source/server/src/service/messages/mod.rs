mod engine;
use futures::future::poll_fn;
use std::io::{Error, ErrorKind};
use tower::Service;

use self::engine::EngineService;
use crate::{engine::channel::RequestSender, message::Message};

#[derive(Debug, Clone)]
pub struct MessageHandler {
    pub engine_service: EngineService,
}

impl MessageHandler {
    pub fn new(tx: RequestSender) -> Self {
        MessageHandler {
            engine_service: EngineService::new(tx),
        }
    }
    pub async fn process(&mut self, message: Message) -> Result<(), Error> {
        poll_fn(|cx| self.engine_service.poll_ready(cx)).await?;
        self.engine_service.call(message).await
    }
}
