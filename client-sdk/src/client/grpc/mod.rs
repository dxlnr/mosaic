pub mod msflp {
    tonic::include_proto!("mosaic.protos");
}

use derive_more::Display;
use thiserror::Error;
use tonic::transport::Channel;

use msflp::msflp_client::MsflpClient;

/// Error returned upon failing to build a new [`GRPCClient`].
#[derive(Clone, Debug, Display, Error)]
pub enum GRPCClientError {
    /// Initialization of gRPC client failed: {0}.
    InitError,
}

#[derive(Clone, Debug)]
pub struct GRPCClient {
    inner: Option<MsflpClient<Channel>>,
    // server_endpoint: &'a str,
    server_address: String,
}

impl GRPCClient
{
    // pub fn new(grpc_client: Option<C>) -> Self {
    //     Self { inner: grpc_client }
    // }

    pub fn new(server_address: String) -> Self {
        Self { inner: None, server_address: server_address.clone() }
    }

    pub async fn try_connect(&mut self) -> Result<(), GRPCClientError> {
        self.inner = Some(MsflpClient::<Channel>::connect(self.server_address.clone()).await.map_err(|e| GRPCClientError::InitError)?);
        Ok(())
    }

}
