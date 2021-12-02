use byteorder::{BigEndian, ByteOrder};
//use std::convert::Infallible;
use std::slice::Chunks;
//use thiserror::Error;

#[derive(Default, Debug, Clone)]
pub struct MessageHandler {
    pub message_bundle: Vec<Vec<f64>>,
}

impl MessageHandler {
    pub fn new() -> Self {
        MessageHandler {
            message_bundle: Vec::new(),
        }
    }
    // pub async fn handle_message(&mut self, data: Vec<u8>) -> Result<(), _> {
    //     let message = data
    // }
    pub async fn add_msg(&mut self, data: Vec<f64>) {
        self.message_bundle.push(data);
    }
}

pub struct Message {
    pub bytes: Vec<u8>,
    pub dtype: DataType,
}

impl Message {
    /// Returns the length of a message.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    /// Creates an iterator that iterates over fixed chunks depending on the datatype.
    pub fn iter(&self) -> Chunks<u8> {
        match self.dtype {
            DataType::F32 => self.bytes.chunks(4),
            DataType::F64 => self.bytes.chunks(8),
        }
    }
}

#[derive(Debug)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F32,
    F64,
}

// /// Errors related to model conversion into primitives.
// pub struct CastingError {
//     sequence: Vec<u8>,
//     target: DataType,
// }

pub trait IntoPrimitives<P: 'static>: Sized {
    /// Byte sequences are converted into primitive values.
    fn into_primitives(self) -> P;

    /// Byte sequences are converted into primitive values.
    fn to_primitives(&self) -> P;
}

impl IntoPrimitives<Vec<f32>> for Message {
    fn into_primitives(self) -> Vec<f32> {
        self.iter().map(|r| BigEndian::read_f32(&r)).collect()
    }
    fn to_primitives(&self) -> Vec<f32> {
        self.iter().map(|r| BigEndian::read_f32(&r)).collect()
    }
}

impl IntoPrimitives<Vec<f64>> for Message {
    fn into_primitives(self) -> Vec<f64> {
        self.iter().map(|r| BigEndian::read_f64(&r)).collect()
    }
    fn to_primitives(&self) -> Vec<f64> {
        self.iter().map(|r| BigEndian::read_f64(&r)).collect()
    }
}
