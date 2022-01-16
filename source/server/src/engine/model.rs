use byteorder::{BigEndian, ByteOrder};
use derive_more::Display;
use num::{bigint::BigInt, rational::Ratio, traits::float::FloatCore};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::sync::Arc;
use thiserror::Error;

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

// impl std::ops::Deref for Model {
//     type Target = Self;
//     fn deref(&self) -> &Self::Target {
//         &self
//     }
// }

impl Model {
    /// Returns the number of weights/parameters of a model.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    fn from_bytes_array_f32(&mut self, bytes: &Vec<Vec<u8>>) {
        self.0 = bytes
            .iter()
            .map(|l| {
                l.chunks(4)
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
            .map(|l| {
                l.chunks(8)
                    .map(|x| Ratio::from_float(BigEndian::read_f64(&x)).unwrap())
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    pub fn deserialize(&mut self, bytes: Vec<Vec<u8>>, dtype: &DataType) {
        match dtype {
            DataType::F32 => self.from_bytes_array_f32(&bytes),
            DataType::F64 => self.from_bytes_array_f64(&bytes),
        }
    }

    fn into_bytes_array_32(&self) -> Vec<Vec<u8>> {
        let res = self
            .0
            .iter()
            .map(|l| {
                l.iter()
                    .map(|x| {
                        ratio_to_float::<f32>(&x)
                            .ok_or(CastingError {
                                weight: x.clone(),
                                target: DataType::F32,
                            })
                            .unwrap()
                            .to_be_bytes()
                            .to_vec()
                    })
                    .flatten()
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        res
    }
    fn into_bytes_array_64(&self) -> Vec<Vec<u8>> {
        let res = self
            .0
            .iter()
            .map(|l| {
                l.iter()
                    .map(|x| {
                        ratio_to_float::<f64>(&x)
                            .ok_or(CastingError {
                                weight: x.clone(),
                                target: DataType::F64,
                            })
                            .unwrap()
                            .to_be_bytes()
                            .to_vec()
                    })
                    .flatten()
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        res
    }

    pub fn serialize(&self, dtype: &DataType) -> Vec<Vec<u8>> {
        match dtype {
            DataType::F32 => self.into_bytes_array_32(),
            DataType::F64 => self.into_bytes_array_64(),
        }
    }
}

pub(crate) fn ratio_to_float<F: FloatCore>(ratio: &Ratio<BigInt>) -> Option<F> {
    let min_value = Ratio::from_float(F::min_value()).unwrap();
    let max_value = Ratio::from_float(F::max_value()).unwrap();
    if ratio < &min_value || ratio > &max_value {
        return None;
    }

    let mut numer = ratio.numer().clone();
    let mut denom = ratio.denom().clone();
    // safe loop: terminates after at most bit-length of ratio iterations
    loop {
        if let (Some(n), Some(d)) = (F::from(numer.clone()), F::from(denom.clone())) {
            if n == F::zero() || d == F::zero() {
                break Some(F::zero());
            } else {
                let float = n / d;
                if float.is_finite() {
                    break Some(float);
                }
            }
        } else {
            numer >>= 1_usize;
            denom >>= 1_usize;
        }
    }
}

#[derive(Error, Debug)]
#[error("Could not convert weight {weight} to floating point number {target}")]
/// Errors related to model converting Ratio to floats.
pub struct CastingError {
    weight: Ratio<BigInt>,
    target: DataType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
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
