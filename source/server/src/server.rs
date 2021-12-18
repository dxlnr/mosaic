//! Sets up a gRPC server.
use std::sync::Mutex;

use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;

use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

use crate::{message::Message, service::messages::MessageHandler, settings::APISettings};

pub mod mosaic {
    tonic::include_proto!("mosaic");
}

use mosaic::communication_server::{Communication, CommunicationServer};
#[allow(unused_imports)]
use mosaic::{
    ClientDefault, ClientMessage, ClientUpdate, Parameters, ServerDefault, ServerMessage,
    ServerModel,
};

#[derive(Debug)]
pub struct Communicator {
    model: Arc<Mutex<Vec<f64>>>,
    // features: Arc<Mutex<Vec<Vec<Vec<u8>>>>>,
    /// Shared handle for passing messages from participant to engine.
    handler: MessageHandler,
}

impl Communicator {
    fn new(handler: MessageHandler, model_length: usize) -> Self {
        Communicator {
            model: Arc::new(Mutex::new(vec![0.0; model_length])),
            // features: Arc::new(Mutex::new(Vec::new())),
            handler,
        }
    }

    async fn handle_message(msg: Message, mut handler: MessageHandler) -> Result<(), Infallible> {
        let _ = handler.process(msg).await.map_err(|e| {
            info!("failed to handle message: {:?}", e);
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
        println!(
            "Request received from client {:?} requesting a global model.",
            request.remote_addr().unwrap()
        );

        let global_model = Arc::clone(&self.model);
        let cupd = global_model.lock().unwrap();

        let tensor = into_bytes_array(&*cupd);

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
        // let feature_list = Arc::clone(&self.features);
        // let mut flist = feature_list.lock().unwrap();
        // flist.push(request.into_inner().parameters.unwrap().tensor);
        //
        // println!("feature list {:?}", &flist);
        // self
        // println!(
        //     "features {:?}",
        //     &request.into_inner().parameters.unwrap().tensor
        // );

        let handle_request = self.handler.clone();
        // let mut handles = handle_request.lock().unwrap();

        let req = Message {
            data: request.into_inner().parameters.unwrap().tensor,
        };

        Communicator::handle_message(req, handle_request);
        // self.handler
        //     .engine_service
        //     .handle
        //     .0
        //     .unbounded_send(engine_req);

        // handles.process(engine_req).await;

        // let feature_list = Arc::clone(&self.features);
        // let mut flist = feature_list.lock().unwrap();
        // flist.push(request.into_inner().parameters.unwrap().tensor);

        let server_msg = mosaic::ServerMessage {
            msg: "success".to_string(),
        };
        Ok(Response::new(server_msg))
    }
}

pub async fn start(
    api_settings: APISettings,
    message_handler: MessageHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    let com = Communicator::new(message_handler, 4);

    info!("Communication Server listening on {}", api_settings.address);

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

fn into_bytes_array(primitives: &Vec<f64>) -> Vec<Vec<u8>> {
    primitives
        .iter()
        .map(|r| r.to_be_bytes().to_vec())
        .collect::<Vec<_>>()
        .to_vec()
}
