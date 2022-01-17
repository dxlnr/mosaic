/// Module for setting the messages that are exchanged between server and engine.
use crate::engine::model::DataType;

/// main message object
#[derive(Debug, Clone)]
pub struct Message {
    pub data: Vec<Vec<u8>>,
    pub dtype: DataType,
}
