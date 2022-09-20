pub mod msflp {
    tonic::include_proto!("mosaic.protos");
}

use derive_more::Display;
use thiserror::Error;
use tonic::{transport::Channel, server};

use msflp::msflp_client::MsflpClient;

/// Error returned upon failing to build a new [`GRPCClient`].
#[derive(Debug, Display, Error)]
pub enum GRPCClientError {
    /// Initialization of gRPC client failed: {0}.
    InitError(tonic::transport::Error),
}

#[derive(Debug)]
pub struct GRPCClient {
    inner: Option<MsflpClient<Channel>>,
    server_endpoint: String,
}

impl GRPCClient
{
    // pub fn new(grpc_client: Option<C>) -> Self {
    //     Self { inner: grpc_client }
    // }

    pub fn new(server_address: String) -> Self {
        // let channel = Channel::from_static(&server_address).connect_lazy();

        Self { inner: None, server_endpoint: server_address }
    }

    fn set_server_endpoint(&mut self, url: String) {
        self.server_endpoint = url;
    }
    // pub async fn new(server_address: std::net::SocketAddr) -> Result<Self, GRPCClientError> {
    //     let client = MsflpClient::connect(server_address.to_string())
    //         .await
    //         .map_err(|e| GRPCClientError::InitError(e))?;
    //     // let client = MsflpClient::new(T);
    //     // let client = Self::init(server_address);
    //     Ok(Self { inner: client })
    // }

    // pub async fn init(server_address: std::net::SocketAddr) -> Result<MsflpClient<Channel>, GRPCClientError> {
    //     let client = MsflpClient::connect(server_address.to_string())
    //         .await
    //         .map_err(|e| GRPCClientError::InitError(e))?;

    //     Ok(client)
    // }

}
