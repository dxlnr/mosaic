//! Implements the `Aggregator` as an independent GRPC server application.
//!
// pub mod msflp {
//     tonic::include_proto!("mosaic.protos");
// }

use futures::Stream;
use std::{convert::Infallible, error::Error, io::ErrorKind, pin::Pin};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tracing::{info, warn};

use crate::{configs::APISettings, services::messages::MessageHandler};

// use msflp::{
//     msflp_server::{Msflp, MsflpServer},
//     server_message,
//     server_message::ServerStatus,
//     ClientMessage, ServerMessage,
// };

use mosaic_core::protos::mosaic::protos::{
    msflp_server::{Msflp, MsflpServer},
    server_message,
    server_message::ServerStatus,
    ClientMessage, ServerMessage,
};

/// Result type of handling the bidirectional stream
/// between client and server.
///
type HandleResult<T> = Result<Response<T>, Status>;
/// Response type of handling the bidirectional stream
/// between client and server.
///
type ResponseStream = Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send>>;

/// Handling IO error while streaming.
///
fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        // // h2::Error do not expose std::io::Error with `source()`
        // // https://github.com/hyperium/h2/pull/462
        // if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
        //     if let Some(io_err) = h2_err.get_io() {
        //         return Some(io_err);
        //     }
        // }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}

/// [`GRPCServer`] Implements the MSFLP server trait.
///
/// The [`GRPCServer`] has two main tasks:
///     - Implementing the service trait generated from our service definition.
///     - Running a gRPC server to listen for requests from clients.
///
#[derive(Debug, Clone)]
pub struct GRPCServer {
    /// Shared handle for passing messages from participant to engine.
    handler: MessageHandler,
}

impl GRPCServer {
    /// Constructs a new [`GRPCServer`].
    fn new(handler: MessageHandler) -> Self {
        GRPCServer { handler }
    }
}

#[tonic::async_trait]
impl Msflp for GRPCServer {
    type HandleStream = ResponseStream;

    /// Mosaic Secure Federated Learning Protocol.
    ///
    /// `handle` implements the logic by which the bidirectional
    /// stream between client and server is defined.
    ///
    /// Client: ClientDisconnect -- Server: ReconnectClient.
    ///
    async fn handle(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> HandleResult<Self::HandleStream> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => tx
                        .send(Ok(ServerMessage {
                            msg: Some(server_message::Msg::Status(ServerStatus {
                                status: "OK".to_string(),
                            })),
                        }))
                        .await
                        .expect("working rx"),
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                warn!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        }
                    }
                }
            }
            info!("Connection between client and aggregation server is finished.");
        });

        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(out_stream) as Self::HandleStream))
    }
}

pub async fn start<F>(
    api_settings: &APISettings,
    message_handler: MessageHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Aggregation Server listening on {}",
        api_settings.server_address
    );

    let aggr_server = GRPCServer::new(message_handler);
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
