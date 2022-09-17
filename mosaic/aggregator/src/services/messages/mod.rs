pub mod state_engine;
pub mod message_parser;

use futures::future::poll_fn;
use tower::Service;

// use mosaic_core::message::Message;
use self::state_engine::StateEngineService;
use crate::{
    state_engine::channel::{RequestSender, StateEngineRequest},
    services::error::ServiceError,
};

pub type BoxedServiceFuture<Response, Error> = std::pin::Pin<
    Box<dyn futures::Future<Output = Result<Response, Error>> + 'static + Send + Sync>,
>;

#[derive(Debug, Clone)]
pub struct MessageHandler {
    pub state_engine_service: StateEngineService,
    pub parser: message_parser::MessageParser,
}

impl MessageHandler {
    pub fn new(tx: RequestSender) -> Self {
        MessageHandler {
            state_engine_service: StateEngineService::new(tx),
            parser: message_parser::MessageParser::new(),
        }
    }
    // /// parsing the incoming client requests.
    // async fn parse(&mut self, req: ClientUpdate) -> Result<Message, ServiceError> {
    //     poll_fn(|cx| self.parser.poll_ready(cx)).await?;
    //     self.parser.call(req).await
    // }
    /// forwards a message to the ['StateEngine']
    pub async fn forward(&mut self, req: StateEngineRequest) -> Result<(), ServiceError> {
        poll_fn(|cx| self.state_engine_service.poll_ready(cx)).await?;
        self.state_engine_service.call(req).await
    }
    // /// handles all incoming client requests.
    // pub async fn handle(&mut self, req: ClientUpdate) -> Result<(), ServiceError> {
    //     let message = self.parse(req).await?;
    //     self.forward(message).await
    // }
}
