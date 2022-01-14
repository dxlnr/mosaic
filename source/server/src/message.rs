/// Module for setting the messages that are exchanged between server and engine.
use byteorder::{BigEndian, ByteOrder};

#[derive(Debug, Clone)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F64,
    F32,
}

/// main message object
#[derive(Debug, Clone)]
pub struct Message {
    //pub data: Vec<Vec<u8>>,
    pub data: Vec<Vec<u8>>,
    pub dtype: DataType,
}

pub trait IntoBytes<T: 'static>: Sized {
    /// Conversion from floating point values to Byte sequences.
    fn into_bytes_array(&self) -> Vec<Vec<T>>;
}

pub trait FromBytes<T: 'static>: Sized {
    /// Conversion from Byte sequences into floating point values.
    fn from_bytes_array(&self) -> Vec<Vec<T>>;
}

impl FromBytes<f32> for Message {
    fn from_bytes_array(&self) -> Vec<Vec<f32>> {
        self.data
            .iter()
            .map(|r| {
                r.chunks(4)
                    .map(|x| BigEndian::read_f32(&x))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
}

impl FromBytes<f64> for Message {
    fn from_bytes_array(&self) -> Vec<Vec<f64>> {
        self.data
            .iter()
            .map(|r| {
                r.chunks(8)
                    .map(|x| BigEndian::read_f64(&x))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
}
