//use std::fmt;
use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use std::convert::Infallible;
use tokio::signal;
use tracing::info;

use crate::engine::{channel::EngineRequest, Engine, ServerState};

/// The name of the current state.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum StateName {
    #[display(fmt = "Idle")]
    Idle,
    #[display(fmt = "Collect")]
    Collect,
    #[display(fmt = "Shutdown")]
    Shutdown,
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait State {
    /// The name of the current state.
    const NAME: StateName;

    /// Performs the tasks of this state.
    async fn perform(&mut self) -> Result<(), Infallible>;

    /// Moves from the current to the next state.
    async fn next(self) -> Option<Engine>;
}

pub struct StateCondition<S> {
    pub(in crate::engine) private: S,
    /// Some shared state.
    pub shared: ServerState,
}

impl<S> StateCondition<S>
where
    Self: State,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<Engine> {
        let state = Self::NAME;

        info!("Engine runs in state: {:?}", &state);

        async move {
            if let Err(_err) = self.perform().await {
                println!("{:?}", "state error");
            }
            self.next().await
        }
        .await
    }
    /// Receives the next [`Request`] from gRPC server.
    pub async fn next_request(&mut self) -> EngineRequest {
        info!("waiting for the next request");
        self.shared.rx.next().await.unwrap()
    }
}

/// A trait that must be implemented by a state to handle a request.
#[async_trait]
pub trait Handler {
    /// Handling a request.
    async fn handle_request(&mut self, req: EngineRequest) -> Result<(), Infallible>;
}

impl<S> StateCondition<S>
where
    Self: State + Handler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), Infallible> {
        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
                next = self.next_request() => {
                    let req = next;
                    self.process_single(req).await;
                }
            }
        }
    }
    async fn process_single(&mut self, req: EngineRequest) {
        let _response = self.handle_request(req).await;
    }
}
