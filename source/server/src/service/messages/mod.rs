mod engine;
mod parsing;

use futures::future::poll_fn;
use tower::Service;

use self::{engine::EngineService, parsing::MessageParser};
use crate::{engine::channel::RequestSender, message::Message, service::error::ServiceError};
use crate::server::mosaic::ClientUpdate;

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

#[derive(Debug, Clone)]
pub struct MessageHandler {
    pub engine_service: EngineService,
    pub parser: MessageParser,
}

impl MessageHandler {
    pub fn new(tx: RequestSender) -> Self {
        MessageHandler {
            engine_service: EngineService::new(tx),
            parser: MessageParser::new(),
        }
    }
    /// parsing the incoming client requests.
    async fn parse(&mut self, req: ClientUpdate) -> Result<Message, ServiceError> {
        poll_fn(|cx| self.parser.poll_ready(cx)).await?;
        self.parser.call(req).await
    }
    /// forwards a message to the ['Engine']
    pub async fn forward(&mut self, message: Message) -> Result<(), ServiceError> {
        poll_fn(|cx| self.engine_service.poll_ready(cx)).await?;
        self.engine_service.call(message).await
    }
    /// handles all incoming client requests.
    pub async fn handle(&mut self, req: ClientUpdate) -> Result<(), ServiceError> {
        let message = self.parse(req).await?;
        self.forward(message).await
    }
}

