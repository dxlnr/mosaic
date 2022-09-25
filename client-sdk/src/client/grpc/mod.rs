pub mod traits;
pub mod msflp {
    tonic::include_proto!("mosaic.protos");
}

use derive_more::Display;
use thiserror::Error;
use tonic::transport::Channel;

use msflp::msflp_client::MsflpClient;
use crate::client::grpc::traits::Msflp;

/// Error returned upon failing to build a new [`GRPCClient`].
#[derive(Debug, Display, Error)]
pub enum GRPCClientError {
    /// Initialization of gRPC client failed: {0}.
    InitError(tonic::transport::Error),
}

#[derive(Clone, Debug)]
pub struct GRPCClient {
    inner: Option<MsflpClient<Channel>>,
    // server_endpoint: &'a str,
    server_address: String,
}

impl GRPCClient {
    pub fn new(server_address: String) -> Self {
        // let rt = tokio::runtime::Builder::new_current_thread()
        // .enable_all()
        // .build()
        // .unwrap();

        // let res = rt.block_on(async {
        //     let mut connected_client = MsflpClient::<Channel>::connect(server_address.clone())
        //         .await
        //         .map_err(|e| GRPCClientError::InitError)?;
        //     Ok::<MsflpClient<Channel>, GRPCClientError>(connected_client)
        // })?;
        // Ok(Self { inner: res, server_address: server_address.clone() })

        Self {
            inner: None,
            server_address: server_address.clone(),
        }
    }

    pub async fn try_connect(&mut self) -> Result<(), GRPCClientError> {
        self.inner = Some(
            MsflpClient::<Channel>::connect(self.server_address.clone())
                .await
                .map_err(|e| GRPCClientError::InitError(e))?,
        );
        Ok(())
    }
}


impl Msflp for GRPCClient {
    fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}