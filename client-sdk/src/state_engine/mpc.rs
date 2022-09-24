use std::error::Error;

use crate::{client::grpc::{GRPCClient, GRPCClientError}, state_engine::Notifier};

/// [`Smpc`]: Message Passing Communication for State Engine.
/// 
pub struct Smpc {
    grpc_client: GRPCClient,
    notifier: Notifier,
    // store: S,
}

impl Smpc {
    pub fn new(grpc_client: GRPCClient, notifier: Notifier) -> Self {
        Self {
            grpc_client,
            notifier,
        }
    }
}

impl Smpc {
    async fn try_connect(&mut self) -> Result<(), GRPCClientError> {
        self.grpc_client.try_connect().await?;
        Ok(())
    }

    fn notify_connect(&mut self) {
        self.notifier.connect()
    }
}