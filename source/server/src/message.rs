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

impl Message {
    // pub fn into_bytes_array(primitives: &Vec<f64>) -> Vec<Vec<u8>> {
    //     primitives
    //         .iter()
    //         .map(|r| r.to_be_bytes().to_vec())
    //         .collect::<Vec<_>>()
    //         .to_vec()
    // }
    //
    // pub fn from_bytes_array(bytes: &Vec<Vec<u8>>) -> Vec<f64> {
    //     bytes
    //         .iter()
    //         .map(|r| BigEndian::read_f64(&r))
    //         .collect::<Vec<_>>()
    //         .to_vec()
    // }
    //
    fn from_bytes_array_f32(&self) -> Vec<Vec<f32>> {
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

    fn from_bytes_array_f64(&self) -> Vec<Vec<f64>> {
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

// impl<T> Message<T> {
//     pub fn from_bytes(&self) -> Vec<Vec<T>> {
//         match self.dtype {
//             DataType::F32 => self.from_bytes_array_f32(),
//             DataType::F64 => self.from_bytes_array_f64(),
//         }
//     }
// }
//
// pub fn test(bytes: &Vec<Vec<u8>>, dtype: &DataType) {
//     match dtype {
//         DataType::F32 => from_bytes_array!(bytes, f32),
//         DataType::F64 => from_bytes_array!(bytes, f64),
//     };
// }

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

// #[macro_export]
// macro_rules! from_bytes {
//     ($msg:expr, $data_type:ty) => {
//         // impl $crate::FromBytes for $data_type {
//         //     from_bytes_array()
//         // }
//         $msg.from_bytes_array().collect::<Vec<Vec<$data_type>>>()
//     };
// }
