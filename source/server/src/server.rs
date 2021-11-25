//! Sets up a gRPC server.
use std::convert::Infallible;

use thiserror::Error;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    engine::message::{DataType, IntoPrimitives, Message},
    settings::APISettings,
};

pub mod mosaic {
    tonic::include_proto!("mosaic");
}

use mosaic::communication_server::{Communication, CommunicationServer};
#[allow(unused_imports)]
use mosaic::{ClientMessage, Parameters, ServerMessage};

#[derive(Default)]
pub struct Communicator {}

#[tonic::async_trait]
impl Communication for Communicator {
    async fn broadcast(
        &self,
        request: Request<ClientMessage>,
    ) -> Result<Response<ServerMessage>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let params = mosaic::Parameters {
            tensors: vec![1, 3, 4, 5],
            tensor_type: "TestTensor".to_string(),
        };

        let msgs = Message {
            bytes: request.into_inner().parameters.unwrap().tensors,
            dtype: DataType::F64,
        };

        let testtest: Vec<f64> = Message::into_primitives(msgs);
        println!("{:?}", testtest);

        let server_msg = mosaic::ServerMessage {
            parameters: Some(params),
        };

        Ok(Response::new(server_msg))
    }
}

pub async fn start(api_settings: APISettings) -> Result<(), Box<dyn std::error::Error>> {
    let com = Communicator::default();

    println!("Communication Server listening on {}", api_settings.address);

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
