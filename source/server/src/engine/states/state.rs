use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use std::io::{Error, ErrorKind};
use tokio::signal;
use tracing::{debug, info, warn};

use crate::{
    engine::{channel::ResponseSender, Engine, ServerState},
    message::Message,
};

/// The name of the current state.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum StateName {
    #[display(fmt = "Idle")]
    Idle,
    #[display(fmt = "Collect")]
    Collect,
    #[display(fmt = "Aggregate")]
    Aggregate,
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
#[allow(dead_code)]
pub struct StateCondition<S> {
    pub(in crate::engine) private: S,
    /// Some shared server state.
    pub shared: ServerState,
}

impl<S> StateCondition<S>
where
    Self: State,
{
    /// Runs the current State to completion.
    pub async fn run_state(mut self) -> Option<Engine> {
        info!("Engine runs in state: {:?}", &Self::NAME);
        async move {
            if let Err(_err) = self.perform().await {
                warn!("{:?}", "state error");
            }
            self.next().await
        }
        .await
    }
    /// Receives the next [`Request`] from gRPC server.
    pub async fn next_request(&mut self) -> Result<(Message, ResponseSender), Error> {
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
    async fn handle_request(&mut self, req: Message) -> Result<(), Error>;
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
                    let (req, tx) = next?;
                    self.process_single(req, tx, counter.as_mut()).await;
                }
            }
            if counter.is_reached() {
                break Ok(());
            }
        }
    }
    async fn process_single(&mut self, req: Message, tx: ResponseSender, counter: &mut Counter) {
        let response = self.handle_request(req).await;
        if response.is_ok() {
            counter.increment_accepted();
        } else {
            counter.increment_rejected();
        }
        let _ = tx.send(response);
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
    /// Increments the counter for accepted messages.
    fn increment_accepted(&mut self) {
        self.accepted += 1;
        info!(
            "{} messages accepted -- at least {} participants required.",
            self.accepted, self.kp,
        );
    }
    /// Increments the counter for rejected messages.
    fn increment_rejected(&mut self) {
        self.rejected += 1;
        debug!("{} messages rejected.", self.rejected);
    }
}
