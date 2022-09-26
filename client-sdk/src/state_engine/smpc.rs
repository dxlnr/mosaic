use crate::{
    client::grpc::{GRPCClient, GRPCClientError},
    state_engine::EventSender,
};

#[derive(Debug)]
/// [`Smpc`]: Message Passing Communication for State Engine.
///
pub struct Smpc {
    grpc_client: GRPCClient,
    event_sender: EventSender,
    // store: S,
}

impl Smpc {
    pub fn new(grpc_client: GRPCClient, event_sender: EventSender) -> Self {
        Self {
            grpc_client,
            event_sender,
        }
    }
}

impl Smpc {
    async fn try_connect(&mut self) -> Result<(), GRPCClientError> {
        self.grpc_client.try_connect().await?;
        Ok(())
    }
    pub fn notify_idle(&mut self) {
        self.event_sender.idle()
    }
    pub fn notify_new_task(&mut self) {
        self.event_sender.new_task()
    }
    pub fn notify_update(&mut self) {
        self.event_sender.update()
    }
}
