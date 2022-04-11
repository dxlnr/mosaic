//!
use byteorder::{BigEndian, ByteOrder};
use derive_more::Display;
use rayon::prelude::*;
use rug::Float;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    slice::{Iter, IterMut},
    str::FromStr,
    sync::Arc,
};

use crate::{proxy::server::mosaic_proto::Parameters, service::error::ServiceError};

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
    // /// model precision.
    // pub precision: u32,
}

impl ModelWrapper {
    pub fn new(model: Model, dtype: DataType, model_version: u32) -> Option<Self> {
        Some(Self {
            model: Arc::new(model),
            dtype,
            model_version,
        })
    }
    /// Turns the model wrapper into proto parameter type.
    ///
    /// message Parameters {
    ///    bytes tensor = 1;
    ///    string data_type = 2;
    ///    uint32 model_version = 3;
    /// }
    ///
    pub fn wrapper_to_params(self) -> Parameters {
        let model = Model::serialize(&self.model, &DataType::F32);

        Parameters {
            tensor: model,
            data_type: self.dtype.to_string(),
            model_version: self.model_version,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
/// A representation of a machine learning model as vector object.
pub struct Model(pub Vec<Float>);

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
    /// Returns bool whether tuple is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns model with all zeros given a fixed length.
    pub fn zeros(length: &usize) -> Self {
        Model(vec![Float::new(53); *length])
    }
    /// Creates an iterator that yields references to the weights/parameters of this model.
    pub fn iter(&self) -> Iter<Float> {
        self.0.iter()
    }
    /// Creates an iterator that yields mutable references to the weights/parameters of this model.
    pub fn iter_mut(&mut self) -> IterMut<Float> {
        self.0.iter_mut()
    }
    fn from_bytes_array_f32(&mut self, bytes: Vec<u8>) {
        self.0 = bytes
            .par_chunks(4)
            .map(|x| Float::with_val(32, BigEndian::read_f32(x)))
            .collect::<Vec<_>>()
            .to_vec()
    }
    /// Conversion from bytes to Ratio for DataType F64
    fn from_bytes_array_f64(&mut self, bytes: Vec<u8>) {
        self.0 = bytes
            .par_chunks(8)
            .map(|x| Float::with_val(53, BigEndian::read_f64(x)))
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
            .map(|x| Float::to_f32(x).to_be_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>()
            .to_vec()
    }
    /// Conversion from Ratio to bytes for DataType F64
    fn primitive_to_bytes_64(&self) -> Vec<u8> {
        self.0
            .par_iter()
            .map(|x| Float::to_f64(x).to_be_bytes().to_vec())
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
