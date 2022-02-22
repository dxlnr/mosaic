use serde::{Deserialize, Serialize};

/// process statistics update event.
pub type StatsUpdate = Option<Stats>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub msgs: Vec<Single>,
}

/// single meta data message from specific client.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Single {
    pub client_id: u32,
    pub round_id: u32,
    pub loss: f32,
}

impl Single {
    pub fn new(client_id: u32, round_id: u32, loss: f32) -> Self {
        Self {
            client_id,
            round_id,
            loss,
        }
    }
}
