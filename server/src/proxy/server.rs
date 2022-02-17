//! Sets up a gRPC server.
use std::convert::Infallible;
// use std::io::{Error, ErrorKind};
use thiserror::Error;
use tracing::{info, warn};

use tonic::{transport::Server, Code, Request, Response, Status};

use crate::{
    core::model::ModelUpdate,
    service::{error::ServiceError, fetch::Fetch, messages::MessageHandler},
    settings::APISettings,
};

pub mod mosaic {
    tonic::include_proto!("mosaic");
}

use mosaic::communication_server::{Communication, CommunicationServer};
#[allow(unused_imports)]
use mosaic::{ClientMessage, ClientUpdate, Parameters, ServerMessage, ServerModel};

#[derive(Debug, Clone)]
pub struct Communicator<F> {
    /// Shared handle for passing messages from participant to engine.
    handler: MessageHandler,
    /// Shared fetcher for passing messages from engine to client.
    fetcher: F,
}

impl<F> Communicator<F>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    /// Constructs a new CommunicationServer
    fn new(handler: MessageHandler, fetcher: F) -> Self {
        Communicator { handler, fetcher }
    }
    /// Forwards the incoming request to the ['Engine'].
    async fn handle_message(
        req: ClientUpdate,
        mut handler: MessageHandler,
    ) -> Result<(), ServiceError> {
        let _ = handler.handle(req).await.map_err(|e| {
            warn!("failed to handle ClientRequest: {:?}", e);
        });
        Ok(())
    }
    /// Handles the request for the latest global model from the ['Engine'].
    async fn handle_model(mut fetcher: F) -> Result<ModelUpdate, ServiceError> {
        Ok(fetcher.fetch_model().await?)
    }
}

#[tonic::async_trait]
impl<F> Communication for Communicator<F>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    async fn get_global_model(
        &self,
        request: Request<ClientMessage>,
    ) -> Result<Response<ServerModel>, Status> {
        info!(
            "Request received from client {:?} requesting a global model.",
            request.remote_addr().unwrap()
        );
        let fetch = self.fetcher.clone();
        let res = Communicator::handle_model(fetch).await.map_err(|e| {
            warn!("Returning the global model failed: {:?}", e);
            Status::new(Code::Internal, e.to_string())
        })?;

        let server_msg = mosaic::ServerModel {
            parameters: res.map(|r| r.wrapper_to_params()),
        };
        Ok(Response::new(server_msg))
    }

    async fn update(
        &self,
        request: Request<ClientUpdate>,
    ) -> Result<Response<ServerMessage>, Status> {
        info!(
            "Request received from client {}: Sending an update to engine.",
            &request.remote_addr().unwrap()
        );
        let handle = self.handler.clone();
        let res = Communicator::<F>::handle_message(request.into_inner().clone(), handle).await;

        let server_msg = mosaic::ServerMessage {
            status: match res {
                Ok(()) => 0,
                _ => 1,
            },
        };
        Ok(Response::new(server_msg))
    }
}

pub async fn start<F>(
    api_settings: &APISettings,
    message_handler: MessageHandler,
    fetcher: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fetch + Sync + Send + 'static + Clone,
{
    info!("Communication Server listening on {}", api_settings.address);

    let com = Communicator::new(message_handler, fetcher);
    Server::builder()
        .add_service(CommunicationServer::new(com))
        .serve(api_settings.address)
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
/// Depicts server error.
pub enum ServerError {
    #[error("invalid TLS configuration was provided")]
    InvalidTlsConfig,
}

impl From<Infallible> for ServerError {
    fn from(infallible: Infallible) -> ServerError {
        match infallible {}
    }
}
