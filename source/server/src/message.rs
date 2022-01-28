/// Module for setting the messages that are exchanged between server and engine.
/// 
use crate::{engine::model::DataType, server::mosaic::ClientUpdate};

/// message buffer object
pub struct MessageBuffer<T> {
    inner: T,
}

/// main message object
#[derive(Debug, Clone)]
pub struct Message {
    /// client key to check if message is authorized.
    pub ckey: u32,
    /// version of the model (training round) that was sent. 
    pub model_version: u32,
    /// actual model data.
    pub data: Vec<Vec<u8>>,
    /// data type of the model (F64 || F32)
    pub dtype: DataType,
    // pub params: Params,
}

impl Message {
    // pub fn new(req: ClientUpdate) -> Self {
    //     let params = req.parameters.unwrap();
    //     Message { 
    //         ckey: req.id, 
    //         model_version: params.model_version,
    //         data: params.tensor,
    //         dtype: DataType::from_str(&params.data_type).unwrap(),
    //     }
    // }
    pub fn new(ckey: u32, model_version: u32, data: Vec<Vec<u8>>, dtype: DataType) -> Self {
        Message { ckey, model_version, data, dtype }
    }
}
