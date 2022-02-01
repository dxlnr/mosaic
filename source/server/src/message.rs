/// Module for setting the messages that are exchanged between server and engine.
///
use crate::engine::model::DataType;

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
    pub data: Vec<Vec<u8>>,
    /// data type of the model (F64 || F32)
    pub dtype: DataType,
    // pub params: Params,
}

impl Message {
    pub fn new(key: u32, model_version: u32, data: Vec<Vec<u8>>, dtype: DataType) -> Self {
        Message {
            key,
            model_version,
            data,
            dtype,
        }
    }
}
