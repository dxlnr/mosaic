use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use tokio::signal;
use tracing::{info, warn};

use crate::{
    engine::{
        channel::ResponseSender, states::error::StateError, utils::MessageCounter, Cache, Engine,
        ServerState,
    },
    proxy::message::Message,
    service::error::ServiceError,
};

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
/// The name of the current state.
pub enum StateName {
    #[display(fmt = "Idle")]
    Idle,
    #[display(fmt = "Collect")]
    Collect,
    #[display(fmt = "Aggregate")]
    Aggregate,
    #[display(fmt = "Failure")]
    Failure,
    #[display(fmt = "Shutdown")]
    Shutdown,
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait State {
    /// The name of the current state.
    const NAME: StateName;

    /// Performs the attached tasks of the state.
    async fn perform(&mut self) -> Result<(), StateError>;

    /// Moves from the current to the next state.
    async fn next(self) -> Option<Engine>;
}

#[allow(dead_code)]
pub struct StateCondition<S> {
    pub(in crate::engine) private: S,
    /// Some shared server state.
    pub shared: ServerState,
    /// caching state.
    pub cache: Cache,
}

impl<S> StateCondition<S>
where
    Self: State,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<Engine> {
        info!("Engine runs in state: {:?}", &Self::NAME);
        async move {
            if let Err(err) = self.perform().await {
                warn!("{:?}", err);
            }
            self.next().await
        }
        .await
    }
    /// Receives the next ['Request'] from gRPC server.
    pub async fn next_request(&mut self) -> Result<(Message, ResponseSender), StateError> {
        info!("Waiting for the next request.");
        self.shared
            .rx
            .next()
            .await
            .ok_or(StateError::RequestChannel(
                "Error when receiving next request.",
            ))
    }
}

/// A trait that must be implemented by a state to handle a request.
#[async_trait]
pub trait Handler {
    /// Handling a request.
    async fn handle_request(&mut self, req: Message) -> Result<(), ServiceError>;
}

impl<S> StateCondition<S>
where
    Self: State + Handler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), StateError> {
        let mut counter = MessageCounter::new(self.shared.round_params.per_round_participants);
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
            if counter.reached_ceiling(&self.cache.round_id) {
                break Ok(());
            }
        }
    }
    /// Processing a single request from client.
    async fn process_single(
        &mut self,
        req: Message,
        tx: ResponseSender,
        counter: &mut MessageCounter,
    ) {
        let model_idx = req.model_version;
        let cid = req.cid;
        let response = self.handle_request(req).await;
        counter.include(&response, &model_idx, &cid);
        let _ = tx.send(response);
    }
}
