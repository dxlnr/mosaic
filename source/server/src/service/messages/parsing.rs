use futures::{future, task::Context};
use std::{task::Poll, str::FromStr};
use tower::Service;

use crate::{message::Message, service::error::ServiceError, engine::model::DataType, server::mosaic::ClientUpdate};

/// Message parsing object
#[derive(Debug, Clone, Default)]
pub struct MessageParser;

impl MessageParser {
    /// Create a new (tower) service for parsing any incoming message (request).
    pub fn new() -> Self {
        Self 
    }
}

impl Service<ClientUpdate> for MessageParser
{
    type Response = Message;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ClientUpdate) -> Self::Future {
        let resp = req.parameters.ok_or( ServiceError::ParsingError);

        let dtype = match &resp {
            Ok(resp) => Ok(DataType::from_str(&resp.data_type).unwrap()),
            _ => Err(ServiceError::ParsingError),
        };

        let params = resp.unwrap();
        future::ready(match dtype {
            Ok(dtype) => Ok(Message::new(req.id, params.model_version, params.tensor, dtype)),
            _ => Err(ServiceError::ParsingError),
        })
    }
}