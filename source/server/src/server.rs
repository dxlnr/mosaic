//! Sets up a gRPC server.
use futures::future::{ready, Ready};
use std::mem::drop;
use std::sync::Mutex;

use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;

use hyper::{service::make_service_fn, Body};
use std::{
    task::{Context, Poll},
    time::Duration,
};
use tonic::{body::BoxBody, transport::Server, Request, Response, Status};
use tower::{Layer, Service};

use crate::{
    engine::{
        message::{DataType, IntoPrimitives, Message, MessageHandler},
        model::Model,
    },
    services::fetcher::Fetchers,
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

#[derive(Default, Debug)]
pub struct Communicator {
    features: Arc<Mutex<Vec<f64>>>,
    counter: Arc<Mutex<u32>>,
}

impl Communicator {
    fn new() -> Self {
        Communicator {
            features: Arc::new(Mutex::new(vec![0.0, 0.0, 0.0, 0.0])),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn read(mut handler: MessageHandler, data: Vec<f64>) {
        handler.add_msg(data).await;
    }
}

#[tonic::async_trait]
impl Communication for Communicator {
    async fn broadcast(
        &self,
        request: Request<ClientDefault>,
    ) -> Result<Response<ServerDefault>, Status> {
        let global_counter = Arc::clone(&self.counter);
        let global_model = Arc::clone(&self.features);

        let mut num = global_counter.lock().unwrap();
        if *num == 10 {
            let mut cupd = global_model.lock().unwrap();
            *cupd = cupd.iter().map(|&v| v / *num as f64).collect();

            println!("global model after aggregation: {:?}", cupd);
        }
        //println!("Got a request from {:?}", request.remote_addr());

        let params = mosaic::Parameters {
            tensors: vec![1, 3, 4, 5],
            data_type: "f64".to_string(),
        };

        let msgs = Message {
            bytes: request.into_inner().parameters.unwrap().tensors,
            dtype: DataType::F64,
        };

        *num += 1;

        let single_model: Vec<f64> = Message::into_primitives(msgs);

        let mut cupd = global_model.lock().unwrap();
        *cupd = cupd
            .iter()
            .zip(single_model.iter())
            .map(|(&b, &v)| b + v)
            .collect();

        let server_msg = mosaic::ServerDefault {
            parameters: Some(params),
        };

        Ok(Response::new(server_msg))
    }

    async fn get_global_model(
        &self,
        request: Request<ClientMessage>,
    ) -> Result<Response<ServerModel>, Status> {
        todo!()
    }

    async fn send_update(
        &self,
        request: Request<ClientUpdate>,
    ) -> Result<Response<ServerMessage>, Status> {
        todo!()
    }
}

pub async fn start(api_settings: APISettings) -> Result<(), Box<dyn std::error::Error>> {
    let com = Communicator::new();

    //let test_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(Engine) });

    println!("Communication Server listening on {}", api_settings.address);

    // The stack of middleware that our service will be wrapped in
    // let layer = tower::ServiceBuilder::new()
    //     // Apply middleware from tower
    //     // Apply our own middleware
    //     .layer(EngineLayer::default())
    //     // Interceptors can be also be applied as middleware
    //     .layer(tonic::service::interceptor(intercept))
    //     .into_inner();

    println!("Communicator object: {:?}", &com);

    Server::builder()
        //.add_service(test_service)
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
