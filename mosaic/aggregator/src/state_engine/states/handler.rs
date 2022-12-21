use async_trait::async_trait;
use std::collections::HashMap;
use tokio::signal;
use tracing::{debug, info, Span, warn};

use crate::{
    state_engine::{
        channel::{RequestError, ResponseSender, StateEngineRequest},
        states::{State, StateCondition, StateError},
    },
    storage::Storage,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MessageCounter {
    /// Hashmap containing a message counting object for every training round.
    pub counter: HashMap<u32, Counter>,
    /// The number of messages that should be processed to close the collect state
    /// and perform aggregation.
    k: u32,
}

impl MessageCounter {
    pub fn new(k: u32) -> Self {
        Self {
            counter: HashMap::new(),
            k,
        }
    }
    /// Checks if the enough messages arrived from participants for closing the message collecting
    /// for a specific training round.
    ///
    pub fn reached_k(&mut self, round_id: &u32) -> bool {
        match self.counter.get_mut(round_id) {
            Some(counter) => counter.accepted >= self.k,
            _ => {
                debug!("Index {} corresponding to round_id in message counter object is not accessible.", round_id);
                false
            }
        }
    }
    /// Include the message to the counter.
    pub fn increment(&mut self, req_result: &Result<(), RequestError>, round_id: &u32) {
        if !self.counter.contains_key(round_id) {
            self.counter.insert(*round_id, Counter::default());
        }

        if let Some(c) = self.counter.get_mut(round_id) {
            match req_result {
                Ok(()) => c.increment_accepted(&self.k, round_id),
                _ => c.increment_rejected(round_id),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A counting object keep track of handled messages from participants.
pub struct Counter {
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

impl Default for Counter {
    /// Creates a new default [`MessageCounter`].
    fn default() -> Self {
        Self {
            accepted: 0,
            rejected: 0,
        }
    }
}

impl Counter {
    /// Increments the counter for accepted messages.
    pub fn increment_accepted(&mut self, k: &u32, round_id: &u32) {
        self.accepted += 1;
        info!(
            "[{}/{}] messages accepted for training round {}.",
            self.accepted, k, round_id
        );
    }
    /// Increments the counter for rejected messages.
    pub fn increment_rejected(&mut self, round_id: &u32) {
        self.rejected += 1;
        debug!(
            "{} messages rejected for training round {}.",
            self.rejected, round_id
        );
    }
}
/// A trait that must be implemented by a state to handle a request.
///
#[async_trait]
pub trait StateHandler {
    /// Handling the request implementation.
    ///
    async fn handle_request(&mut self, req: StateEngineRequest) -> Result<(), RequestError>;
}

impl<S, T> StateCondition<S, T>
where
    S: Send,
    T: Storage,
    Self: State<T> + StateHandler,
{
    /// Processes requests.
    pub async fn process(&mut self) -> Result<(), StateError> {
        let mut counter = MessageCounter::new(self.shared.aggr.round_params.per_round_participants);

        if self.shared.aggr.round_params.per_round_participants == 0 {
            warn!("Participants per round parameter is 0. Consider setting `participants` in .toml config file.");
            return Ok(());
        }
        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
                next = self.next_request() => {
                    let (req, span, tx) = next?;
                    self.process_single(req, span, tx, &mut counter).await;
                }
            }
            if counter.reached_k(&self.shared.aggr.round_id) {
                break Ok(());
            }
        }
    }
    /// Processing a single request from a client.
    async fn process_single(
        &mut self,
        req: StateEngineRequest,
        span: Span,
        tx: ResponseSender,
        counter: &mut MessageCounter,
    ) {
        let _span_guard = span.enter();
        let response = self.handle_request(req).await;
        counter.increment(&response, &self.shared.aggr.round_id);
        let _ = tx.send(response);
    }
}
