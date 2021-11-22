//! Sets up a gRPC server.
use std::convert::Infallible;
use thiserror::Error;
use tonic::{transport::Server, Request, Response, Status};

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
    async fn test(
        &self,
        request: Request<ClientMessage>,
    ) -> Result<Response<ServerMessage>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let params = mosaic::Parameters {
            tensors: vec![1, 3, 4, 5],
            tensor_type: "TestTensor".to_string(),
        };

        let back = request.into_inner();
        println!("{:?}", back.parameters.unwrap());

        let server_msg = mosaic::ServerMessage {
            parameters: Some(params),
        };

        Ok(Response::new(server_msg))
    }
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let com = Communicator::default();

    println!("Communication Server listening on {}", addr);

    Server::builder()
        .add_service(CommunicationServer::new(com))
        .serve(addr)
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
