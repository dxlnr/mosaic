//use std::fmt;
use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use std::convert::Infallible;
use std::io::{Error, ErrorKind};
use tokio::signal;
use tracing::{debug, info};

use crate::{
    engine::{Engine, ServerState},
    message::Message,
};

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
    async fn perform(&mut self) -> Result<(), Error>;

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
    pub async fn next_request(&mut self) -> Result<Message, Error> {
        info!("Waiting for the next request");
        self.shared
            .rx
            .next()
            .await
            .ok_or_else(|| Error::new(ErrorKind::Other, "Error when receiving next request."))
    }
}

/// A trait that must be implemented by a state to handle a request.
#[async_trait]
pub trait Handler {
    /// Handling a request.
    async fn handle_request(&mut self, req: Message) -> Result<(), Infallible>;
}

impl<S> StateCondition<S>
where
    Self: State + Handler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), Error> {
        let mut counter = Counter::new(self.shared.participants);
        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
                next = self.next_request() => {
                    let req = next?;
                    println!("{:?}", &req);
                    info!("received something");
                    self.process_single(req, counter.as_mut()).await;
                }
            }
            if counter.is_reached() {
                break Ok(());
            }
        }
    }
    async fn process_single(&mut self, req: Message, counter: &mut Counter) {
        let response = self.handle_request(req).await;
        if response.is_ok() {
            counter.increment_accepted();
        }
    }
}

/// A counting object keep track of handled messages from participants.
struct Counter {
    /// The number of messages that should be processed to close the collect state.
    kp: u32,
    /// The number of messages successfully processed.
    accepted: u32,
    /// The number of messages failed to processed.
    rejected: u32,
}

impl AsMut<Counter> for Counter {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl Counter {
    /// Creates a new message counter.
    fn new(kp: u32) -> Self {
        Self {
            kp,
            accepted: 0,
            rejected: 0,
        }
    }
    /// Checks if the enough messages arrived from participants.
    fn is_reached(&self) -> bool {
        self.accepted >= self.kp
    }

    /// Increments the counter for accepted requests.
    fn increment_accepted(&mut self) {
        self.accepted += 1;
        debug!(
            "{} messages accepted -- at least {} participants required.",
            self.accepted, self.kp,
        );
    }
}
