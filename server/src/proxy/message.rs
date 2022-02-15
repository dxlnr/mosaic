/// Module for setting the messages that are exchanged between server and engine.
///
use crate::core::model::DataType;

// /// message buffer object
// pub struct MessageBuffer<T> {
//     inner: T,
// }

/// main message object
#[derive(Debug, Clone)]
pub struct Message {
    /// client key to check if message is authorized.
    pub key: u32,
    /// version of the model (training round) that was sent.
    pub model_version: u32,
    /// actual model data.
    pub data: Vec<u8>,
    /// data type of the model (F64 || F32)
    pub dtype: DataType,
    // /// training parameters.
    // pub params: Params,
    /// Weighting factor that determines the proportion of the local to the global model.
    pub stake: u32,
}

impl Message {
    pub fn new(key: u32, model_version: u32, data: Vec<u8>, dtype: DataType, stake: u32) -> Self {
        Message {
            key,
            model_version,
            data,
            dtype,
            stake,
        }
    }
}
