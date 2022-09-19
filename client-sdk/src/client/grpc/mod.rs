pub mod msflp {
    tonic::include_proto!("mosaic.protos");
}

use derive_more::Display;
use thiserror::Error;
use tonic::transport::Channel;

use msflp::msflp_client::MsflpClient;

/// Error returned upon failing to build a new [`GRPCClient`].
#[derive(Debug, Display, Error)]
pub enum GRPCClientError {
    /// Initialization of gRPC client failed: {0}.
    InitError(tonic::transport::Error),
}

#[derive(Debug)]
pub struct GRPCClient {
    inner: MsflpClient<Channel>,
}

impl GRPCClient {
    pub async fn new(server_address: std::net::SocketAddr) -> Result<Self, GRPCClientError> {
        let client = MsflpClient::connect(server_address.to_string())
            .await
            .map_err(|e| GRPCClientError::InitError(e))?;

        Ok(Self { inner: client })
    }
}
