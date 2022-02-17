use serde::{Deserialize, Serialize};

/// process statistics update event.
pub type StatsUpdate = Option<Stats>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub loss: Vec<f32>,
}
