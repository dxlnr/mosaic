use serde::{Deserialize, Serialize};

/// process statistics update event.
pub type StatsUpdate = Option<Stats>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// [`Stats`] holding a vector of [`Single`] messages from individual clients which contain meta data about the training.
pub struct Stats {
    pub msgs: Vec<Single>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// [`Single`] meta data message from specific client.
pub struct Single {
    pub client_id: u32,
    pub round_id: u32,
    pub loss: f32,
    pub samples: u32,
}

impl Single {
    pub fn new(client_id: u32, round_id: u32, loss: f32, samples: u32) -> Self {
        Self {
            client_id,
            round_id,
            loss,
            samples,
        }
    }
}
