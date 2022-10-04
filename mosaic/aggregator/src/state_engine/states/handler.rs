use async_trait::async_trait;
use tokio::signal;

use crate::aggr::counter::MessageCounter;

use crate::state_engine::{
    channel::{ResponseSender, RequestError, StateEngineRequest},
    states::{State, StateCondition, StateError},
};

/// A trait that must be implemented by a state to handle a request.
/// 
#[async_trait]
pub trait StateHandler {
    /// Handling the request implementation.
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError>;
}

impl<S> StateCondition<S>
where
    Self: State + StateHandler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), StateError> {
        let mut counter = MessageCounter::new(self.shared.aggr.params.k);
        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
                next = self.next_request() => {
                    let (req, tx) = next?;
                    self.process_single(req, tx, &mut counter).await;
                }
            }
            if counter.reached_ceiling(&0) {
                break Ok(());
            }
        }
    }
    /// Processing a single request from a client.
    async fn process_single(&mut self, req: StateEngineRequest, tx: ResponseSender, counter: &mut MessageCounter) {
        let response = self.handle_request(req).await;
        let _ = tx.send(response);
    }
}
