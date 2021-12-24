//! Sets up a gRPC server.
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use tracing::{info, warn};

use tonic::{transport::Server, Request, Response, Status};

use crate::{
    engine::watch::Subscriber,
    message::Message,
    service::{fetch::Fetcher, messages::MessageHandler},
    settings::APISettings,
};

pub mod mosaic {
    tonic::include_proto!("mosaic");
}

use mosaic::communication_server::{Communication, CommunicationServer};
#[allow(unused_imports)]
use mosaic::{
    ClientDefault, ClientMessage, ClientUpdate, Parameters, ServerDefault, ServerMessage,
    ServerModel,
};

#[derive(Debug, Clone)]
pub struct Communicator {
    model: Arc<Mutex<Vec<f64>>>,
    /// Shared handle for passing messages from participant to engine.
    handler: MessageHandler,
    fetcher: Fetcher,
}

impl Communicator {
    /// Constructs a new CommunicationServer
    fn new(handler: MessageHandler, fetcher: Fetcher, model_length: usize) -> Self {
        Communicator {
            model: Arc::new(Mutex::new(vec![0.0; model_length])),
            handler,
            fetcher,
        }
    }
    /// Forwards the incoming request to the ['Engine']
    async fn handle_message(msg: Message, mut handler: MessageHandler) -> Result<(), Infallible> {
        let _ = handler.forward(msg).await.map_err(|e| {
            warn!("failed to handle message: {:?}", e);
        });
        Ok(())
    }

    async fn handle_model(mut fetcher: Fetcher) -> Result<(), Infallible> {
        let _ = fetcher.fetch().await.map_err(|_| {
            warn!("failed to fetch model.");
        });
        Ok(())
    }
}

#[tonic::async_trait]
impl Communication for Communicator {
    async fn get_global_model(
        &self,
        request: Request<ClientMessage>,
    ) -> Result<Response<ServerModel>, Status> {
        info!(
            "Request received from client {:?} requesting a global model.",
            request.remote_addr().unwrap()
        );

        let fetch = self.fetcher.clone();
        let res = Communicator::handle_model(fetch).await;

        println!("{:?}", &res.unwrap());

        let global_model = Arc::clone(&self.model);
        let cupd = global_model.lock().unwrap();

        let tensor = Message::into_bytes_array(&*cupd);

        let params = mosaic::Parameters {
            tensor: tensor.to_vec(),
            data_type: "f64".to_string(),
        };

        let server_msg = mosaic::ServerModel {
            parameters: Some(params),
            id: request.into_inner().id,
        };
        Ok(Response::new(server_msg))
    }

    async fn update(
        &self,
        request: Request<ClientUpdate>,
    ) -> Result<Response<ServerMessage>, Status> {
        info!(
            "Request received from client {}: Sending an update to engine.",
            request.remote_addr().unwrap()
        );

        let req = Message {
            data: Message::from_bytes_array(&request.into_inner().parameters.unwrap().tensor),
        };

        let handle = self.handler.clone();
        let _res = Communicator::handle_message(req, handle).await;

        let server_msg = mosaic::ServerMessage {
            msg: "success".to_string(),
        };
        Ok(Response::new(server_msg))
    }
}

pub async fn start(
    api_settings: APISettings,
    message_handler: MessageHandler,
    fetcher: Fetcher,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Communication Server listening on {}", api_settings.address);

    let com = Communicator::new(message_handler, fetcher, 4);
    Server::builder()
        .add_service(CommunicationServer::new(com))
        .serve(api_settings.address)
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
/// Depict server error.
pub enum ServerError {
    #[error("invalid TLS configuration was provided")]
    InvalidTlsConfig,
}

impl From<Infallible> for ServerError {
    fn from(infallible: Infallible) -> ServerError {
        match infallible {}
    }
}
