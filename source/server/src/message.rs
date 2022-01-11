/// Module for setting the messages that are exchanged between server and engine.
use byteorder::{BigEndian, ByteOrder};

/// main message object
#[derive(Debug, Clone)]
pub struct Message {
    //pub data: Vec<Vec<u8>>,
    pub data: Vec<f64>,
}

impl Message {
    pub fn into_bytes_array(primitives: &Vec<f64>) -> Vec<Vec<u8>> {
        primitives
            .iter()
            .map(|r| r.to_be_bytes().to_vec())
            .collect::<Vec<_>>()
            .to_vec()
    }

    pub fn from_bytes_array(bytes: &Vec<Vec<u8>>) -> Vec<f64> {
        bytes
            .iter()
            .map(|r| BigEndian::read_f64(&r))
            .collect::<Vec<_>>()
            .to_vec()
    }

    pub fn from_bytes_array_test(bytes: &Vec<Vec<u8>>) -> Vec<Vec<f32>> {
        bytes
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

// pub enum DataType {
//     F64,
//     F32,
// }
