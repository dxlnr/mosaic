use byteorder::{BigEndian, ByteOrder};
use derive_more::Display;
use num::{
    bigint::BigInt,
    rational::Ratio,
    traits::{float::FloatCore, Zero},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    slice::{Iter, IterMut},
    str::FromStr,
    sync::Arc,
};
use thiserror::Error;

use crate::{proxy::server::mosaic::Parameters, service::error::ServiceError};

/// Global model update event.
pub type ModelUpdate = Option<ModelWrapper>;

/// Model wrapper for passing metadata alongside the actual model data.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelWrapper {
    /// actual ['Model'] content.
    pub model: Arc<Model>,
    /// associated DataType
    pub dtype: DataType,
    /// model version which returns the round_id in which the model was aggregated.
    pub model_version: u32,
}

impl ModelWrapper {
    pub fn new(model: Model, dtype: DataType, model_version: u32) -> Option<Self> {
        Some(Self {
            model: Arc::new(model),
            dtype,
            model_version,
        })
    }
    pub fn wrapper_to_params(self) -> Parameters {
        let model = Model::serialize(&self.model, &DataType::F32);

        Parameters {
            tensor: model,
            data_type: self.dtype.to_string(),
            model_version: self.model_version,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A representation of a machine learning model as vector object.
pub struct Model(pub Vec<Ratio<BigInt>>);

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
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns model with all zeros given a fixed length.
    pub fn zeros(length: &usize) -> Self {
        Model(vec![Ratio::<BigInt>::zero(); *length])
    }
    /// Creates an iterator that yields references to the weights/parameters of this model.
    pub fn iter(&self) -> Iter<Ratio<BigInt>> {
        self.0.iter()
    }
    /// Creates an iterator that yields mutable references to the weights/parameters of this model.
    pub fn iter_mut(&mut self) -> IterMut<Ratio<BigInt>> {
        self.0.iter_mut()
    }
    /// Conversion from bytes to Ratio for DataType F32
    fn from_bytes_array_f32(&mut self, bytes: Vec<u8>) {
        self.0 = bytes
            .par_chunks(4)
            .map(|x| {
                Ratio::from_float(BigEndian::read_f32(x)).unwrap_or_else(Ratio::<BigInt>::zero)
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
    /// Conversion from bytes to Ratio for DataType F64
    fn from_bytes_array_f64(&mut self, bytes: Vec<u8>) {
        self.0 = bytes
            .par_chunks(8)
            .map(|x| {
                Ratio::from_float(BigEndian::read_f64(x)).unwrap_or_else(Ratio::<BigInt>::zero)
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
    pub fn deserialize(&mut self, bytes: Vec<u8>, dtype: &DataType) {
        match dtype {
            DataType::F32 => self.from_bytes_array_f32(bytes),
            DataType::F64 => self.from_bytes_array_f64(bytes),
        }
    }
    /// Conversion from Ratio to bytes for DataType F32
    fn primitive_to_bytes_32(&self) -> Vec<u8> {
        self.0
            .par_iter()
            .map(|x| {
                ratio_to_float::<f32>(x)
                    .unwrap_or_else(Zero::zero)
                    .to_be_bytes()
                    .to_vec()
            })
            .flatten()
            .collect::<Vec<_>>()
            .to_vec()
    }
    /// Conversion from Ratio to bytes for DataType F64
    fn primitive_to_bytes_64(&self) -> Vec<u8> {
        self.0
            .par_iter()
            .map(|x| {
                ratio_to_float::<f64>(x)
                    .unwrap_or_else(Zero::zero)
                    .to_be_bytes()
                    .to_vec()
            })
            .flatten()
            .collect::<Vec<_>>()
            .to_vec()
    }
    pub fn serialize(&self, dtype: &DataType) -> Vec<u8> {
        match dtype {
            DataType::F32 => self.primitive_to_bytes_32(),
            DataType::F64 => self.primitive_to_bytes_64(),
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

impl FromStr for DataType {
    type Err = ServiceError;
    fn from_str(input: &str) -> Result<DataType, Self::Err> {
        match input {
            "F32" => Ok(DataType::F32),
            "F64" => Ok(DataType::F64),
            _ => Err(ServiceError::ParsingError(format!(
                "failed to parse from unknown data type {}",
                input
            ))),
        }
    }
}
