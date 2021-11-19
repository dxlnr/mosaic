use tonic::{transport::Server, Request, Response, Status};

use mosaic::communication_server::{Communication, CommunicationServer};
use mosaic::{ClientMessage, Parameters, ServerMessage};

pub mod mosaic {
    tonic::include_proto!("mosaic");
}

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

        let server_msg = mosaic::ServerMessage {
            parameters: Some(params),
        };

        Ok(Response::new(server_msg))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let addr = "[::1]:50051".parse().unwrap();
    let addr = "127.0.0.1:8080".parse().unwrap();
    let com = Communicator::default();

    println!("Communication Server listening on {}", addr);

    Server::builder()
        .add_service(CommunicationServer::new(com))
        .serve(addr)
        .await?;

    Ok(())
}
