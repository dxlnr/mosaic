use std::collections::HashMap;
use tracing::{debug, info};

use crate::services::error::ServiceError;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MessageCounter {
    /// Hashmap containing a message counting object for every training round.
    pub counter: HashMap<u32, Counter>,
    /// The number of messages that should be processed to close the collect state and perform aggregation.
    ceiling: u32,
}

impl MessageCounter {
    pub fn new(ceiling: u32) -> Self {
        Self {
            counter: HashMap::new(),
            ceiling,
        }
    }
    /// Checks if the enough messages arrived from participants for closing the message collecting
    /// for a specific training round.
    pub fn reached_ceiling(&mut self, round_id: &u32) -> bool {
        match self.counter.get_mut(round_id) {
            Some(counter) => counter.accepted >= self.ceiling,
            _ => {
                debug!("index {} corresponding to round_id in message counter object is not accessible.", round_id);
                false
            }
        }
    }
    /// Include the message to the counter.
    pub fn include(&mut self, msg_result: &Result<(), ServiceError>, msg_idx: &u32, cid: &u32) {
        if !self.counter.contains_key(msg_idx) {
            self.counter.insert(*msg_idx, Counter::default());
        }

        if let Some(c) = self.counter.get_mut(msg_idx) {
            match msg_result {
                Ok(()) => c.increment_accepted(&self.ceiling, msg_idx, cid),
                _ => c.increment_rejected(msg_idx),
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
    pub fn increment_accepted(&mut self, ceiling: &u32, round_id: &u32, client_id: &u32) {
        self.accepted += 1;
        info!(
            "[{}/{}] messages accepted for training round {}. Sent by client {}",
            self.accepted, ceiling, round_id, client_id
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