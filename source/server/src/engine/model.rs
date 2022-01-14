use byteorder::{BigEndian, ByteOrder};
use num::{bigint::BigInt, rational::Ratio};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::sync::Arc;

/// Global model update event.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelUpdate {
    Invalidate,
    New(Arc<Model>),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A representation of a machine learning model as vector object.
pub struct Model(pub Vec<Vec<Ratio<BigInt>>>);

impl std::convert::AsRef<Model> for Model {
    fn as_ref(&self) -> &Model {
        self
    }
}

impl Model {
    /// Returns the number of weights/parameters of a model.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    fn from_bytes_array_f32(&mut self, bytes: &Vec<Vec<u8>>) {
        self.0 = bytes
            .iter()
            .map(|r| {
                r.chunks(4)
                    .map(|x| Ratio::from_float(BigEndian::read_f32(&x)).unwrap())
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
    fn from_bytes_array_f64(&mut self, bytes: &Vec<Vec<u8>>) {
        self.0 = bytes
            .iter()
            .map(|r| {
                r.chunks(8)
                    .map(|x| Ratio::from_float(BigEndian::read_f64(&x)).unwrap())
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    pub fn conversion(&mut self, bytes: Vec<Vec<u8>>, dtype: &DataType) {
        match dtype {
            DataType::F32 => self.from_bytes_array_f32(&bytes),
            DataType::F64 => self.from_bytes_array_f64(&bytes),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F32,
    F64,
}

impl TryFrom<u8> for DataType {
    type Error = ErrorKind;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(DataType::F32),
            1 => Ok(DataType::F64),
            _ => Err(ErrorKind::InvalidData),
        }
    }
}

// #[macro_export]
// macro_rules! from_bytes {
//     ($model:expr, $msg:expr, $data_type:ty) => {
//         impl $crate::FromBytes for $data_type {
//             $model.from_bytes_array($msg)
//         }
//     };
// }
