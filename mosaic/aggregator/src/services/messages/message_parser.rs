use futures::{future, task::Context};
use std::{str::FromStr, task::Poll};
use tower::Service;

use mosaic_core::message::Message;
use crate::{
    services::error::ServiceError,
};

/// [`MessageParser`] object
#[derive(Debug, Clone, Default)]
pub struct MessageParser;

impl MessageParser {
    /// Create a new (tower) service for parsing any incoming message (request).
    pub fn new() -> Self {
        Self
    }

    // fn parse(req: ClientUpdate) -> Result<Message, ServiceError> {
    //     todo!()
    // }
}

// impl Service<ClientUpdate> for MessageParser {
//     type Response = Message;
//     type Error = ServiceError;
//     type Future = future::Ready<Result<Self::Response, Self::Error>>;

//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: ClientUpdate) -> Self::Future {
//         future::ready(MessageParser::parse(req))
//     }
// }
