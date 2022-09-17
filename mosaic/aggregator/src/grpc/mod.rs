//! Implements the `Aggregator` as an independent GRPC server application.
//!
pub mod msflp {
    tonic::include_proto!("mosaic.protos");
}

use std::{convert::Infallible, pin::Pin};

use futures::Stream;
use thiserror::Error;
use tracing::info;

use tonic::{transport::Server, Status};

use crate::{
    configs::APISettings,
    services::messages::MessageHandler
};

use msflp::msflp_server::{Msflp, MsflpServer};
use msflp::{ClientMessage, ServerMessage};

type ResponseStream = Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send>>;

/// [`AggrServer`] Implements the MSFLP server trait.
/// 
/// The [`AggrServer`] has two main tasks:
///     - Implementing the service trait generated from our service definition.
///     - Running a gRPC server to listen for requests from clients.
/// 
#[derive(Debug, Clone)]
pub struct AggrServer {
    /// Shared handle for passing messages from participant to engine.
    handler: MessageHandler,
}

impl AggrServer
{
    /// Constructs a new [`AggrServer`].
    fn new(handler: MessageHandler) -> Self {
        AggrServer { handler }
    }
}

#[tonic::async_trait]
impl Msflp for AggrServer
{   
    type HandleStream = ResponseStream;

    async fn handle(
        &self,
        request: tonic::Request<tonic::Streaming<ClientMessage>>,
    ) -> Result<tonic::Response<Self::HandleStream>, tonic::Status>
    {
        todo!()
    }
}

pub async fn start<F>(
    api_settings: &APISettings,
    message_handler: MessageHandler,
) -> Result<(), Box<dyn std::error::Error>>
{
    info!("Aggregation Server listening on {}", api_settings.server_address);

    let aggr_server = AggrServer::new(message_handler);
    Server::builder()
        .add_service(MsflpServer::new(aggr_server))
        .serve(api_settings.server_address)
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
/// Depicts [`ServerError`].
pub enum ServerError {
    #[error("invalid TLS configuration was provided")]
    InvalidTlsConfig,
}

impl From<Infallible> for ServerError {
    fn from(infallible: Infallible) -> ServerError {
        match infallible {}
    }
}