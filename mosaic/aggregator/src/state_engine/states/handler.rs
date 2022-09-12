use async_trait::async_trait;

use crate::{
    state_engine::{
        channel::ResponseSender, states::StateError
        ServerState,
    },
    service::error::ServiceError,
};

/// A trait that must be implemented by a state to handle a request.
#[async_trait]
pub trait StateHandler {
    /// Handling a request.
    async fn handle_request(&mut self, req: Message) -> Result<(), ServiceError>;
}

impl<S> StateCondition<S>
where
    Self: State + StateHandler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), StateError> {
        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
                next = self.next_request() => {
                    let (req, tx) = next?;
                    self.process_single(req, tx).await;
                }
            }
        }
    }
    /// Processing a single request from a client.
    async fn process_single(
        &mut self,
        req: Message,
        tx: ResponseSender,
    ) {
        todo!()
    }
}
